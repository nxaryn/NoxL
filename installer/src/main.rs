//! NoxL Installer — a graphical installer.
//!
//! Produces a genuinely bootable NoxL installation:
//! 1. Disk selection and partitioning (UEFI + root)
//! 2. Filesystem creation (ext4 or Btrfs)
//! 3. Base system installation (pacstrap)
//! 4. User account creation
//! 5. Bootloader setup (systemd-boot or GRUB)
//! 6. Desktop installation
//!
//! The installer runs from the Live ISO and installs to the target disk.

use anyhow::Context;
use clap::Parser;
use tracing_subscriber::EnvFilter;

mod disk;
mod install;
mod user;
mod bootloader;
mod view;

#[derive(Debug, Parser)]
#[command(name = "noxl-installer", about = "NoxL Installer")]
struct Opts {
    #[arg(long, default_value = "info", env = "RUST_LOG")]
    log_level: String,
    /// Target disk (e.g. /dev/sda, /dev/nvme0n1).
    #[arg(long)]
    disk: Option<String>,
    /// Filesystem type: ext4 or btrfs.
    #[arg(long, default_value = "ext4")]
    fs: String,
    /// Username for the first user account.
    #[arg(long)]
    username: Option<String>,
    /// Timezone (e.g. UTC, America/New_York).
    #[arg(long, default_value = "UTC")]
    timezone: String,
    /// Keyboard layout (e.g. us, de).
    #[arg(long, default_value = "us")]
    keyboard: String,
    /// Enable disk encryption (LUKS).
    #[arg(long)]
    encrypt: bool,
}

fn main() -> anyhow::Result<()> {
    let opts = Opts::parse();

    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new(&opts.log_level)),
        )
        .init();

    tracing::info!("NoxL Installer starting");

    let config = noxl_config::NoxlConfig::load().context("failed to load config")?;

    // If a disk is specified, run in non-interactive mode.
    if let Some(disk) = opts.disk {
        let username = opts.username.unwrap_or_else(|| "noxl".into());
        return install::run(
            &disk,
            &opts.fs,
            &username,
            &opts.timezone,
            &opts.keyboard,
            opts.encrypt,
        );
    }

    // Otherwise run the graphical installer.
    view::run(config, opts)
}
