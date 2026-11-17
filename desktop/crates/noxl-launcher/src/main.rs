//! NoxL application launcher.
//!
//! A layer-shell surface (top layer) that shows a searchable list of installed
//! applications, categories, pinned/recent apps, and power controls.
//!
//! Applications are read from the real freedesktop `.desktop` files in
//! `/usr/share/applications`, `~/.local/share/applications`, etc. Launching an
//! app sends a `Spawn` command to the compositor over IPC, which executes the
//! real process.

use anyhow::Context;
use clap::Parser;
use tracing_subscriber::EnvFilter;

use noxl_launcher::{apps, launcher, search};

#[derive(Debug, Parser)]
#[command(name = "noxl-launcher", about = "NoxL application launcher")]
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

    tracing::info!("NoxL launcher starting");

    let config = noxl_config::NoxlConfig::load().context("failed to load config")?;

    // Load the real application list.
    let apps = apps::load_applications()?;
    tracing::info!(count = apps.len(), "loaded applications");

    // Run the launcher (creates the layer-shell surface and event loop).
    launcher::run(config, apps)
}
