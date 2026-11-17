//! Theme model — the OBSIDIAN design language as data.
//!
//! A `Theme` is a complete description of colors, typography, shapes, and
//! effects. The compositor, panel, launcher, and applications all render from
//! this struct, so changing a value here updates the whole desktop consistently
//! without a restart.

use serde::{Deserialize, Serialize};

/// A complete theme definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Theme {
    /// Theme name (e.g. "OBSIDIAN").
    pub name: String,
    /// Theme version (for forward-compat / migration).
    pub version: u32,
    /// Human description.
    pub description: String,
    /// "dark", "light", or "custom".
    pub color_mode: ColorMode,
    /// Color palette.
    pub colors: Colors,
    /// Typography.
    pub typography: Typography,
    /// Shape metrics (radii, spacing).
    pub shape: Shape,
    /// Effects (blur, shadows, animations).
    pub effects: Effects,
}

impl Default for Theme {
    fn default() -> Self {
        Self::obsidian()
    }
}

impl Theme {
    /// The built-in OBSIDIAN theme.
    pub fn obsidian() -> Self {
        Self {
            name: "OBSIDIAN".into(),
            version: 1,
            description: "NoxL default theme. Minimalistic, futuristic, elegant."
                .into(),
            color_mode: ColorMode::Dark,
            colors: Colors::obsidian(),
            typography: Typography::default(),
            shape: Shape::default(),
            effects: Effects::default(),
        }
    }

    /// Load a theme from a JSON file (themes are distributed as JSON).
    pub fn from_json_str(s: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(s)
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ColorMode {
    Dark,
    Light,
    Custom,
}

/// The OBSIDIAN color palette.
///
/// Every color is a hex string like "#09090D". Components parse these into
/// RGBA as needed. Keeping them as strings makes the config human-readable and
/// matches the design spec exactly.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Colors {
    pub bg_primary: String,
    pub bg_secondary: String,
    pub bg_elevated: String,
    pub accent_primary: String,
    pub accent_secondary: String,
    pub accent_soft: String,
    pub text_primary: String,
    pub text_secondary: String,
    pub border: String,
}

impl Default for Colors {
    fn default() -> Self {
        Self::obsidian()
    }
}

impl Colors {
    pub fn obsidian() -> Self {
        Self {
            bg_primary: "#09090D".into(),
            bg_secondary: "#111116".into(),
            bg_elevated: "#181820".into(),
            accent_primary: "#B44CFF".into(),
            accent_secondary: "#7028D9".into(),
            accent_soft: "#E0B4FF".into(),
            text_primary: "#F2EFF7".into(),
            text_secondary: "#96919F".into(),
            border: "#292532".into(),
        }
    }

    /// Parse a hex color string into (r, g, b, a) floats in 0.0..=1.0.
    /// Accepts "#RRGGBB" and "#RRGGBBAA".
    pub fn parse_hex(s: &str) -> Option<(f32, f32, f32, f32)> {
        let s = s.trim();
        let hex = s.strip_prefix('#').unwrap_or(s);
        let (r, g, b, a) = match hex.len() {
            6 => {
                let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
                let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
                let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
                (r, g, b, 255)
            }
            8 => {
                let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
                let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
                let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
                let a = u8::from_str_radix(&hex[6..8], 16).ok()?;
                (r, g, b, a)
            }
            _ => return None,
        };
        Some((
            r as f32 / 255.0,
            g as f32 / 255.0,
            b as f32 / 255.0,
            a as f32 / 255.0,
        ))
    }
}

/// Typography settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Typography {
    pub font_sans: String,
    pub font_mono: String,
    pub font_size_base: u32,
    pub font_size_small: u32,
    pub font_size_large: u32,
}

impl Default for Typography {
    fn default() -> Self {
        Self {
            font_sans: "Inter".into(),
            font_mono: "JetBrains Mono".into(),
            font_size_base: 13,
            font_size_small: 11,
            font_size_large: 18,
        }
    }
}

/// Shape metrics.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Shape {
    pub radius_panel: u32,
    pub radius_window: u32,
    pub radius_control: u32,
    pub spacing_unit: u32,
}

impl Default for Shape {
    fn default() -> Self {
        Self {
            radius_panel: 8,
            radius_window: 10,
            radius_control: 6,
            spacing_unit: 4,
        }
    }
}

/// Effects.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Effects {
    pub blur: bool,
    pub blur_radius: u32,
    pub shadows: bool,
    pub animations: bool,
    pub animation_duration_ms: u32,
}

impl Default for Effects {
    fn default() -> Self {
        Self {
            blur: true,
            blur_radius: 24,
            shadows: true,
            animations: true,
            animation_duration_ms: 160,
        }
    }
}
