//! User account creation.

use anyhow::{Context, Result};

/// Create a user account.
pub fn create_user(username: &str) -> Result<()> {
    // Create the user with a home directory and wheel group.
    let status = std::process::Command::new("arch-chroot")
        .args([
            "/mnt",
            "useradd",
            "-m",
            "-G",
            "wheel,audio,video,storage",
            "-s",
            "/bin/bash",
            username,
        ])
        .status()
        .context("failed to create user")?;
    if !status.success() {
        anyhow::bail!("failed to create user {}", username);
    }

    // Set the password (interactive).
    tracing::info!(username, "set password for user (interactive)");
    let status = std::process::Command::new("arch-chroot")
        .args(["/mnt", "passwd", username])
        .status()
        .context("failed to set password")?;
    if !status.success() {
        anyhow::bail!("failed to set password");
    }

    // Enable sudo for wheel group.
    let status = std::process::Command::new("arch-chroot")
        .args(["/mnt", "sh", "-c", "echo '%wheel ALL=(ALL) ALL' > /etc/sudoers.d/wheel"])
        .status()
        .context("failed to configure sudo")?;
    if !status.success() {
        anyhow::bail!("failed to configure sudo");
    }

    tracing::info!(username, "user created");
    Ok(())
}
