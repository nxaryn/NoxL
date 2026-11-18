//! Font rendering using `ab_glyph` (pure Rust, no system dependencies).
//!
//! Provides real glyph rasterization so NoxL applications can draw readable
//! text instead of placeholder blocks.
//!
//! Fonts are loaded via fontconfig (matching by family name like "Inter"
//! or "DejaVu Sans"). Glyphs are rasterized to 8-bit alpha masks and
//! composited into the pixel buffer.

use std::collections::HashMap;

use ab_glyph::{Font as AbFont, FontArc, FontVec, Glyph, Point, PxScale, ScaleFont};
use anyhow::{Context, Result};

/// A loaded font face.
pub struct Font {
    font: FontVec,
    /// Glyph cache: (char, size) -> bitmap.
    cache: HashMap<(char, u32), Option<GlyphBitmap>>,
    /// Current pixel size.
    size: f32,
}

/// A rasterized glyph.
#[derive(Debug, Clone)]
pub struct GlyphBitmap {
    pub width: i32,
    pub height: i32,
    pub left: i32,
    pub top: i32,
    pub advance: i32,
    /// 8-bit alpha mask (width * height bytes).
    pub buffer: Vec<u8>,
}

impl Font {
    /// Load a font by family name (e.g. "DejaVu Sans", "Inter").
    ///
    /// Uses fontconfig to find the font file, then `ab_glyph` to load it.
    pub fn load(family: &str) -> Result<Self> {
        let font_path = find_font_file(family)
            .context("fontconfig could not find a matching font")?;

        let data = std::fs::read(&font_path)
            .with_context(|| format!("failed to read font: {}", font_path.display()))?;

        let font = FontVec::try_from_vec(data)
            .context("failed to parse font")?;

        Ok(Self {
            font,
            cache: HashMap::new(),
            size: 13.0,
        })
    }

    /// Load the default sans-serif font.
    pub fn load_default() -> Result<Self> {
        Self::load("DejaVu Sans")
    }

    /// Load the default monospace font.
    pub fn load_mono() -> Result<Self> {
        Self::load("DejaVu Sans Mono")
    }

    /// Set the pixel size for this font.
    pub fn set_pixel_size(&mut self, size: u32) {
        self.size = size as f32;
        self.cache.clear();
    }

    /// Rasterize a character at the current pixel size.
    pub fn glyph(&mut self, c: char) -> Option<&GlyphBitmap> {
        let key = (c, self.size as u32);
        if !self.cache.contains_key(&key) {
            let bitmap = self.rasterize(c);
            self.cache.insert(key, bitmap);
        }
        self.cache.get(&key).and_then(|g| g.as_ref())
    }

    /// Rasterize a character.
    fn rasterize(&mut self, c: char) -> Option<GlyphBitmap> {
        let scale = PxScale::from(self.size);
        let scaled = self.font.as_scaled(scale);

        let glyph = Glyph {
            id: self.font.glyph_id(c),
            scale: scale.into(),
            position: Point { x: 0.0, y: 0.0 },
        };

        if glyph.id.0 == 0 {
            return None;
        }

        let glyph_id = glyph.id;
        let mut outline = self.font.outline_glyph(glyph)?;
        let bounds = outline.px_bounds();

        let width = bounds.width().ceil() as i32;
        let height = bounds.height().ceil() as i32;

        if width == 0 || height == 0 {
            let advance = scaled.h_advance(glyph_id);
            return Some(GlyphBitmap {
                width: 0,
                height: 0,
                left: 0,
                top: 0,
                advance: advance as i32,
                buffer: Vec::new(),
            });
        }

        let mut buffer = vec![0u8; (width * height) as usize];
        outline.draw(|x, y, alpha| {
            let px = x as i32;
            let py = y as i32;
            if px >= 0 && px < width && py >= 0 && py < height {
                let idx = (py * width + px) as usize;
                if idx < buffer.len() {
                    buffer[idx] = (alpha * 255.0) as u8;
                }
            }
        });

        let advance = scaled.h_advance(glyph_id) as i32;

        Some(GlyphBitmap {
            width,
            height,
            left: bounds.min.x as i32,
            top: bounds.max.y as i32,
            advance,
            buffer,
        })
    }

    /// Measure the width of a string in pixels.
    pub fn measure(&mut self, text: &str) -> i32 {
        let scale = PxScale::from(self.size);
        let scaled = self.font.as_scaled(scale);
        let mut width = 0.0;
        for c in text.chars() {
            let id = self.font.glyph_id(c);
            width += scaled.h_advance(id);
        }
        width as i32
    }
}

/// Find a font file by family name using fontconfig.
fn find_font_file(family: &str) -> Result<std::path::PathBuf> {
    use fontconfig::Fontconfig;

    let fc = Fontconfig::new().context("failed to init fontconfig")?;
    let mut pattern = fontconfig::Pattern::new(&fc);
    let family_cstr = std::ffi::CString::new(family).unwrap();
    let name_cstr = std::ffi::CString::new("family").unwrap();
    pattern.add_string(&name_cstr, &family_cstr);
    let matched = pattern.font_match();

    let path = matched.filename()
        .context("font has no filename")?
        .to_string();

    Ok(std::path::PathBuf::from(path))
}

/// Draw text into a pixel buffer.
///
/// `x`, `y` is the top-left corner of the text.
/// `r`, `g`, `b`, `a` is the text color.
pub fn draw_text(
    painter: &mut crate::Painter,
    font: &mut Font,
    text: &str,
    x: i32,
    y: i32,
    r: u8,
    g: u8,
    b: u8,
    a: u8,
) {
    let mut cx = x;
    for c in text.chars() {
        if let Some(glyph) = font.glyph(c) {
            let gx = cx + glyph.left;
            let gy = y - glyph.top;

            for row in 0..glyph.height {
                for col in 0..glyph.width {
                    let src_idx = (row * glyph.width + col) as usize;
                    if src_idx < glyph.buffer.len() {
                        let alpha = glyph.buffer[src_idx];
                        if alpha > 0 {
                            let px = gx + col;
                            let py = gy + row;
                            if px >= 0 && px < painter.width && py >= 0 && py < painter.height {
                                let offset = (py * painter.stride + px * 4) as usize;
                                if offset + 3 < painter.data.len() {
                                    let bg_b = painter.data[offset];
                                    let bg_g = painter.data[offset + 1];
                                    let bg_r = painter.data[offset + 2];
                                    let bg_a = painter.data[offset + 3];

                                    let fa = alpha as u32;
                                    let ba = bg_a as u32;
                                    let out_a = fa + ba * (255 - fa) / 255;

                                    if out_a > 0 {
                                        let fr = (r as u32 * fa + bg_r as u32 * ba * (255 - fa) / 255) / out_a;
                                        let fg = (g as u32 * fa + bg_g as u32 * ba * (255 - fa) / 255) / out_a;
                                        let fb = (b as u32 * fa + bg_b as u32 * ba * (255 - fa) / 255) / out_a;
                                        painter.data[offset] = fb as u8;
                                        painter.data[offset + 1] = fg as u8;
                                        painter.data[offset + 2] = fr as u8;
                                        painter.data[offset + 3] = out_a as u8;
                                    }
                                }
                            }
                        }
                    }
                }
            }

            cx += glyph.advance;
        }
    }
}
