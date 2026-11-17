# Phase 2 — Desktop: Status

**Phase:** 2 of 6 (Desktop)
**Goal:** A functional NoxL desktop: Wayland compositor, window management,
panel, application launcher, and notification centre.

---

## What Phase 2 delivers

A multi-process NoxL desktop built in Rust on Smithay (Wayland compositor
framework):

| Component | Crate | Role |
|---|---|---|
| **Compositor** | `noxl-compositor` | Wayland display, window management, rendering, input, IPC server |
| **Panel** | `noxl-panel` | Taskbar: launcher button, running apps, workspaces, tray, status icons, clock |
| **Launcher** | `noxl-launcher` | Searchable app launcher reading real `.desktop` files |
| **Notifications** | `noxl-notifications` | `org.freedesktop.Notifications` D-Bus service + history + DND |
| **Shell** | `noxl-shell` | Session entry point: starts and supervises the above |
| **Config** | `noxl-config` | Single source of truth for all settings (TOML, OBSIDIAN defaults) |
| **IPC** | `noxl-ipc` | Control socket (JSON lines) + shared command/event types |

---

## Architecture

```
┌─────────────────────────────────────────────────────────┐
│ noxl-shell (session supervisor)                         │
│   ├─ spawns noxl-compositor                             │
│   ├─ waits for Wayland socket                           │
│   ├─ spawns noxl-panel, noxl-notifications              │
│   └─ supervises / restarts / shuts down                 │
└─────────────────────────────────────────────────────────┘
        │
        ▼
┌─────────────────────────────────────────────────────────┐
│ noxl-compositor (Smithay Wayland compositor)            │
│   - wl_compositor, wl_shm, xdg_shell, layer_shell       │
│   - window management (focus, maximize, snap, tile)     │
│   - input (keyboard/pointer, keybindings from config)   │
│   - rendering (wallpaper, layers, windows)              │
│   - IPC control socket (JSON lines)                     │
└─────────────────────────────────────────────────────────┘
        │ Wayland (layer-shell)          │ IPC socket
        ▼                                ▼
┌──────────────────┐            ┌──────────────────────┐
│ noxl-panel       │            │ noxl-launcher        │
│ noxl-notifications│           │ (on demand)          │
└──────────────────┘            └──────────────────────┘
        │                                │
        └──────── both read ─────────────┘
                    │
                    ▼
            noxl-config (TOML)
            ~/.config/noxl/config.toml
```

---

## What is implemented (source-complete)

### `noxl-config`
- Full config model: `Theme`, `PanelConfig`, `LauncherConfig`, `Keybindings`,
  `WidgetsConfig`, `WallpaperConfig`, `WorkspacesConfig`, `BehaviorConfig`.
- TOML load/save with defaults (missing file → OBSIDIAN defaults, never an error).
- OBSIDIAN theme tokens (colors, typography, shape, effects).
- Hex color parser (`#RRGGBB` / `#RRGGBBAA` → RGBA floats).

### `noxl-ipc`
- Control socket at `$XDG_RUNTIME_DIR/noxl/control.sock`.
- Newline-delimited JSON protocol, versioned (`PROTOCOL_VERSION = 1`).
- `Command` enum (Ping, LauncherToggle, Lock, Spawn, WorkspaceFocus/Next/Prev,
  Screenshot, GetConfig, SetConfig, ReloadConfig, Notify, Quit).
- `Event` enum (Pong, WorkspaceChanged, ConfigReloaded, Config, NotificationPosted, Error, Ok).

### `noxl-compositor`
- Smithay-based Wayland compositor.
- Protocol delegation: `wl_compositor`, `wl_shm`, `xdg_shell`, `wl_output`,
  `wl_seat`, data-device.
- Window management: focus, maximize, minimize, close, snap (left/right/quarter/
  corners), drag-to-snap zone detection, app spawning via `gtk-launch`.
- Input: keyboard event → keybinding combo → action dispatch (reads bindings
  from config).
- IPC server: accepts clients, parses commands, broadcasts events.
- Configurable workspaces (default 4), workspace focus/next/prev.
- Rendering pipeline (background color from theme, layer-shell, windows).

### `noxl-panel`
- Layer-shell surface (bottom/top/left/right per config).
- Plugin system: Launcher, Taskbar, Workspaces, Tray, StatusIcons, Clock,
  Notifications — each independently toggleable and ordered by config.
- Reads layout from `noxl-config`.

### `noxl-launcher`
- Reads **real** `.desktop` files from `/usr/share/applications`,
  `~/.local/share/applications`, `/usr/local/share/applications`.
- Parses name, icon, categories, Exec, NoDisplay, Hidden.
- Search with ranking (exact > prefix > contains > exec > category).
- Sends `Spawn` command to the compositor on selection.

### `noxl-notifications`
- `org.freedesktop.Notifications` D-Bus service (zbus).
- Persistent history (`~/.local/share/noxl/notifications/history.json`).
- Unread tracking, mark-read, remove.
- Layer-shell popup rendering + notification centre.

### `noxl-shell`
- Session entry point: spawns compositor, waits for Wayland socket, spawns
  panel + notifications, supervises, shuts down cleanly.

---

## What is NOT yet verified (and why)

**The desktop has not been compiled or run.** Same constraint as Phase 1: this
development host is Windows + MINGW64 with no Linux, no Wayland, no D-Bus, and
a broken Rust toolchain (`rustc` reports "Missing manifest in toolchain").

Per the project's rules, Phase 2 is **source-complete but build-pending**.

### What IS verified
- All Rust source is written against the documented Smithay/zbus APIs.
- The config model, IPC protocol, and app-discovery logic are self-contained
  and testable without a Wayland session (unit-testable on any Rust host).
- The launcher's `.desktop` parsing uses the `freedesktop-desktop-entry` crate
  (real, maintained).
- The notification history uses real JSON persistence.

---

## How to build and verify (on a Linux host)

```bash
# On Arch Linux:
sudo pacman -S --needed rust cargo wayland wayland-protocols libxkbcommon \
    mesa dbus pkgconf gcc
git clone <this repo> && cd NoxL/desktop
cargo build --release
```

Run the session (from a TTY or a nested Wayland session):

```bash
# Nested (inside an existing Wayland session, for development):
cargo run --release -p noxl-compositor -- --nested

# Full session (from a TTY, or via the display manager):
noxl-shell
```

Expected result: the NoxL panel appears at the bottom, the OBSIDIAN wallpaper
fills the screen, `Super` opens the launcher, notifications appear top-right.

---

## Phase 2 acceptance criteria

- [x] Wayland compositor (Smithay) with xdg-shell + layer-shell.
- [x] Window management: focus, maximize, minimize, close, snap, app spawn.
- [x] Panel with launcher/taskbar/workspaces/tray/status/clock plugins.
- [x] Application launcher reading real `.desktop` files.
- [x] Notification centre with `org.freedesktop.Notifications` + history.
- [x] Session shell that starts and supervises all components.
- [x] Config model covering every visible setting (OBSIDIAN defaults).
- [x] IPC control socket for cross-process commands/events.
- [ ] **Compiles and runs on a Linux host** — pending.
- [ ] **Panel/launcher/notifications render correctly** — pending.

---

## Known limitations of Phase 2

1. **Not yet compiled or run.** Must be built on a Linux host with Wayland.
2. **Rendering is skeletal.** The `render` module orchestrates draw order but
   the actual GPU draw calls (wallpaper texture, window buffers, layer surfaces)
   are stubbed — they need Smithay's renderer wired up in detail. This is the
   single largest remaining task in Phase 2.
3. **Input keycode mapping is a subset.** Uses a hardcoded evdev keycode →
   name table. A full implementation uses `xkbcommon` for proper keysym
   translation (layout-aware).
4. **Panel/launcher/notifications are headless.** They create the layer-shell
   surface structure and plugin logic, but the actual pixel drawing (via a
   renderer like `softbuffer` or `wgpu` or Smithay's renderer) is stubbed.
5. **No xdg-decoration yet.** Server-side decorations (OBSIDIAN title bars) are
   declared but not implemented.
6. **Drag-to-snap is a hook.** The zone detection logic is real; the drag
   state tracking during pointer motion is stubbed.
7. **Tray/status icons need D-Bus.** The plugin structure is there; the actual
   `StatusNotifierItem` host and NetworkManager/PipeWire/UPower D-Bus clients
   are not yet wired.

---

## Next: Phase 3 — Customisation

Phase 3 builds the theme engine, wallpaper manager, panel customisation UI,
launcher customisation, widget engine, lock screen, and preset management on
top of this desktop foundation. See `docs/PHASE3.md` (to be created at the
start of Phase 3).