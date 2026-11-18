//! Bootloader installation (systemd-boot or GRUB).

use anyhow::{Context, Result};

use crate::disk::PartitionLayout;

/// Install the bootloader.
pub fn install(layout: &PartitionLayout) -> Result<()> {
    // Install systemd-boot (UEFI).
    let status = std::process::Command::new("arch-chroot")
        .args(["/mnt", "bootctl", "install"])
        .status()
        .context("failed to install systemd-boot — is systemd installed?")?;
    if !status.success() {
        anyhow::bail!("failed to install systemd-boot");
    }

    // Create the loader entry.
    let root_uuid = get_partition_uuid(&layout.root_partition)?;
    let entry = format!(
        "title   NoxL\nlinux   /vmlinuz-linux\ninitrd  /initramfs-linux.img\noptions root=UUID={} rw quiet splash\n",
        root_uuid
    );

    let status = std::process::Command::new("arch-chroot")
        .args(["/mnt", "sh", "-c", &format!("echo '{}' > /efi/loader/entries/noxl.conf", entry)])
        .status()
        .context("failed to create loader entry")?;
    if !status.success() {
        anyhow::bail!("failed to create loader entry");
    }

    // Set the default entry.
    let status = std::process::Command::new("arch-chroot")
        .args(["/mnt", "sh", "-c", "echo 'default noxl.conf' > /efi/loader/loader.conf"])
        .status()
        .context("failed to set default entry")?;
    if !status.success() {
        anyhow::bail!("failed to set default entry");
    }

    tracing::info!("bootloader installed");
    Ok(())
}

/// Get the UUID of a partition.
fn get_partition_uuid(partition: &str) -> Result<String> {
    let output = std::process::Command::new("blkid")
        .args(["-s", "UUID", "-o", "value", partition])
        .output()
        .context("failed to run blkid")?;
    if !output.status.success() {
        anyhow::bail!("failed to get UUID for {}", partition);
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}
