//! IPC control-socket server.
//!
//! Listens on `$XDG_RUNTIME_DIR/noxl/control.sock` and speaks the NoxL control
//! protocol (newline-delimited JSON). Clients (panel, launcher, settings,
//! scripts) connect, send a `Command`, and receive `Event`s.
//!
//! The server is non-blocking from the compositor's perspective: `poll_commands`
//! drains any pending client commands, and `send_event` queues events for the
//! next flush.

use std::{
    collections::VecDeque,
    io::{BufRead, BufReader, Write},
    os::unix::net::UnixListener,
    path::Path,
    sync::mpsc::{channel, Receiver, Sender},
    thread,
};

use tracing::{debug, error, warn};

use noxl_ipc::{control_socket_path, Command, Event, Frame};

/// The IPC server.
pub struct IpcServer {
    /// Commands received from clients, drained by the compositor.
    command_rx: Receiver<Command>,
    /// Events to send to clients.
    event_tx: Sender<Event>,
    /// Handle to the listener thread.
    _thread: Option<thread::JoinHandle<()>>,
}

impl IpcServer {
    /// Start the IPC server, returning a handle.
    pub fn new() -> anyhow::Result<Self> {
        let path = control_socket_path();

        // Ensure the socket directory exists.
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        // Remove a stale socket from a previous run.
        if path.exists() {
            let _ = std::fs::remove_file(&path);
        }

        let listener = UnixListener::bind(&path)?;
        tracing::info!(path = %path.display(), "IPC socket bound");

        let (cmd_tx, cmd_rx) = channel::<Command>();
        let (event_tx, event_rx) = channel::<Event>();

        let thread = thread::spawn(move || {
            ipc_thread(listener, cmd_tx, event_rx);
        });

        Ok(Self {
            command_rx: cmd_rx,
            event_tx,
            _thread: Some(thread),
        })
    }

    /// Drain all pending commands from clients.
    pub fn poll_commands(&self) -> Vec<Command> {
        let mut cmds = Vec::new();
        while let Ok(cmd) = self.command_rx.try_recv() {
            cmds.push(cmd);
        }
        cmds
    }

    /// Queue an event to be sent to connected clients.
    pub fn send_event(&self, event: Event) {
        let _ = self.event_tx.send(event);
    }
}

/// The listener thread: accepts clients and reads commands.
fn ipc_thread(
    listener: UnixListener,
    cmd_tx: Sender<Command>,
    event_rx: Receiver<Event>,
) {
    // Track connected clients so we can broadcast events.
    let mut clients: Vec<std::os::unix::net::UnixStream> = Vec::new();

    // Set the listener to non-blocking so we can also drain events.
    if let Err(e) = listener.set_nonblocking(true) {
        error!(error = %e, "failed to set IPC listener nonblocking");
        return;
    }

    loop {
        // Accept new clients.
        match listener.accept() {
            Ok((stream, _)) => {
                debug!("IPC client connected");
                let _ = stream.set_nonblocking(true);
                clients.push(stream);
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {}
            Err(e) => {
                error!(error = %e, "IPC accept error");
                continue;
            }
        }

        // Read commands from each client.
        let mut i = 0;
        while i < clients.len() {
            let mut disconnected = false;
            let mut lines = Vec::new();
            {
                let mut reader = BufReader::new(clients[i].try_clone().unwrap());
                loop {
                    let mut line = String::new();
                    match reader.read_line(&mut line) {
                        Ok(0) => {
                            disconnected = true;
                            break;
                        }
                        Ok(_) => {
                            if line.ends_with('\n') {
                                lines.push(line);
                            } else {
                                break;
                            }
                        }
                        Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => break,
                        Err(e) => {
                            warn!(error = %e, "IPC read error");
                            disconnected = true;
                            break;
                        }
                    }
                }
            }

            for line in lines {
                let trimmed = line.trim();
                if trimmed.is_empty() {
                    continue;
                }
                match Frame::decode(trimmed) {
                    Ok(frame) => match serde_json::from_value::<Command>(frame.payload) {
                        Ok(cmd) => {
                            if cmd_tx.send(cmd).is_err() {
                                break;
                            }
                        }
                        Err(e) => {
                            warn!(error = %e, "invalid IPC command");
                        }
                    },
                    Err(e) => {
                        warn!(error = %e, "invalid IPC frame");
                    }
                }
            }

            if disconnected {
                clients.remove(i);
            } else {
                i += 1;
            }
        }

        // Drain and broadcast events.
        while let Ok(event) = event_rx.try_recv() {
            let frame = Frame::new(serde_json::to_value(&event).unwrap_or_default());
            let data = frame.encode();
            for client in &mut clients {
                let _ = client.write_all(data.as_bytes());
            }
        }

        thread::sleep(std::time::Duration::from_millis(4));
    }
}
