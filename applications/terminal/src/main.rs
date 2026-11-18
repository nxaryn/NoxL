//! NoxL Terminal — a native terminal emulator.
//!
//! Uses `portable-pty` to spawn a real PTY (your shell), `vte` to parse the
//! terminal output, and a layer-shell surface to display it.
//!
//! Features:
//! - Real PTY (runs your actual shell: bash, zsh, fish, etc.)
//! - Multiple tabs
//! - OBSIDIAN theme (colors from config)
//! - Configurable font (from theme)

use anyhow::Context;
use clap::Parser;
use tracing_subscriber::EnvFilter;

mod term;
mod view;

#[derive(Debug, Parser)]
#[command(name = "noxl-terminal", about = "NoxL Terminal")]
struct Opts {
    #[arg(long, default_value = "info", env = "RUST_LOG")]
    log_level: String,
    /// Shell to run (defaults to $SHELL or /bin/bash).
    #[arg(long)]
    shell: Option<String>,
    /// Working directory (defaults to $HOME).
    #[arg(long)]
    workdir: Option<String>,
}

fn main() -> anyhow::Result<()> {
    let opts = Opts::parse();

    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new(&opts.log_level)),
        )
        .init();

    tracing::info!("NoxL Terminal starting");

    let config = noxl_config::NoxlConfig::load().context("failed to load config")?;

    let shell = opts.shell.unwrap_or_else(|| {
        std::env::var("SHELL").unwrap_or_else(|_| "/bin/bash".into())
    });
    let workdir = opts.workdir.unwrap_or_else(|| {
        std::env::var("HOME").unwrap_or_else(|_| "/tmp".into())
    });

    tracing::info!(%shell, %workdir, "spawning terminal");

    // Spawn the PTY.
    let mut term = term::Terminal::new(&shell, &workdir)?;

    // Run the view (layer-shell surface + event loop).
    view::run(config, &mut term)
}