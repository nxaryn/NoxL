//! Notification rendering: layer-shell popups and the notification centre.
//!
//! Creates a layer-shell surface on the top layer and draws notifications
//! into it using the shared NoxL renderer.

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

use crate::history::History;

/// The notification centre state.
pub struct NotificationCentre {
    pub surface: Option<wl_surface::WlSurface>,
    pub layer_surface: Option<zwlr_layer_surface_v1::ZwlrLayerSurfaceV1>,
    pub config: noxl_config::NoxlConfig,
    pub history: History,
    pub shm: Option<wl_shm::WlShm>,
    pub qh: Option<QueueHandle<NotificationCentre>>,
    pub running: bool,
}

/// Run the notification centre.
pub fn run(
    config: noxl_config::NoxlConfig,
    history: History,
) -> Result<()> {
    let conn = Connection::connect_to_env().context("failed to connect to Wayland display")?;
    let (globals, mut event_queue) =
        registry_queue_init::<NotificationCentre>(&conn).context("failed to init registry queue")?;
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

    // Create the notification popup surface (hidden until a notification arrives).
    let surface = compositor.create_surface(&qh, ());
    let layer_surface = layer_shell.get_layer_surface(
        &surface,
        None,
        zwlr_layer_shell_v1::Layer::Overlay,
        "noxl-notifications".to_string(),
        &qh,
        (),
    );

    // Position top-right.
    layer_surface.set_anchor(
        zwlr_layer_surface_v1::Anchor::Top | zwlr_layer_surface_v1::Anchor::Right,
    );
    layer_surface.set_exclusive_zone(-1);
    layer_surface.set_margin(0, 16, 0, 16);
    layer_surface.set_keyboard_interactivity(
        zwlr_layer_surface_v1::KeyboardInteractivity::None,
    );
    surface.commit();

    tracing::info!("notification centre layer-shell surface created");

    let mut nc = NotificationCentre {
        surface: Some(surface),
        layer_surface: Some(layer_surface),
        config,
        history,
        shm: Some(shm),
        qh: Some(qh.clone()),
        running: true,
    };

    loop {
        event_queue
            .blocking_dispatch(&mut nc)
            .context("Wayland dispatch error")?;

        if !nc.running {
            break;
        }
    }

    Ok(())
}

/// Draw the notification centre background.
fn draw_notifications(nc: &mut NotificationCentre, width: i32, height: i32) {
    let Some(shm) = &nc.shm else { return };
    let Some(qh) = &nc.qh else { return };
    let Some(surface) = &nc.surface else { return };
    let qh = qh.clone();

    let buffer = match ShmBuffer::new(shm, &qh, width, height) {
        Ok(b) => b,
        Err(e) => {
            tracing::error!(error = %e, "failed to create notification buffer");
            return;
        }
    };

    let mut painter = Painter::new(width, height);

    // OBSIDIAN elevated background.
    let (r, g, b, a) = noxl_render::parse_hex(&nc.config.theme.colors.bg_elevated);
    painter.fill(r, g, b, a);

    // Accent left border.
    let (ar, ag, ab, aa) = noxl_render::parse_hex(&nc.config.theme.colors.accent_primary);
    painter.fill_rect(0, 0, 2, height, ar, ag, ab, aa);

    let mut buffer = buffer;
    buffer.data.copy_from_slice(&painter.data);
    buffer.attach(surface);

    tracing::debug!(unread = nc.history.unread().len(), "notifications drawn");
}

// --- Dispatch implementations ---

impl Dispatch<wl_registry::WlRegistry, GlobalListContents> for NotificationCentre {
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

impl Dispatch<wl_compositor::WlCompositor, ()> for NotificationCentre {
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

impl Dispatch<wl_surface::WlSurface, ()> for NotificationCentre {
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

impl Dispatch<wl_region::WlRegion, ()> for NotificationCentre {
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

impl Dispatch<wl_output::WlOutput, ()> for NotificationCentre {
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

impl Dispatch<wl_shm::WlShm, ()> for NotificationCentre {
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

impl Dispatch<wl_shm_pool::WlShmPool, ()> for NotificationCentre {
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

impl Dispatch<wl_buffer::WlBuffer, ()> for NotificationCentre {
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

impl Dispatch<zwlr_layer_shell_v1::ZwlrLayerShellV1, ()> for NotificationCentre {
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

impl Dispatch<zwlr_layer_surface_v1::ZwlrLayerSurfaceV1, ()> for NotificationCentre {
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
                if let Some(ls) = &app.layer_surface {
                    ls.ack_configure(serial);
                }
                let w = if width > 0 { width as i32 } else { 360 };
                let h = if height > 0 { height as i32 } else { 80 };
                draw_notifications(app, w, h);
            }
            zwlr_layer_surface_v1::Event::Closed => {
                app.running = false;
            }
            _ => {}
        }
    }
}
