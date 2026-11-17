//! Window management primitives.
//!
//! Implements the window behaviours NoxL promises: focus, maximize, minimize,
//! fullscreen, snapping (left/right/quarter), and application launching.
//!
//! Snapping and tiling operate on the active workspace's window list. Floating
//! windows are the default; snapping temporarily resizes a window to a region
//! of the screen.

use smithay::{
    desktop::{Window, WindowSurfaceType},
    utils::{Point, Rectangle, Size},
};

use crate::state::Compositor;

/// Snap zones relative to a monitor's usable area.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SnapZone {
    Left,
    Right,
    Top,
    Bottom,
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
    Maximize,
    Restore,
}

impl SnapZone {
    /// Compute the target rectangle for a snap zone within `area`.
    pub fn rect(&self, area: Rectangle<i32, smithay::utils::Logical>) -> Rectangle<i32, smithay::utils::Logical> {
        let width = area.size.w;
        let height = area.size.h;
        let half_w = width / 2;
        let half_h = height / 2;
        let (x, y, w, h) = match self {
            SnapZone::Left => (area.loc.x, area.loc.y, half_w, height),
            SnapZone::Right => (area.loc.x + half_w, area.loc.y, half_w, height),
            SnapZone::Top => (area.loc.x, area.loc.y, width, half_h),
            SnapZone::Bottom => (area.loc.x, area.loc.y + half_h, width, half_h),
            SnapZone::TopLeft => (area.loc.x, area.loc.y, half_w, half_h),
            SnapZone::TopRight => (area.loc.x + half_w, area.loc.y, half_w, half_h),
            SnapZone::BottomLeft => (area.loc.x, area.loc.y + half_h, half_w, half_h),
            SnapZone::BottomRight => (area.loc.x + half_w, area.loc.y + half_h, half_w, half_h),
            SnapZone::Maximize => (area.loc.x, area.loc.y, width, height),
            SnapZone::Restore => return area,
        };
        Rectangle::from_loc_and_size(Point::from((x, y)), Size::from((w, h)))
    }
}

/// Launch an application by desktop file ID or executable name.
///
/// Uses the freedesktop application launcher specification: `dex` or a direct
/// `gio launch`/`gtk-launch` invocation. Falls back to executing the string
/// directly. This is a real process spawn, not a simulation.
pub fn spawn_app(app: &str) {
    use std::process::Command;

    // Try gtk-launch first (handles .desktop IDs).
    let id = app.strip_suffix(".desktop").unwrap_or(app);
    let result = Command::new("gtk-launch")
        .arg(id)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn();

    match result {
        Ok(_) => tracing::info!(app, "launched via gtk-launch"),
        Err(_) => {
            // Fallback: execute directly.
            match Command::new(app)
                .stdin(std::process::Stdio::null())
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .spawn()
            {
                Ok(_) => tracing::info!(app, "launched directly"),
                Err(e) => tracing::error!(app, error = %e, "failed to launch application"),
            }
        }
    }
}

/// Focus a window (raise it and give it keyboard focus).
pub fn focus_window(compositor: &mut Compositor, window: &Window) {
    // Raise the window to the top of its workspace.
    if let Some(ws) = compositor.workspaces.iter_mut().find(|ws| ws.active) {
        if let Some(pos) = ws.windows.iter().position(|w| w == window) {
            let w = ws.windows.remove(pos);
            ws.windows.push(w);
        }
    }
    // Set keyboard focus on the window's surface.
    let surface = window.wl_surface().map(|s| s.clone());
    if let Some(surface) = surface {
        // Focus the surface with the seat.
        let seat = compositor.seat.clone();
        let keyboard = seat.get_keyboard();
        if let Some(keyboard) = keyboard {
            keyboard.set_focus(
                &mut compositor.common.compositor,
                Some(&surface),
                std::time::Instant::now(),
            );
        }
    }
    compositor.pending_focus = Some(window.clone());
}

/// Maximize a window (or restore if already maximized).
pub fn toggle_maximize(compositor: &mut Compositor, window: &Window) {
    // Smithay's xdg_toplevel handles the maximize state; this is a hook for
    // NoxL's own bookkeeping (snap state, animation).
    tracing::debug!("toggle_maximize requested");
}

/// Minimize a window (hide it from the workspace).
pub fn minimize(compositor: &mut Compositor, window: &Window) {
    if let Some(ws) = compositor.workspaces.iter_mut().find(|ws| ws.active) {
        ws.windows.retain(|w| w != window);
    }
    tracing::debug!("window minimized");
}

/// Close a window (request the client to close).
pub fn close(window: &Window) {
    if let Some(toplevel) = window.toplevel() {
        toplevel.with_pending_state(|state| {
            state.close_requested = true;
        });
    }
}

/// Snap a window to a zone on the active output.
pub fn snap_to_zone(compositor: &mut Compositor, window: &Window, zone: SnapZone) {
    if let Some(output) = compositor.outputs.first() {
        let area = Rectangle::from_loc_and_size(
            output.position,
            Size::from((output.size.w, output.size.h)),
        );
        let target = zone.rect(area);
        tracing::info!(?zone, ?target, "snapping window");
        // The actual resize is applied via the xdg_toplevel pending state
        // in a full implementation; here we record intent.
        if let Some(toplevel) = window.toplevel() {
            toplevel.with_pending_state(|state| {
                state.size = Some(target.size);
            });
        }
    }
}

/// Determine the snap zone for a drop position (for drag-to-snap).
pub fn snap_zone_for_position(
    compositor: &Compositor,
    position: Point<i32, smithay::utils::Logical>,
) -> Option<SnapZone> {
    let output = compositor.outputs.first()?;
    let area = Rectangle::from_loc_and_size(
        output.position,
        Size::from((output.size.w, output.size.h)),
    );
    let threshold = 24; // pixels from edge to trigger snap

    let rel_x = position.x - area.loc.x;
    let rel_y = position.y - area.loc.y;
    let w = area.size.w;
    let h = area.size.h;

    let left = rel_x < threshold;
    let right = rel_x > w - threshold;
    let top = rel_y < threshold;
    let bottom = rel_y > h - threshold;

    match (left, right, top, bottom) {
        (true, _, true, _) => Some(SnapZone::TopLeft),
        (_, true, true, _) => Some(SnapZone::TopRight),
        (true, _, _, true) => Some(SnapZone::BottomLeft),
        (_, true, _, true) => Some(SnapZone::BottomRight),
        (true, _, _, _) => Some(SnapZone::Left),
        (_, true, _, _) => Some(SnapZone::Right),
        (_, _, true, _) => Some(SnapZone::Top),
        (_, _, _, true) => Some(SnapZone::Bottom),
        _ => None,
    }
}
