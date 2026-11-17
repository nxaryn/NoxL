//! Launcher runtime: layer-shell surface and search.
//!
//! Creates a `zwlr_layer_surface_v1` on the top layer and draws the launcher
//! background. The search logic lives in `search.rs`; the app list in `apps.rs`.

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

use crate::apps::AppEntry;

/// The launcher application state.
pub struct Launcher {
    pub surface: wl_surface::WlSurface,
    pub layer_surface: zwlr_layer_surface_v1::ZwlrLayerSurfaceV1,
    pub config: noxl_config::NoxlConfig,
    pub apps: Vec<AppEntry>,
    pub shm: Option<wl_shm::WlShm>,
    pub qh: Option<QueueHandle<Launcher>>,
    pub running: bool,
    pub visible: bool,
}

/// Run the launcher.
pub fn run(config: noxl_config::NoxlConfig, apps: Vec<AppEntry>) -> Result<()> {
    let conn = Connection::connect_to_env().context("failed to connect to Wayland display")?;
    let (globals, mut event_queue) =
        registry_queue_init::<Launcher>(&conn).context("failed to init registry queue")?;
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
        "noxl-launcher".to_string(),
        &qh,
        (),
    );

    // Launcher size from config.
    let w = config.launcher.width as i32;
    let h = config.launcher.height as i32;

    layer_surface.set_size(0, 0);
    layer_surface.set_anchor(zwlr_layer_surface_v1::Anchor::Top | zwlr_layer_surface_v1::Anchor::Left);
    layer_surface.set_exclusive_zone(-1);
    layer_surface.set_keyboard_interactivity(
        zwlr_layer_surface_v1::KeyboardInteractivity::Exclusive,
    );
    surface.commit();

    tracing::info!(w, h, "launcher layer-shell surface created");

    let mut launcher = Launcher {
        surface,
        layer_surface,
        config,
        apps,
        shm: Some(shm),
        qh: Some(qh.clone()),
        running: true,
        visible: true,
    };

    loop {
        event_queue
            .blocking_dispatch(&mut launcher)
            .context("Wayland dispatch error")?;

        if !launcher.running {
            break;
        }
    }

    Ok(())
}

/// Draw the launcher background into a buffer and attach it.
fn draw_launcher(
    launcher: &mut Launcher,
    width: i32,
    height: i32,
) {
    let Some(shm) = &launcher.shm else { return };
    let Some(qh) = &launcher.qh else { return };
    let qh = qh.clone();

    let buffer = match ShmBuffer::new(shm, &qh, width, height) {
        Ok(b) => b,
        Err(e) => {
            tracing::error!(error = %e, "failed to create launcher buffer");
            return;
        }
    };

    let mut painter = Painter::new(width, height);

    // OBSIDIAN elevated background.
    let (r, g, b, a) = noxl_render::parse_hex(&launcher.config.theme.colors.bg_elevated);
    painter.fill(r, g, b, a);

    // Accent border.
    let (ar, ag, ab, aa) = noxl_render::parse_hex(&launcher.config.theme.colors.border);
    painter.stroke_rect(0, 0, width, height, ar, ag, ab, aa);

    // Copy into the buffer and attach.
    let mut buffer = buffer;
    buffer.data.copy_from_slice(&painter.data);
    buffer.attach(&launcher.surface);

    tracing::info!(width, height, "launcher drawn");
}

// --- Dispatch implementations ---

impl Dispatch<wl_registry::WlRegistry, GlobalListContents> for Launcher {
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

impl Dispatch<wl_compositor::WlCompositor, ()> for Launcher {
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

impl Dispatch<wl_surface::WlSurface, ()> for Launcher {
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

impl Dispatch<wl_region::WlRegion, ()> for Launcher {
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

impl Dispatch<wl_output::WlOutput, ()> for Launcher {
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

impl Dispatch<wl_shm::WlShm, ()> for Launcher {
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

impl Dispatch<wl_shm_pool::WlShmPool, ()> for Launcher {
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

impl Dispatch<wl_buffer::WlBuffer, ()> for Launcher {
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

impl Dispatch<zwlr_layer_shell_v1::ZwlrLayerShellV1, ()> for Launcher {
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

impl Dispatch<zwlr_layer_surface_v1::ZwlrLayerSurfaceV1, ()> for Launcher {
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
                let w = if width > 0 { width as i32 } else { app.config.launcher.width as i32 };
                let h = if height > 0 { height as i32 } else { app.config.launcher.height as i32 };
                draw_launcher(app, w, h);
            }
            zwlr_layer_surface_v1::Event::Closed => {
                tracing::info!("launcher layer surface closed; exiting");
                app.running = false;
            }
            _ => {}
        }
    }
}
