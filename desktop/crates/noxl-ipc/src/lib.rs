//! NoxL inter-process communication.
//!
//! NoxL is a multi-process desktop: the compositor owns the Wayland display,
//! and the panel, launcher, notifications, and widgets are separate processes
//! that talk to it. This crate defines:
//!
//! 1. The **control socket** protocol — a line-delimited JSON protocol over a
//!    Unix domain socket (`$XDG_RUNTIME_DIR/noxl/control.sock`) that lets
//!    clients send commands to the compositor and receive events.
//! 2. Shared **event** and **command** types used across all NoxL processes.
//!
//! The protocol is intentionally simple, versioned, and human-debuggable (JSON),
//! so third-party tools and scripts can drive the desktop.

use serde::{Deserialize, Serialize};

/// Protocol version. Bump when the message schema changes incompatibly.
pub const PROTOCOL_VERSION: u32 = 1;

/// Path to the control socket.
pub fn control_socket_path() -> std::path::PathBuf {
    let runtime = std::env::var_os("XDG_RUNTIME_DIR")
        .map(std::path::PathBuf::from)
        .or_else(|| {
            // Fallback for environments where XDG_RUNTIME_DIR is not set
            // (e.g. running as root in a live environment).
            let uid = unsafe { libc::getuid() };
            let path = std::path::PathBuf::from(format!("/run/user/{}", uid));
            if path.exists() {
                Some(path)
            } else {
                Some(std::path::PathBuf::from("/tmp"))
            }
        })
        .unwrap_or_else(|| std::path::PathBuf::from("/tmp"));
    runtime.join("noxl").join("control.sock")
}

/// Find the Wayland display name.
///
/// Checks (in order):
/// 1. WAYLAND_DISPLAY env var
/// 2. The noxl-wayland.sock file written by the compositor
/// 3. Defaults to "wayland-1"
pub fn wayland_display_name() -> String {
    if let Ok(display) = std::env::var("WAYLAND_DISPLAY") {
        return display;
    }

    // Read the socket file written by the compositor.
    let runtime = std::env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| "/tmp".into());
    let socket_file = std::path::Path::new(&runtime).join("noxl-wayland.sock");
    if let Ok(contents) = std::fs::read_to_string(&socket_file) {
        let name = contents.trim().to_string();
        if !name.is_empty() {
            return name;
        }
    }

    "wayland-1".into()
}

/// Set up the Wayland environment for child processes.
///
/// Call this at the start of any NoxL client that needs to connect to the
    // compositor. It sets WAYLAND_DISPLAY and XDG_RUNTIME_DIR if missing.
pub fn setup_wayland_env() {
    if std::env::var_os("XDG_RUNTIME_DIR").is_none() {
        let uid = unsafe { libc::getuid() };
        let fallback = format!("/run/user/{}", uid);
        if std::path::Path::new(&fallback).exists() {
            std::env::set_var("XDG_RUNTIME_DIR", &fallback);
        } else {
            std::env::set_var("XDG_RUNTIME_DIR", "/tmp");
        }
    }

    if std::env::var_os("WAYLAND_DISPLAY").is_none() {
        let display = wayland_display_name();
        std::env::set_var("WAYLAND_DISPLAY", &display);
    }

    // Set up D-Bus session bus address if missing.
    if std::env::var_os("DBUS_SESSION_BUS_ADDRESS").is_none() {
        let runtime = std::env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| "/tmp".into());
        let bus_path = std::path::Path::new(&runtime).join("bus");
        if bus_path.exists() {
            std::env::set_var("DBUS_SESSION_BUS_ADDRESS", format!("unix:path={}", bus_path.display()));
        }
        // If no session bus is available, D-Bus connections will fail gracefully.
        // The notification centre will log the error and continue without D-Bus.
    }
}

/// A message sent from a client to the compositor.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "cmd", rename_all = "snake_case")]
pub enum Command {
    /// Ping — returns a Pong event.
    Ping,
    /// Open the application launcher.
    LauncherToggle,
    /// Lock the screen.
    Lock,
    /// Spawn an application by desktop file ID or executable.
    Spawn { app: String },
    /// Focus a workspace by index (0-based).
    WorkspaceFocus { index: u32 },
    /// Next / previous workspace.
    WorkspaceNext,
    WorkspacePrev,
    /// Take a screenshot (region/fullscreen/window).
    Screenshot { mode: ScreenshotMode },
    /// Get the current config as JSON.
    GetConfig,
    /// Replace the current config (full NoxlConfig JSON).
    SetConfig { config: serde_json::Value },
    /// Reload config from disk.
    ReloadConfig,
    /// Show a notification (used by the notification daemon test path).
    Notify {
        summary: String,
        body: String,
        app_icon: Option<String>,
    },
    /// Quit a client request (e.g. panel restart).
    Quit,
}

/// Screenshot capture mode.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScreenshotMode {
    Fullscreen,
    Region,
    Window,
}

/// An event sent from the compositor to a client.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event", rename_all = "snake_case")]
pub enum Event {
    /// Response to Ping.
    Pong { protocol: u32 },
    /// A workspace was changed.
    WorkspaceChanged { index: u32 },
    /// The config was reloaded.
    ConfigReloaded,
    /// The current config (response to GetConfig).
    Config(serde_json::Value),
    /// A notification was posted.
    NotificationPosted { summary: String, body: String },
    /// An error occurred.
    Error { message: String },
    /// Acknowledgement with no payload.
    Ok,
}

/// A framed message on the wire: newline-delimited JSON.
///
/// Each message is a single line of UTF-8 JSON terminated by `\n`. This keeps
/// the protocol trivially implementable in any language (bash, python, C, Rust).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Frame {
    pub version: u32,
    pub payload: serde_json::Value,
}

impl Frame {
    pub fn new(payload: serde_json::Value) -> Self {
        Self {
            version: PROTOCOL_VERSION,
            payload,
        }
    }

    pub fn encode(&self) -> String {
        // serde_json guarantees no embedded newlines in strings are emitted
        // unescaped, so a single line is safe.
        let mut s = serde_json::to_string(self).expect("frame serialization cannot fail");
        s.push('\n');
        s
    }

    pub fn decode(line: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(line)
    }
}
