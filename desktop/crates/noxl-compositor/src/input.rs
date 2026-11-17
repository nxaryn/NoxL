//! Input handling: keyboard, pointer, and keybindings.
//!
//! Translates raw key events into NoxL actions using the user's keybindings
//! from the config.

use smithay::{
    backend::input::KeyState,
    input::keyboard::{FilterResult, KeyboardHandle, ModifiersState},
};

use crate::{state::Noxl, window};

/// Handle a keyboard event through the seat's keyboard.
///
/// This is called from the winit event loop. The keyboard handle processes
/// the event and calls the closure for each key press, where we can inspect
/// the modifiers and dispatch keybindings.
pub fn handle_keyboard_event(
    compositor: &mut Noxl,
    keyboard: &KeyboardHandle<Noxl>,
    keycode: smithay::input::keyboard::Keycode,
    state: KeyState,
) {
    // Clone the keyboard handle to avoid borrowing issues.
    let keyboard = keyboard.clone();

    // Use the input method with a filter that captures modifiers.
    let combo = std::cell::RefCell::new(None);
    let combo_ref = &combo;

    keyboard.input::<(), _>(
        compositor,
        keycode,
        state,
        0.into(),
        0,
        |_, modifiers, _| {
            if state == KeyState::Pressed {
                let combo_str = format_combo(keycode, modifiers);
                *combo_ref.borrow_mut() = Some(combo_str);
            }
            FilterResult::Forward
        },
    );

    // Only act on key press.
    if state != KeyState::Pressed {
        return;
    }

    if let Some(combo_str) = combo.into_inner() {
        dispatch_keybinding(compositor, &combo_str);
    }
}

/// Format a keycode + modifiers into a combo string like "Super+Q".
fn format_combo(keycode: smithay::input::keyboard::Keycode, modifiers: &ModifiersState) -> String {
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
    let name = keycode_name(keycode);
    parts.push(&name);
    parts.join("+")
}

/// Map an evdev keycode to a readable name.
fn keycode_name(keycode: smithay::input::keyboard::Keycode) -> String {
    let code: u32 = keycode.into();
    match code {
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
        99 => "Print".into(),
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
        _ => format!("Key{:?}", keycode),
    }
}

/// Dispatch a keybinding combo to the appropriate action.
fn dispatch_keybinding(compositor: &mut Noxl, combo: &str) {
    let bindings = &compositor.config.keybindings.bindings;

    let action = bindings
        .iter()
        .find(|(_, bound)| bound.as_str() == combo)
        .map(|(action, _)| action.clone());

    let Some(action) = action else {
        return;
    };

    tracing::debug!(combo, action, "keybinding matched");

    match action.as_str() {
        "launcher.toggle" => {
            tracing::debug!("launcher toggle (sent to panel via IPC)");
        }
        "window.close" => {
            let window = compositor.workspaces.iter().find(|ws| ws.active)
                .and_then(|ws| ws.windows.last().cloned());
            if let Some(window) = window {
                window::close(&window);
            }
        }
        "window.maximize" => {
            let window = compositor.workspaces.iter().find(|ws| ws.active)
                .and_then(|ws| ws.windows.last().cloned());
            if let Some(window) = window {
                window::toggle_maximize(compositor, &window);
            }
        }
        "window.minimize" => {
            let window = compositor.workspaces.iter().find(|ws| ws.active)
                .and_then(|ws| ws.windows.last().cloned());
            if let Some(window) = window {
                window::minimize(compositor, &window);
            }
        }
        "window.snap_left" => {
            let window = compositor.workspaces.iter().find(|ws| ws.active)
                .and_then(|ws| ws.windows.last().cloned());
            if let Some(window) = window {
                window::snap_to_zone(compositor, &window, window::SnapZone::Left);
            }
        }
        "window.snap_right" => {
            let window = compositor.workspaces.iter().find(|ws| ws.active)
                .and_then(|ws| ws.windows.last().cloned());
            if let Some(window) = window {
                window::snap_to_zone(compositor, &window, window::SnapZone::Right);
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
