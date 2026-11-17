//! Application launcher configuration.

use serde::{Deserialize, Serialize};

/// Application launcher (start menu) configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct LauncherConfig {
    /// Launcher width in pixels.
    pub width: u32,
    /// Launcher height in pixels.
    pub height: u32,
    /// Show a search field.
    pub show_search: bool,
    /// Show recently used applications.
    pub show_recent: bool,
    /// Show recently installed applications.
    pub show_recent_installed: bool,
    /// Number of recent apps to show.
    pub recent_count: u32,
    /// Show application categories.
    pub show_categories: bool,
    /// Pin specific apps (desktop file IDs) to the top.
    pub pinned_apps: Vec<String>,
    /// Power buttons shown at the bottom.
    pub show_power_controls: bool,
    /// Search history enabled.
    pub search_history: bool,
    /// Keyboard navigation enabled.
    pub keyboard_nav: bool,
}

impl Default for LauncherConfig {
    fn default() -> Self {
        Self {
            width: 640,
            height: 560,
            show_search: true,
            show_recent: true,
            show_recent_installed: true,
            recent_count: 8,
            show_categories: true,
            pinned_apps: Vec::new(),
            show_power_controls: true,
            search_history: true,
            keyboard_nav: true,
        }
    }
}
