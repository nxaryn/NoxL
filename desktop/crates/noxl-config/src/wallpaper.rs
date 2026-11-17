//! Wallpaper configuration.
//!
//! Supports static images, slideshows, per-monitor wallpapers, scaling modes,
//! and (optionally) video wallpapers.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// How the wallpaper fills the screen.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum WallpaperScaling {
    /// Fill the screen, cropping if needed.
    Fill,
    /// Fit the whole image, letterboxing if needed.
    Fit,
    /// Stretch to fill (may distort).
    Stretch,
    /// Center at native size.
    Center,
    /// Tile across the screen.
    Tile,
}

impl Default for WallpaperScaling {
    fn default() -> Self {
        Self::Fill
    }
}

/// Wallpaper mode.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "mode", rename_all = "snake_case")]
pub enum WallpaperConfig {
    /// A single static image.
    Static {
        /// Path to the image file.
        path: String,
        /// Scaling mode.
        scaling: WallpaperScaling,
    },
    /// A slideshow of images.
    Slideshow {
        /// Directory containing images, or explicit list.
        images: Vec<String>,
        /// Seconds per image.
        interval_secs: u32,
        /// Randomize order.
        random: bool,
        /// Scaling mode.
        scaling: WallpaperScaling,
    },
    /// A solid color.
    Solid {
        /// Hex color like "#09090D".
        color: String,
    },
    /// Per-monitor wallpapers (connector name -> config).
    PerMonitor {
        /// Map of connector name (e.g. "DP-1") to a static config.
        monitors: HashMap<String, StaticWallpaper>,
    },
}

/// A simple static wallpaper descriptor, used inside PerMonitor.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct StaticWallpaper {
    pub path: String,
    pub scaling: WallpaperScaling,
}

impl Default for StaticWallpaper {
    fn default() -> Self {
        Self {
            path: String::new(),
            scaling: WallpaperScaling::Fill,
        }
    }
}

impl Default for WallpaperConfig {
    fn default() -> Self {
        // Default to the OBSIDIAN wallpaper shipped in the live image.
        StaticWallpaper::default();
        Self::Static {
            path: "/usr/share/noxl/wallpapers/obsidian-default.png".into(),
            scaling: WallpaperScaling::Fill,
        }
    }
}
