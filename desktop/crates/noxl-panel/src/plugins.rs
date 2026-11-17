//! Panel plugins: launcher button, taskbar, workspaces, tray, status icons, clock.
//!
//! Each plugin is a section of the panel. Plugins are enabled/disabled and
//! ordered by `panel_config.plugins`.

use anyhow::Context;

/// A panel plugin.
pub trait Plugin {
    /// Render the plugin into the panel.
    fn render(&self, config: &noxl_config::NoxlConfig) -> anyhow::Result<()>;
    /// Handle a click on the plugin.
    fn on_click(&mut self, x: u32) {}
}

/// The set of active plugins.
pub struct PluginSet {
    pub plugins: Vec<Box<dyn Plugin>>,
}

impl PluginSet {
    pub fn new() -> Self {
        Self { plugins: Vec::new() }
    }
}

impl Default for PluginSet {
    fn default() -> Self {
        Self::new()
    }
}

/// Initialize the plugins enabled in the config.
pub fn init(config: &noxl_config::NoxlConfig) -> anyhow::Result<PluginSet> {
    let mut plugins: Vec<Box<dyn Plugin>> = Vec::new();

    for plugin in &config.panel.plugins {
        match plugin {
            noxl_config::PanelPlugin::Launcher => {
                plugins.push(Box::new(LauncherPlugin));
            }
            noxl_config::PanelPlugin::Taskbar => {
                plugins.push(Box::new(TaskbarPlugin));
            }
            noxl_config::PanelPlugin::Workspaces => {
                plugins.push(Box::new(WorkspacesPlugin));
            }
            noxl_config::PanelPlugin::Tray => {
                plugins.push(Box::new(TrayPlugin));
            }
            noxl_config::PanelPlugin::StatusIcons => {
                plugins.push(Box::new(StatusIconsPlugin));
            }
            noxl_config::PanelPlugin::Clock => {
                plugins.push(Box::new(ClockPlugin));
            }
            noxl_config::PanelPlugin::Notifications => {
                plugins.push(Box::new(NotificationsPlugin));
            }
        }
    }

    Ok(PluginSet { plugins })
}

impl PluginSet {
    pub fn render(&self, config: &noxl_config::NoxlConfig) -> anyhow::Result<()> {
        for plugin in &self.plugins {
            plugin.render(config)?;
        }
        Ok(())
    }
}

/// Application launcher button.
struct LauncherPlugin;
impl Plugin for LauncherPlugin {
    fn render(&self, _config: &noxl_config::NoxlConfig) -> anyhow::Result<()> {
        // Draw the NoxL launcher icon. On click, send LauncherToggle to the
        // compositor via IPC.
        Ok(())
    }
}

/// Running/pinned application list.
struct TaskbarPlugin;
impl Plugin for TaskbarPlugin {
    fn render(&self, config: &noxl_config::NoxlConfig) -> anyhow::Result<()> {
        // For each pinned app, draw its icon. For each running app on the
        // active workspace, draw a window button.
        let _ = &config.panel.pinned_apps;
        Ok(())
    }
}

/// Workspace switcher.
struct WorkspacesPlugin;
impl Plugin for WorkspacesPlugin {
    fn render(&self, config: &noxl_config::NoxlConfig) -> anyhow::Result<()> {
        // Draw one button per workspace (config.workspaces.count).
        let _ = config.workspaces.count;
        Ok(())
    }
}

/// System tray (StatusNotifierItem).
struct TrayPlugin;
impl Plugin for TrayPlugin {
    fn render(&self, _config: &noxl_config::NoxlConfig) -> anyhow::Result<()> {
        // Connect to the SNI host on D-Bus and render tray icons.
        Ok(())
    }
}

/// Network/audio/battery status icons.
struct StatusIconsPlugin;
impl Plugin for StatusIconsPlugin {
    fn render(&self, _config: &noxl_config::NoxlConfig) -> anyhow::Result<()> {
        // Network: NetworkManager via D-Bus.
        // Audio: PipeWire via D-Bus.
        // Battery: upower via D-Bus.
        Ok(())
    }
}

/// Clock and calendar popup.
struct ClockPlugin;
impl Plugin for ClockPlugin {
    fn render(&self, _config: &noxl_config::NoxlConfig) -> anyhow::Result<()> {
        // Draw the current time (HH:MM). On click, open the calendar popup.
        Ok(())
    }
}

/// Notification indicator.
struct NotificationsPlugin;
impl Plugin for NotificationsPlugin {
    fn render(&self, _config: &noxl_config::NoxlConfig) -> anyhow::Result<()> {
        // Show a badge with the unread notification count.
        Ok(())
    }
}
