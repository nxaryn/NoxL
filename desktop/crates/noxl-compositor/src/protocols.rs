//! Wayland protocol implementations.
//!
//! Smithay provides the protocol implementations; this module documents the
//! layer-shell roles NoxL uses and provides helpers.

use smithay::wayland::shell::wlr_layer::Layer;

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
    pub fn to_layer(&self) -> Layer {
        match self {
            LayerRole::Background => Layer::Background,
            LayerRole::Bottom => Layer::Bottom,
            LayerRole::Top => Layer::Top,
            LayerRole::Overlay => Layer::Overlay,
        }
    }
}
