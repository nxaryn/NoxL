//! Shared-memory buffer rendering for the panel.
//!
//! Re-exports the shared NoxL renderer so the panel can use it directly.

pub use noxl_render::{draw_text, parse_hex, Font, Painter, ShmBuffer};
