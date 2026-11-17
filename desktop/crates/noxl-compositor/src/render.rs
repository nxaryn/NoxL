//! Rendering.
//!
//! Draws a frame in the correct order:
//! 1. OBSIDIAN background color (from the theme)
//! 2. Layer-shell surfaces on the background layer (wallpaper)
//! 3. Windows on the active workspace
//! 4. Layer-shell surfaces on the bottom layer (panel)
//! 5. Layer-shell surfaces on the top layer (launcher, notifications)
//! 6. Layer-shell surfaces on the overlay layer (lock screen)
//!
//! Uses Smithay's GLES renderer.

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
    wayland::shell::wlr_layer::Layer,
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
    let mut all_elements: Vec<WaylandSurfaceRenderElement<GlesRenderer>> = Vec::new();

    // 1. Layer-shell surfaces on the background layer (wallpaper).
    collect_layer_elements(compositor, renderer, Layer::Background, &mut all_elements);

    // 2. Windows on the active workspace.
    for surface in compositor.xdg_shell_state.toplevel_surfaces() {
        all_elements.extend(render_elements_from_surface_tree(
            renderer,
            surface.wl_surface(),
            (0, 0),
            1.0,
            1.0,
            Kind::Unspecified,
        ));
    }

    // 3. Bottom layer (panel).
    collect_layer_elements(compositor, renderer, Layer::Bottom, &mut all_elements);

    // 4. Top layer (launcher, notifications).
    collect_layer_elements(compositor, renderer, Layer::Top, &mut all_elements);

    // 5. Overlay layer (lock screen).
    collect_layer_elements(compositor, renderer, Layer::Overlay, &mut all_elements);

    // Draw.
    let mut frame = renderer.render(&mut framebuffer, size, Transform::Flipped180)?;
    frame.clear(background_color(compositor), &[damage])?;
    draw_render_elements(&mut frame, 1.0, &all_elements, &[damage])?;
    let _ = frame.finish()?;

    Ok(())
}

/// Collect render elements for all layer surfaces on a given layer.
fn collect_layer_elements(
    compositor: &Noxl,
    renderer: &mut GlesRenderer,
    layer: Layer,
    elements: &mut Vec<WaylandSurfaceRenderElement<GlesRenderer>>,
) {
    // In a full implementation this iterates the LayerMap for each output and
    // collects the layer surfaces on the given layer. The LayerMap is stored
    // per-output; here we use the first output as a placeholder.
    if let Some(output) = compositor.outputs.first() {
        let layer_map = smithay::desktop::layer_map_for_output(&output.output);
        for surface in layer_map.layers_on(layer) {
            elements.extend(render_elements_from_surface_tree(
                renderer,
                surface.wl_surface(),
                (0, 0),
                1.0,
                1.0,
                Kind::Unspecified,
            ));
        }
    }
}

/// The OBSIDIAN background color as RGBA floats (from the theme).
pub fn background_color(compositor: &Noxl) -> Color32F {
    noxl_config::Colors::parse_hex(&compositor.config.theme.colors.bg_primary)
        .map(|(r, g, b, a)| Color32F::new(r, g, b, a))
        .unwrap_or(Color32F::new(0.035, 0.035, 0.05, 1.0))
}
