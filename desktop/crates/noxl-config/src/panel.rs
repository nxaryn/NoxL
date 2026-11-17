//! Panel / taskbar configuration.
//!
//! The panel is fully configurable: position, size, which plugins appear, and
//! auto-hide behaviour. Each plugin is independently toggleable.

use serde::{Deserialize, Serialize};

/// Which screen edge the panel is anchored to.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum PanelPosition {
    Top,
    Bottom,
    Left,
    Right,
}

impl Default for PanelPosition {
    fn default() -> Self {
        Self::Bottom
    }
}

/// Auto-hide mode.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum PanelAutohide {
    /// Always visible.
    Never,
    /// Hide when the cursor leaves.
    OnCursorLeave,
    /// Hide when a window is maximized.
    OnMaximized,
}

impl Default for PanelAutohide {
    fn default() -> Self {
        Self::Never
    }
}

/// A single panel instance. NoxL supports multiple panels (e.g. one per monitor).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct PanelConfig {
    /// Panel position on screen.
    pub position: PanelPosition,
    /// Panel thickness in pixels.
    pub size: u32,
    /// Auto-hide behaviour.
    pub autohide: PanelAutohide,
    /// Transparency 0.0 (opaque) .. 1.0 (invisible).
    pub opacity: f32,
    /// Enable background blur.
    pub blur: bool,
    /// Spacing between panel items, in pixels.
    pub item_spacing: u32,
    /// Which plugins are enabled, in display order.
    pub plugins: Vec<PanelPlugin>,
    /// Show the application launcher button.
    pub show_launcher: bool,
    /// Show the workspace switcher.
    pub show_workspaces: bool,
    /// Show the system tray.
    pub show_tray: bool,
    /// Show the clock.
    pub show_clock: bool,
    /// Show network/audio/battery indicators.
    pub show_status_icons: bool,
    /// Pin specific apps to the panel (desktop file IDs).
    pub pinned_apps: Vec<String>,
}

impl Default for PanelConfig {
    fn default() -> Self {
        Self {
            position: PanelPosition::Bottom,
            size: 48,
            autohide: PanelAutohide::Never,
            opacity: 0.92,
            blur: true,
            item_spacing: 6,
            plugins: vec![
                PanelPlugin::Launcher,
                PanelPlugin::Taskbar,
                PanelPlugin::Workspaces,
                PanelPlugin::Tray,
                PanelPlugin::StatusIcons,
                PanelPlugin::Clock,
            ],
            show_launcher: true,
            show_workspaces: true,
            show_tray: true,
            show_clock: true,
            show_status_icons: true,
            pinned_apps: Vec::new(),
        }
    }
}

/// A panel plugin/section.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum PanelPlugin {
    /// Application launcher button.
    Launcher,
    /// Running/pinned application list.
    Taskbar,
    /// Workspace switcher.
    Workspaces,
    /// System tray (StatusNotifierItem).
    Tray,
    /// Network/audio/battery indicators.
    StatusIcons,
    /// Clock and calendar popup.
    Clock,
    /// Notification indicator.
    Notifications,
}
