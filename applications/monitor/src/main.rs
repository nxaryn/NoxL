//! NoxL Monitor — a system monitor.
//!
//! Shows real system data: CPU, RAM, GPU, processes, storage, network.
//! Uses the `sysinfo` crate for real data (no fake numbers).

use anyhow::Context;
use clap::Parser;
use tracing_subscriber::EnvFilter;

mod view;

#[derive(Debug, Parser)]
#[command(name = "noxl-monitor", about = "NoxL Monitor")]
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

    tracing::info!("NoxL Monitor starting");

    let config = noxl_config::NoxlConfig::load().context("failed to load config")?;

    view::run(config)
}
