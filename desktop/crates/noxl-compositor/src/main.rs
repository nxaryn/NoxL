//! NoxL Wayland compositor.
//!
//! Built on Smithay. This is the heart of NoxL Desktop: it owns the Wayland
//! display, manages windows (floating + tiling + snapping), handles input,
//! renders the desktop (background, windows, and layer-shell surfaces for the
//! panel/launcher/notifications), and exposes the control socket for the other
//! NoxL processes.
//!
//! Architecture (see docs/PHASE2.md):
//! - `state`      — global compositor state (workspaces, windows, outputs)
//! - `window`     — window management primitives (focus, snap, tile, fullscreen)
//! - `input`      — keyboard/pointer handling and keybindings
//! - `protocols`  — Wayland protocol implementations (xdg-shell, layer-shell,
//!                  xdg-decoration, output, seat)
//! - `ipc`        — control-socket server (JSON lines)
//! - `render`     — drawing (background, windows, layer surfaces)

pub mod state;
pub mod window;
pub mod input;
pub mod protocols;
pub mod ipc;
pub mod render;

use anyhow::Context;
use clap::Parser;
use tracing_subscriber::EnvFilter;

/// Command-line options.
#[derive(Debug, Parser)]
#[command(name = "noxl-compositor", about = "NoxL Wayland compositor")]
struct Opts {
    /// Log level (trace, debug, info, warn, error).
    #[arg(long, default_value = "info", env = "RUST_LOG")]
    log_level: String,
    /// Disable the control socket (for debugging).
    #[arg(long)]
    no_ipc: bool,
    /// Start nested (for development inside an existing Wayland session).
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

    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .context("failed to build tokio runtime")?;

    rt.block_on(async move {
        let config = noxl_config::NoxlConfig::load().context("failed to load config")?;
        tracing::info!(theme = %config.theme.name, "config loaded");

        let mut compositor = state::Compositor::new(config, opts.no_ipc, opts.nested)
            .context("failed to initialize compositor state")?;

        compositor.run().await
    })
}
