//! Installer view: layer-shell surface that displays the installer UI.

use anyhow::{Context, Result};

use wayland_client::{
    globals::{registry_queue_init, GlobalListContents},
    protocol::{
        wl_buffer, wl_compositor, wl_output, wl_registry, wl_region, wl_seat, wl_shm, wl_shm_pool,
        wl_surface,
    },
    Connection, Dispatch, Proxy, QueueHandle,
};

use wayland_protocols_wlr::layer_shell::v1::client::{
    zwlr_layer_shell_v1, zwlr_layer_surface_v1,
};

use noxl_render::{draw_text, Font, Painter, ShmBuffer};

use crate::disk;

/// The installer view state.
pub struct InstallerView {
    pub surface: wl_surface::WlSurface,
    pub layer_surface: zwlr_layer_surface_v1::ZwlrLayerSurfaceV1,
    pub config: noxl_config::NoxlConfig,
    pub shm: Option<wl_shm::WlShm>,
    pub qh: Option<QueueHandle<InstallerView>>,
    pub running: bool,
    pub width: i32,
    pub height: i32,
    pub disks: Vec<disk::Disk>,
    pub selected_disk: Option<usize>,
    pub step: InstallStep,
    pub status: String,
}

/// Installation steps.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstallStep {
    Welcome,
    DiskSelect,
    UserConfig,
    Confirm,
    Installing,
    Done,
}

/// Run the installer view.
pub fn run(config: noxl_config::NoxlConfig, opts: crate::Opts) -> Result<()> {
    let conn = Connection::connect_to_env().context("failed to connect to Wayland display")?;
    let (globals, mut event_queue) =
        registry_queue_init::<InstallerView>(&conn).context("failed to init registry queue")?;
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
    let _seat: wl_seat::WlSeat = globals
        .bind(&qh, 1..=9, ())
        .context("wl_seat not available")?;

    let surface = compositor.create_surface(&qh, ());

    let layer_surface = layer_shell.get_layer_surface(
        &surface,
        None,
        zwlr_layer_shell_v1::Layer::Overlay,
        "noxl-installer".to_string(),
        &qh,
        (),
    );

    layer_surface.set_size(0, 0);
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

    tracing::info!("installer layer-shell surface created");

    // Detect disks.
    let disks = disk::detect_disks().unwrap_or_default();
    tracing::info!(count = disks.len(), "detected disks");

    let mut view = InstallerView {
        surface,
        layer_surface,
        config,
        shm: Some(shm),
        qh: Some(qh.clone()),
        running: true,
        width: 1024,
        height: 768,
        disks,
        selected_disk: None,
        step: InstallStep::Welcome,
        status: "Welcome to NoxL Installer".into(),
    };

    loop {
        event_queue
            .blocking_dispatch(&mut view)
            .context("Wayland dispatch error")?;

        draw_installer(&mut view);

        if !view.running {
            break;
        }
    }

    Ok(())
}

/// Draw the installer into a buffer and attach it.
fn draw_installer(view: &mut InstallerView) {
    let Some(shm) = &view.shm else { return };
    let Some(qh) = &view.qh else { return };
    let qh = qh.clone();

    let w = view.width;
    let h = view.height;

    let buffer = match ShmBuffer::new(shm, &qh, w, h) {
        Ok(b) => b,
        Err(e) => {
            tracing::error!(error = %e, "failed to create installer buffer");
            return;
        }
    };

    let mut painter = Painter::new(w, h);

    // OBSIDIAN background.
    let (r, g, b, a) = noxl_render::parse_hex(&view.config.theme.colors.bg_primary);
    painter.fill(r, g, b, a);

    // Title bar.
    let (br, bg, bb, ba) = noxl_render::parse_hex(&view.config.theme.colors.bg_secondary);
    painter.fill_rect(0, 0, w, 60, br, bg, bb, ba);

    // Accent line.
    let (ar, ag, ab, aa) = noxl_render::parse_hex(&view.config.theme.colors.accent_primary);
    painter.fill_rect(0, 60, w, 2, ar, ag, ab, aa);

    // Title.
    let (tr, tg, tb, ta) = noxl_render::parse_hex(&view.config.theme.colors.text_primary);
    if let Ok(mut font) = Font::load_default() {
        font.set_pixel_size(24);
        draw_text(&mut painter, &mut font, "NoxL Installer", 40, 15, tr, tg, tb, ta);
    }

    // Step content.
    let (sr, sg, sb, sa) = noxl_render::parse_hex(&view.config.theme.colors.text_secondary);
    if let Ok(mut font) = Font::load_default() {
        font.set_pixel_size(14);
        draw_text(&mut painter, &mut font, &view.status, 40, 100, tr, tg, tb, ta);
    }

    // Disk list.
    if view.step == InstallStep::DiskSelect {
        let y_start = 140;
        for (i, disk) in view.disks.iter().enumerate() {
            let y = y_start + (i as i32) * 40;
            let selected = view.selected_disk == Some(i);

            // Highlight selected disk.
            if selected {
                painter.fill_rect(30, y - 5, w - 60, 35, ar, ag, ab, aa);
            }

            if let Ok(mut font) = Font::load_default() {
                font.set_pixel_size(13);
                let text = format!("{} ({:.0} GB) - {}", disk.path, disk.size_gb, disk.model);
                draw_text(&mut painter, &mut font, &text, 40, y, tr, tg, tb, ta);
            }
        }
    }

    // Status / progress.
    if view.step == InstallStep::Installing {
        if let Ok(mut font) = Font::load_default() {
            font.set_pixel_size(14);
            draw_text(&mut painter, &mut font, &view.status, 40, h - 60, tr, tg, tb, ta);
        }
    }

    // Bottom hint.
    if let Ok(mut font) = Font::load_default() {
        font.set_pixel_size(11);
        draw_text(&mut painter, &mut font, "Press Enter to continue, Escape to cancel", 40, h - 30, sr, sg, sb, sa);
    }

    let mut buffer = buffer;
    buffer.data.copy_from_slice(&painter.data);
    buffer.attach(&view.surface);
}

// --- Dispatch implementations ---

impl Dispatch<wl_registry::WlRegistry, GlobalListContents> for InstallerView {
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

impl Dispatch<wl_compositor::WlCompositor, ()> for InstallerView {
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

impl Dispatch<wl_surface::WlSurface, ()> for InstallerView {
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

impl Dispatch<wl_region::WlRegion, ()> for InstallerView {
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

impl Dispatch<wl_output::WlOutput, ()> for InstallerView {
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

impl Dispatch<wl_seat::WlSeat, ()> for InstallerView {
    fn event(
        _: &mut Self,
        _: &wl_seat::WlSeat,
        _: wl_seat::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<wl_shm::WlShm, ()> for InstallerView {
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

impl Dispatch<wl_shm_pool::WlShmPool, ()> for InstallerView {
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

impl Dispatch<wl_buffer::WlBuffer, ()> for InstallerView {
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

impl Dispatch<zwlr_layer_shell_v1::ZwlrLayerShellV1, ()> for InstallerView {
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

impl Dispatch<zwlr_layer_surface_v1::ZwlrLayerSurfaceV1, ()> for InstallerView {
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
