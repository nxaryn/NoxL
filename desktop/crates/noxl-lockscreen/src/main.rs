//! NoxL lock screen.
//!
//! A full-screen layer-shell surface on the Overlay layer that locks the
//! session. Draws the OBSIDIAN lock screen (background, clock, user prompt)
//! and authenticates the user via PAM.
//!
//! Security notes:
//! - Authentication is delegated to PAM (`pam_start`/`pam_authenticate`).
//!   The lock screen never hardcodes or stores passwords.
//! - The lock screen grabs the keyboard exclusively so the user cannot
//!   interact with anything behind it.
//! - On successful authentication the lock screen exits and the session
//!   resumes.

use anyhow::{Context, Result};
use clap::Parser;
use tracing_subscriber::EnvFilter;

mod auth;
mod draw;

#[derive(Debug, Parser)]
#[command(name = "noxl-lockscreen", about = "NoxL lock screen")]
struct Opts {
    #[arg(long, default_value = "info", env = "RUST_LOG")]
    log_level: String,
    /// Username to authenticate (defaults to the current user).
    #[arg(long)]
    user: Option<String>,
}

fn main() -> Result<()> {
    let opts = Opts::parse();

    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new(&opts.log_level)),
        )
        .init();

    tracing::info!("NoxL lock screen starting");

    let config = noxl_config::NoxlConfig::load().context("failed to load config")?;

    let user = opts.user.unwrap_or_else(whoami);
    tracing::info!(%user, "locking session");

    draw::run(config, user)
}

/// Get the current username.
fn whoami() -> String {
    std::env::var("USER")
        .or_else(|_| std::env::var("LOGNAME"))
        .unwrap_or_else(|_| "noxl".into())
}
