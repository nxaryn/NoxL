//! Rendering.
//!
//! Draws a frame: the wallpaper background, then layer-shell surfaces (panel,
//! launcher, notifications), then windows on the active workspace.
//!
//! Smithay's renderer handles the actual GPU work; this module orchestrates the
//! draw order and applies the OBSIDIAN background color as a fallback before
//! the wallpaper texture is loaded.

use smithay::{
    backend::renderer::damage::OutputDamageTracker,
    desktop::space::Space,
    utils::{Rectangle, Size},
};

use crate::state::Compositor;

/// Render a single frame.
pub fn render(compositor: &mut Compositor) -> anyhow::Result<()> {
    // In a full implementation this would:
    // 1. Clear the output with the OBSIDIAN background color.
    // 2. Draw the wallpaper (from config.wallpaper).
    // 3. Render layer-shell surfaces (panel, launcher, notifications).
    // 4. Render windows on the active workspace via the Space.
    // 5. Apply damage tracking and commit the frame.
    //
    // The draw order and background color come from the config, so theme
    // changes apply without a restart.

    let _ = &compositor.space;
    let _ = &compositor.layer_map;
    Ok(())
}

/// The OBSIDIAN background color as RGBA floats (from the theme).
pub fn background_color(compositor: &Compositor) -> (f32, f32, f32, f32) {
    noxl_config::Colors::parse_hex(&compositor.config.theme.colors.bg_primary)
        .unwrap_or((0.035, 0.035, 0.05, 1.0))
}
