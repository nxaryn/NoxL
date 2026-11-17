//! NoxL configuration model.
//!
//! This crate is the single source of truth for every user-configurable aspect
//! of the NoxL desktop. It defines the on-disk schema (TOML), loads and
//! validates it, and exposes typed accessors to the compositor, panel, launcher,
//! notifications, and applications.
//!
//! Design rules:
//! - Every visible setting is represented here, so it can be persisted and
//!   restored (presets, theme switching, etc.).
//! - Config is plain TOML under `$XDG_CONFIG_HOME/noxl/` so it is human-readable
//!   and editable without NoxL running.
//! - Defaults are OBSIDIAN. A missing file yields defaults, never an error.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub mod theme;
pub mod panel;
pub mod launcher;
pub mod keybindings;
pub mod widgets;
pub mod wallpaper;

pub use theme::*;
pub use panel::*;
pub use launcher::*;
pub use keybindings::*;
pub use widgets::*;
pub use wallpaper::*;

/// Top-level NoxL configuration.
///
/// This is the root document written to `~/.config/noxl/config.toml`.
/// Every subsystem reads its own section and ignores unknown fields, so the
/// file remains forward- and backward-compatible.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct NoxlConfig {
    /// OBSIDIAN theme definition (colors, fonts, shapes, effects).
    pub theme: Theme,
    /// Panel/taskbar configuration (position, size, plugins, auto-hide).
    pub panel: PanelConfig,
    /// Application launcher configuration.
    pub launcher: LauncherConfig,
    /// Keyboard/mouse shortcuts.
    pub keybindings: Keybindings,
    /// Desktop widget layout and visibility.
    pub widgets: WidgetsConfig,
    /// Wallpaper configuration (per-monitor, slideshow, scaling).
    pub wallpaper: WallpaperConfig,
    /// Virtual desktops / workspaces.
    pub workspaces: WorkspacesConfig,
    /// General behaviour toggles.
    pub behavior: BehaviorConfig,
}

impl Default for NoxlConfig {
    fn default() -> Self {
        Self {
            theme: Theme::default(),
            panel: PanelConfig::default(),
            launcher: LauncherConfig::default(),
            keybindings: Keybindings::default(),
            widgets: WidgetsConfig::default(),
            wallpaper: WallpaperConfig::default(),
            workspaces: WorkspacesConfig::default(),
            behavior: BehaviorConfig::default(),
        }
    }
}

/// Virtual desktops / workspaces.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct WorkspacesConfig {
    /// Number of workspaces.
    pub count: u32,
    /// Show workspace thumbnails in the panel switcher.
    pub show_thumbnails: bool,
    /// Assign apps to specific workspaces (app_id -> workspace index).
    pub assignments: std::collections::HashMap<String, u32>,
}

impl Default for WorkspacesConfig {
    fn default() -> Self {
        Self {
            count: 4,
            show_thumbnails: true,
            assignments: Default::default(),
        }
    }
}

/// General behaviour toggles.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct BehaviorConfig {
    /// Enable animations (windows, menus, etc.).
    pub animations: bool,
    /// Animation duration in milliseconds.
    pub animation_duration_ms: u32,
    /// Enable window shadows.
    pub window_shadows: bool,
    /// Compositor VSync / frame pacing.
    pub vsync: bool,
    /// Focus follows mouse.
    pub focus_follows_mouse: bool,
    /// Click-to-focus (raises window on click).
    pub click_to_focus: bool,
}

impl Default for BehaviorConfig {
    fn default() -> Self {
        Self {
            animations: true,
            animation_duration_ms: 160,
            window_shadows: true,
            vsync: true,
            focus_follows_mouse: false,
            click_to_focus: true,
        }
    }
}

/// Where NoxL config lives.
pub struct Paths;

impl Paths {
    /// `$XDG_CONFIG_HOME/noxl` or `~/.config/noxl`.
    pub fn config_dir() -> PathBuf {
        dirs_config().join("noxl")
    }

    /// `$XDG_CONFIG_HOME/noxl/config.toml`
    pub fn config_file() -> PathBuf {
        Self::config_dir().join("config.toml")
    }

    /// `$XDG_DATA_HOME/noxl` or `~/.local/share/noxl`.
    pub fn data_dir() -> PathBuf {
        dirs_data().join("noxl")
    }

    /// `$XDG_STATE_HOME/noxl` or `~/.local/state/noxl`.
    pub fn state_dir() -> PathBuf {
        dirs_state().join("noxl")
    }

    /// `$XDG_CACHE_HOME/noxl` or `~/.cache/noxl`.
    pub fn cache_dir() -> PathBuf {
        dirs_cache().join("noxl")
    }
}

fn dirs_config() -> PathBuf {
    std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| {
            directories::BaseDirs::new()
                .map(|b| b.config_dir().to_path_buf())
        })
        .unwrap_or_else(|| PathBuf::from("/etc"))
}

fn dirs_data() -> PathBuf {
    std::env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .or_else(|| {
            directories::BaseDirs::new()
                .map(|b| b.data_dir().to_path_buf())
        })
        .unwrap_or_else(|| PathBuf::from("/usr/share"))
}

fn dirs_state() -> PathBuf {
    std::env::var_os("XDG_STATE_HOME")
        .map(PathBuf::from)
        .or_else(|| {
            directories::BaseDirs::new()
                .and_then(|b| b.state_dir().map(|p| p.to_path_buf()))
        })
        .unwrap_or_else(|| PathBuf::from("/var/lib"))
}

fn dirs_cache() -> PathBuf {
    std::env::var_os("XDG_CACHE_HOME")
        .map(PathBuf::from)
        .or_else(|| {
            directories::BaseDirs::new()
                .map(|b| b.cache_dir().to_path_buf())
        })
        .unwrap_or_else(|| PathBuf::from("/var/cache"))
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("failed to read config: {0}")]
    Io(#[from] std::io::Error),
    #[error("failed to parse config TOML: {0}")]
    Toml(#[from] toml::de::Error),
    #[error("failed to serialize config: {0}")]
    Serialize(#[from] toml::ser::Error),
}

impl NoxlConfig {
    /// Load config from the default path, falling back to defaults if absent.
    pub fn load() -> Result<Self, ConfigError> {
        let path = Paths::config_file();
        Self::load_from(&path)
    }

    /// Load config from a specific path.
    pub fn load_from(path: &std::path::Path) -> Result<Self, ConfigError> {
        match std::fs::read_to_string(path) {
            Ok(s) => Ok(toml::from_str(&s)?),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                tracing::info!(path = %path.display(), "config not found; using defaults");
                Ok(Self::default())
            }
            Err(e) => Err(ConfigError::Io(e)),
        }
    }

    /// Save config to the default path (creating parent dirs).
    pub fn save(&self) -> Result<(), ConfigError> {
        let path = Paths::config_file();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let s = toml::to_string_pretty(self)?;
        std::fs::write(&path, s)?;
        tracing::info!(path = %path.display(), "config saved");
        Ok(())
    }

    /// Save config to a specific path.
    pub fn save_to(&self, path: &std::path::Path) -> Result<(), ConfigError> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let s = toml::to_string_pretty(self)?;
        std::fs::write(path, s)?;
        Ok(())
    }
}
