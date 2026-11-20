//! NoxL notification centre.
//!
//! Implements the `org.freedesktop.Notifications` D-Bus interface so any
//! application can send notifications. Notifications are stored in a history,
//! displayed as layer-shell surfaces, and can be silenced with Do Not Disturb.
//!
//! Per-application settings and sounds are configurable.

use anyhow::Context;
use clap::Parser;
use tracing_subscriber::EnvFilter;

use noxl_notifications::{daemon, history, render};

#[derive(Debug, Parser)]
#[command(name = "noxl-notifications", about = "NoxL notification centre")]
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

    tracing::info!("NoxL notification centre starting");

    // Set up Wayland environment (find the compositor socket).
    noxl_ipc::setup_wayland_env();

    let config = noxl_config::NoxlConfig::load().context("failed to load config")?;

    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .context("failed to build tokio runtime")?;

    rt.block_on(async move {
        // Register the org.freedesktop.Notifications D-Bus service.
        daemon::run(config).await
    })
}
