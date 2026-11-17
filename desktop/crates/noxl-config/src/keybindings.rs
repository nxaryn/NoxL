//! Keybinding configuration.
//!
//! Each action maps to a key combo string like "Super+Return". The compositor
//! parses these into its key-state machine. Users can rebind everything.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Keyboard/mouse shortcuts.
///
/// The map key is the action name; the value is the key combo. Unknown actions
/// are ignored, so new actions can be added without breaking old configs.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Keybindings {
    /// action -> key combo (e.g. "launcher.toggle" -> "Super")
    pub bindings: HashMap<String, String>,
}

impl Default for Keybindings {
    fn default() -> Self {
        let mut bindings = HashMap::new();
        // Launcher
        bindings.insert("launcher.toggle".into(), "Super".into());
        // Window management
        bindings.insert("window.close".into(), "Super+Q".into());
        bindings.insert("window.maximize".into(), "Super+Up".into());
        bindings.insert("window.minimize".into(), "Super+Down".into());
        bindings.insert("window.fullscreen".into(), "Super+F".into());
        bindings.insert("window.snap_left".into(), "Super+Left".into());
        bindings.insert("window.snap_right".into(), "Super+Right".into());
        // Workspaces
        bindings.insert("workspace.prev".into(), "Ctrl+Alt+Left".into());
        bindings.insert("workspace.next".into(), "Ctrl+Alt+Right".into());
        for i in 1..=9 {
            bindings.insert(format!("workspace.focus_{}", i), format!("Super+{}", i));
        }
        // Session
        bindings.insert("session.lock".into(), "Super+L".into());
        bindings.insert("session.logout".into(), "Ctrl+Alt+Delete".into());
        // Screenshot
        bindings.insert("screenshot.region".into(), "Print".into());
        bindings.insert("screenshot.full".into(), "Shift+Print".into());
        // Terminal
        bindings.insert("terminal.spawn".into(), "Super+Return".into());
        Self { bindings }
    }
}

impl Keybindings {
    /// Look up the combo for an action, if bound.
    pub fn get(&self, action: &str) -> Option<&str> {
        self.bindings.get(action).map(|s| s.as_str())
    }

    /// Set or replace a binding.
    pub fn set(&mut self, action: impl Into<String>, combo: impl Into<String>) {
        self.bindings.insert(action.into(), combo.into());
    }
}
