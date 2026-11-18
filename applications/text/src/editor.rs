//! Text editor state: tabs, content, cursor, syntax highlighting.

use std::path::PathBuf;

use anyhow::Result;

/// A syntax highlighting mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyntaxMode {
    Plain,
    Rust,
    Python,
    JavaScript,
    C,
    Shell,
    Toml,
    Json,
    Markdown,
}

impl SyntaxMode {
    /// Detect syntax mode from a file extension.
    pub fn from_extension(path: &std::path::Path) -> Self {
        match path.extension().and_then(|e| e.to_str()) {
            Some("rs") => SyntaxMode::Rust,
            Some("py") => SyntaxMode::Python,
            Some("js") | Some("ts") => SyntaxMode::JavaScript,
            Some("c") | Some("h") | Some("cpp") => SyntaxMode::C,
            Some("sh") | Some("bash") => SyntaxMode::Shell,
            Some("toml") => SyntaxMode::Toml,
            Some("json") => SyntaxMode::Json,
            Some("md") => SyntaxMode::Markdown,
            _ => SyntaxMode::Plain,
        }
    }
}

/// A single tab.
#[derive(Debug, Clone)]
pub struct Tab {
    pub path: Option<PathBuf>,
    pub content: String,
    pub cursor: (usize, usize), // (line, col)
    pub modified: bool,
    pub syntax: SyntaxMode,
}

impl Tab {
    pub fn new() -> Self {
        Self {
            path: None,
            content: String::new(),
            cursor: (0, 0),
            modified: false,
            syntax: SyntaxMode::Plain,
        }
    }

    pub fn from_file(path: &std::path::Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        Ok(Self {
            path: Some(path.to_path_buf()),
            content,
            cursor: (0, 0),
            modified: false,
            syntax: SyntaxMode::from_extension(path),
        })
    }
}

/// The editor state.
pub struct Editor {
    pub tabs: Vec<Tab>,
    pub active_tab: usize,
}

impl Editor {
    pub fn new() -> Self {
        Self {
            tabs: vec![Tab::new()],
            active_tab: 0,
        }
    }

    pub fn open(&mut self, path: &str) -> Result<()> {
        let tab = Tab::from_file(std::path::Path::new(path))?;
        self.tabs.push(tab);
        self.active_tab = self.tabs.len() - 1;
        Ok(())
    }

    pub fn new_tab(&mut self) {
        self.tabs.push(Tab::new());
        self.active_tab = self.tabs.len() - 1;
    }

    pub fn close_tab(&mut self, index: usize) {
        if self.tabs.len() > 1 {
            self.tabs.remove(index);
            if self.active_tab >= self.tabs.len() {
                self.active_tab = self.tabs.len() - 1;
            }
        }
    }

    pub fn save(&mut self) -> Result<()> {
        let tab = &mut self.tabs[self.active_tab];
        if let Some(path) = &tab.path {
            std::fs::write(path, &tab.content)?;
            tab.modified = false;
        }
        Ok(())
    }

    pub fn save_as(&mut self, path: &str) -> Result<()> {
        let tab = &mut self.tabs[self.active_tab];
        std::fs::write(path, &tab.content)?;
        tab.path = Some(PathBuf::from(path));
        tab.modified = false;
        Ok(())
    }

    pub fn insert_char(&mut self, c: char) {
        let tab = &mut self.tabs[self.active_tab];
        let (line, col) = tab.cursor;
        let lines: Vec<&str> = tab.content.split('\n').collect();
        if line < lines.len() {
            let mut new_lines: Vec<String> = lines.iter().map(|s| s.to_string()).collect();
            let l = &mut new_lines[line];
            let byte_pos = l.char_indices().nth(col).map(|(i, _)| i).unwrap_or(l.len());
            l.insert(byte_pos, c);
            tab.content = new_lines.join("\n");
            tab.cursor.1 += 1;
            tab.modified = true;
        }
    }

    pub fn insert_newline(&mut self) {
        let tab = &mut self.tabs[self.active_tab];
        let (line, col) = tab.cursor;
        let lines: Vec<&str> = tab.content.split('\n').collect();
        if line < lines.len() {
            let mut new_lines: Vec<String> = lines.iter().map(|s| s.to_string()).collect();
            let l = &mut new_lines[line];
            let byte_pos = l.char_indices().nth(col).map(|(i, _)| i).unwrap_or(l.len());
            l.insert(byte_pos, '\n');
            tab.content = new_lines.join("\n");
            tab.cursor = (line + 1, 0);
            tab.modified = true;
        }
    }

    pub fn backspace(&mut self) {
        let tab = &mut self.tabs[self.active_tab];
        let (line, col) = tab.cursor;
        if col > 0 {
            let lines: Vec<&str> = tab.content.split('\n').collect();
            if line < lines.len() {
                let mut new_lines: Vec<String> = lines.iter().map(|s| s.to_string()).collect();
                let l = &mut new_lines[line];
                let byte_pos = l.char_indices().nth(col - 1).map(|(i, _)| i).unwrap_or(0);
                l.remove(byte_pos);
                tab.content = new_lines.join("\n");
                tab.cursor.1 -= 1;
                tab.modified = true;
            }
        }
    }

    pub fn line_count(&self) -> usize {
        self.tabs[self.active_tab].content.split('\n').count()
    }
}
