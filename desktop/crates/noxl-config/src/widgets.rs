//! Widget system configuration.
//!
//! Widgets are small, functional desktop components (CPU, clock, notes, etc.).
//! This module defines the layout model: which widgets exist, where they are,
//! how big they are, and whether they are visible. The widget runtime (Phase 3)
//! reads this to place and render widgets.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A widget instance placed on the desktop.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct WidgetInstance {
    /// Widget type identifier (e.g. "system.cpu", "productivity.clock").
    pub widget_type: String,
    /// Position in pixels from the top-left of the desktop area.
    pub x: i32,
    pub y: i32,
    /// Size in pixels.
    pub width: u32,
    pub height: u32,
    /// Opacity 0.0..1.0.
    pub opacity: f32,
    /// Whether the widget is visible.
    pub visible: bool,
    /// Which workspace this widget appears on (0 = all).
    pub workspace: u32,
    /// Per-widget settings (widget-specific keys).
    pub settings: HashMap<String, String>,
}

impl Default for WidgetInstance {
    fn default() -> Self {
        Self {
            widget_type: String::new(),
            x: 0,
            y: 0,
            width: 200,
            height: 120,
            opacity: 1.0,
            visible: true,
            workspace: 0,
            settings: HashMap::new(),
        }
    }
}

/// The full widget layout.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct WidgetsConfig {
    /// All widget instances.
    pub widgets: Vec<WidgetInstance>,
    /// Enable the widget system entirely.
    pub enabled: bool,
    /// Allow widgets to be moved/resized interactively.
    pub interactive: bool,
    /// Snap widgets to a grid.
    pub snap_to_grid: bool,
    /// Grid size in pixels (when snap_to_grid is true).
    pub grid_size: u32,
}

impl Default for WidgetsConfig {
    fn default() -> Self {
        Self {
            widgets: Vec::new(),
            enabled: true,
            interactive: true,
            snap_to_grid: false,
            grid_size: 16,
        }
    }
}

impl WidgetsConfig {
    /// Add or replace a widget instance (keyed by widget_type + position).
    pub fn upsert(&mut self, widget: WidgetInstance) {
        if let Some(existing) = self
            .widgets
            .iter_mut()
            .find(|w| w.widget_type == widget.widget_type && w.x == widget.x && w.y == widget.y)
        {
            *existing = widget;
        } else {
            self.widgets.push(widget);
        }
    }

    /// Remove a widget by type and position.
    pub fn remove(&mut self, widget_type: &str, x: i32, y: i32) {
        self.widgets
            .retain(|w| !(w.widget_type == widget_type && w.x == x && w.y == y));
    }
}
