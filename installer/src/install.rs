//! Installation steps: mount, pacstrap, config.

use std::path::Path;

use anyhow::{Context, Result};

use crate::disk::PartitionLayout;
use crate::user;
use crate::bootloader;

/// Run the full installation.
pub fn run(
    disk: &str,
    fs: &str,
    username: &str,
    timezone: &str,
    keyboard: &str,
    encrypt: bool,
) -> Result<()> {
    tracing::info!(disk, fs, username, "starting installation");

    // Step 1: Partition the disk.
    tracing::info!("Partitioning disk...");
    let layout = crate::disk::partition_disk(disk, fs, encrypt)?;

    // Step 2: Format partitions.
    tracing::info!("Formatting partitions...");
    crate::disk::format_partitions(&layout, fs)?;

    // Step 3: Mount partitions.
    tracing::info!("Mounting partitions...");
    mount_partitions(&layout, encrypt)?;

    // Step 4: Install base system.
    tracing::info!("Installing base system (pacstrap)...");
    install_base()?;
    tracing::info!("Base system installed.");

    // Step 5: Configure system.
    tracing::info!("Configuring system...");
    configure_system(timezone, keyboard)?;

    // Step 6: Create user.
    tracing::info!("Creating user account...");
    user::create_user(username)?;

    // Step 7: Install bootloader.
    tracing::info!("Installing bootloader...");
    bootloader::install(&layout)?;

    // Step 8: Unmount.
    tracing::info!("Unmounting...");
    unmount();

    tracing::info!("Installation complete! Reboot to use NoxL.");
    Ok(())
}

/// Mount the partitions.
fn mount_partitions(layout: &PartitionLayout, _encrypt: bool) -> Result<()> {
    let target = Path::new("/mnt");

    // Create mount points.
    std::fs::create_dir_all(target.join("boot"))?;
    std::fs::create_dir_all(target.join("efi"))?;

    // Mount root.
    let status = std::process::Command::new("mount")
        .args([&layout.root_partition, "/mnt"])
        .status()
        .context("failed to mount root partition")?;
    if !status.success() {
        anyhow::bail!("failed to mount root partition");
    }

    // Mount EFI.
    let status = std::process::Command::new("mount")
        .args(["--mkdir", &layout.efi_partition, "/mnt/efi"])
        .status()
        .context("failed to mount EFI partition")?;
    if !status.success() {
        anyhow::bail!("failed to mount EFI partition");
    }

    tracing::info!(layout = ?layout, "partitions mounted");
    Ok(())
}

/// Install the base system with pacstrap.
fn install_base() -> Result<()> {
    let status = std::process::Command::new("pacstrap")
        .args(["/mnt", "base", "linux", "linux-firmware", "systemd", "networkmanager", "pipewire", "mesa"])
        .status()
        .context("failed to run pacstrap — is arch-install-scripts installed?")?;
    if !status.success() {
        anyhow::bail!("pacstrap failed");
    }
    Ok(())
}

/// Configure the installed system.
fn configure_system(timezone: &str, keyboard: &str) -> Result<()> {
    // Set timezone.
    let status = std::process::Command::new("arch-chroot")
        .args(["/mnt", "ln", "-sf", &format!("/usr/share/zoneinfo/{}", timezone), "/etc/localtime"])
        .status()
        .context("failed to set timezone")?;
    if !status.success() {
        anyhow::bail!("failed to set timezone");
    }

    // Set hostname.
    let status = std::process::Command::new("arch-chroot")
        .args(["/mnt", "sh", "-c", "echo noxl > /etc/hostname"])
        .status()
        .context("failed to set hostname")?;
    if !status.success() {
        anyhow::bail!("failed to set hostname");
    }

    // Set keyboard layout.
    let status = std::process::Command::new("arch-chroot")
        .args(["/mnt", "localectl", "set-keymap", keyboard])
        .status()
        .context("failed to set keyboard layout")?;
    if !status.success() {
        anyhow::bail!("failed to set keyboard layout");
    }

    // Enable NetworkManager.
    let status = std::process::Command::new("arch-chroot")
        .args(["/mnt", "systemctl", "enable", "NetworkManager"])
        .status()
        .context("failed to enable NetworkManager")?;
    if !status.success() {
        anyhow::bail!("failed to enable NetworkManager");
    }

    tracing::info!(timezone, keyboard, "system configured");
    Ok(())
}

/// Unmount all partitions.
fn unmount() {
    let _ = std::process::Command::new("umount").args(["/mnt/efi"]).status();
    let _ = std::process::Command::new("umount").args(["/mnt"]).status();
}
