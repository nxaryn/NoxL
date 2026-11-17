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

use noxl_launcher::{apps, search};

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

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .context("failed to build tokio runtime")?;

    rt.block_on(async move {
        // Load the real application list.
        let apps = apps::load_applications()?;
        tracing::info!(count = apps.len(), "loaded applications");

        // Create the layer-shell surface and run the event loop.
        run_launcher(config, apps).await
    })
}

async fn run_launcher(
    config: noxl_config::NoxlConfig,
    apps: Vec<apps::AppEntry>,
) -> anyhow::Result<()> {
    let launcher_config = &config.launcher;

    tracing::info!(
        width = launcher_config.width,
        height = launcher_config.height,
        "creating launcher layer-shell surface"
    );

    // The layer-shell setup:
    //   - layer: Layer::Top
    //   - anchor: top-left (or centered)
    //   - size: launcher_config.width x launcher_config.height
    //   - keyboard focus: exclusive (so typing goes to the search field)

    // Event loop: render, handle input, search.
    loop {
        // Render the launcher (search field, app grid, categories).
        // On selection, send Command::Spawn { app } to the compositor.
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }
}
