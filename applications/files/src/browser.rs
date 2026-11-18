//! Directory browser: reads real files from the filesystem.

use std::path::PathBuf;

use anyhow::Result;

/// A file or directory entry.
#[derive(Debug, Clone)]
pub struct Entry {
    pub name: String,
    pub path: PathBuf,
    pub is_dir: bool,
    pub size: u64,
}

/// The browser state.
pub struct Browser {
    pub current_path: PathBuf,
    pub entries: Vec<Entry>,
    pub history: Vec<PathBuf>,
    pub history_index: usize,
}

impl Browser {
    /// Create a browser at the given path.
    pub fn new(path: &str) -> Result<Self> {
        let current_path = PathBuf::from(path);
        let mut browser = Self {
            current_path: current_path.clone(),
            entries: Vec::new(),
            history: vec![current_path.clone()],
            history_index: 0,
        };
        browser.refresh()?;
        Ok(browser)
    }

    /// Reload the current directory.
    pub fn refresh(&mut self) -> Result<()> {
        self.entries.clear();
        for entry in std::fs::read_dir(&self.current_path)? {
            let entry = entry?;
            let path = entry.path();
            let metadata = entry.metadata()?;
            let name = entry.file_name().to_string_lossy().to_string();
            self.entries.push(Entry {
                name,
                path,
                is_dir: metadata.is_dir(),
                size: if metadata.is_dir() { 0 } else { metadata.len() },
            });
        }
        // Sort: directories first, then alphabetical.
        self.entries.sort_by(|a, b| {
            match (a.is_dir, b.is_dir) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
            }
        });
        Ok(())
    }

    /// Navigate to a path.
    pub fn navigate(&mut self, path: PathBuf) -> Result<()> {
        self.current_path = path;
        self.refresh()?;
        // Truncate forward history.
        self.history.truncate(self.history_index + 1);
        self.history.push(self.current_path.clone());
        self.history_index = self.history.len() - 1;
        Ok(())
    }

    /// Go to parent directory.
    pub fn go_up(&mut self) -> Result<()> {
        if let Some(parent) = self.current_path.parent() {
            self.navigate(parent.to_path_buf())?;
        }
        Ok(())
    }

    /// Go back.
    pub fn go_back(&mut self) -> Result<()> {
        if self.history_index > 0 {
            self.history_index -= 1;
            self.current_path = self.history[self.history_index].clone();
            self.refresh()?;
        }
        Ok(())
    }

    /// Go forward.
    pub fn go_forward(&mut self) -> Result<()> {
        if self.history_index < self.history.len() - 1 {
            self.history_index += 1;
            self.current_path = self.history[self.history_index].clone();
            self.refresh()?;
        }
        Ok(())
    }

    /// Open a directory.
    pub fn open(&mut self, index: usize) -> Result<()> {
        if let Some(entry) = self.entries.get(index) {
            if entry.is_dir {
                self.navigate(entry.path.clone())?;
            }
        }
        Ok(())
    }

    /// Delete a file.
    pub fn delete(&mut self, index: usize) -> Result<()> {
        if let Some(entry) = self.entries.get(index) {
            if entry.is_dir {
                std::fs::remove_dir_all(&entry.path)?;
            } else {
                std::fs::remove_file(&entry.path)?;
            }
            self.refresh()?;
        }
        Ok(())
    }

    /// Rename a file.
    pub fn rename(&mut self, index: usize, new_name: &str) -> Result<()> {
        if let Some(entry) = self.entries.get(index) {
            let new_path = entry.path.with_file_name(new_name);
            std::fs::rename(&entry.path, &new_path)?;
            self.refresh()?;
        }
        Ok(())
    }

    /// Format file size for display.
    pub fn format_size(entry: &Entry) -> String {
        if entry.is_dir {
            return "<DIR>".into();
        }
        const KB: u64 = 1024;
        const MB: u64 = KB * 1024;
        const GB: u64 = MB * 1024;
        if entry.size >= GB {
            format!("{:.1} GB", entry.size as f64 / GB as f64)
        } else if entry.size >= MB {
            format!("{:.1} MB", entry.size as f64 / MB as f64)
        } else if entry.size >= KB {
            format!("{:.1} KB", entry.size as f64 / KB as f64)
        } else {
            format!("{} B", entry.size)
        }
    }
}
