//! Terminal PTY handling.
//!
//! Spawns a real shell in a pseudo-terminal and reads its output.

use std::io::Read;

use anyhow::{Context, Result};
use portable_pty::{native_pty_system, CommandBuilder, PtySize};

/// A terminal session backed by a real PTY.
pub struct Terminal {
    /// The PTY master (we write to it, read from the child).
    pub writer: Box<dyn std::io::Write + Send>,
    /// The PTY reader (child output).
    pub reader: Box<dyn Read + Send>,
    /// The child process.
    pub child: Box<dyn portable_pty::Child + Send + Sync>,
    /// Parsed screen state (via vte).
    pub parser: vte::Parser,
    /// The screen contents (grid of cells).
    pub screen: Vec<Vec<char>>,
    /// Cursor position.
    pub cursor: (u16, u16),
    /// Terminal size.
    pub size: PtySize,
}

impl Terminal {
    /// Spawn a new terminal running `shell` in `workdir`.
    pub fn new(shell: &str, workdir: &str) -> Result<Self> {
        let pty_system = native_pty_system();
        let pair = pty_system
            .openpty(PtySize {
                rows: 24,
                cols: 80,
                pixel_width: 0,
                pixel_height: 0,
            })
            .context("failed to open PTY")?;

        let mut cmd = CommandBuilder::new(shell);
        cmd.cwd(workdir);
        cmd.env("TERM", "xterm-256color");
        cmd.env("COLORTERM", "truecolor");

        let child = pair
            .slave
            .spawn_command(cmd)
            .context("failed to spawn shell")?;

        let reader = pair
            .master
            .try_clone_reader()
            .context("failed to clone PTY reader")?;
        let writer = pair
            .master
            .take_writer()
            .context("failed to take PTY writer")?;

        Ok(Self {
            writer,
            reader,
            child,
            parser: vte::Parser::new(),
            screen: vec![vec![' '; 80]; 24],
            cursor: (0, 0),
            size: PtySize {
                rows: 24,
                cols: 80,
                pixel_width: 0,
                pixel_height: 0,
            },
        })
    }

    /// Read available output from the shell.
    pub fn read_output(&mut self) -> std::io::Result<usize> {
        let mut buf = [0u8; 4096];
        let n = self.reader.read(&mut buf)?;
        if n > 0 {
            // Feed the bytes to the vte parser.
            let mut performer = VtePerformer {
                screen: &mut self.screen,
                cursor: &mut self.cursor,
            };
            for byte in &buf[..n] {
                self.parser.advance(&mut performer, *byte);
            }
        }
        Ok(n)
    }

    /// Write input to the shell (keystrokes).
    pub fn write_input(&mut self, data: &[u8]) -> std::io::Result<()> {
        self.writer.write_all(data)?;
        self.writer.flush()
    }

    /// Resize the terminal.
    pub fn resize(&mut self, rows: u16, cols: u16) {
        self.size.rows = rows;
        self.size.cols = cols;
        self.screen = vec![vec![' '; cols as usize]; rows as usize];
        // In a full implementation this sends a resize to the PTY master.
        // portable-pty's resize is done via the master's resize method.
    }

    /// Check if the shell has exited.
    pub fn has_exited(&mut self) -> bool {
        matches!(self.child.try_wait(), Ok(Some(_)))
    }
}

/// vte performer that updates the screen grid.
struct VtePerformer<'a> {
    screen: &'a mut Vec<Vec<char>>,
    cursor: &'a mut (u16, u16),
}

impl vte::Perform for VtePerformer<'_> {
    fn print(&mut self, c: char) {
        let (x, y) = *self.cursor;
        if (y as usize) < self.screen.len() && (x as usize) < self.screen[y as usize].len() {
            self.screen[y as usize][x as usize] = c;
            self.cursor.0 += 1;
        }
    }

    fn execute(&mut self, byte: u8) {
        match byte {
            b'\n' => {
                self.cursor.0 = 0;
                self.cursor.1 += 1;
                if self.cursor.1 as usize >= self.screen.len() {
                    self.cursor.1 = self.screen.len() as u16 - 1;
                }
            }
            b'\r' => {
                self.cursor.0 = 0;
            }
            _ => {}
        }
    }

    fn csi_dispatch(
        &mut self,
        params: &vte::Params,
        _intermediates: &[u8],
        _ignore: bool,
        c: char,
    ) {
        // Handle cursor movement.
        let (mut x, mut y) = *self.cursor;
        match c {
            'H' | 'f' => {
                // CUP: cursor position.
                let row = params.iter().next().map(|v| v[0]).unwrap_or(1);
                let col = params.iter().nth(1).map(|v| v[0]).unwrap_or(1);
                y = (row.saturating_sub(1)) as u16;
                x = (col.saturating_sub(1)) as u16;
            }
            'A' => {
                let n = params.iter().next().map(|v| v[0]).unwrap_or(1);
                y = y.saturating_sub(n as u16);
            }
            'B' => {
                let n = params.iter().next().map(|v| v[0]).unwrap_or(1);
                y += n as u16;
            }
            'C' => {
                let n = params.iter().next().map(|v| v[0]).unwrap_or(1);
                x += n as u16;
            }
            'D' => {
                let n = params.iter().next().map(|v| v[0]).unwrap_or(1);
                x = x.saturating_sub(n as u16);
            }
            'J' => {
                // Clear screen.
                let mode = params.iter().next().map(|v| v[0]).unwrap_or(0);
                if mode == 2 {
                    for row in self.screen.iter_mut() {
                        for cell in row.iter_mut() {
                            *cell = ' ';
                        }
                    }
                }
            }
            'K' => {
                // Clear line.
                let mode = params.iter().next().map(|v| v[0]).unwrap_or(0);
                if (y as usize) < self.screen.len() {
                    let row = &mut self.screen[y as usize];
                    match mode {
                        0 => {
                            for cell in row.iter_mut().skip(x as usize) {
                                *cell = ' ';
                            }
                        }
                        1 => {
                            for cell in row.iter_mut().take(x as usize + 1) {
                                *cell = ' ';
                            }
                        }
                        2 => {
                            for cell in row.iter_mut() {
                                *cell = ' ';
                            }
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
        *self.cursor = (x, y);
    }
}
