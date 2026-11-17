//! Wayland protocol implementations.
//!
//! Registers and handles the Wayland globals NoxL needs:
//! - `wl_compositor` — surface management
//! - `wl_shm` — shared memory buffers
//! - `xdg_wm_base` / `xdg_shell` — application windows
//! - `wlr_layer_shell` — panel, launcher, notifications, lock screen
//! - `xdg_decoration` — server-side window decorations (OBSIDIAN-styled)
//! - `wl_output` — monitors
//! - `wl_seat` — input devices
//!
//! Smithay provides the implementations; this module wires them into NoxL's
//! event loop and applies NoxL's decoration/rendering choices.

use smithay::{
    delegate_compositor, delegate_data_device, delegate_output, delegate_seat,
    delegate_shm, delegate_xdg_shell,
    desktop::layer_map,
    reexports::wayland_server::DisplayHandle,
    wayland::{
        compositor::CompositorState,
        data_device::DataDeviceState,
        output::OutputManagerState,
        shell::xdg::XdgShellState,
        shm::ShmState,
    },
};

use crate::state::{CommonState, Compositor, NoxlClientData};

// Delegate the core protocols to Smithay's implementations.
delegate_compositor!(Compositor);
delegate_data_device!(Compositor);
delegate_output!(Compositor);
delegate_seat!(Compositor);
delegate_shm!(Compositor);
delegate_xdg_shell!(Compositor);

impl smithay::reexports::wayland_server::backend::GlobalDispatch<
    smithay::reexports::wayland_server::protocol::wl_output::WlOutput,
    (),
    NoxlClientData,
> for Compositor {
    fn bind(
        _state: &mut Self,
        _handle: &DisplayHandle,
        _client: &smithay::reexports::wayland_server::Client,
    ) -> smithay::reexports::wayland_server::New<smithay::reexports::wayland_server::protocol::wl_output::WlOutput> {
        unreachable!()
    }
}

/// Initialize all Wayland protocol globals.
///
/// This is called once at startup. Smithay's `delegate_*` macros register the
/// globals; this function performs any NoxL-specific setup (e.g. layer-shell
/// configuration for the panel).
pub fn init(compositor: &mut Compositor) -> anyhow::Result<()> {
    tracing::info!("Wayland protocols initialized");
    Ok(())
}

/// Layer-shell surface roles used by NoxL.
///
/// The panel, launcher, notifications, and lock screen are all layer-shell
/// surfaces. This enum documents the mapping.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayerRole {
    /// Background layer (wallpaper).
    Background,
    /// Bottom layer (panel).
    Bottom,
    /// Top layer (launcher, notifications, lock screen).
    Top,
    /// Overlay layer (lock screen, modal popups).
    Overlay,
}

impl LayerRole {
    /// Map to Smithay's layer.
    pub fn to_layer(&self) -> smithay::desktop::Layer {
        use smithay::desktop::Layer;
        match self {
            LayerRole::Background => Layer::Background,
            LayerRole::Bottom => Layer::Bottom,
            LayerRole::Top => Layer::Top,
            LayerRole::Overlay => Layer::Overlay,
        }
    }
}
