//! Screenshot capture via wlr-screencopy.

use anyhow::{Context, Result};

/// Capture the screen.
///
/// In a full implementation this uses the `wlr-screencopy-unstable-v1`
/// protocol to capture the screen contents into a buffer, then saves it
/// as a PNG.
///
/// The stub documents the real implementation:
/// 1. Bind `zwlr_screencopy_manager_v1`
/// 2. Capture the output (or a region)
/// 3. Receive the pixel data via a buffer
/// 4. Save as PNG using the `image` crate

pub fn capture(mode: &str, output: &str) -> Result<()> {
    tracing::info!(mode, output, "capturing screenshot");

    // The real implementation:
    //
    //   let conn = Connection::connect_to_env()?;
    //   let (globals, mut event_queue) = registry_queue_init::<State>(&conn)?;
    //   let screencopy = globals.bind(&qh, 1..=3, ())?;
    //   let output = globals.bind(&qh, 1..=4, ())?;
    //
    //   // Capture the output.
    //   let frame = screencopy.capture_output(&output, &qh, ());
    //
    //   // On BufferDone event, save the pixels.
    //   // frame.buffer_done(...) -> save as PNG
    //
    //   // Save with the image crate:
    //   let img = image::RgbaImage::from_raw(w, h, pixels)?;
    //   img.save(output)?;

    // Stub: create a placeholder PNG so the command doesn't fail silently.
    let img = image::RgbaImage::new(1920, 1080);
    img.save(output).context("failed to save screenshot")?;

    tracing::info!(output, "screenshot saved (placeholder — screencopy not yet implemented)");
    Ok(())
}
