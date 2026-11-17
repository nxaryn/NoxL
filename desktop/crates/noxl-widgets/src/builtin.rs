//! Built-in widget implementations.
//!
//! Each widget reads from the `WidgetContext` and produces a `WidgetRender`
//! (text + optional progress). The actual pixel drawing is done by the
//! compositor; this crate produces the data.

use crate::context::WidgetContext;
use noxl_config::WidgetInstance;

/// What a widget renders.
#[derive(Debug, Clone)]
pub struct WidgetRender {
    /// Primary text (e.g. "42%").
    pub text: String,
    /// Optional progress 0.0 - 1.0 (for usage bars).
    pub progress: Option<f32>,
    /// Optional secondary text (e.g. "3.2 GB / 8.0 GB").
    pub detail: Option<String>,
}

/// A built-in widget.
pub struct BuiltinWidget {
    pub widget_type: &'static str,
    pub render: fn(&WidgetContext, &WidgetInstance) -> anyhow::Result<WidgetRender>,
}

/// All built-in widgets.
pub static BUILTIN_WIDGETS: &[BuiltinWidget] = &[
    BuiltinWidget { widget_type: "system.cpu", render: render_cpu },
    BuiltinWidget { widget_type: "system.memory", render: render_memory },
    BuiltinWidget { widget_type: "system.uptime", render: render_uptime },
    BuiltinWidget { widget_type: "productivity.clock", render: render_clock },
];

/// Get a widget renderer by type.
pub fn get(widget_type: &str) -> Option<fn(&WidgetContext, &WidgetInstance) -> anyhow::Result<WidgetRender>> {
    BUILTIN_WIDGETS
        .iter()
        .find(|w| w.widget_type == widget_type)
        .map(|w| w.render)
}

fn render_cpu(ctx: &WidgetContext, _inst: &WidgetInstance) -> anyhow::Result<WidgetRender> {
    let usage = ctx.cpu_usage();
    Ok(WidgetRender {
        text: format!("{:.0}%", usage),
        progress: Some(usage / 100.0),
        detail: Some(format!("{} CPUs", ctx.cpu_count())),
    })
}

fn render_memory(ctx: &WidgetContext, _inst: &WidgetInstance) -> anyhow::Result<WidgetRender> {
    let used = ctx.used_memory();
    let total = ctx.total_memory();
    let pct = if total > 0 { (used as f64 / total as f64) * 100.0 } else { 0.0 };
    Ok(WidgetRender {
        text: format!("{:.0}%", pct),
        progress: Some((pct / 100.0) as f32),
        detail: Some(format_bytes(used) + " / " + &format_bytes(total)),
    })
}

fn render_uptime(ctx: &WidgetContext, _inst: &WidgetInstance) -> anyhow::Result<WidgetRender> {
    let secs = ctx.uptime();
    let days = secs / 86400;
    let hours = (secs % 86400) / 3600;
    let mins = (secs % 3600) / 60;
    Ok(WidgetRender {
        text: format!("{}d {}h {}m", days, hours, mins),
        progress: None,
        detail: None,
    })
}

fn render_clock(_ctx: &WidgetContext, _inst: &WidgetInstance) -> anyhow::Result<WidgetRender> {
    let now = chrono::Local::now();
    Ok(WidgetRender {
        text: now.format("%H:%M").to_string(),
        progress: None,
        detail: Some(now.format("%A, %B %d").to_string()),
    })
}

fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    if bytes >= GB {
        format!("{:.1} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.0} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.0} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}
