//! NoxL Wayland compositor.
//!
//! Built on Smithay 0.7. This is the heart of NoxL Desktop: it owns the Wayland
//! display, manages windows, handles input, and renders the desktop.
//!
//! In nested mode it runs inside an existing Wayland/X11 session (for
//! development). In standalone mode it would use DRM/libinput/udev (Phase 6).

pub mod state;
pub mod window;
pub mod input;
pub mod protocols;
pub mod ipc;
pub mod render;

use anyhow::Context;
use clap::Parser;
use tracing_subscriber::EnvFilter;

use smithay::{
    backend::{
        input::{InputEvent, KeyboardKeyEvent},
        renderer::gles::GlesRenderer,
        winit::{self as smithay_winit, WinitEvent},
    },
    reexports::wayland_server::{Display, ListeningSocket},
};

#[derive(Debug, Parser)]
#[command(name = "noxl-compositor", about = "NoxL Wayland compositor")]
struct Opts {
    #[arg(long, default_value = "info", env = "RUST_LOG")]
    log_level: String,
    #[arg(long)]
    no_ipc: bool,
    /// Start nested inside an existing Wayland session (development).
    #[arg(long)]
    nested: bool,
}

fn main() -> anyhow::Result<()> {
    let opts = Opts::parse();

    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new(&opts.log_level)),
        )
        .init();

    tracing::info!("NoxL compositor starting");

    let config = noxl_config::NoxlConfig::load().context("failed to load config")?;
    tracing::info!(theme = %config.theme.name, "config loaded");

    // Create the Wayland display.
    let mut display: Display<state::Noxl> = Display::new().map_err(|e| anyhow::anyhow!("{}", e))?;;;
    let dh = display.handle();

    // Set up the winit backend (nested mode).
    let (mut backend, mut winit) = smithay_winit::init::<GlesRenderer>().map_err(|e| anyhow::anyhow!("{}", e))?;;
    let size = backend.window_size();

    // Create the compositor state.
    let mut compositor = state::Noxl::new(&display, config, opts.no_ipc, opts.nested)
        .context("failed to initialize compositor state")?;

    // Add a keyboard.
    let keyboard = compositor
        .seat
        .add_keyboard(Default::default(), 200, 200)
        .context("failed to add keyboard")?;

    // Set up the Wayland socket.
    let listener = ListeningSocket::bind("wayland-1")?;
    std::env::set_var("WAYLAND_DISPLAY", "wayland-1");
    let mut clients = Vec::new();

    let start_time = std::time::Instant::now();

    // Event loop.
    loop {
        let status = winit.dispatch_new_events(|event| match event {
            WinitEvent::Resized { .. } => {}
            WinitEvent::Input(event) => match event {
                InputEvent::Keyboard { event } => {
                    input::handle_keyboard_event(&mut compositor, &keyboard, event.key_code(), event.state());
                }
                _ => {}
            },
            _ => (),
        });

        // Process IPC commands.
        if let Some(ipc) = &mut compositor.ipc {
            for cmd in ipc.poll_commands() {
                compositor.handle_command(cmd);
            }
        }

        // Render.
        render::render(&mut compositor, &mut backend, &mut winit, size)?;

        // Dispatch Wayland clients.
        display.dispatch_clients(&mut compositor)?;
        display.flush_clients()?;

        // Accept new clients.
        if let Some(stream) = listener.accept()? {
            let client = display
                .handle()
                .insert_client(stream, std::sync::Arc::new(state::ClientState::default()))
                .ok();
            if let Some(client) = client {
                clients.push(client);
            }
        }

        // Send frame callbacks.
        // (Frame callbacks are sent by the renderer in a full implementation.)

        match status {
            winit::platform::pump_events::PumpStatus::Continue => (),
            winit::platform::pump_events::PumpStatus::Exit(_) => return Ok(()),
        }
    }
}
