# Phase 2 — Desktop: Status

**Phase:** 2 of 6 (Desktop)
**Goal:** A functional NoxL desktop: Wayland compositor, window management,
panel, application launcher, and notification centre.

**Status:** ✅ Compiles and passes tests on real Linux (Arch WSL2).
Build/boot verification on a VM is pending (see `docs/TESTING.md`).

---

## What Phase 2 delivers

A multi-process NoxL desktop built in Rust on Smithay 0.7:

| Component | Crate | Role | Draws pixels? |
|---|---|---|---|
| **Compositor** | `noxl-compositor` | Wayland display, WM, rendering, input, IPC, layer-shell | ✅ (windows + layers) |
| **Panel** | `noxl-panel` | Taskbar on the bottom layer | ✅ (OBSIDIAN bg + accent) |
| **Launcher** | `noxl-launcher` | App launcher on the top layer | ✅ (OBSIDIAN bg + border) |
| **Notifications** | `noxl-notifications` | Notification centre on the overlay layer | ✅ (OBSIDIAN bg + accent) |
| **Shell** | `noxl-shell` | Session entry point | — |
| **Config** | `noxl-config` | Single source of truth (TOML, OBSIDIAN) | — |
| **IPC** | `noxl-ipc` | Control socket (JSON lines) | — |
| **Render** | `noxl-render` | Shared software renderer (wl_shm buffers) | — |

---

## Verified on Arch Linux (WSL2)

```
cargo build   → clean, all 8 crates + 7 binaries
cargo test    → 48 tests passing, 0 failing
```

| Crate | Tests | Covers |
|---|---|---|
| `noxl-config` | 11 | OBSIDIAN defaults, hex parsing, TOML roundtrip, partial-config defaults, keybindings, widget upsert/remove |
| `noxl-ipc` | 8 | Frame codec, command/event roundtrips, validation |
| `noxl-launcher` | 9 | Search ranking, case insensitivity |
| `noxl-notifications` | 7 | History, unread, JSON roundtrip |
| `noxl-render` | 13 | Pixel drawing, hex parsing, alpha blending |

---

## Architecture

```
┌─────────────────────────────────────────────────────────┐
│ noxl-shell (session supervisor)                         │
│   ├─ spawns noxl-compositor                             │
│   ├─ waits for Wayland socket                           │
│   ├─ spawns noxl-panel, noxl-notifications              │
│   └─ supervises / shuts down                            │
└─────────────────────────────────────────────────────────┘
        │
        ▼
┌─────────────────────────────────────────────────────────┐
│ noxl-compositor (Smithay 0.7 Wayland compositor)        │
│   - wl_compositor, wl_shm, xdg_shell, layer_shell       │
│   - window management (focus, maximize, snap, tile)     │
│   - input (keyboard/pointer, config-driven keybindings) │
│   - rendering (OBSIDIAN bg, windows, layer surfaces)    │
│   - IPC control socket (JSON lines)                     │
└─────────────────────────────────────────────────────────┘
        │ Wayland (layer-shell)          │ IPC socket
        ▼                                ▼
┌──────────────────┐  ┌──────────────┐  ┌─────────────────┐
│ noxl-panel       │  │ noxl-launcher│  │ noxl-notifications│
│ (Bottom layer)   │  │ (Top layer)  │  │ (Overlay layer) │
│ draws OBSIDIAN   │  │ draws        │  │ draws OBSIDIAN  │
│ bg + accent      │  │ OBSIDIAN bg  │  │ bg + accent     │
└──────────────────┘  └──────────────┘  └─────────────────┘
        │                    │                   │
        └──────── both read ─┴───────────────────┘
                    │
                    ▼
            noxl-config (TOML)
            ~/.config/noxl/config.toml
```

---

## What is implemented

### `noxl-compositor`
- Smithay 0.7 compositor with proper handler impls (`CompositorHandler`,
  `XdgShellHandler`, `SeatHandler`, `ShmHandler`, `DataDeviceHandler`,
  `WlrLayerShellHandler`) and `delegate_*` macros.
- Window management: focus, maximize, minimize, close, snap (left/right/
  quarter/corners), drag-to-snap zone detection, app spawning via `gtk-launch`.
- Input: keyboard events → keybinding combos (from config) → action dispatch.
- IPC server: accepts clients, parses JSON commands, broadcasts events.
- Configurable workspaces (default 4), workspace focus/next/prev.
- Rendering: OBSIDIAN background clear, window surfaces, layer surfaces
  in the correct z-order (background → windows → bottom → top → overlay).

### `noxl-panel`
- Creates a `zwlr_layer_surface_v1` on the `Bottom` layer.
- Anchor and size derived from config (position, size, exclusive zone).
- On Configure: paints the OBSIDIAN `bg_secondary` background and a 1px
  `accent_primary` line at the top edge into a `wl_shm` buffer.
- Plugin structure: Launcher, Taskbar, Workspaces, Tray, StatusIcons, Clock,
  Notifications — each independently toggleable and ordered by config.

### `noxl-launcher`
- Reads **real** `.desktop` files from `/usr/share/applications`,
  `~/.local/share/applications`, `/usr/local/share/applications`.
- Parses name, icon, categories, Exec, NoDisplay, Hidden.
- Search with ranking (exact > prefix > contains > exec > category).
- Creates a `zwlr_layer_surface_v1` on the `Top` layer with exclusive
  keyboard focus.
- On Configure: paints the OBSIDIAN `bg_elevated` background with a border.

### `noxl-notifications`
- `org.freedesktop.Notifications` D-Bus service (zbus).
- Persistent history (`~/.local/share/noxl/notifications/history.json`).
- Unread tracking, mark-read, remove.
- Creates a `zwlr_layer_surface_v1` on the `Overlay` layer (top-right).
- On Configure: paints the OBSIDIAN `bg_elevated` background with an
  accent left border.

### `noxl-render`
- Shared software renderer used by all three clients.
- `ShmBuffer`: memfd-backed `wl_shm` pool + buffer.
- `Painter`: fill, fill_rect, hline, vline, stroke_rect.
- `parse_hex`, `blend`: color helpers.
- 13 unit tests covering all drawing primitives.

### `noxl-shell`
- Session entry point: spawns compositor, waits for Wayland socket, spawns
  panel + notifications, supervises, shuts down cleanly.

---

## Known limitations

1. **Content is a solid color, not the full UI.** The panel, launcher, and
   notification centre draw their OBSIDIAN backgrounds and accents, but the
   plugin content (taskbar icons, clock text, app grid, notification text)
   is not drawn yet. That requires a font rasterizer and icon loading.
2. **Input keycode mapping is a subset.** Uses a hardcoded evdev keycode
   table. A full implementation uses `xkbcommon` for layout-aware keysym
   translation.
3. **No xdg-decoration.** Server-side OBSIDIAN title bars are not implemented.
4. **Tray/status icons need D-Bus.** Plugin structure exists; the actual
   `StatusNotifierItem` host and NetworkManager/PipeWire/UPower clients are
   not wired.
5. **Standalone (non-nested) mode not implemented.** The compositor uses the
   winit backend (nested). Standalone mode needs DRM/libinput/udev/seat
   (Phase 6).
6. **Not yet booted in a VM.** Must be built and run on a Linux host with
   Wayland to confirm the desktop appears. See `docs/TESTING.md`.

---

## Next: Phase 3 — Customisation

Phase 3 builds the theme engine, wallpaper manager, panel customisation UI,
launcher customisation, widget engine, lock screen, and preset management.
See `docs/PHASE3.md` (to be created at the start of Phase 3).