//! Application discovery from freedesktop `.desktop` files.
//!
//! Reads the real installed applications from the standard application
//! directories, parses their `.desktop` entries, and returns a list of
//! `AppEntry`. This is how the launcher retrieves real installed applications
//! from the operating system.

use std::path::PathBuf;

use freedesktop_desktop_entry::DesktopEntry;

/// A discovered application.
#[derive(Debug, Clone)]
pub struct AppEntry {
    /// Desktop file ID (e.g. "firefox.desktop").
    pub id: String,
    /// Display name.
    pub name: String,
    /// Icon name.
    pub icon: String,
    /// Categories (e.g. ["Network", "WebBrowser"]).
    pub categories: Vec<String>,
    /// Executable to run (from the Exec= line).
    pub exec: String,
    /// Whether the app should be shown in the launcher.
    pub visible: bool,
}

/// The standard application directories, in priority order.
fn application_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();

    // User applications (highest priority).
    if let Some(data_home) = dirs::data_dir() {
        dirs.push(data_home.join("applications"));
    }

    // System applications.
    dirs.push(PathBuf::from("/usr/share/applications"));
    dirs.push(PathBuf::from("/usr/local/share/applications"));

    dirs
}

/// Load all installed applications.
pub fn load_applications() -> anyhow::Result<Vec<AppEntry>> {
    let mut apps = Vec::new();
    let mut seen_ids = std::collections::HashSet::new();

    for dir in application_dirs() {
        let entries = match std::fs::read_dir(&dir) {
            Ok(e) => e,
            Err(_) => continue,
        };

        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("desktop") {
                continue;
            }

            let id = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_string();

            // Skip duplicates (user dir overrides system dir).
            if !seen_ids.insert(id.clone()) {
                continue;
            }

            if let Some(app) = parse_desktop_entry(&path, &id) {
                if app.visible {
                    apps.push(app);
                }
            }
        }
    }

    // Sort by name.
    apps.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

    Ok(apps)
}

/// Parse a single .desktop file.
fn parse_desktop_entry(path: &std::path::Path, id: &str) -> Option<AppEntry> {
    let entry = DesktopEntry::from_str(&std::fs::read_to_string(path).ok()?, true).ok()?;

    // Skip hidden entries.
    if entry.hidden() {
        return None;
    }

    // Only show apps that have a name and an Exec line.
    let name = entry.name::<String>(&["en"]).or_else(|| {
        // Fall back to any available locale.
        entry.name::<String>(&[]).clone()
    })?;

    let exec = entry.exec()?.to_string();

    let icon = entry.icon().map(|s| s.to_string()).unwrap_or_default();

    let categories: Vec<String> = entry
        .categories()
        .map(|cats| cats.iter().map(|s| s.to_string()).collect())
        .unwrap_or_default();

    // Skip NoDisplay entries.
    let visible = !entry.no_display();

    Some(AppEntry {
        id: id.to_string(),
        name,
        icon,
        categories,
        exec,
        visible,
    })
}
