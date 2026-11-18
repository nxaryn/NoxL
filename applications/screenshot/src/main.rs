//! NoxL Screenshot — capture the screen.
//!
//! Uses the `wlr-screencopy` protocol to capture the screen.
//! Supports fullscreen, window, and region capture.

use anyhow::Context;
use clap::Parser;
use tracing_subscriber::EnvFilter;

mod capture;

#[derive(Debug, Parser)]
#[command(name = "noxl-screenshot", about = "NoxL Screenshot")]
struct Opts {
    #[arg(long, default_value = "info", env = "RUST_LOG")]
    log_level: String,
    /// Output file (defaults to ~/Pictures/screenshot-<timestamp>.png).
    #[arg(long)]
    output: Option<String>,
    /// Capture mode: fullscreen, window, region.
    #[arg(long, default_value = "fullscreen")]
    mode: String,
}

fn main() -> anyhow::Result<()> {
    let opts = Opts::parse();

    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new(&opts.log_level)),
        )
        .init();

    tracing::info!("NoxL Screenshot starting");

    let output = opts.output.unwrap_or_else(|| {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        format!("{}/Pictures/screenshot-{}.png",
            std::env::var("HOME").unwrap_or_else(|_| "/tmp".into()),
            timestamp)
    });

    capture::capture(&opts.mode, &output)
}
