//! Monitor view: layer-shell surface that displays system stats.

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

use noxl_render::{Painter, ShmBuffer, Font, draw_text};

/// The monitor view state.
pub struct MonitorView {
    pub surface: wl_surface::WlSurface,
    pub layer_surface: zwlr_layer_surface_v1::ZwlrLayerSurfaceV1,
    pub config: noxl_config::NoxlConfig,
    pub shm: Option<wl_shm::WlShm>,
    pub qh: Option<QueueHandle<MonitorView>>,
    pub running: bool,
    pub width: i32,
    pub height: i32,
    pub system: sysinfo::System,
}

/// Run the monitor view.
pub fn run(config: noxl_config::NoxlConfig) -> Result<()> {
    let conn = Connection::connect_to_env().context("failed to connect to Wayland display")?;
    let (globals, mut event_queue) =
        registry_queue_init::<MonitorView>(&conn).context("failed to init registry queue")?;
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
        "noxl-monitor".to_string(),
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

    tracing::info!("monitor layer-shell surface created");

    let mut view = MonitorView {
        surface,
        layer_surface,
        config,
        shm: Some(shm),
        qh: Some(qh.clone()),
        running: true,
        width: 600,
        height: 500,
        system: sysinfo::System::new_all(),
    };

    loop {
        // Refresh system data.
        view.system.refresh_all();

        event_queue
            .blocking_dispatch(&mut view)
            .context("Wayland dispatch error")?;

        draw_monitor(&mut view);

        if !view.running {
            break;
        }

        // Update once per second.
        std::thread::sleep(std::time::Duration::from_millis(1000));
    }

    Ok(())
}

/// Draw the monitor into a buffer and attach it.
fn draw_monitor(view: &mut MonitorView) {
    let Some(shm) = &view.shm else { return };
    let Some(qh) = &view.qh else { return };
    let qh = qh.clone();

    let w = view.width;
    let h = view.height;

    let buffer = match ShmBuffer::new(shm, &qh, w, h) {
        Ok(b) => b,
        Err(e) => {
            tracing::error!(error = %e, "failed to create monitor buffer");
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

    // CPU usage bar.
    let cpu = view.system.global_cpu_info().cpu_usage();
    let cpu_y = 60;
    painter.fill_rect(20, cpu_y, w - 40, 20, br, bg, bb, ba);
    painter.fill_rect(20, cpu_y, ((w - 40) as f32 * cpu / 100.0) as i32, 20, ar, ag, ab, aa);

    // RAM usage bar.
    let ram_used = view.system.used_memory();
    let ram_total = view.system.total_memory();
    let ram_pct = if ram_total > 0 { (ram_used as f64 / ram_total as f64) * 100.0 } else { 0.0 };
    let ram_y = 100;
    painter.fill_rect(20, ram_y, w - 40, 20, br, bg, bb, ba);
    painter.fill_rect(20, ram_y, ((w - 40) as f64 * ram_pct / 100.0) as i32, 20, ar, ag, ab, aa);

    // CPU info text (real font rendering).
    let (tr, tg, tb, ta) = noxl_render::parse_hex(&view.config.theme.colors.text_primary);
    let mut font = Font::load_default().ok();
    if let Some(ref mut f) = font {
        f.set_pixel_size(13);
    }

    let cpu_text = format!("CPU: {:.0}%", cpu);
    if let Some(ref mut f) = font {
        draw_text(&mut painter, f, &cpu_text, 20, 85, tr, tg, tb, ta);
    }

    // RAM info text.
    let ram_text = format!("RAM: {:.0}% ({:.1}/{:.1} GB)", ram_pct, ram_used as f64 / 1e9, ram_total as f64 / 1e9);
    if let Some(ref mut f) = font {
        draw_text(&mut painter, f, &ram_text, 20, 125, tr, tg, tb, ta);
    }

    // Uptime.
    let uptime = sysinfo::System::uptime();
    let hours = uptime / 3600;
    let mins = (uptime % 3600) / 60;
    let uptime_text = format!("Uptime: {}h {}m", hours, mins);
    if let Some(ref mut f) = font {
        draw_text(&mut painter, f, &uptime_text, 20, 155, tr, tg, tb, ta);
    }

    // CPU brand.
    let brand = view.system.cpus().first()
        .map(|c| c.brand().trim())
        .unwrap_or("Unknown");
    let brand_text = format!("CPU: {}", brand);
    if let Some(ref mut f) = font {
        draw_text(&mut painter, f, &brand_text, 20, 185, tr, tg, tb, ta);
    }

    // Process count.
    let proc_text = format!("Processes: {}", view.system.processes().len());
    if let Some(ref mut f) = font {
        draw_text(&mut painter, f, &proc_text, 20, 215, tr, tg, tb, ta);
    }

    let mut buffer = buffer;
    buffer.data.copy_from_slice(&painter.data);
    buffer.attach(&view.surface);
}

// --- Dispatch implementations ---

impl Dispatch<wl_registry::WlRegistry, GlobalListContents> for MonitorView {
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

impl Dispatch<wl_compositor::WlCompositor, ()> for MonitorView {
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

impl Dispatch<wl_surface::WlSurface, ()> for MonitorView {
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

impl Dispatch<wl_region::WlRegion, ()> for MonitorView {
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

impl Dispatch<wl_output::WlOutput, ()> for MonitorView {
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

impl Dispatch<wl_shm::WlShm, ()> for MonitorView {
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

impl Dispatch<wl_shm_pool::WlShmPool, ()> for MonitorView {
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

impl Dispatch<wl_buffer::WlBuffer, ()> for MonitorView {
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

impl Dispatch<zwlr_layer_shell_v1::ZwlrLayerShellV1, ()> for MonitorView {
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

impl Dispatch<zwlr_layer_surface_v1::ZwlrLayerSurfaceV1, ()> for MonitorView {
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
