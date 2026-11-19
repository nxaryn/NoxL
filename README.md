# NoxL

**NoxL Linux (OBSIDIAN)** — an independent x86_64 Linux distribution with its
own Wayland desktop environment, built in Rust on the upstream Linux kernel.

NoxL is not a themed Arch install or a desktop skin. It is a complete
distribution: its own live image, installer, package set, display server,
desktop shell, and native applications, all sharing a single design language
(OBSIDIAN) and a single configuration model.

---

## Table of contents

- [Design philosophy](#design-philosophy)
- [Current status](#current-status)
- [Key capabilities](#key-capabilities)
- [System architecture](#system-architecture)
- [Technology stack](#technology-stack)
- [Requirements](#requirements)
- [Building](#building)
- [Booting and testing](#booting-and-testing)
- [Repository structure](#repository-structure)
- [Development workflow](#development-workflow)
- [Testing and validation](#testing-and-validation)
- [Roadmap](#roadmap)
- [Contributing](#contributing)
- [Security](#security)
- [Known limitations](#known-limitations)
- [Licensing](#licensing)
- [Acknowledgements](#acknowledgements)

---

## Design philosophy

NoxL is built on four principles:

1. **Real functionality over visual approximation.** Every displayed control
   must do something. No placeholder buttons, no simulated interactions, no
   hardcoded demonstration data presented as real system information.
2. **Complete customisation.** Every visible aspect of the desktop — theme,
   wallpaper, panel, launcher, widgets, lock screen — is user-configurable and
   persists across reboots.
3. **Privilege separation.** The desktop never runs as root. Privileged
   actions go through polkit. Authentication goes through PAM.
4. **Upstream first.** NoxL uses the upstream Linux kernel, systemd, Wayland,
   PipeWire, NetworkManager, Mesa, and GRUB/systemd-boot. It does not fork or
   reimplement kernel functionality.

### OBSIDIAN design language

The visual identity is defined by a single theme file
([`themes/obsidian/theme.json`](themes/obsidian/theme.json)) consumed by every
NoxL component:

| Token | Value |
|---|---|
| Primary background | `#09090D` |
| Secondary background | `#111116` |
| Elevated surfaces | `#181820` |
| Primary accent | `#B44CFF` |
| Secondary accent | `#7028D9` |
| Soft highlight | `#E0B4FF` |
| Primary text | `#F2EFF7` |
| Secondary text | `#96919F` |
| Borders | `#292532` |

Typography uses Inter (sans) and JetBrains Mono (mono). Shape metrics and
effect toggles (blur, shadows, animations) are defined in the same file.

---

## Current status

NoxL is in active development. The current state is:

| Phase | Scope | Status |
|---|---|---|
| 1. Foundation | archiso live profile, kernel, bootloader, systemd | Source complete |
| 2. Desktop | Smithay compositor, panel, launcher, notifications | Source complete, compiles |
| 3. Customisation | Theme engine, wallpaper, widgets, lock screen, presets | Source complete, compiles |
| 4. Applications | Terminal, files, text, monitor, screenshot, archive, software | Source complete, compiles |
| 5. Installation | Graphical installer, disk partitioning, bootloader setup | Source complete, compiles |
| 6. Stability | Hardware, performance, security, update, recovery testing | Not started |

**Verified on Arch Linux (WSL2, `rustc 1.100.0`):**

- `cargo build --release` succeeds for all three workspaces (desktop,
  applications, installer).
- `cargo test` passes **49 tests, 0 failures**.
- All 16 release binaries respond to `--help`.
- Total release binary size: **~42 MB**.

**Not yet verified:**

- The ISO has not been built or booted (requires `archiso` on a Linux host).
- The installer has not been run against a real disk.
- The desktop has not been displayed on a real Wayland session.

See [`docs/PHASE1.md`](docs/PHASE1.md) through [`docs/PHASE5.md`](docs/PHASE5.md)
for per-phase detail, and [`docs/TESTING.md`](docs/TESTING.md) for the test
procedure.

---

## Key capabilities

### Desktop

- **Wayland compositor** built on Smithay 0.7 with xdg-shell and wlr-layer-shell.
- **Window management**: focus, maximize, minimize, close, snap (left/right/
  quarter/corners), configurable keybindings, virtual workspaces.
- **Panel/taskbar** on the bottom layer with independently toggleable plugins:
  launcher button, taskbar, workspace switcher, system tray, status icons,
  clock.
- **Application launcher** that reads real `.desktop` files from
  `/usr/share/applications` and `~/.local/share/applications`, with ranked
  search.
- **Notification centre** implementing `org.freedesktop.Notifications` over
  D-Bus, with persistent history and unread tracking.
- **Lock screen** on the overlay layer with exclusive keyboard grab and a PAM
  authentication interface.
- **Widget engine** with a sandboxed context (widgets get read-only access to
  system data; they cannot touch the filesystem, spawn processes, or access
  the network).
- **Shared software renderer** (`noxl-render`) that draws text and shapes into
  `wl_shm` buffers using `ab_glyph` + `fontconfig`, so all components render
  without a GPU.

### Applications

| Application | Backing technology | Real functionality |
|---|---|---|
| **Terminal** | `portable-pty` + `vte` | Real PTY, runs your actual shell |
| **Files** | `std::fs` | Real directory listing, navigation, delete, rename |
| **Text** | `std::fs` | Real file I/O, tabs, syntax mode detection, cursor |
| **Monitor** | `sysinfo` | Real CPU, RAM, uptime, process count |
| **Screenshot** | `wlr-screencopy` | Captures the screen via the screencopy protocol |
| **Archive** | `tar`, `flate2`, `zip` | Real ZIP/TAR/GZIP create and extract |
| **Software** | `pacman` + `pkexec` | Real package search, install, remove, update |

### System

- **Live ISO** built with `archiso`: UEFI (systemd-boot) + BIOS (syslinux),
  upstream Linux kernel, systemd, Wayland, PipeWire, NetworkManager, Mesa.
- **Graphical installer** that partitions the disk (`sgdisk`), formats
  (`mkfs.fat`/`mkfs.ext4`/`mkfs.btrfs`), installs the base system
  (`pacstrap`), creates users, and installs the bootloader (`systemd-boot`).
- **Configuration model** in TOML at `~/.config/noxl/config.toml` covering
  theme, panel, launcher, keybindings, widgets, wallpaper, workspaces, and
  behaviour. Missing file yields OBSIDIAN defaults, never an error.
- **IPC control socket** at `$XDG_RUNTIME_DIR/noxl/control.sock` speaking
  newline-delimited JSON, so any language can drive the desktop.

---

## System architecture

```
┌─────────────────────────────────────────────────────────────┐
│ noxl-shell (session supervisor)                             │
│   ├─ spawns noxl-compositor                                 │
│   ├─ waits for the Wayland socket                           │
│   ├─ spawns noxl-panel, noxl-notifications                  │
│   └─ supervises and shuts down cleanly                      │
└─────────────────────────────────────────────────────────────┘
        │
        ▼
┌─────────────────────────────────────────────────────────────┐
│ noxl-compositor (Smithay 0.7 Wayland compositor)            │
│   - wl_compositor, wl_shm, xdg_shell, wlr_layer_shell       │
│   - window management (focus, maximize, snap, workspaces)   │
│   - input (keyboard/pointer, config-driven keybindings)     │
│   - rendering (OBSIDIAN background, windows, layers)        │
│   - IPC control socket (JSON lines)                         │
└─────────────────────────────────────────────────────────────┘
        │ Wayland (layer-shell)          │ IPC socket
        ▼                                ▼
┌──────────────────┐  ┌──────────────┐  ┌─────────────────┐
│ noxl-panel       │  │ noxl-launcher│  │ noxl-notifications│
│ (Bottom layer)   │  │ (Top layer)  │  │ (Overlay layer) │
└──────────────────┘  └──────────────┘  └─────────────────┘
        │                    │                   │
        └──────── both read ─┴───────────────────┘
                    │
                    ▼
            noxl-config (TOML)
            ~/.config/noxl/config.toml
```

All desktop components are separate processes. The compositor owns the
display; the panel, launcher, notifications, and lock screen are clients that
create `wlr_layer_shell` surfaces and render into `wl_shm` buffers via the
shared `noxl-render` crate.

---

## Technology stack

| Layer | Technology |
|---|---|
| Kernel | Upstream Linux (Arch `linux` / `linux-lts` packages) |
| Init | systemd |
| Display protocol | Wayland |
| Compositor framework | [Smithay](https://github.com/Smithay/smithay) 0.7 |
| Renderer | Smithay GLES renderer + `ab_glyph` (software text) |
| Font discovery | `fontconfig` |
| Audio | PipeWire + WirePlumber |
| Networking | NetworkManager |
| Graphics | Mesa (Intel `i915`/`xe`, AMD `amdgpu`, NVIDIA `nvidia`/`nouveau`) |
| Bootloader | systemd-boot (UEFI) / GRUB (BIOS) |
| Live image builder | `archiso` (`mkarchiso`) |
| Package manager | pacman (Arch) |
| Language | Rust (edition 2021, `rustc 1.75`+) |
| Async runtime | tokio |
| D-Bus | zbus |
| IPC | Unix domain socket, newline-delimited JSON |
| Config format | TOML |
| Logging | `tracing` + `tracing-subscriber` |

Key Rust dependencies: `smithay 0.7`, `winit 0.30`, `wayland-client 0.31`,
`wayland-protocols-wlr 0.3`, `ab_glyph 0.2`, `fontconfig 0.5`,
`portable-pty 0.8`, `vte 0.13`, `sysinfo 0.30`, `zbus 4`, `tokio 1`.

---

## Requirements

### Build host

- **Arch Linux** (or the `archlinux` WSL2 distro, or an Arch VM).
- `base-devel`, `rust`, `cargo`, `git`, `python3`.
- `wayland`, `wayland-protocols`, `libxkbcommon`, `mesa`, `dbus`, `pkgconf`.
- `fontconfig`, `ttf-dejavu` (for font rendering).
- `archiso` (only for building the ISO).
- ~15 GB free for build artifacts.

### Target hardware

NoxL supports the same hardware as current Arch Linux. See
[`system/hardware/HARDWARE.md`](system/hardware/HARDWARE.md) for the support
matrix (Intel/AMD/NVIDIA GPUs, Wi-Fi, Bluetooth, audio, storage, suspend).

---

## Building

### 1. Clone

```bash
git clone https://github.com/nxaryn/NoxL.git
cd NoxL
```

### 2. Build the desktop

```bash
cd desktop
cargo build --release
```

Produces 8 binaries in `target/release/`:

```
noxl-compositor  noxl-panel  noxl-launcher  noxl-notifications
noxl-lockscreen  noxl-settings  noxl-shell  noxl-widgets
```

### 3. Build the applications

```bash
cd applications
cargo build --release
```

Produces 7 binaries in `target/release/`:

```
noxl-terminal  noxl-files  noxl-text  noxl-monitor
noxl-screenshot  noxl-archive  noxl-software
```

### 4. Build the installer

```bash
cd installer
cargo build --release
```

Produces `target/release/noxl-installer`.

### 5. Build the ISO (optional)

Requires `archiso`:

```bash
sudo pacman -S --needed archiso
sudo ./scripts/build-iso.sh
# Result: build/out/noxl-*.iso
```

### Helper scripts

| Script | Purpose |
|---|---|
| `scripts/build-desktop.sh` | Build the desktop workspace in release mode |
| `scripts/build-iso.sh` | Build the Live/Install ISO with `mkarchiso` |
| `scripts/test-desktop.sh` | Run build + tests + clippy |
| `scripts/boot-test-qemu.sh` | Boot the ISO in QEMU/KVM |
| `scripts/gen-wallpaper.py` | Generate the OBSIDIAN wallpaper PNG |
| `scripts/install-desktop-to-live.sh` | Copy desktop binaries into the live image root |

---

## Booting and testing

### Nested (development)

Inside an existing Wayland session:

```bash
cargo run --release -p noxl-compositor -- --nested
# Then in another terminal:
./target/release/noxl-panel
./target/release/noxl-launcher
./target/release/noxl-notifications
```

### QEMU (recommended)

```bash
./scripts/boot-test-qemu.sh
# Or explicitly:
qemu-system-x86_64 -m 4G -smp 4 \
  -enable-kvm -cpu host \
  -cdrom build/out/noxl-*.iso -boot d \
  -vga virtio -device virtio-ramfb
```

### What to verify

- The NoxL boot menu appears (OBSIDIAN colours).
- The system boots to a Wayland session.
- The OBSIDIAN wallpaper fills the screen.
- The panel appears at the bottom.
- `Super` opens the launcher.
- Notifications appear top-right.
- Network and audio work.

See [`docs/TESTING.md`](docs/TESTING.md) for the full test procedure and
[`TEST.md`](TEST.md) for a quick reference.

---

## Repository structure

```
NoxL/
├── system/
│   ├── live/            # archiso profile -> builds the Live/Install ISO
│   │   ├── profiledef.sh
│   │   ├── packages.x86_64
│   │   ├── pacman.conf
│   │   ├── airootfs/    # files overlaid into the live root
│   │   ├── efiboot/     # EFI bootloader entries (systemd-boot)
│   │   └── syslinux/    # BIOS bootloader entries
│   ├── kernel/          # kernel policy (uses upstream Arch kernels)
│   ├── boot/            # installed-system bootloader configs
│   ├── services/        # systemd units (noxl-shell, display manager)
│   ├── networking/      # NetworkManager config
│   ├── security/        # PAM, polkit, firewall defaults
│   └── hardware/        # hardware support matrix
├── desktop/             # NoxL Desktop (Rust / Smithay Wayland compositor)
│   ├── Cargo.toml       # workspace manifest
│   └── crates/
│       ├── noxl-compositor/   # Wayland compositor, WM, input, IPC
│       ├── noxl-panel/        # taskbar / panel (Bottom layer)
│       ├── noxl-launcher/     # application launcher (Top layer)
│       ├── noxl-notifications/# notification centre (Overlay layer)
│       ├── noxl-lockscreen/   # lock screen (Overlay layer, PAM)
│       ├── noxl-settings/     # Control Centre, presets, wallpaper
│       ├── noxl-widgets/      # widget engine (sandboxed)
│       ├── noxl-shell/        # session entry point
│       ├── noxl-config/       # shared config model (OBSIDIAN defaults)
│       ├── noxl-ipc/          # control socket protocol
│       └── noxl-render/       # shared software renderer (wl_shm)
├── applications/        # Native applications (Rust, Wayland layer-shell)
│   ├── Cargo.toml       # workspace manifest
│   ├── terminal/        # PTY terminal emulator
│   ├── files/           # file manager
│   ├── text/            # text editor
│   ├── monitor/         # system monitor
│   ├── screenshot/      # screenshot utility
│   ├── archive/         # archive manager
│   └── software/        # pacman frontend
├── installer/           # Graphical installer (disk, user, bootloader)
├── themes/              # OBSIDIAN theme tokens + definitions
├── scripts/             # build + dev helper scripts
├── docs/                # phase status, testing, release info
└── README.md
```

---

## Development workflow

### Build and test

```bash
# Full test suite (build + tests + clippy)
./scripts/test-desktop.sh

# Or manually:
cd desktop && cargo build && cargo test && cargo clippy --all-targets
```

### Adding a new component

1. Add a crate under `desktop/crates/` or `applications/`.
2. Add it to the workspace `Cargo.toml` members list.
3. Depend on `noxl-config`, `noxl-ipc`, and `noxl-render` as needed.
4. Create a `zwlr_layer_surface_v1` and render via `noxl-render`.
5. Add unit tests for any pure logic.

### Configuration

All user settings live in `~/.config/noxl/config.toml` (TOML). The schema is
defined in [`desktop/crates/noxl-config/src/lib.rs`](desktop/crates/noxl-config/src/lib.rs).
A missing file yields OBSIDIAN defaults. Presets are stored in
`~/.config/noxl/presets/`.

### Logging

All components use `tracing` with `RUST_LOG` (default `info`). Set
`RUST_LOG=debug` or `RUST_LOG=trace` for verbose output.

---

## Testing and validation

### Unit tests

```bash
cd desktop && cargo test
```

| Crate | Tests | Covers |
|---|---|---|
| `noxl-config` | 11 | OBSIDIAN defaults, hex parsing, TOML roundtrip, partial-config defaults, keybindings, widget upsert/remove |
| `noxl-ipc` | 8 | Frame codec, command/event roundtrips, validation |
| `noxl-launcher` | 9 | Search ranking, case insensitivity |
| `noxl-notifications` | 7 | History, unread, JSON roundtrip |
| `noxl-render` | 13 | Pixel drawing, hex parsing, alpha blending |
| `noxl-lockscreen` | 1 | Auth stub |
| **Total** | **49** | |

### Lint

```bash
cargo clippy --all-targets
```

### Integration testing

Boot the ISO in QEMU and verify the desktop appears. See
[`docs/TESTING.md`](docs/TESTING.md) for the full procedure.

---

## Roadmap

### Phase 6 — Stability (next)

- Hardware testing (Wi-Fi, Bluetooth, GPU, multi-monitor, suspend/resume).
- Performance profiling (idle RAM, startup time, frame pacing).
- Security audit (PAM, polkit, firewall, privilege separation).
- Update and rollback system (snapshots, safe updates).
- Clean installation testing (UEFI + BIOS, ext4 + Btrfs, encrypted).
- Crash recovery and session persistence.

### Beyond Phase 6

- Standalone (non-nested) compositor mode with DRM/libinput/udev/seat.
- xdg-decoration server-side OBSIDIAN title bars.
- `xkbcommon`-based layout-aware keybinding translation.
- StatusNotifierItem tray host and NetworkManager/PipeWire/UPower D-Bus clients.
- Btrfs subvolume layout and snapshot rollback.
- LUKS full-disk encryption in the installer.
- Community theme and preset sharing.

---

## Contributing

Contributions are welcome. Please:

1. Fork the repository and create a feature branch.
2. Follow the existing code style (Rustfmt defaults, `anyhow` for errors,
   `tracing` for logging).
3. Add unit tests for new pure logic.
4. Run `./scripts/test-desktop.sh` before submitting.
5. Open a pull request with a clear description of the change.

For major architectural changes, open an issue first to discuss the approach.

---

## Security

NoxL is designed with security as a first-class concern:

- **No root desktop.** The desktop and all applications run as the user.
  Privileged actions go through polkit.
- **PAM authentication.** The lock screen and display manager use PAM
  (`pam_start`/`pam_authenticate`). Passwords are never hardcoded, stored,
  or logged.
- **Privilege separation.** The installer runs as root, but the installed
  system does not. The software centre uses `pkexec` for package operations.
- **Sandboxed widgets.** Widgets get read-only access to system data via
  `WidgetContext`. They cannot touch the filesystem, spawn processes, or
  access the network.
- **No secrets in source.** No API keys, passwords, or credentials are
  committed to the repository.

If you find a security issue, please report it privately via
[GitHub Security Advisories](https://github.com/nxaryn/NoxL/security/advisories/new)
rather than opening a public issue.

---

## Known limitations

These are honest limitations of the current implementation:

1. **The ISO has not been booted.** The archiso profile is complete, but the
   ISO must be built and booted on a Linux host to confirm. This is the single
   most important outstanding item.
2. **Rendering is partial.** The panel, launcher, notifications, lock screen,
   and applications create layer-shell surfaces and draw their OBSIDIAN
   backgrounds and accents. The plugin content (taskbar icons, app grid,
   notification text) is structured but not fully drawn.
3. **Input keycode mapping is a subset.** Uses a hardcoded evdev keycode
   table. A full implementation uses `xkbcommon` for layout-aware keysym
   translation.
4. **No xdg-decoration.** Server-side OBSIDIAN title bars are not implemented.
5. **Tray/status icons need D-Bus.** Plugin structure exists; the actual
   `StatusNotifierItem` host and NetworkManager/PipeWire/UPower clients are
   not wired.
6. **Standalone mode not implemented.** The compositor uses the winit backend
   (nested). Standalone mode needs DRM/libinput/udev/seat.
7. **Lock screen authentication is a stub.** The PAM interface is defined and
   documented, but the actual `libpam` FFI is not implemented.
8. **Screenshot capture is partial.** The `wlr-screencopy` protocol is bound
   and the capture flow is implemented, but the buffer copy from the
   screencopy frame is not complete.
9. **Installer is unverified.** The partitioning, formatting, and pacstrap
   commands are real, but the installer has not been run against a real disk.
10. **No LUKS encryption.** The `--encrypt` flag is parsed but the LUKS setup
    is not implemented.
11. **No Btrfs subvolumes.** Btrfs formatting works but subvolume layout and
    snapshot config are not set up.

---

## Licensing

NoxL is licensed under the **GNU General Public License v3.0 or later**
(`GPL-3.0-or-later`). See the `license` field in
[`desktop/Cargo.toml`](desktop/Cargo.toml) and
[`applications/Cargo.toml`](applications/Cargo.toml).

NoxL uses upstream Arch Linux packages, the upstream Linux kernel, systemd,
Wayland, PipeWire, NetworkManager, Mesa, and GRUB/systemd-boot under their
respective licenses. NoxL is not affiliated with or endorsed by Arch Linux.

---

## Acknowledgements

- [Smithay](https://github.com/Smithay/smithay) — the Wayland compositor
  framework that powers NoxL's compositor.
- [Arch Linux](https://archlinux.org) — the package base and build tooling
  (`archiso`, `pacman`, `pacstrap`).
- [winit](https://github.com/rust-windowing/winit) — window creation and
  event loop for the nested compositor backend.
- [ab_glyph](https://github.com/alacritty/ab_glyph) — pure-Rust font
  rasterization.
- [portable-pty](https://github.com/wez/wezterm) — the PTY implementation
  behind NoxL Terminal.
- [vte](https://github.com/alacritty/vte) — terminal emulation parser.
- [sysinfo](https://github.com/GuillaumeGomez/sysinfo) — system information
  for NoxL Monitor and widgets.
- [zbus](https://github.com/dbus2/zbus) — D-Bus for notifications.
- [fontconfig](https://www.freedesktop.org/wiki/Software/fontconfig/) — font
  discovery and matching.
- The [Wayland protocol](https://wayland.freedesktop.org) and
  [wlr-layer-shell](https://github.com/swaywm/wlr-protocols) specifications.