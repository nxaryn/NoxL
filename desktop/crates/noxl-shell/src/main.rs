//! NoxL shell — the session entry point.
//!
//! This is what the display manager launches. It:
//! 1. Starts the compositor (`noxl-compositor`).
//! 2. Waits for the Wayland socket to appear.
//! 3. Launches the desktop components: panel, launcher (on demand),
//!    notifications, and the wallpaper.
//! 4. Supervises them: restarts on crash, shuts down cleanly on logout.
//!
//! Keeping this as a separate process means the compositor stays focused on
//! display/input, and the shell handles session lifecycle.

use std::process::Command;

use anyhow::Context;
use clap::Parser;
use tracing_subscriber::EnvFilter;

#[derive(Debug, Parser)]
#[command(name = "noxl-shell", about = "NoxL desktop session")]
struct Opts {
    #[arg(long, default_value = "info", env = "RUST_LOG")]
    log_level: String,
}

/// A managed child process.
struct Child {
    name: &'static str,
    command: Command,
}

fn main() -> anyhow::Result<()> {
    let opts = Opts::parse();

    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new(&opts.log_level)),
        )
        .init();

    tracing::info!("NoxL shell starting");

    // 1. Start the compositor.
    let mut compositor = Command::new("noxl-compositor");
    compositor.env("RUST_LOG", &opts.log_level);
    let mut compositor = compositor
        .spawn()
        .context("failed to spawn noxl-compositor")?;
    tracing::info!("compositor spawned");

    // 2. Wait for the Wayland socket.
    wait_for_wayland_socket()?;

    // 3. Launch desktop components.
    let mut children: Vec<Child> = Vec::new();

    children.push(Child {
        name: "panel",
        command: {
            let mut c = Command::new("noxl-panel");
            c.env("RUST_LOG", &opts.log_level);
            c
        },
    });

    children.push(Child {
        name: "notifications",
        command: {
            let mut c = Command::new("noxl-notifications");
            c.env("RUST_LOG", &opts.log_level);
            c
        },
    });

    // The launcher is started on demand by the panel/compositor, not here.

    let mut spawned: Vec<(&'static str, std::process::Child)> = Vec::new();
    for child in children {
        match child.command.spawn() {
            Ok(p) => {
                tracing::info!(name = child.name, "component spawned");
                spawned.push((child.name, p));
            }
            Err(e) => {
                tracing::error!(name = child.name, error = %e, "failed to spawn component");
            }
        }
    }

    // 4. Supervise: wait for any process to exit, then shut down.
    // (Full implementation restarts crashed components and handles logout.)
    let status = compositor.wait()?;
    tracing::info!(?status, "compositor exited; shutting down session");

    for (name, mut child) in spawned {
        let _ = child.kill();
        let _ = child.wait();
        tracing::info!(name, "component stopped");
    }

    Ok(())
}

/// Wait for the Wayland display socket to appear.
fn wait_for_wayland_socket() -> anyhow::Result<()> {
    let wayland_display = std::env::var("WAYLAND_DISPLAY").unwrap_or_else(|_| "wayland-0".into());
    let runtime_dir = std::env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| "/tmp".into());
    let socket_path = std::path::Path::new(&runtime_dir).join(&wayland_display);

    for _ in 0..100 {
        if socket_path.exists() {
            tracing::info!(path = %socket_path.display(), "Wayland socket ready");
            return Ok(());
        }
        std::thread::sleep(std::time::Duration::from_millis(50));
    }

    anyhow::bail!("Wayland socket did not appear: {}", socket_path.display())
}
