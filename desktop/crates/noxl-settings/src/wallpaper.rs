//! Wallpaper management: browse, set, slideshow, per-monitor.
//!
//! Scans the user's wallpaper directories and the system NoxL wallpapers,
//! and provides operations to set the wallpaper (static, slideshow, per-monitor).

use std::path::PathBuf;

use noxl_config::{WallpaperConfig, WallpaperScaling};

/// A wallpaper image.
#[derive(Debug, Clone)]
pub struct Wallpaper {
    pub path: PathBuf,
    pub name: String,
}

/// Directories to scan for wallpapers.
fn wallpaper_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();

    // User wallpapers.
    if let Some(data) = directories::BaseDirs::new().map(|b| b.data_dir().to_path_buf()) {
        dirs.push(std::path::PathBuf::from(data).join("noxl").join("wallpapers"));
    }
    if let Some(pictures) = directories::UserDirs::new().and_then(|u| u.picture_dir().map(|p| p.to_path_buf())) {
        dirs.push(pictures);
    }

    // System wallpapers.
    dirs.push(PathBuf::from("/usr/share/noxl/wallpapers"));
    dirs.push(PathBuf::from("/usr/share/wallpapers"));

    dirs
}

/// List available wallpapers.
pub fn list() -> Vec<Wallpaper> {
    let mut wallpapers = Vec::new();
    let extensions = ["png", "jpg", "jpeg", "webp", "bmp"];

    for dir in wallpaper_dirs() {
        let entries = match std::fs::read_dir(&dir) {
            Ok(e) => e,
            Err(_) => continue,
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                if extensions.contains(&ext.to_lowercase().as_str()) {
                    let name = path
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("wallpaper")
                        .to_string();
                    wallpapers.push(Wallpaper { path, name });
                }
            }
        }
    }

    wallpapers
}

/// Set a static wallpaper.
pub fn set_static(path: &str, scaling: WallpaperScaling) -> WallpaperConfig {
    WallpaperConfig::Static {
        path: path.to_string(),
        scaling,
    }
}

/// Set a slideshow wallpaper.
pub fn set_slideshow(images: Vec<String>, interval_secs: u32, random: bool) -> WallpaperConfig {
    WallpaperConfig::Slideshow {
        images,
        interval_secs,
        random,
        scaling: WallpaperScaling::Fill,
    }
}

/// Set a solid color wallpaper.
pub fn set_solid(color: &str) -> WallpaperConfig {
    WallpaperConfig::Solid {
        color: color.to_string(),
    }
}
