//! Settings categories.
//!
//! Each category exposes get/set operations that read and write the real
//! configuration. The UI layer calls these; the category implementations
//! handle persistence and system integration.

use noxl_config::NoxlConfig;

/// A settings category.
pub trait Category {
    /// Category ID (e.g. "personalisation").
    fn id(&self) -> &str;
    /// Display name.
    fn name(&self) -> &str;
    /// Apply a setting change (key, value as JSON).
    fn set(&mut self, config: &mut NoxlConfig, key: &str, value: serde_json::Value) -> anyhow::Result<()>;
}

/// Personalisation category.
pub struct Personalisation;

impl Category for Personalisation {
    fn id(&self) -> &str { "personalisation" }
    fn name(&self) -> &str { "Personalisation" }

    fn set(&mut self, config: &mut NoxlConfig, key: &str, value: serde_json::Value) -> anyhow::Result<()> {
        match key {
            // Theme
            "theme.accent_primary" => {
                config.theme.colors.accent_primary = serde_json::from_value(value)?;
            }
            "theme.accent_secondary" => {
                config.theme.colors.accent_secondary = serde_json::from_value(value)?;
            }
            "theme.bg_primary" => {
                config.theme.colors.bg_primary = serde_json::from_value(value)?;
            }
            "theme.blur" => {
                config.theme.effects.blur = serde_json::from_value(value)?;
            }
            "theme.animations" => {
                config.theme.effects.animations = serde_json::from_value(value)?;
            }
            // Wallpaper
            "wallpaper" => {
                config.wallpaper = serde_json::from_value(value)?;
            }
            // Panel
            "panel.position" => {
                config.panel.position = serde_json::from_value(value)?;
            }
            "panel.size" => {
                config.panel.size = serde_json::from_value(value)?;
            }
            "panel.autohide" => {
                config.panel.autohide = serde_json::from_value(value)?;
            }
            "panel.opacity" => {
                config.panel.opacity = serde_json::from_value(value)?;
            }
            // Launcher
            "launcher.width" => {
                config.launcher.width = serde_json::from_value(value)?;
            }
            "launcher.show_recent" => {
                config.launcher.show_recent = serde_json::from_value(value)?;
            }
            // Widgets
            "widgets" => {
                config.widgets = serde_json::from_value(value)?;
            }
            _ => anyhow::bail!("unknown personalisation key: {}", key),
        }
        config.save()?;
        Ok(())
    }
}

/// System category.
pub struct System;

impl Category for System {
    fn id(&self) -> &str { "system" }
    fn name(&self) -> &str { "System" }

    fn set(&mut self, config: &mut NoxlConfig, key: &str, value: serde_json::Value) -> anyhow::Result<()> {
        match key {
            "behavior.vsync" => {
                config.behavior.vsync = serde_json::from_value(value)?;
            }
            "behavior.focus_follows_mouse" => {
                config.behavior.focus_follows_mouse = serde_json::from_value(value)?;
            }
            "workspaces.count" => {
                config.workspaces.count = serde_json::from_value(value)?;
            }
            _ => anyhow::bail!("unknown system key: {}", key),
        }
        config.save()?;
        Ok(())
    }
}

/// Connectivity category (Wi-Fi, Ethernet, Bluetooth, VPN).
pub struct Connectivity;

impl Category for Connectivity {
    fn id(&self) -> &str { "connectivity" }
    fn name(&self) -> &str { "Connectivity" }

    fn set(&mut self, _config: &mut NoxlConfig, key: &str, value: serde_json::Value) -> anyhow::Result<()> {
        // Connectivity settings are applied via NetworkManager D-Bus, not
        // stored in noxl config. This is a hook for the UI.
        tracing::debug!(key, ?value, "connectivity setting (applied via NetworkManager)");
        Ok(())
    }
}

/// Security category.
pub struct Security;

impl Category for Security {
    fn id(&self) -> &str { "security" }
    fn name(&self) -> &str { "Security" }

    fn set(&mut self, _config: &mut NoxlConfig, key: &str, value: serde_json::Value) -> anyhow::Result<()> {
        // Security settings (firewall, permissions) go through polkit and
        // system tools, not noxl config.
        tracing::debug!(key, ?value, "security setting (applied via polkit)");
        Ok(())
    }
}
