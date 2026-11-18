//! NoxL Software — a graphical package manager.
//!
//! Frontend for pacman. Supports:
//! - Search packages
//! - View package info
//! - Install / remove packages
//! - Update system
//! - Show installed packages
//!
//! Security: package operations go through polkit (privilege separation).
//! The UI never runs as root.

use anyhow::Context;
use clap::Parser;
use tracing_subscriber::EnvFilter;

mod packages;
mod view;

#[derive(Debug, Parser)]
#[command(name = "noxl-software", about = "NoxL Software")]
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

    tracing::info!("NoxL Software starting");

    let config = noxl_config::NoxlConfig::load().context("failed to load config")?;

    view::run(config)
}
