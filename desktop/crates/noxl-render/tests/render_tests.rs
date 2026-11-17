//! Tests for the NoxL software renderer (pure pixel logic).

use noxl_render::*;

#[test]
fn parse_hex_6_digit() {
    let (r, g, b, a) = parse_hex("#B44CFF");
    assert_eq!(r, 0xB4);
    assert_eq!(g, 0x4C);
    assert_eq!(b, 0xFF);
    assert_eq!(a, 255);
}

#[test]
fn parse_hex_8_digit() {
    let (r, g, b, a) = parse_hex("#09090D80");
    assert_eq!(r, 0x09);
    assert_eq!(g, 0x09);
    assert_eq!(b, 0x0D);
    assert_eq!(a, 0x80);
}

#[test]
fn parse_hex_no_hash() {
    let (r, g, b, _) = parse_hex("111116");
    assert_eq!(r, 0x11);
    assert_eq!(g, 0x11);
    assert_eq!(b, 0x16);
}

#[test]
fn parse_hex_invalid_returns_black() {
    let (r, g, b, a) = parse_hex("zzz");
    assert_eq!((r, g, b, a), (0, 0, 0, 255));
}

#[test]
fn painter_fill_sets_every_pixel() {
    let mut p = Painter::new(4, 4);
    p.fill(10, 20, 30, 255);
    // Check first and last pixel.
    assert_eq!(&p.data[0..4], &[30, 20, 10, 255]); // BGRA
    let last = (4 * 4 - 1) * 4;
    assert_eq!(&p.data[last..last + 4], &[30, 20, 10, 255]);
}

#[test]
fn painter_fill_rect() {
    let mut p = Painter::new(10, 10);
    p.fill_rect(2, 2, 3, 3, 255, 0, 0, 255);
    // Pixel inside the rect.
    let offset = (2 * 10 + 2) * 4;
    assert_eq!(&p.data[offset..offset + 4], &[0, 0, 255, 255]);
    // Pixel outside the rect.
    let offset = (0 * 10 + 0) * 4;
    assert_eq!(&p.data[offset..offset + 4], &[0, 0, 0, 0]);
}

#[test]
fn painter_set_pixel_out_of_bounds_is_noop() {
    let mut p = Painter::new(4, 4);
    p.set_pixel(-1, 0, 255, 0, 0, 255);
    p.set_pixel(0, -1, 255, 0, 0, 255);
    p.set_pixel(4, 0, 255, 0, 0, 255);
    p.set_pixel(0, 4, 255, 0, 0, 255);
    // All pixels should still be zero.
    assert!(p.data.iter().all(|&b| b == 0));
}

#[test]
fn painter_hline_and_vline() {
    let mut p = Painter::new(10, 10);
    p.hline(1, 5, 4, 255, 255, 255, 255);
    // Check the horizontal line.
    for x in 1..5 {
        let offset = (5 * 10 + x) * 4;
        assert_eq!(&p.data[offset..offset + 4], &[255, 255, 255, 255]);
    }
    // Pixel just outside the line.
    let offset = (5 * 10 + 5) * 4;
    assert_eq!(&p.data[offset..offset + 4], &[0, 0, 0, 0]);

    p.vline(7, 1, 4, 0, 255, 0, 255);
    for y in 1..5 {
        let offset = (y * 10 + 7) * 4;
        assert_eq!(&p.data[offset..offset + 4], &[0, 255, 0, 255]);
    }
}

#[test]
fn painter_stroke_rect() {
    let mut p = Painter::new(10, 10);
    p.stroke_rect(1, 1, 6, 6, 255, 0, 0, 255);
    // Corner.
    let offset = (1 * 10 + 1) * 4;
    assert_eq!(&p.data[offset..offset + 4], &[0, 0, 255, 255]);
    // Center should be empty.
    let offset = (3 * 10 + 3) * 4;
    assert_eq!(&p.data[offset..offset + 4], &[0, 0, 0, 0]);
}

#[test]
fn blend_opaque_src() {
    let src = (255, 0, 0, 255);
    let dst = (0, 0, 255, 255);
    let (r, g, b, a) = blend(src, dst);
    assert_eq!((r, g, b, a), (255, 0, 0, 255));
}

#[test]
fn blend_transparent_src() {
    let src = (255, 0, 0, 0);
    let dst = (0, 0, 255, 255);
    let (r, g, b, a) = blend(src, dst);
    assert_eq!((r, g, b, a), (0, 0, 255, 255));
}

#[test]
fn blend_both_transparent() {
    let (r, g, b, a) = blend((0, 0, 0, 0), (0, 0, 0, 0));
    assert_eq!((r, g, b, a), (0, 0, 0, 0));
}

#[test]
fn painter_stride_matches_width() {
    let p = Painter::new(7, 3);
    assert_eq!(p.stride, 28);
    assert_eq!(p.data.len(), 28 * 3);
}
