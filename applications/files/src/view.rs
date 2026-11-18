//! Files view: layer-shell surface that displays the file browser.

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

use crate::browser::Browser;

/// The files view state.
pub struct FilesView {
    pub surface: wl_surface::WlSurface,
    pub layer_surface: zwlr_layer_surface_v1::ZwlrLayerSurfaceV1,
    pub config: noxl_config::NoxlConfig,
    pub shm: Option<wl_shm::WlShm>,
    pub qh: Option<QueueHandle<FilesView>>,
    pub running: bool,
    pub width: i32,
    pub height: i32,
}

/// Run the files view.
pub fn run(config: noxl_config::NoxlConfig, browser: &mut Browser) -> Result<()> {
    let conn = Connection::connect_to_env().context("failed to connect to Wayland display")?;
    let (globals, mut event_queue) =
        registry_queue_init::<FilesView>(&conn).context("failed to init registry queue")?;
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
        "noxl-files".to_string(),
        &qh,
        (),
    );

    layer_surface.set_size(0, 0);
    layer_surface.set_anchor(zwlr_layer_surface_v1::Anchor::Top | zwlr_layer_surface_v1::Anchor::Left);
    layer_surface.set_exclusive_zone(-1);
    layer_surface.set_keyboard_interactivity(
        zwlr_layer_surface_v1::KeyboardInteractivity::Exclusive,
    );
    surface.commit();

    tracing::info!("files layer-shell surface created");

    let mut view = FilesView {
        surface,
        layer_surface,
        config,
        shm: Some(shm),
        qh: Some(qh.clone()),
        running: true,
        width: 900,
        height: 600,
    };

    loop {
        event_queue
            .blocking_dispatch(&mut view)
            .context("Wayland dispatch error")?;

        draw_files(&mut view, browser);

        if !view.running {
            break;
        }
    }

    Ok(())
}

/// Draw the file browser into a buffer and attach it.
fn draw_files(view: &mut FilesView, browser: &Browser) {
    let Some(shm) = &view.shm else { return };
    let Some(qh) = &view.qh else { return };
    let qh = qh.clone();

    let w = view.width;
    let h = view.height;

    let buffer = match ShmBuffer::new(shm, &qh, w, h) {
        Ok(b) => b,
        Err(e) => {
            tracing::error!(error = %e, "failed to create files buffer");
            return;
        }
    };

    let mut painter = Painter::new(w, h);

    // OBSIDIAN background.
    let (r, g, b, a) = noxl_render::parse_hex(&view.config.theme.colors.bg_primary);
    painter.fill(r, g, b, a);

    // Top bar with the current path.
    let (br, bg, bb, ba) = noxl_render::parse_hex(&view.config.theme.colors.bg_secondary);
    painter.fill_rect(0, 0, w, 40, br, bg, bb, ba);

    // Accent line below the top bar.
    let (ar, ag, ab, aa) = noxl_render::parse_hex(&view.config.theme.colors.accent_primary);
    painter.fill_rect(0, 40, w, 1, ar, ag, ab, aa);

    // Draw file entries.
    let row_h = 28;
    let start_y = 50;
    let (tr, tg, tb, ta) = noxl_render::parse_hex(&view.config.theme.colors.text_primary);
    let (sr, sg, sb, sa) = noxl_render::parse_hex(&view.config.theme.colors.text_secondary);

    for (i, entry) in browser.entries.iter().enumerate() {
        let y = start_y + (i as i32) * row_h;
        if y + row_h > h {
            break;
        }

        // Alternate row background.
        if i % 2 == 0 {
            painter.fill_rect(0, y, w, row_h, br, bg, bb, ba);
        }

        // Directory icon (a filled square) or file icon (a smaller square).
        let (ir, ig, ib, ia) = if entry.is_dir { (ar, ag, ab, aa) } else { (sr, sg, sb, sa) };
        let icon_size = if entry.is_dir { 16 } else { 12 };
        painter.fill_rect(12, y + 6, icon_size, icon_size, ir, ig, ib, ia);

        // File name (as a block — a real implementation uses a font rasterizer).
        let name_x = 36;
        for (j, _) in entry.name.chars().take(40).enumerate() {
            let cx = name_x + (j as i32) * 7;
            painter.fill_rect(cx, y + 8, 5, 12, tr, tg, tb, ta);
        }

        // Size (right-aligned, as blocks).
        let size_str = Browser::format_size(entry);
        let size_x = w - (size_str.len() as i32) * 7 - 20;
        for (j, _) in size_str.chars().enumerate() {
            let cx = size_x + (j as i32) * 7;
            painter.fill_rect(cx, y + 8, 5, 12, sr, sg, sb, sa);
        }
    }

    let mut buffer = buffer;
    buffer.data.copy_from_slice(&painter.data);
    buffer.attach(&view.surface);
}

// --- Dispatch implementations ---

impl Dispatch<wl_registry::WlRegistry, GlobalListContents> for FilesView {
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

impl Dispatch<wl_compositor::WlCompositor, ()> for FilesView {
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

impl Dispatch<wl_surface::WlSurface, ()> for FilesView {
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

impl Dispatch<wl_region::WlRegion, ()> for FilesView {
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

impl Dispatch<wl_output::WlOutput, ()> for FilesView {
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

impl Dispatch<wl_shm::WlShm, ()> for FilesView {
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

impl Dispatch<wl_shm_pool::WlShmPool, ()> for FilesView {
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

impl Dispatch<wl_buffer::WlBuffer, ()> for FilesView {
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

impl Dispatch<zwlr_layer_shell_v1::ZwlrLayerShellV1, ()> for FilesView {
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

impl Dispatch<zwlr_layer_surface_v1::ZwlrLayerSurfaceV1, ()> for FilesView {
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
