//! Shared-memory buffer rendering for the panel.
//!
//! Creates a `wl_shm` buffer, draws the panel background and plugin content
//! into it with a simple software rasterizer (no GPU dependency), and attaches
//! it to the layer surface.
//!
//! This keeps the panel dependency-light: it works on any system with a
//! Wayland compositor, without requiring OpenGL or a rendering toolkit.

use anyhow::{Context, Result};

use wayland_client::{
    protocol::{wl_buffer, wl_shm, wl_shm_pool, wl_surface},
    Connection, Dispatch, Proxy, QueueHandle,
};

use crate::panel::Panel;

use std::os::unix::io::AsFd;

/// A shared-memory buffer with pixel data.
pub struct ShmBuffer {
    pub buffer: wl_buffer::WlBuffer,
    pub width: i32,
    pub height: i32,
    pub stride: i32,
    pub data: Vec<u8>,
}

impl ShmBuffer {
    /// Create a new shared-memory buffer of the given size.
    pub fn new(
        shm: &wl_shm::WlShm,
        qh: &QueueHandle<Panel>,
        width: i32,
        height: i32,
    ) -> Result<Self> {
        let stride = width * 4;
        let size = (stride * height) as usize;

        // Create a memfd for the buffer data.
        let fd = create_memfd(size)?;
        let pool = shm.create_pool(fd.as_fd(), size as i32, qh, ());
        let buffer = pool.create_buffer(0, width, height, stride, wl_shm::Format::Argb8888, qh, ());
        pool.destroy();

        Ok(Self {
            buffer,
            width,
            height,
            stride,
            data: vec![0u8; size],
        })
    }

    /// Attach this buffer to a surface.
    pub fn attach(&self, surface: &wl_surface::WlSurface) {
        surface.attach(Some(&self.buffer), 0, 0);
        surface.damage_buffer(0, 0, self.width, self.height);
        surface.commit();
    }
}

/// Create an anonymous memory file descriptor of the given size.
fn create_memfd(size: usize) -> Result<std::os::unix::io::OwnedFd> {
    use std::io::Write;

    // Create a memfd via /dev/shm (portable, no libc memfd_create needed).
    let path = format!("/dev/shm/noxl-panel-{}", std::process::id());
    let file = std::fs::File::create(&path)?;
    file.set_len(size as u64)?;
    let mut f = file;
    f.write_all(&vec![0u8; size])?;
    f.sync_all()?;

    // Reopen read-write and keep the fd.
    let file = std::fs::OpenOptions::new().write(true).read(true).open(&path)?;
    let _ = std::fs::remove_file(&path);

    Ok(file.into())
}

/// Simple pixel drawing helpers.
pub struct Painter {
    pub width: i32,
    pub height: i32,
    pub stride: i32,
    pub data: Vec<u8>,
}

impl Painter {
    pub fn new(width: i32, height: i32) -> Self {
        let stride = width * 4;
        Self {
            width,
            height,
            stride,
            data: vec![0u8; (stride * height) as usize],
        }
    }

    /// Fill the entire buffer with a color (r, g, b, a as u8).
    pub fn fill(&mut self, r: u8, g: u8, b: u8, a: u8) {
        for y in 0..self.height {
            for x in 0..self.width {
                self.set_pixel(x, y, r, g, b, a);
            }
        }
    }

    /// Set a single pixel.
    pub fn set_pixel(&mut self, x: i32, y: i32, r: u8, g: u8, b: u8, a: u8) {
        if x < 0 || x >= self.width || y < 0 || y >= self.height {
            return;
        }
        let offset = (y * self.stride + x * 4) as usize;
        // ARGB8888 in little-endian: B, G, R, A
        self.data[offset] = b;
        self.data[offset + 1] = g;
        self.data[offset + 2] = r;
        self.data[offset + 3] = a;
    }

    /// Fill a rectangle.
    pub fn fill_rect(&mut self, x: i32, y: i32, w: i32, h: i32, r: u8, g: u8, b: u8, a: u8) {
        for py in y..(y + h) {
            for px in x..(x + w) {
                self.set_pixel(px, py, r, g, b, a);
            }
        }
    }
}

/// Parse a hex color string into (r, g, b, a) u8 values.
pub fn parse_hex(s: &str) -> (u8, u8, u8, u8) {
    let hex = s.trim().strip_prefix('#').unwrap_or(s.trim());
    let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0);
    let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0);
    let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0);
    let a = if hex.len() >= 8 {
        u8::from_str_radix(&hex[6..8], 16).unwrap_or(255)
    } else {
        255
    };
    (r, g, b, a)
}

impl Dispatch<wl_buffer::WlBuffer, ()> for Panel {
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

impl Dispatch<wl_shm_pool::WlShmPool, ()> for Panel {
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
