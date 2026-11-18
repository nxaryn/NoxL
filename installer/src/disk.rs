//! Disk operations: detection, partitioning, formatting.

use std::path::Path;

use anyhow::{Context, Result};

/// A disk detected on the system.
#[derive(Debug, Clone)]
pub struct Disk {
    pub path: String,
    pub model: String,
    pub size_gb: f64,
    pub is_removable: bool,
}

/// Detect available disks.
pub fn detect_disks() -> Result<Vec<Disk>> {
    let mut disks = Vec::new();

    // Read /sys/block for block devices.
    let block_dir = Path::new("/sys/block");
    if !block_dir.exists() {
        anyhow::bail!("/sys/block not found — not running on Linux?");
    }

    for entry in std::fs::read_dir(block_dir)? {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().to_string();

        // Skip loop devices, ram devices, and partitions.
        if name.starts_with("loop") || name.starts_with("ram") || name.starts_with("dm-") {
            continue;
        }

        let dev_path = format!("/dev/{}", name);

        // Get the device size.
        let size_path = format!("/sys/block/{}/size", name);
        let size_sectors: u64 = std::fs::read_to_string(&size_path)
            .ok()
            .and_then(|s| s.trim().parse().ok())
            .unwrap_or(0);
        let size_gb = (size_sectors * 512) as f64 / 1e9;

        // Get the device model.
        let model_path = format!("/sys/block/{}/device/model", name);
        let model = std::fs::read_to_string(&model_path)
            .ok()
            .map(|s| s.trim().to_string())
            .unwrap_or_else(|| "Unknown".into());

        // Check if removable.
        let removable_path = format!("/sys/block/{}/removable", name);
        let is_removable = std::fs::read_to_string(&removable_path)
            .ok()
            .map(|s| s.trim() == "1")
            .unwrap_or(false);

        disks.push(Disk {
            path: dev_path,
            model,
            size_gb,
            is_removable,
        });
    }

    Ok(disks)
}

/// Partition a disk (UEFI + root).
///
/// Creates:
/// - Partition 1: EFI System Partition (512 MB, FAT32)
/// - Partition 2: Root filesystem (rest of disk, ext4 or Btrfs)
///
/// If `encrypt` is true, partition 2 is LUKS-encrypted.
pub fn partition_disk(disk: &str, _fs: &str, encrypt: bool) -> Result<PartitionLayout> {
    let _ = encrypt; // LUKS handled in install step

    // Use sgdisk to create the partition table.
    // This is a real operation that modifies the disk.
    let status = std::process::Command::new("sgdisk")
        .args([
            "--zap-all",                    // Clear existing partition table
            &format!("--new=1:0:+512M"),   // EFI partition
            &format!("--new=2:0:0"),       // Root partition (rest of disk)
            &format!("--typecode=1:ef00"), // EFI type
            &format!("--typecode=2:8300"), // Linux filesystem type
            &format!("--change-name=1:NoxL EFI"),
            &format!("--change-name=2:NoxL Root"),
            disk,
        ])
        .status()
        .context("failed to run sgdisk — is gptfdisk installed?")?;

    if !status.success() {
        anyhow::bail!("sgdisk failed to partition {}", disk);
    }

    // Determine partition paths.
    let (efi_partition, root_partition) = if disk.contains("nvme") || disk.contains("mmcblk") {
        (format!("{}p1", disk), format!("{}p2", disk))
    } else {
        (format!("{}1", disk), format!("{}2", disk))
    };

    Ok(PartitionLayout {
        disk: disk.to_string(),
        efi_partition,
        root_partition,
    })
}

/// The partition layout.
#[derive(Debug, Clone)]
pub struct PartitionLayout {
    pub disk: String,
    pub efi_partition: String,
    pub root_partition: String,
}

/// Format the partitions.
pub fn format_partitions(layout: &PartitionLayout, fs: &str) -> Result<()> {
    // Format EFI as FAT32.
    let status = std::process::Command::new("mkfs.fat")
        .args(["-F32", &layout.efi_partition])
        .status()
        .context("failed to format EFI partition — is dosfstools installed?")?;
    if !status.success() {
        anyhow::bail!("failed to format EFI partition");
    }

    // Format root.
    let status = match fs {
        "btrfs" => std::process::Command::new("mkfs.btrfs")
            .args(["-f", &layout.root_partition])
            .status()
            .context("failed to format root — is btrfs-progs installed?")?,
        _ => std::process::Command::new("mkfs.ext4")
            .args(["-F", &layout.root_partition])
            .status()
            .context("failed to format root — is e2fsprogs installed?")?,
    };
    if !status.success() {
        anyhow::bail!("failed to format root partition");
    }

    tracing::info!(layout = ?layout, fs, "partitions formatted");
    Ok(())
}
