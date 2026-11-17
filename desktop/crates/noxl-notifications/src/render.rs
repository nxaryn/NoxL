//! Notification rendering: layer-shell popups and the notification centre.

use crate::history::History;

/// Render visible notifications.
pub fn render(_config: &noxl_config::NoxlConfig, history: &History) -> anyhow::Result<()> {
    // For each unread notification, draw a layer-shell popup at the top-right
    // (or configured position). Popups auto-dismiss after expire_timeout.
    let unread = history.unread();
    tracing::trace!(count = unread.len(), "rendering notifications");
    Ok(())
}
