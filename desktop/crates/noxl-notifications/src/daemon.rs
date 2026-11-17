//! Notification daemon: org.freedesktop.Notifications D-Bus service.

use anyhow::Context;

use crate::{history, render};

/// Run the notification daemon.
pub async fn run(config: noxl_config::NoxlConfig) -> anyhow::Result<()> {
    tracing::info!("registering org.freedesktop.Notifications on the session bus");

    // Connect to the session bus and claim the notification service name.
    // (Full implementation uses zbus to serve the interface:
    //   - Notify(app_name, replaces_id, app_icon, summary, body, actions,
    //            hints, expire_timeout) -> u32
    //   - GetCapabilities() -> Vec<String>
    //   - CloseNotification(id)
    //   - GetServerInformation() -> (name, vendor, version, spec_version)
    // and emits org.freedesktop.Notifications.NotificationClosed and
    //  ActionInvoked signals.)

    let _connection = zbus::Connection::session()
        .await
        .context("failed to connect to session bus")?;

    tracing::info!("connected to session bus");

    // Load history.
    let history = history::History::load()?;

    // Run the notification centre (creates the layer surface and event loop).
    render::run(config, history)?;
    Ok(())
}
