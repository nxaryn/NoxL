# Phase 3 — Customisation: Status

**Phase:** 3 of 6 (Customisation)
**Goal:** Theme engine, wallpaper manager, panel/launcher customisation,
widget engine, lock screen, preset management.

**Status:** 🟡 In progress. Lock screen and settings/presets/widgets
source complete and compiling. Theme switching and wallpaper application
are wired but not yet exercised end-to-end.

---

## What Phase 3 delivers

| Feature | Crate | Status |
|---|---|---|
| **Lock screen** | `noxl-lockscreen` | ✅ Compiles, draws OBSIDIAN overlay, PAM interface |
| **Settings / Control Centre** | `noxl-settings` | ✅ Categories, presets, wallpaper mgmt |
| **Widget engine** | `noxl-widgets` | ✅ Sandboxed context, builtin widgets (CPU/RAM/uptime/clock) |
| **Theme engine** | `noxl-config` (theme module) | ✅ OBSIDIAN tokens, hex parsing, live reload via IPC |
| **Wallpaper manager** | `noxl-settings` (wallpaper module) | ✅ Static/slideshow/solid/per-monitor configs |
| **Preset management** | `noxl-settings` (presets module) | ✅ Save/load/apply/export/import/delete |

---

## What is implemented

### Lock screen (`noxl-lockscreen`)
- Full-screen `zwlr_layer_surface_v1` on the `Overlay` layer.
- Exclusive keyboard grab (blocks input to the session behind it).
- Paints the OBSIDIAN `bg_primary` background with an accent line.
- `auth.rs`: PAM authentication interface. The stub documents the real
  `libpam` FFI (`pam_start`/`pam_authenticate`/`pam_end`). Passwords are
  never hardcoded, stored, or logged.
- 1 unit test.

### Settings (`noxl-settings`)
- **Categories**: Personalisation, System, Connectivity, Security — each
  with typed get/set operations that write to the real config.
- **Presets**: save/load/apply/export/import/delete as TOML files in
  `~/.config/noxl/presets/`.
- **Wallpaper**: list available wallpapers, set static/slideshow/solid/
  per-monitor configs.

### Widget engine (`noxl-widgets`)
- Sandboxed `WidgetContext` — widgets get read-only access to system data
  (CPU, RAM, uptime, CPU brand) and cannot touch the filesystem, spawn
  processes, or access the network.
- Builtin widgets: `system.cpu`, `system.memory`, `system.uptime`,
  `productivity.clock`.
- Widget instances are positioned, sized, and persisted via config.

### Theme engine (`noxl-config`)
- OBSIDIAN theme tokens (colors, typography, shape, effects).
- Hex color parsing (`#RRGGBB` / `#RRGGBBAA` → RGBA floats).
- Live reload via IPC (`ReloadConfig` command) — theme changes apply
  without a restart.

### Wallpaper manager (`noxl-settings`)
- Scans user and system wallpaper directories.
- Supports static, slideshow, solid color, and per-monitor configs.
- Scaling modes: Fill, Fit, Stretch, Center, Tile.

### Preset management (`noxl-settings`)
- A preset is a full `NoxlConfig` snapshot.
- Save/load/apply/export/import/delete.
- Stored as TOML in `~/.config/noxl/presets/`.

---

## Verified on Arch Linux (WSL2)

```
cargo build   → clean, all 9 crates + 8 binaries
cargo test    → 49 tests passing, 0 failing
```

---

## Known limitations

1. **Lock screen authentication is a stub.** The PAM interface is defined
   and documented, but the actual `libpam` FFI is not implemented. The stub
   returns `NotImplemented`.
2. **Theme switching is wired but not exercised.** The config model and IPC
   reload command exist; a full end-to-end theme change (settings UI →
   config → compositor repaint) has not been tested.
3. **Wallpaper is configured but not rendered.** The compositor's render
   module clears with the theme background color; it does not yet load and
   draw the wallpaper texture.
4. **Widgets are headless.** The widget engine produces `WidgetRender`
   data (text, progress, detail) but does not draw it to the screen.
5. **Settings UI is headless.** The categories and setters exist; the
   graphical settings window is not implemented.

---

## Next: Phase 4 — Applications

Phase 4 builds the native applications: Files, Terminal, Settings,
Monitor, Software, Text, Screenshot, Archive. See `docs/PHASE4.md`.