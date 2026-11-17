//! Rendering.
//!
//! Draws a frame: the OBSIDIAN background color, then windows on the active
//! workspace. Uses Smithay's GLES renderer.

use smithay::{
    backend::{
        renderer::{
            element::{
                surface::{render_elements_from_surface_tree, WaylandSurfaceRenderElement},
                Kind,
            },
            gles::GlesRenderer,
            utils::draw_render_elements,
            Color32F, Frame, Renderer,
        },
        winit::WinitGraphicsBackend,
    },
    utils::{Rectangle, Transform},
};

use crate::state::Noxl;

/// Render a single frame.
pub fn render(
    compositor: &mut Noxl,
    backend: &mut WinitGraphicsBackend<GlesRenderer>,
    _winit: &mut smithay::backend::winit::WinitEventLoop,
    size: smithay::utils::Size<i32, smithay::utils::Physical>,
) -> anyhow::Result<()> {
    let damage = Rectangle::from_size(size);

    let (renderer, mut framebuffer) = backend.bind()?;
    let elements = compositor
        .xdg_shell_state
        .toplevel_surfaces()
        .iter()
        .flat_map(|surface| {
            render_elements_from_surface_tree(
                renderer,
                surface.wl_surface(),
                (0, 0),
                1.0,
                1.0,
                Kind::Unspecified,
            )
        })
        .collect::<Vec<WaylandSurfaceRenderElement<GlesRenderer>>>();

    let mut frame = renderer.render(&mut framebuffer, size, Transform::Flipped180)?;
    frame.clear(background_color(compositor), &[damage])?;
    draw_render_elements(&mut frame, 1.0, &elements, &[damage])?;
    let _ = frame.finish()?;

    Ok(())
}

/// The OBSIDIAN background color as RGBA floats (from the theme).
pub fn background_color(compositor: &Noxl) -> Color32F {
    noxl_config::Colors::parse_hex(&compositor.config.theme.colors.bg_primary)
        .map(|(r, g, b, a)| Color32F::new(r, g, b, a))
        .unwrap_or(Color32F::new(0.035, 0.035, 0.05, 1.0))
}
