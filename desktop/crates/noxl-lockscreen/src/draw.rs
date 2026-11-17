//! Lock screen rendering: full-screen OBSIDIAN overlay.
//!
//! Creates a `zwlr_layer_surface_v1` on the Overlay layer covering the whole
//! screen and paints the OBSIDIAN lock screen background.

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

/// The lock screen state.
pub struct LockScreen {
    pub surface: wl_surface::WlSurface,
    pub layer_surface: zwlr_layer_surface_v1::ZwlrLayerSurfaceV1,
    pub config: noxl_config::NoxlConfig,
    pub user: String,
    pub shm: Option<wl_shm::WlShm>,
    pub qh: Option<QueueHandle<LockScreen>>,
    pub running: bool,
    pub unlocked: bool,
}

/// Run the lock screen.
pub fn run(config: noxl_config::NoxlConfig, user: String) -> Result<()> {
    let conn = Connection::connect_to_env().context("failed to connect to Wayland display")?;
    let (globals, mut event_queue) =
        registry_queue_init::<LockScreen>(&conn).context("failed to init registry queue")?;
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
        zwlr_layer_shell_v1::Layer::Overlay,
        "noxl-lockscreen".to_string(),
        &qh,
        (),
    );

    // Cover the whole screen and grab the keyboard exclusively.
    layer_surface.set_anchor(
        zwlr_layer_surface_v1::Anchor::Top
            | zwlr_layer_surface_v1::Anchor::Bottom
            | zwlr_layer_surface_v1::Anchor::Left
            | zwlr_layer_surface_v1::Anchor::Right,
    );
    layer_surface.set_exclusive_zone(-1);
    layer_surface.set_keyboard_interactivity(
        zwlr_layer_surface_v1::KeyboardInteractivity::Exclusive,
    );
    surface.commit();

    tracing::info!(%user, "lock screen surface created");

    let mut lock = LockScreen {
        surface,
        layer_surface,
        config,
        user,
        shm: Some(shm),
        qh: Some(qh.clone()),
        running: true,
        unlocked: false,
    };

    loop {
        event_queue
            .blocking_dispatch(&mut lock)
            .context("Wayland dispatch error")?;

        if !lock.running || lock.unlocked {
            break;
        }
    }

    if lock.unlocked {
        tracing::info!("session unlocked");
    }

    Ok(())
}

/// Draw the lock screen background.
fn draw_lock_screen(lock: &mut LockScreen, width: i32, height: i32) {
    let Some(shm) = &lock.shm else { return };
    let Some(qh) = &lock.qh else { return };
    let qh = qh.clone();

    let buffer = match ShmBuffer::new(shm, &qh, width, height) {
        Ok(b) => b,
        Err(e) => {
            tracing::error!(error = %e, "failed to create lock screen buffer");
            return;
        }
    };

    let mut painter = Painter::new(width, height);

    // OBSIDIAN primary background (near-black).
    let (r, g, b, a) = noxl_render::parse_hex(&lock.config.theme.colors.bg_primary);
    painter.fill(r, g, b, a);

    // A subtle accent line near the top.
    let (ar, ag, ab, aa) = noxl_render::parse_hex(&lock.config.theme.colors.accent_primary);
    painter.fill_rect(0, height / 3, width, 1, ar, ag, ab, aa);

    let mut buffer = buffer;
    buffer.data.copy_from_slice(&painter.data);
    buffer.attach(&lock.surface);

    tracing::info!(width, height, "lock screen drawn");
}

// --- Dispatch implementations ---

impl Dispatch<wl_registry::WlRegistry, GlobalListContents> for LockScreen {
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

impl Dispatch<wl_compositor::WlCompositor, ()> for LockScreen {
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

impl Dispatch<wl_surface::WlSurface, ()> for LockScreen {
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

impl Dispatch<wl_region::WlRegion, ()> for LockScreen {
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

impl Dispatch<wl_output::WlOutput, ()> for LockScreen {
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

impl Dispatch<wl_shm::WlShm, ()> for LockScreen {
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

impl Dispatch<wl_shm_pool::WlShmPool, ()> for LockScreen {
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

impl Dispatch<wl_buffer::WlBuffer, ()> for LockScreen {
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

impl Dispatch<zwlr_layer_shell_v1::ZwlrLayerShellV1, ()> for LockScreen {
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

impl Dispatch<zwlr_layer_surface_v1::ZwlrLayerSurfaceV1, ()> for LockScreen {
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
                let w = if width > 0 { width as i32 } else { 1920 };
                let h = if height > 0 { height as i32 } else { 1080 };
                draw_lock_screen(app, w, h);
            }
            zwlr_layer_surface_v1::Event::Closed => {
                app.running = false;
            }
            _ => {}
        }
    }
}
