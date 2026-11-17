//! NoxL shared software renderer.
//!
//! Provides pixel drawing into `wl_shm` buffers so any NoxL client (panel,
//! launcher, notifications, lock screen) can render without a GPU or toolkit.
//!
//! The renderer is deliberately minimal: fill, rectangle, and text-free
//! primitives. Text rendering is left to the client (which can use a font
//! rasterizer if needed).

use std::os::unix::io::AsFd;

use anyhow::{Context, Result};

use wayland_client::{
    protocol::{wl_buffer, wl_shm, wl_shm_pool, wl_surface},
    Connection, Dispatch, Proxy, QueueHandle,
};

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
    ///
    /// `state` must implement `Dispatch<WlShm, ()>`, `Dispatch<WlShmPool, ()>`,
    /// and `Dispatch<WlBuffer, ()>`.
    pub fn new<State>(
        shm: &wl_shm::WlShm,
        qh: &QueueHandle<State>,
        width: i32,
        height: i32,
    ) -> Result<Self>
    where
        State: Dispatch<wl_shm::WlShm, ()>
            + Dispatch<wl_shm_pool::WlShmPool, ()>
            + Dispatch<wl_buffer::WlBuffer, ()>
            + 'static,
    {
        let stride = width * 4;
        let size = (stride * height) as usize;

        let fd = create_memfd(size)?;
        let pool = shm.create_pool(fd.as_fd(), size as i32, qh, ());
        let buffer = pool.create_buffer(
            0,
            width,
            height,
            stride,
            wl_shm::Format::Argb8888,
            qh,
            (),
        );
        pool.destroy();

        Ok(Self {
            buffer,
            width,
            height,
            stride,
            data: vec![0u8; size],
        })
    }

    /// Attach this buffer to a surface and commit.
    pub fn attach(&self, surface: &wl_surface::WlSurface) {
        surface.attach(Some(&self.buffer), 0, 0);
        surface.damage_buffer(0, 0, self.width, self.height);
        surface.commit();
    }
}

/// Create an anonymous memory file descriptor of the given size.
fn create_memfd(size: usize) -> Result<std::os::unix::io::OwnedFd> {
    use std::io::Write;

    let path = format!("/dev/shm/noxl-render-{}", std::process::id());
    let file = std::fs::File::create(&path).context("failed to create memfd file")?;
    file.set_len(size as u64)?;
    let mut f = file;
    f.write_all(&vec![0u8; size])?;
    f.sync_all()?;

    let file = std::fs::OpenOptions::new()
        .write(true)
        .read(true)
        .open(&path)?;
    let _ = std::fs::remove_file(&path);

    Ok(file.into())
}

/// A software painter that draws into a pixel buffer.
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

    /// Fill the entire buffer with a color.
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

    /// Draw a 1px horizontal line.
    pub fn hline(&mut self, x: i32, y: i32, w: i32, r: u8, g: u8, b: u8, a: u8) {
        self.fill_rect(x, y, w, 1, r, g, b, a);
    }

    /// Draw a 1px vertical line.
    pub fn vline(&mut self, x: i32, y: i32, h: i32, r: u8, g: u8, b: u8, a: u8) {
        self.fill_rect(x, y, 1, h, r, g, b, a);
    }

    /// Draw a rectangle border.
    pub fn stroke_rect(&mut self, x: i32, y: i32, w: i32, h: i32, r: u8, g: u8, b: u8, a: u8) {
        self.hline(x, y, w, r, g, b, a);
        self.hline(x, y + h - 1, w, r, g, b, a);
        self.vline(x, y, h, r, g, b, a);
        self.vline(x + w - 1, y, h, r, g, b, a);
    }
}

/// Parse a hex color string into (r, g, b, a) u8 values.
///
/// Accepts "#RRGGBB" and "#RRGGBBAA".
pub fn parse_hex(s: &str) -> (u8, u8, u8, u8) {
    let hex = s.trim().strip_prefix('#').unwrap_or(s.trim());
    if hex.len() < 6 {
        return (0, 0, 0, 255);
    }
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

/// Blend two colors (src over dst) with alpha.
pub fn blend(src: (u8, u8, u8, u8), dst: (u8, u8, u8, u8)) -> (u8, u8, u8, u8) {
    let sa = src.3 as u32;
    let da = dst.3 as u32;
    let out_a = sa + da * (255 - sa) / 255;
    if out_a == 0 {
        return (0, 0, 0, 0);
    }
    let r = ((src.0 as u32 * sa + dst.0 as u32 * da * (255 - sa) / 255) / out_a) as u8;
    let g = ((src.1 as u32 * sa + dst.1 as u32 * da * (255 - sa) / 255) / out_a) as u8;
    let b = ((src.2 as u32 * sa + dst.2 as u32 * da * (255 - sa) / 255) / out_a) as u8;
    (r, g, b, out_a as u8)
}
