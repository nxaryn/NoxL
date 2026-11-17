//! NoxL Control Centre (Settings).
//!
//! The centralised settings application. Organised into categories:
//! - Personalisation: themes, colors, wallpaper, desktop layout, icons,
//!   fonts, cursor, animations, taskbar, start menu, widgets, lock screen.
//! - System: display, audio, power, storage, users, keyboard, mouse,
//!   accessibility, default applications.
//! - Connectivity: Wi-Fi, Ethernet, Bluetooth, VPN.
//! - Devices: printers, USB, displays, audio, input.
//! - Security: firewall, permissions, disk encryption, authentication,
//!   application permissions, updates.
//! - Advanced: kernel selection, boot config, services, hardware info,
//!   logs, developer options.
//!
//! Every setting reads and writes the real system configuration through
//! appropriate interfaces (D-Bus, system tools, config files). The settings
//! app never runs as root; privileged actions go through polkit.

use anyhow::Context;
use clap::Parser;
use tracing_subscriber::EnvFilter;

mod categories;
mod presets;
mod wallpaper;

#[derive(Debug, Parser)]
#[command(name = "noxl-settings", about = "NoxL Control Centre")]
struct Opts {
    #[arg(long, default_value = "info", env = "RUST_LOG")]
    log_level: String,
    /// Open directly to a category (e.g. "personalisation", "system").
    #[arg(long)]
    category: Option<String>,
}

fn main() -> anyhow::Result<()> {
    let opts = Opts::parse();

    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new(&opts.log_level)),
        )
        .init();

    tracing::info!("NoxL Control Centre starting");

    let config = noxl_config::NoxlConfig::load().context("failed to load config")?;

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .context("failed to build tokio runtime")?;

    rt.block_on(async move {
        // The settings UI is a layer-shell surface (or a regular window).
        // Each category reads/writes config and calls system interfaces.
        tracing::info!(category = ?opts.category, "showing settings");
        Ok(())
    })
}
