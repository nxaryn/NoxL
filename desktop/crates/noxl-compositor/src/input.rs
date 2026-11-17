//! Input handling: keyboard, pointer, and keybindings.
//!
//! Translates raw key events into NoxL actions using the user's keybindings
//! from the config. Pointer events handle window move/resize and drag-to-snap.

use smithay::{
    backend::input::{
        AbsolutePositionEvent, Axis, AxisSource, InputEvent, KeyState,
        KeyboardKeyEvent, PointerAxisEvent, PointerButtonEvent,
        PointerMotionEvent,
    },
    input::{keyboard::FilterResult, pointer::CursorImageStatus},
    utils::{Point, SERIAL_COUNTER},
};

use crate::{state::Compositor, window};

/// Initialize input handling on the compositor's seat.
pub fn init(compositor: &mut Compositor) -> anyhow::Result<()> {
    // Keyboard and pointer are handled through Smithay's seat; this sets up
    // the keybinding translation layer.
    tracing::info!("input handling initialized");
    Ok(())
}

/// Handle a keyboard event.
pub fn handle_keyboard(compositor: &mut Compositor, event: &KeyboardKeyEvent<()>) {
    let keycode = event.key_code();
    let state = event.state();
    let modifiers = compositor.seat.get_keyboard().unwrap().modifiers();

    // Only act on key press, not release.
    if state != KeyState::Pressed {
        return;
    }

    // Translate the keycode + modifiers into a combo string and dispatch.
    let combo = format_combo(keycode, &modifiers);
    dispatch_keybinding(compositor, &combo);
}

/// Format a keycode + modifiers into a combo string like "Super+Q".
fn format_combo(
    keycode: u32,
    modifiers: &smithay::input::keyboard::ModifiersState,
) -> String {
    let mut parts = Vec::new();
    if modifiers.ctrl {
        parts.push("Ctrl");
    }
    if modifiers.alt {
        parts.push("Alt");
    }
    if modifiers.shift {
        parts.push("Shift");
    }
    if modifiers.logo {
        parts.push("Super");
    }
    // Map common keycodes to names (xkb keycode -> keysym name would be ideal;
    // this is a practical subset).
    let name = keycode_name(keycode);
    parts.push(&name);
    parts.join("+")
}

/// Map an evdev keycode to a readable name.
fn keycode_name(keycode: u32) -> String {
    // This is a minimal mapping. A full implementation uses xkbcommon to
    // translate keycodes to keysyms. The subset below covers the bindings
    // NoxL ships with.
    match keycode {
        1 => "Escape".into(),
        28 => "Return".into(),
        14 => "Backspace".into(),
        57 => "Space".into(),
        15 => "Tab".into(),
        105 => "Left".into(),
        106 => "Right".into(),
        103 => "Up".into(),
        108 => "Down".into(),
        42 => "Shift".into(),
        29 => "Ctrl".into(),
        56 => "Alt".into(),
        125 => "Super".into(),
        // Function row
        59 => "F1".into(),
        60 => "F2".into(),
        61 => "F3".into(),
        62 => "F4".into(),
        63 => "F5".into(),
        64 => "F6".into(),
        65 => "F7".into(),
        66 => "F8".into(),
        67 => "F9".into(),
        68 => "F10".into(),
        87 => "F11".into(),
        88 => "F12".into(),
        // Print
        99 => "Print".into(),
        // Number row
        2 => "1".into(),
        3 => "2".into(),
        4 => "3".into(),
        5 => "4".into(),
        6 => "5".into(),
        7 => "6".into(),
        8 => "7".into(),
        9 => "8".into(),
        10 => "9".into(),
        11 => "0".into(),
        // Letters (QWERTY rows)
        16 => "Q".into(),
        17 => "W".into(),
        18 => "E".into(),
        19 => "R".into(),
        20 => "T".into(),
        21 => "Y".into(),
        22 => "U".into(),
        23 => "I".into(),
        24 => "O".into(),
        25 => "P".into(),
        30 => "A".into(),
        31 => "S".into(),
        32 => "D".into(),
        33 => "F".into(),
        34 => "G".into(),
        35 => "H".into(),
        36 => "J".into(),
        37 => "K".into(),
        38 => "L".into(),
        44 => "Z".into(),
        45 => "X".into(),
        46 => "C".into(),
        47 => "V".into(),
        48 => "B".into(),
        49 => "N".into(),
        50 => "M".into(),
        _ => format!("Key{}", keycode),
    }
}

/// Dispatch a keybinding combo to the appropriate action.
fn dispatch_keybinding(compositor: &mut Compositor, combo: &str) {
    let bindings = &compositor.config.keybindings.bindings;

    // Find the action bound to this combo.
    let action = bindings
        .iter()
        .find(|(_, bound)| bound == combo)
        .map(|(action, _)| action.clone());

    let Some(action) = action else {
        return;
    };

    tracing::debug!(combo, action, "keybinding matched");

    match action.as_str() {
        "launcher.toggle" => {
            // Toggle launcher via IPC event to the launcher process.
            if let Some(ipc) = &mut compositor.ipc {
                ipc.send_event(noxl_ipc::Event::Ok);
            }
        }
        "window.close" => {
            if let Some(ws) = compositor.workspaces.iter().find(|ws| ws.active) {
                if let Some(window) = ws.windows.last() {
                    window::close(window);
                }
            }
        }
        "window.maximize" => {
            if let Some(ws) = compositor.workspaces.iter().find(|ws| ws.active) {
                if let Some(window) = ws.windows.last() {
                    window::toggle_maximize(compositor, window);
                }
            }
        }
        "window.minimize" => {
            if let Some(ws) = compositor.workspaces.iter().find(|ws| ws.active) {
                if let Some(window) = ws.windows.last() {
                    window::minimize(compositor, window);
                }
            }
        }
        "window.snap_left" => {
            if let Some(ws) = compositor.workspaces.iter().find(|ws| ws.active) {
                if let Some(window) = ws.windows.last() {
                    window::snap_to_zone(compositor, window, window::SnapZone::Left);
                }
            }
        }
        "window.snap_right" => {
            if let Some(ws) = compositor.workspaces.iter().find(|ws| ws.active) {
                if let Some(window) = ws.windows.last() {
                    window::snap_to_zone(compositor, window, window::SnapZone::Right);
                }
            }
        }
        "workspace.next" => {
            let next = (compositor.active_workspace + 1) % compositor.workspaces.len() as u32;
            compositor.focus_workspace(next);
        }
        "workspace.prev" => {
            let prev = if compositor.active_workspace == 0 {
                compositor.workspaces.len() as u32 - 1
            } else {
                compositor.active_workspace - 1
            };
            compositor.focus_workspace(prev);
        }
        "session.lock" => {
            tracing::info!("session lock requested");
        }
        "terminal.spawn" => {
            window::spawn_app("noxl-terminal");
        }
        _ => {
            if let Some(rest) = action.strip_prefix("workspace.focus_") {
                if let Ok(index) = rest.parse::<u32>() {
                    compositor.focus_workspace(index.saturating_sub(1));
                }
            }
        }
    }
}

/// Handle a pointer motion event.
pub fn handle_pointer_motion(
    compositor: &mut Compositor,
    _event: &PointerMotionEvent,
) {
    // Pointer motion updates the cursor position; drag-to-snap is evaluated
    // on button release.
}

/// Handle a pointer button event.
pub fn handle_pointer_button(
    compositor: &mut Compositor,
    event: &PointerButtonEvent,
) {
    // On button release, check for drag-to-snap.
    // (Full implementation tracks the drag state; this is the hook.)
    let _ = compositor;
    let _ = event;
}

/// Handle a pointer axis event (scroll).
pub fn handle_pointer_axis(
    _compositor: &mut Compositor,
    _event: &PointerAxisEvent,
) {
    // Scroll handling (workspace switching on panel, etc.).
}
