//! Panel runtime: layer-shell surface, layout, and event loop.

use anyhow::Context;

use crate::plugins;

/// Run the panel.
pub async fn run(config: noxl_config::NoxlConfig) -> anyhow::Result<()> {
    let panel_config = &config.panel;

    tracing::info!(
        position = ?panel_config.position,
        size = panel_config.size,
        "creating panel layer-shell surface"
    );

    // Connect to the Wayland display.
    // (Full implementation creates a wl_surface + zwlr_layer_surface_v1,
    //  sets the anchor based on panel_config.position, and draws the plugins.)
    //
    // The layer-shell setup:
    //   - anchor: top/bottom/left/right per config
    //   - size: panel_config.size
    //   - layer: Layer::Bottom (or Top if autohide is on)
    //   - keyboard interactivity: on-demand (for the launcher button)

    // Initialize plugins.
    let mut plugins = plugins::init(&config)?;

    // Connect to the compositor IPC socket for live updates.
    let ipc = noxl_ipc::control_socket_path();
    tracing::info!(path = %ipc.display(), "panel will connect to compositor IPC");

    // Event loop: render, handle input, receive IPC events.
    loop {
        // Render the panel.
        plugins.render(&config)?;

        // Poll IPC for workspace/window updates.
        // (In the full implementation this is an async select over the
        //  Wayland fd and the IPC socket.)

        // Sleep briefly to avoid busy-looping.
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }
}
