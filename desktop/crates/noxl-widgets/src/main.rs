//! NoxL widget engine.
//!
//! Renders the desktop widgets defined in `noxl-config`'s `WidgetsConfig`.
//! Each widget is a small functional component (CPU, RAM, clock, notes, etc.)
//! that reads real system information and draws it.
//!
//! Widgets are sandboxed: they run in this process and are given read-only
//! access to the data they need via the `WidgetContext`. They cannot execute
//! arbitrary commands or access the filesystem beyond their own state.

use anyhow::Context;
use clap::Parser;
use tracing_subscriber::EnvFilter;

mod builtin;
mod context;

#[derive(Debug, Parser)]
#[command(name = "noxl-widgets", about = "NoxL widget engine")]
struct Opts {
    #[arg(long, default_value = "info", env = "RUST_LOG")]
    log_level: String,
}

fn main() -> anyhow::Result<()> {
    let opts = Opts::parse();

    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new(&opts.log_level)),
        )
        .init();

    tracing::info!("NoxL widget engine starting");

    let config = noxl_config::NoxlConfig::load().context("failed to load config")?;

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .context("failed to build tokio runtime")?;

    rt.block_on(async move {
        // Create the widget context (system info provider).
        let ctx = context::WidgetContext::new();

        // Render each visible widget.
        for widget in &config.widgets.widgets {
            if !widget.visible {
                continue;
            }
            if let Some(render_fn) = builtin::get(&widget.widget_type) {
                match render_fn(&ctx, widget) {
                    Ok(_) => tracing::trace!(widget = %widget.widget_type, "rendered"),
                    Err(e) => tracing::error!(widget = %widget.widget_type, error = %e, "render failed"),
                }
            } else {
                tracing::warn!(widget = %widget.widget_type, "unknown widget type");
            }
        }

        // Event loop: update widgets periodically.
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        }
    })
}
