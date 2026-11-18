# Phase 4 — Applications: Status

**Phase:** 4 of 6 (Applications)
**Goal:** Native NoxL application ecosystem.

**Status:** ✅ All 7 applications compile on real Linux (Arch WSL2).

---

## Applications

| App | Crate | Features | Draws? |
|---|---|---|---|
| **Terminal** | `noxl-terminal` | Real PTY (portable-pty), vte parsing, tabs, OBSIDIAN theme | ✅ |
| **Files** | `noxl-files` | Real directory listing, navigation, history, delete/rename | ✅ |
| **Text** | `noxl-text` | Tabs, syntax mode detection, line numbers, cursor, save | ✅ |
| **Monitor** | `noxl-monitor` | Real CPU/RAM/uptime/process data (sysinfo) | ✅ |
| **Screenshot** | `noxl-screenshot` | Fullscreen/window/region capture, PNG output | ✅ (placeholder) |
| **Archive** | `noxl-archive` | ZIP/TAR/GZIP/TAR.GZ create/extract/list | ✅ |
| **Software** | `noxl-software` | pacman frontend, search/install/remove/update via pkexec | ✅ |

---

## Architecture

All applications follow the same pattern:
1. Create a `zwlr_layer_surface_v1` on the Top layer
2. Read the OBSIDIAN theme from `noxl-config`
3. Paint pixels into a `wl_shm` buffer via `noxl-render`
4. Attach and commit

This keeps them consistent with the desktop components and each other.

---

## Verified on Arch Linux (WSL2)

```
cargo build   → clean, all 7 binaries produced
```

Binaries:
```
noxl-terminal  noxl-files  noxl-text  noxl-monitor
noxl-screenshot  noxl-archive  noxl-software
```

---

## Known limitations

1. **Text rendering is block-based.** All apps draw text as filled rectangles
   (a placeholder for a real font rasterizer). The layout and content are real;
   the glyphs are not.
2. **Screenshot is a placeholder.** The `wlr-screencopy` protocol interface is
   documented but not implemented. It saves a blank PNG.
3. **No mouse input yet.** The apps create surfaces and draw, but don't
   handle pointer events (clicks, typing).
4. **Software center uses pacman directly.** The search/install/remove logic
   is real (via subprocess), but the UI doesn't wire button clicks to it yet.

---

## Next: Phase 5 — Installation

Phase 5 builds the graphical installer, Live ISO, disk management, bootloader
setup, and installation verification. See `docs/PHASE5.md`.