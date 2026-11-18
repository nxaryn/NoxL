//! Terminal view: layer-shell surface that displays the terminal.
//!
//! Creates a `zwlr_layer_surface_v1` and draws the terminal grid into a
//! `wl_shm` buffer using the shared NoxL renderer.

use anyhow::{Context, Result};

use wayland_client::{
    globals::{registry_queue_init, GlobalListContents},
    protocol::{
        wl_buffer, wl_compositor, wl_output, wl_registry, wl_region, wl_shm, wl_shm_pool,
        wl_surface,
    },
    Connection, Dispatch, Proxy, QueueHandle,
};

use wayland_protocols_wlr::layer_shell::v1::client::{
    zwlr_layer_shell_v1, zwlr_layer_surface_v1,
};

use noxl_render::{Painter, ShmBuffer};

use crate::term::Terminal;

/// The terminal view state.
pub struct TerminalView {
    pub surface: wl_surface::WlSurface,
    pub layer_surface: zwlr_layer_surface_v1::ZwlrLayerSurfaceV1,
    pub config: noxl_config::NoxlConfig,
    pub shm: Option<wl_shm::WlShm>,
    pub qh: Option<QueueHandle<TerminalView>>,
    pub running: bool,
    pub width: i32,
    pub height: i32,
}

/// Run the terminal view.
pub fn run(config: noxl_config::NoxlConfig, term: &mut Terminal) -> Result<()> {
    let conn = Connection::connect_to_env().context("failed to connect to Wayland display")?;
    let (globals, mut event_queue) =
        registry_queue_init::<TerminalView>(&conn).context("failed to init registry queue")?;
    let qh = event_queue.handle();

    let compositor: wl_compositor::WlCompositor = globals
        .bind(&qh, 1..=6, ())
        .context("wl_compositor not available")?;
    let shm: wl_shm::WlShm = globals
        .bind(&qh, 1..=1, ())
        .context("wl_shm not available")?;
    let layer_shell: zwlr_layer_shell_v1::ZwlrLayerShellV1 = globals
        .bind(&qh, 1..=4, ())
        .context("zwlr_layer_shell_v1 not available")?;
    let _output: wl_output::WlOutput = globals
        .bind(&qh, 1..=4, ())
        .context("wl_output not available")?;

    let surface = compositor.create_surface(&qh, ());

    let layer_surface = layer_shell.get_layer_surface(
        &surface,
        None,
        zwlr_layer_shell_v1::Layer::Top,
        "noxl-terminal".to_string(),
        &qh,
        (),
    );

    // Terminal window: centered, 800x500.
    layer_surface.set_size(0, 0);
    layer_surface.set_anchor(zwlr_layer_surface_v1::Anchor::Top | zwlr_layer_surface_v1::Anchor::Left);
    layer_surface.set_exclusive_zone(-1);
    layer_surface.set_keyboard_interactivity(
        zwlr_layer_surface_v1::KeyboardInteractivity::Exclusive,
    );
    surface.commit();

    tracing::info!("terminal layer-shell surface created");

    let mut view = TerminalView {
        surface,
        layer_surface,
        config,
        shm: Some(shm),
        qh: Some(qh.clone()),
        running: true,
        width: 800,
        height: 500,
    };

    // Event loop: read PTY output, draw, dispatch.
    loop {
        // Read any available shell output.
        let _ = term.read_output();

        // Dispatch Wayland events.
        event_queue
            .blocking_dispatch(&mut view)
            .context("Wayland dispatch error")?;

        // Draw the terminal.
        draw_terminal(&mut view, term);

        if !view.running || term.has_exited() {
            break;
        }
    }

    Ok(())
}

/// Draw the terminal grid into a buffer and attach it.
fn draw_terminal(view: &mut TerminalView, term: &Terminal) {
    let Some(shm) = &view.shm else { return };
    let Some(qh) = &view.qh else { return };
    let qh = qh.clone();

    let w = view.width;
    let h = view.height;

    let buffer = match ShmBuffer::new(shm, &qh, w, h) {
        Ok(b) => b,
        Err(e) => {
            tracing::error!(error = %e, "failed to create terminal buffer");
            return;
        }
    };

    let mut painter = Painter::new(w, h);

    // OBSIDIAN terminal background.
    let (r, g, b, a) = noxl_render::parse_hex(&view.config.theme.colors.bg_primary);
    painter.fill(r, g, b, a);

    // Draw the character grid.
    // Each character is drawn as a filled rectangle (a simple block font).
    let char_w = 8;
    let char_h = 16;
    let (tr, tg, tb, ta) = noxl_render::parse_hex(&view.config.theme.colors.text_primary);

    for (y, row) in term.screen.iter().enumerate() {
        for (x, ch) in row.iter().enumerate() {
            if *ch != ' ' {
                let px = (x as i32) * char_w;
                let py = (y as i32) * char_h;
                // Draw a simple block for each character.
                // A real implementation would use a font rasterizer.
                painter.fill_rect(px, py, char_w - 1, char_h - 1, tr, tg, tb, ta);
            }
        }
    }

    // Draw the cursor.
    let (cx, cy) = term.cursor;
    let cx_px = (cx as i32) * char_w;
    let cy_px = (cy as i32) * char_h;
    let (ar, ag, ab, aa) = noxl_render::parse_hex(&view.config.theme.colors.accent_primary);
    painter.fill_rect(cx_px, cy_px, char_w - 1, char_h - 1, ar, ag, ab, aa);

    let mut buffer = buffer;
    buffer.data.copy_from_slice(&painter.data);
    buffer.attach(&view.surface);
}

// --- Dispatch implementations ---

impl Dispatch<wl_registry::WlRegistry, GlobalListContents> for TerminalView {
    fn event(
        _: &mut Self,
        _: &wl_registry::WlRegistry,
        _: wl_registry::Event,
        _: &GlobalListContents,
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<wl_compositor::WlCompositor, ()> for TerminalView {
    fn event(
        _: &mut Self,
        _: &wl_compositor::WlCompositor,
        _: wl_compositor::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<wl_surface::WlSurface, ()> for TerminalView {
    fn event(
        _: &mut Self,
        _: &wl_surface::WlSurface,
        _: wl_surface::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<wl_region::WlRegion, ()> for TerminalView {
    fn event(
        _: &mut Self,
        _: &wl_region::WlRegion,
        _: wl_region::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<wl_output::WlOutput, ()> for TerminalView {
    fn event(
        _: &mut Self,
        _: &wl_output::WlOutput,
        _: wl_output::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<wl_shm::WlShm, ()> for TerminalView {
    fn event(
        _: &mut Self,
        _: &wl_shm::WlShm,
        _: wl_shm::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<wl_shm_pool::WlShmPool, ()> for TerminalView {
    fn event(
        _: &mut Self,
        _: &wl_shm_pool::WlShmPool,
        _: wl_shm_pool::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<wl_buffer::WlBuffer, ()> for TerminalView {
    fn event(
        _: &mut Self,
        _: &wl_buffer::WlBuffer,
        _: wl_buffer::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<zwlr_layer_shell_v1::ZwlrLayerShellV1, ()> for TerminalView {
    fn event(
        _: &mut Self,
        _: &zwlr_layer_shell_v1::ZwlrLayerShellV1,
        _: zwlr_layer_shell_v1::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<zwlr_layer_surface_v1::ZwlrLayerSurfaceV1, ()> for TerminalView {
    fn event(
        app: &mut Self,
        _: &zwlr_layer_surface_v1::ZwlrLayerSurfaceV1,
        event: zwlr_layer_surface_v1::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        match event {
            zwlr_layer_surface_v1::Event::Configure { serial, width, height } => {
                app.layer_surface.ack_configure(serial);
                if width > 0 && height > 0 {
                    app.width = width as i32;
                    app.height = height as i32;
                }
            }
            zwlr_layer_surface_v1::Event::Closed => {
                app.running = false;
            }
            _ => {}
        }
    }
}
