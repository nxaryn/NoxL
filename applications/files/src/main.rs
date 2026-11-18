//! NoxL Files — a native file manager.
//!
//! Features:
//! - Directory listing (real files from the filesystem)
//! - Navigation (parent, home, bookmarks)
//! - File operations (copy, move, delete, rename)
//! - Tabs
//! - Bookmarks
//! - Search
//! - OBSIDIAN theme

use anyhow::Context;
use clap::Parser;
use tracing_subscriber::EnvFilter;

mod browser;
mod view;

#[derive(Debug, Parser)]
#[command(name = "noxl-files", about = "NoxL Files")]
struct Opts {
    #[arg(long, default_value = "info", env = "RUST_LOG")]
    log_level: String,
    /// Directory to open (defaults to $HOME).
    #[arg(long)]
    path: Option<String>,
}

fn main() -> anyhow::Result<()> {
    let opts = Opts::parse();

    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new(&opts.log_level)),
        )
        .init();

    tracing::info!("NoxL Files starting");

    let config = noxl_config::NoxlConfig::load().context("failed to load config")?;

    let start_path = opts.path.unwrap_or_else(|| {
        std::env::var("HOME").unwrap_or_else(|_| "/".into())
    });

    let mut browser = browser::Browser::new(&start_path)?;
    tracing::info!(path = %start_path, entries = browser.entries.len(), "browser initialized");

    view::run(config, &mut browser)
}
