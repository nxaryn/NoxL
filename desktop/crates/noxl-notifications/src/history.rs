//! Notification history: persistent store of past notifications.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// A stored notification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notification {
    /// Unique ID.
    pub id: u32,
    /// Application name (from the Notify caller).
    pub app_name: String,
    /// Icon name or path.
    pub app_icon: String,
    /// Summary (title).
    pub summary: String,
    /// Body text.
    pub body: String,
    /// Unix timestamp (seconds).
    pub timestamp: u64,
    /// Whether the notification has been read.
    pub read: bool,
}

/// Persistent notification history.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct History {
    pub notifications: Vec<Notification>,
    /// Next notification ID.
    pub next_id: u32,
}

impl History {
    /// Path to the history file.
    pub fn path() -> PathBuf {
        noxl_config::Paths::data_dir()
            .join("notifications")
            .join("history.json")
    }

    /// Load history from disk, or return an empty history.
    pub fn load() -> anyhow::Result<Self> {
        let path = Self::path();
        match std::fs::read_to_string(&path) {
            Ok(s) => Ok(serde_json::from_str(&s)?),
            Err(_) => Ok(Self::default()),
        }
    }

    /// Save history to disk.
    pub fn save(&self) -> anyhow::Result<()> {
        let path = Self::path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&path, serde_json::to_string_pretty(self)?)?;
        Ok(())
    }

    /// Add a notification and return its ID.
    pub fn add(&mut self, app_name: String, app_icon: String, summary: String, body: String) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        self.notifications.push(Notification {
            id,
            app_name,
            app_icon,
            summary,
            body,
            timestamp: now,
            read: false,
        });
        id
    }

    /// Mark a notification as read.
    pub fn mark_read(&mut self, id: u32) {
        if let Some(n) = self.notifications.iter_mut().find(|n| n.id == id) {
            n.read = true;
        }
    }

    /// Remove a notification.
    pub fn remove(&mut self, id: u32) {
        self.notifications.retain(|n| n.id != id);
    }

    /// Unread notifications.
    pub fn unread(&self) -> Vec<&Notification> {
        self.notifications.iter().filter(|n| !n.read).collect()
    }
}
