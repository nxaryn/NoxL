//! Package management: search, install, remove, update.
//!
//! Uses pacman via subprocess. Privileged operations go through pkexec
//! (polkit) so the UI never runs as root.

use std::process::Command;

use anyhow::Result;

/// A package.
#[derive(Debug, Clone)]
pub struct Package {
    pub name: String,
    pub version: String,
    pub description: String,
    pub installed: bool,
}

/// Search for packages.
pub fn search(query: &str) -> Result<Vec<Package>> {
    let output = Command::new("pacman")
        .args(["-Ss", query])
        .output()?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut packages = Vec::new();

    for line in stdout.lines() {
        if line.starts_with("extra/") || line.starts_with("core/") || line.starts_with("community/") {
            // Parse: "repo/name version"
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                let name = parts[0].split('/').nth(1).unwrap_or(parts[0]).to_string();
                let version = parts[1].to_string();
                packages.push(Package {
                    name,
                    version,
                    description: String::new(),
                    installed: false,
                });
            }
        }
    }

    Ok(packages)
}

/// List installed packages.
pub fn list_installed() -> Result<Vec<Package>> {
    let output = Command::new("pacman")
        .args(["-Q"])
        .output()?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut packages = Vec::new();

    for line in stdout.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 2 {
            packages.push(Package {
                name: parts[0].to_string(),
                version: parts[1].to_string(),
                description: String::new(),
                installed: true,
            });
        }
    }

    Ok(packages)
}

/// Install a package (via pkexec for privilege separation).
pub fn install(name: &str) -> Result<()> {
    let status = Command::new("pkexec")
        .args(["pacman", "-S", "--noconfirm", name])
        .status()?;
    if !status.success() {
        anyhow::bail!("failed to install {}", name);
    }
    Ok(())
}

/// Remove a package (via pkexec).
pub fn remove(name: &str) -> Result<()> {
    let status = Command::new("pkexec")
        .args(["pacman", "-R", "--noconfirm", name])
        .status()?;
    if !status.success() {
        anyhow::bail!("failed to remove {}", name);
    }
    Ok(())
}

/// Update the system (via pkexec).
pub fn update() -> Result<()> {
    let status = Command::new("pkexec")
        .args(["pacman", "-Syu", "--noconfirm"])
        .status()?;
    if !status.success() {
        anyhow::bail!("failed to update system");
    }
    Ok(())
}
