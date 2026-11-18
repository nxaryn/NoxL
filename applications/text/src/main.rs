//! NoxL Text — a lightweight text editor.
//!
//! Features:
//! - Open and edit text files
//! - Multiple tabs
//! - Syntax highlighting (basic, based on file extension)
//! - Line numbers
//! - OBSIDIAN theme
//! - Save

use anyhow::Context;
use clap::Parser;
use tracing_subscriber::EnvFilter;

mod editor;
mod view;

#[derive(Debug, Parser)]
#[command(name = "noxl-text", about = "NoxL Text")]
struct Opts {
    #[arg(long, default_value = "info", env = "RUST_LOG")]
    log_level: String,
    /// File to open.
    file: Option<String>,
}

fn main() -> anyhow::Result<()> {
    let opts = Opts::parse();

    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new(&opts.log_level)),
        )
        .init();

    tracing::info!("NoxL Text starting");

    let config = noxl_config::NoxlConfig::load().context("failed to load config")?;

    let mut editor = editor::Editor::new();

    if let Some(path) = opts.file {
        editor.open(&path)?;
    }

    view::run(config, &mut editor)
}
