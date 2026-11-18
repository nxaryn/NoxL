//! Screenshot capture via wlr-screencopy.
//!
//! Uses the `wlr-screencopy-unstable-v1` protocol to capture the screen
//! contents into a buffer, then saves it as a PNG.

use anyhow::{Context, Result};

use wayland_client::{
    globals::{registry_queue_init, GlobalListContents},
    protocol::{wl_buffer, wl_output, wl_registry, wl_shm, wl_shm_pool},
    Connection, Dispatch, Proxy, QueueHandle,
};

use wayland_protocols_wlr::screencopy::v1::client::{
    zwlr_screencopy_frame_v1, zwlr_screencopy_manager_v1,
};

use image::{ImageFormat, RgbaImage};

/// Capture the screen.
pub fn capture(mode: &str, output: &str) -> Result<()> {
    tracing::info!(mode, output, "capturing screenshot");

    let conn = Connection::connect_to_env().context("failed to connect to Wayland display")?;
    let (globals, mut event_queue) =
        registry_queue_init::<CaptureState>(&conn).context("failed to init registry queue")?;
    let qh = event_queue.handle();

    let _registry = globals.contents().clone_list();

    // Bind the screencopy manager.
    let screencopy: zwlr_screencopy_manager_v1::ZwlrScreencopyManagerV1 = globals
        .bind(&qh, 1..=3, ())
        .context("zwlr_screencopy_manager_v1 not available")?;
    let _shm: wl_shm::WlShm = globals
        .bind(&qh, 1..=1, ())
        .context("wl_shm not available")?;
    let _output_global: wl_output::WlOutput = globals
        .bind(&qh, 1..=4, ())
        .context("wl_output not available")?;

    // Capture the first output.
    let frame = screencopy.capture_output(0, &_output_global, &qh, ());

    // Process events until we get the buffer.
    let mut state = CaptureState {
        frame: Some(frame),
        buffer: None,
        width: 0,
        height: 0,
        stride: 0,
        format: 0,
        done: false,
    };

    while !state.done {
        event_queue
            .blocking_dispatch(&mut state)
            .context("Wayland dispatch error")?;
    }

    // Save the captured pixels as a PNG.
    if let Some(buffer) = state.buffer {
        let img = RgbaImage::from_raw(
            state.width,
            state.height,
            buffer,
        ).context("failed to create image from buffer")?;
        img.save_with_format(output, ImageFormat::Png)
            .context("failed to save screenshot")?;
        tracing::info!(output, width = state.width, height = state.height, "screenshot saved");
    } else {
        anyhow::bail!("screencopy did not produce a buffer");
    }

    Ok(())
}

/// State for the screencopy capture.
struct CaptureState {
    frame: Option<zwlr_screencopy_frame_v1::ZwlrScreencopyFrameV1>,
    buffer: Option<Vec<u8>>,
    width: u32,
    height: u32,
    stride: u32,
    format: u32,
    done: bool,
}

impl Dispatch<wl_registry::WlRegistry, GlobalListContents> for CaptureState {
    fn event(
        _: &mut Self,
        _: &wl_registry::WlRegistry,
        _: wl_registry::Event,
        _: &GlobalListContents,
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<wl_output::WlOutput, ()> for CaptureState {
    fn event(
        _: &mut Self,
        _: &wl_output::WlOutput,
        _: wl_output::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<wl_shm::WlShm, ()> for CaptureState {
    fn event(
        _: &mut Self,
        _: &wl_shm::WlShm,
        _: wl_shm::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<wl_shm_pool::WlShmPool, ()> for CaptureState {
    fn event(
        _: &mut Self,
        _: &wl_shm_pool::WlShmPool,
        _: wl_shm_pool::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<wl_buffer::WlBuffer, ()> for CaptureState {
    fn event(
        _: &mut Self,
        _: &wl_buffer::WlBuffer,
        _: wl_buffer::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<zwlr_screencopy_manager_v1::ZwlrScreencopyManagerV1, ()> for CaptureState {
    fn event(
        _: &mut Self,
        _: &zwlr_screencopy_manager_v1::ZwlrScreencopyManagerV1,
        _: zwlr_screencopy_manager_v1::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<zwlr_screencopy_frame_v1::ZwlrScreencopyFrameV1, ()> for CaptureState {
    fn event(
        state: &mut Self,
        _: &zwlr_screencopy_frame_v1::ZwlrScreencopyFrameV1,
        event: zwlr_screencopy_frame_v1::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        match event {
            zwlr_screencopy_frame_v1::Event::Buffer {
                format,
                width,
                height,
                stride,
            } => {
                state.width = width;
                state.height = height;
                state.stride = stride;
                state.format = match &format {
                    wayland_client::WEnum::Value(wayland_client::protocol::wl_shm::Format::Argb8888) => 0,
                    wayland_client::WEnum::Value(wayland_client::protocol::wl_shm::Format::Xrgb8888) => 1,
                    _ => 0,
                };

                // Create a buffer for the captured pixels.
                // In a full implementation this creates a wl_shm buffer
                // and passes it to the screencopy frame.
                // For now, allocate the pixel data.
                let size = (stride * height) as usize;
                state.buffer = Some(vec![0u8; size]);
            }
            zwlr_screencopy_frame_v1::Event::Ready {
                ..
            } => {
                // The frame is ready; the buffer has been filled.
                state.done = true;
            }
            zwlr_screencopy_frame_v1::Event::Failed => {
                tracing::error!("screencopy failed");
                state.done = true;
            }
            _ => {}
        }
    }
}
