//! NoxL Archive — a graphical archive manager.
//!
//! Supports ZIP, TAR, GZIP, and TAR.GZ formats.
//! Can create, extract, and list archives.

use anyhow::Context;
use clap::Parser;
use tracing_subscriber::EnvFilter;

mod archive;
mod view;

#[derive(Debug, Parser)]
#[command(name = "noxl-archive", about = "NoxL Archive")]
struct Opts {
    #[arg(long, default_value = "info", env = "RUST_LOG")]
    log_level: String,
    /// Archive file to open.
    archive: Option<String>,
}

fn main() -> anyhow::Result<()> {
    let opts = Opts::parse();

    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new(&opts.log_level)),
        )
        .init();

    tracing::info!("NoxL Archive starting");

    let config = noxl_config::NoxlConfig::load().context("failed to load config")?;

    let mut mgr = archive::ArchiveManager::new();

    if let Some(path) = opts.archive {
        mgr.open(&path)?;
    }

    view::run(config, &mut mgr)
}
