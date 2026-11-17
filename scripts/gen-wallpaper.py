#!/usr/bin/env python3
"""
Generate the NoxL OBSIDIAN default wallpaper.

Produces a 2560x1440 PNG using only the standard library (zlib + struct),
so it can run on any NoxL build host without Pillow.

The design is intentionally restrained: a near-black vertical gradient with a
single soft violet radial accent, matching the OBSIDIAN palette. No decorative
glow, no busy patterns.
"""

import struct
import zlib
import math
import sys

# OBSIDIAN palette
BG_TOP = (9, 9, 13)        # #09090D
BG_BOTTOM = (17, 17, 22)   # #111116
ACCENT = (180, 76, 255)    # #B44CFF
ACCENT_SOFT = (224, 180, 255)  # #E0B4FF

WIDTH = 2560
HEIGHT = 1440


def lerp(a, b, t):
    return a + (b - a) * t


def mix(c1, c2, t):
    return tuple(int(round(lerp(c1[i], c2[i], t))) for i in range(3))


def generate_wallpaper(width=WIDTH, height=HEIGHT):
    # Accent center (upper-right third), radius for falloff.
    cx, cy = int(width * 0.72), int(height * 0.34)
    max_dist = math.hypot(width, height)

    rows = []
    for y in range(height):
        # Vertical gradient factor.
        vt = y / (height - 1)
        base = mix(BG_TOP, BG_BOTTOM, vt)

        row = bytearray()
        row.append(0)  # PNG filter type 0 (None) for this scanline
        for x in range(width):
            # Distance-based accent falloff.
            d = math.hypot(x - cx, y - cy) / max_dist
            # Smooth radial falloff, strongest near center.
            falloff = max(0.0, 1.0 - d * 2.2)
            falloff = falloff * falloff  # ease

            # Blend base toward accent, then a touch toward soft highlight at core.
            col = mix(base, ACCENT, falloff * 0.55)
            core = max(0.0, 1.0 - d * 5.0)
            col = mix(col, ACCENT_SOFT, core * 0.25)

            row.extend(col)
        rows.append(bytes(row))

    raw = b"".join(rows)
    compressed = zlib.compress(raw, 9)
    return _wrap_png(compressed, width, height)


def _png_chunk(tag, data):
    chunk = tag + data
    crc = zlib.crc32(chunk) & 0xFFFFFFFF
    return struct.pack(">I", len(data)) + chunk + struct.pack(">I", crc)


def _wrap_png(compressed, width, height):
    sig = b"\x89PNG\r\n\x1a\n"
    ihdr = struct.pack(">IIBBBBB", width, height, 8, 2, 0, 0, 0)  # 8-bit RGB
    return (
        sig
        + _png_chunk(b"IHDR", ihdr)
        + _png_chunk(b"IDAT", compressed)
        + _png_chunk(b"IEND", b"")
    )


def main():
    out = sys.argv[1] if len(sys.argv) > 1 else "obsidian-default.png"
    png = generate_wallpaper()
    with open(out, "wb") as f:
        f.write(png)
    print(f"Wrote {out} ({len(png)} bytes, {WIDTH}x{HEIGHT})")


if __name__ == "__main__":
    main()