//! NoxL panel / taskbar.
//!
//! A separate process that creates a `wlr_layer_shell` surface on the bottom
//! (or configured) edge of the screen and renders the panel: launcher button,
//! running/pinned apps, workspace switcher, system tray, status icons, and a
//! clock.
//!
//! The panel reads its layout from `noxl-config` and receives live updates
//! (workspace changes, window list) from the compositor over the IPC socket.

use anyhow::Context;
use clap::Parser;
use tracing_subscriber::EnvFilter;

mod draw;
mod panel;
mod plugins;

#[derive(Debug, Parser)]
#[command(name = "noxl-panel", about = "NoxL panel/taskbar")]
struct Opts {
    #[arg(long, default_value = "info", env = "RUST_LOG")]
    log_level: String,
}

fn main() -> anyhow::Result<()> {
    let opts = Opts::parse();

    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new(&opts.log_level)),
        )
        .init();

    tracing::info!("NoxL panel starting");

    // Set up Wayland environment (find the compositor socket).
    noxl_ipc::setup_wayland_env();

    let config = noxl_config::NoxlConfig::load().context("failed to load config")?;

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .context("failed to build tokio runtime")?;

    rt.block_on(async move {
        panel::run(config)?;
        Ok(())
    })
}
