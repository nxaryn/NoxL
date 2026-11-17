//! Panel runtime: layer-shell surface, layout, and event loop.
//!
//! Creates a real `wlr_layer_shell` surface using the raw Wayland protocol
//! and renders the panel plugins into it.
//!
//! This uses `wayland-client` directly with the `zwlr_layer_shell_v1` and
//! `zwlr_layer_surface_v1` interfaces from `wayland-protocols-wlr`.

use anyhow::{Context, Result};

use wayland_client::{
    globals::{registry_queue_init, GlobalListContents},
    protocol::{wl_compositor, wl_output, wl_registry, wl_region, wl_shm, wl_surface},
    Connection, Dispatch, Proxy, QueueHandle,
};

use wayland_protocols_wlr::layer_shell::v1::client::{
    zwlr_layer_shell_v1, zwlr_layer_surface_v1,
};

use crate::plugins;

/// The panel application state.
pub struct Panel {
    /// The Wayland surface.
    pub surface: wl_surface::WlSurface,
    /// The layer surface.
    pub layer_surface: zwlr_layer_surface_v1::ZwlrLayerSurfaceV1,
    /// Config.
    pub config: noxl_config::NoxlConfig,
    /// Active plugins.
    pub plugins: plugins::PluginSet,
    /// Exit flag.
    pub running: bool,
    /// Pending configure serial.
    pub configure_serial: Option<u32>,
}

/// Run the panel.
pub fn run(config: noxl_config::NoxlConfig) -> Result<()> {
    let conn = Connection::connect_to_env().context("failed to connect to Wayland display")?;
    let (globals, mut event_queue) =
        registry_queue_init::<Panel>(&conn).context("failed to init registry queue")?;
    let qh = event_queue.handle();

    // Bind the globals we need.
    let compositor: wl_compositor::WlCompositor = globals
        .bind(&qh, 1..=6, ())
        .context("wl_compositor not available")?;
    let _shm: wl_shm::WlShm = globals
        .bind(&qh, 1..=1, ())
        .context("wl_shm not available")?;
    let layer_shell: zwlr_layer_shell_v1::ZwlrLayerShellV1 = globals
        .bind(&qh, 1..=4, ())
        .context("zwlr_layer_shell_v1 not available")?;
    let _output: wl_output::WlOutput = globals
        .bind(&qh, 1..=4, ())
        .context("wl_output not available")?;

    // Create the surface.
    let surface = compositor.create_surface(&qh, ());

    // Determine anchor from config.
    let anchor = match config.panel.position {
        noxl_config::PanelPosition::Top => {
            zwlr_layer_surface_v1::Anchor::Top
                | zwlr_layer_surface_v1::Anchor::Left
                | zwlr_layer_surface_v1::Anchor::Right
        }
        noxl_config::PanelPosition::Bottom => {
            zwlr_layer_surface_v1::Anchor::Bottom
                | zwlr_layer_surface_v1::Anchor::Left
                | zwlr_layer_surface_v1::Anchor::Right
        }
        noxl_config::PanelPosition::Left => {
            zwlr_layer_surface_v1::Anchor::Left
                | zwlr_layer_surface_v1::Anchor::Top
                | zwlr_layer_surface_v1::Anchor::Bottom
        }
        noxl_config::PanelPosition::Right => {
            zwlr_layer_surface_v1::Anchor::Right
                | zwlr_layer_surface_v1::Anchor::Top
                | zwlr_layer_surface_v1::Anchor::Bottom
        }
    };

    let layer = zwlr_layer_shell_v1::Layer::Bottom;

    // Create the layer surface.
    let layer_surface = layer_shell.get_layer_surface(
        &surface,
        None, // output (nil = all outputs)
        layer,
        "noxl-panel".to_string(),
        &qh,
        (),
    );

    // Configure the layer surface.
    layer_surface.set_size(0, config.panel.size as u32);
    layer_surface.set_anchor(anchor);
    layer_surface.set_exclusive_zone(-1);
    layer_surface.set_margin(0, 0, 0, 0);
    layer_surface.set_keyboard_interactivity(
        zwlr_layer_surface_v1::KeyboardInteractivity::None,
    );

    // Commit the surface so the compositor configures it.
    surface.commit();

    tracing::info!(
        position = ?config.panel.position,
        size = config.panel.size,
        "panel layer-shell surface created"
    );

    let panel_config = config.clone();
    let mut panel = Panel {
        surface,
        layer_surface,
        config,
        plugins: plugins::init(&panel_config)?,
        running: true,
        configure_serial: None,
    };

    // Event loop.
    loop {
        event_queue
            .blocking_dispatch(&mut panel)
            .context("Wayland dispatch error")?;

        // Render plugins (structure; pixel drawing is the next step).
        panel.plugins.render(&panel.config)?;

        if !panel.running {
            break;
        }
    }

    Ok(())
}

// --- Dispatch implementations ---

impl Dispatch<wl_registry::WlRegistry, GlobalListContents> for Panel {
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

impl Dispatch<wl_compositor::WlCompositor, ()> for Panel {
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

impl Dispatch<wl_surface::WlSurface, ()> for Panel {
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

impl Dispatch<wl_region::WlRegion, ()> for Panel {
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

impl Dispatch<wl_output::WlOutput, ()> for Panel {
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

impl Dispatch<wl_shm::WlShm, ()> for Panel {
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

impl Dispatch<zwlr_layer_shell_v1::ZwlrLayerShellV1, ()> for Panel {
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

impl Dispatch<zwlr_layer_surface_v1::ZwlrLayerSurfaceV1, ()> for Panel {
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
                tracing::debug!(serial, width, height, "layer surface configured");
                app.layer_surface.ack_configure(serial);
                app.configure_serial = Some(serial);
            }
            zwlr_layer_surface_v1::Event::Closed => {
                tracing::info!("layer surface closed; exiting");
                app.running = false;
            }
            _ => {}
        }
    }
}
