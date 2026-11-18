//! Software center view: layer-shell surface that displays the package manager.

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

use crate::packages;

/// The software center view state.
pub struct SoftwareView {
    pub surface: wl_surface::WlSurface,
    pub layer_surface: zwlr_layer_surface_v1::ZwlrLayerSurfaceV1,
    pub config: noxl_config::NoxlConfig,
    pub shm: Option<wl_shm::WlShm>,
    pub qh: Option<QueueHandle<SoftwareView>>,
    pub running: bool,
    pub width: i32,
    pub height: i32,
    pub packages: Vec<packages::Package>,
    pub search_query: String,
}

/// Run the software center view.
pub fn run(config: noxl_config::NoxlConfig) -> Result<()> {
    let conn = Connection::connect_to_env().context("failed to connect to Wayland display")?;
    let (globals, mut event_queue) =
        registry_queue_init::<SoftwareView>(&conn).context("failed to init registry queue")?;
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
        "noxl-software".to_string(),
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

    tracing::info!("software center layer-shell surface created");

    // Load installed packages.
    let pkgs = packages::list_installed().unwrap_or_default();
    tracing::info!(count = pkgs.len(), "loaded installed packages");

    let mut view = SoftwareView {
        surface,
        layer_surface,
        config,
        shm: Some(shm),
        qh: Some(qh.clone()),
        running: true,
        width: 800,
        height: 600,
        packages: pkgs,
        search_query: String::new(),
    };

    loop {
        event_queue
            .blocking_dispatch(&mut view)
            .context("Wayland dispatch error")?;

        draw_software(&mut view);

        if !view.running {
            break;
        }
    }

    Ok(())
}

/// Draw the software center into a buffer and attach it.
fn draw_software(view: &mut SoftwareView) {
    let Some(shm) = &view.shm else { return };
    let Some(qh) = &view.qh else { return };
    let qh = qh.clone();

    let w = view.width;
    let h = view.height;

    let buffer = match ShmBuffer::new(shm, &qh, w, h) {
        Ok(b) => b,
        Err(e) => {
            tracing::error!(error = %e, "failed to create software buffer");
            return;
        }
    };

    let mut painter = Painter::new(w, h);

    // OBSIDIAN background.
    let (r, g, b, a) = noxl_render::parse_hex(&view.config.theme.colors.bg_primary);
    painter.fill(r, g, b, a);

    // Title bar.
    let (br, bg, bb, ba) = noxl_render::parse_hex(&view.config.theme.colors.bg_secondary);
    painter.fill_rect(0, 0, w, 40, br, bg, bb, ba);

    // Accent line.
    let (ar, ag, ab, aa) = noxl_render::parse_hex(&view.config.theme.colors.accent_primary);
    painter.fill_rect(0, 40, w, 1, ar, ag, ab, aa);

    // Title.
    let (tr, tg, tb, ta) = noxl_render::parse_hex(&view.config.theme.colors.text_primary);
    let title = "NoxL Software";
    for (j, _) in title.chars().enumerate() {
        let cx = 20 + (j as i32) * 8;
        painter.fill_rect(cx, 12, 6, 14, tr, tg, tb, ta);
    }

    // Search bar.
    let (sr, sg, sb, sa) = noxl_render::parse_hex(&view.config.theme.colors.text_secondary);
    painter.fill_rect(20, 55, w - 40, 28, br, bg, bb, ba);
    painter.stroke_rect(20, 55, w - 40, 28, ar, ag, ab, aa);

    // Package list.
    let row_h = 32;
    let start_y = 95;
    for (i, pkg) in view.packages.iter().enumerate().take(15) {
        let y = start_y + (i as i32) * row_h;
        if y + row_h > h {
            break;
        }

        // Alternate row background.
        if i % 2 == 0 {
            painter.fill_rect(0, y, w, row_h, br, bg, bb, ba);
        }

        // Package name.
        for (j, _) in pkg.name.chars().take(30).enumerate() {
            let cx = 20 + (j as i32) * 8;
            painter.fill_rect(cx, y + 8, 6, 14, tr, tg, tb, ta);
        }

        // Version.
        for (j, _) in pkg.version.chars().take(20).enumerate() {
            let cx = w - 200 + (j as i32) * 8;
            painter.fill_rect(cx, y + 8, 6, 14, sr, sg, sb, sa);
        }

        // "Installed" badge.
        if pkg.installed {
            painter.fill_rect(w - 80, y + 8, 60, 16, ar, ag, ab, aa);
        }
    }

    let mut buffer = buffer;
    buffer.data.copy_from_slice(&painter.data);
    buffer.attach(&view.surface);
}

// --- Dispatch implementations ---

impl Dispatch<wl_registry::WlRegistry, GlobalListContents> for SoftwareView {
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

impl Dispatch<wl_compositor::WlCompositor, ()> for SoftwareView {
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

impl Dispatch<wl_surface::WlSurface, ()> for SoftwareView {
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

impl Dispatch<wl_region::WlRegion, ()> for SoftwareView {
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

impl Dispatch<wl_output::WlOutput, ()> for SoftwareView {
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

impl Dispatch<wl_shm::WlShm, ()> for SoftwareView {
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

impl Dispatch<wl_shm_pool::WlShmPool, ()> for SoftwareView {
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

impl Dispatch<wl_buffer::WlBuffer, ()> for SoftwareView {
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

impl Dispatch<zwlr_layer_shell_v1::ZwlrLayerShellV1, ()> for SoftwareView {
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

impl Dispatch<zwlr_layer_surface_v1::ZwlrLayerSurfaceV1, ()> for SoftwareView {
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
