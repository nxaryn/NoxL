//! Preset management: save, export, import, and switch complete desktop configs.
//!
//! A preset is a full `NoxlConfig` snapshot (theme, wallpaper, panel, widgets,
//! launcher, keybindings, behavior). Presets are stored as TOML files in
//! `~/.config/noxl/presets/`.

use std::path::{Path, PathBuf};

use noxl_config::NoxlConfig;

/// A named preset.
#[derive(Debug, Clone)]
pub struct Preset {
    pub name: String,
    pub config: NoxlConfig,
}

/// Preset directory.
pub fn presets_dir() -> PathBuf {
    noxl_config::Paths::config_dir().join("presets")
}

/// List all saved presets.
pub fn list() -> anyhow::Result<Vec<String>> {
    let dir = presets_dir();
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let mut names = Vec::new();
    for entry in std::fs::read_dir(&dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) == Some("toml") {
            if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                names.push(stem.to_string());
            }
        }
    }
    names.sort();
    Ok(names)
}

/// Save the current config as a named preset.
pub fn save(name: &str, config: &NoxlConfig) -> anyhow::Result<PathBuf> {
    let dir = presets_dir();
    std::fs::create_dir_all(&dir)?;
    let path = dir.join(format!("{}.toml", name));
    config.save_to(&path)?;
    tracing::info!(name, path = %path.display(), "preset saved");
    Ok(path)
}

/// Load a preset by name.
pub fn load(name: &str) -> anyhow::Result<NoxlConfig> {
    let path = presets_dir().join(format!("{}.toml", name));
    Ok(NoxlConfig::load_from(&path)?)
}

/// Apply a preset (save it as the active config).
pub fn apply(name: &str) -> anyhow::Result<NoxlConfig> {
    let config = load(name)?;
    config.save()?;
    tracing::info!(name, "preset applied");
    Ok(config)
}

/// Export a preset to an arbitrary path (for sharing).
pub fn export(name: &str, to: &Path) -> anyhow::Result<()> {
    let config = load(name)?;
    config.save_to(to)?;
    tracing::info!(name, to = %to.display(), "preset exported");
    Ok(())
}

/// Import a preset from a file.
pub fn import(from: &Path, name: &str) -> anyhow::Result<()> {
    let config = NoxlConfig::load_from(from)?;
    save(name, &config)?;
    tracing::info!(name, from = %from.display(), "preset imported");
    Ok(())
}

/// Delete a preset.
pub fn delete(name: &str) -> anyhow::Result<()> {
    let path = presets_dir().join(format!("{}.toml", name));
    if path.exists() {
        std::fs::remove_file(&path)?;
        tracing::info!(name, "preset deleted");
    }
    Ok(())
}
