# Testing NoxL

NoxL is a Linux Wayland desktop. It must be built and run on Linux. This guide
covers the three testing environments, from easiest to most complete.

---

## Environment summary

| Environment | Builds? | Runs desktop? | Good for |
|---|---|---|---|
| WSL2 (Arch) | ✅ Yes | ❌ No (no display) | Compile checks, unit tests |
| QEMU/KVM VM | ✅ Yes | ✅ Yes | Boot test, desktop verification |
| Bare metal | ✅ Yes | ✅ Yes | Final hardware/stability testing |

---

## 1. WSL2 — compile + unit tests

The fastest feedback loop. WSL2 can compile everything and run all unit tests,
but cannot display a Wayland desktop.

### Setup (one time)

```powershell
# On Windows:
wsl --install -d archlinux
```

```bash
# Inside Arch WSL:
sudo pacman -Syu --noconfirm
sudo pacman -S --noconfirm --needed \
    base-devel rust cargo \
    wayland wayland-protocols libxkbcommon mesa dbus pkgconf git
```

### Build

```bash
cd /mnt/c/Users/NestlingPhoenix/Documents/GitHub/Noxaryn/Projects/NoxL/desktop
cargo build
```

Expected: all 7 binaries produced in `target/debug/`:

```
noxl-compositor   noxl-panel     noxl-launcher
noxl-notifications noxl-shell    noxl-settings
noxl-widgets
```

### Unit tests

```bash
cargo test
```

Expected: **35 tests passing, 0 failing**:

| Crate | Tests | Covers |
|---|---|---|
| `noxl-config` | 11 | OBSIDIAN defaults, hex color parsing, TOML roundtrip, partial-config defaults, keybindings, widget upsert/remove, wallpaper defaults, panel plugins |
| `noxl-ipc` | 8 | Frame encode/decode, command/event roundtrips, socket path, invalid JSON rejection |
| `noxl-launcher` | 9 | Search ranking (exact > prefix > contains > exec > category), case insensitivity, ordering |
| `noxl-notifications` | 7 | History add/remove/mark-read, unread tracking, JSON roundtrip, timestamps |

### Lint

```bash
cargo clippy --all-targets
```

---

## 2. QEMU/KVM — boot + desktop test

This is the real test. Requires a Linux host (not WSL2) with QEMU.

### Build the ISO (Phase 1)

```bash
sudo pacman -S --needed archiso
sudo ./scripts/build-iso.sh
# Result: build/out/noxl-*.iso
```

### Boot in QEMU

```bash
./scripts/boot-test-qemu.sh
# Or explicitly:
qemu-system-x86_64 -m 4G -smp 4 \
  -enable-kvm -cpu host \
  -cdrom build/out/noxl-*.iso -boot d \
  -vga virtio -device virtio-ramfb
```

### What to verify

- [ ] The NoxL syslinux/systemd-boot menu appears (OBSIDIAN colors).
- [ ] Boots to a Wayland session (GDM autologin as `noxl` on the live image).
- [ ] The OBSIDIAN wallpaper fills the screen.
- [ ] Network works (`ping`, NetworkManager applet).
- [ ] Audio works (PipeWire).
- [ ] `noxl-shell` starts the panel and notifications.

### Running the desktop directly (from a TTY or nested session)

```bash
# Nested inside an existing Wayland session (development):
cargo run --release -p noxl-compositor -- --nested

# Full session:
noxl-shell
```

---

## 3. Bare metal — final validation

Boot the ISO on real hardware. This is Phase 6 (stability).

### What to verify

- [ ] Boots on UEFI and BIOS.
- [ ] Wi-Fi, Ethernet, Bluetooth work.
- [ ] GPU acceleration (Intel/AMD/NVIDIA).
- [ ] Multiple monitors.
- [ ] Suspend/resume.
- [ ] Audio (headphones, speakers, Bluetooth).
- [ ] Touchpad/keyboard.
- [ ] USB devices.
- [ ] Installation to disk (Phase 5).
- [ ] Settings persist after reboot.

---

## Continuous integration

A `scripts/test-desktop.sh` runs the full WSL2 test suite in one command:

```bash
./scripts/test-desktop.sh
```

It runs:

1. `cargo build` — compile check
2. `cargo test` — unit tests
3. `cargo clippy --all-targets` — lint

---

## Known gaps

These are honest limitations of the current implementation:

1. **Rendering is skeletal.** The compositor's `render` module clears the
   background and draws window surfaces, but layer-shell surfaces (panel,
   launcher, notifications) are not yet composited. The panel/launcher/
   notification processes create their structure but do not draw pixels yet.
2. **Input keycode mapping is a subset.** Uses a hardcoded evdev keycode table.
   A full implementation uses `xkbcommon` for layout-aware keysym translation.
3. **No xdg-decoration.** Server-side OBSIDIAN title bars are not implemented.
4. **Tray/status icons need D-Bus.** Plugin structure exists; the actual
   `StatusNotifierItem` host and NetworkManager/PipeWire/UPower clients are
   not wired.
5. **Standalone (non-nested) mode not implemented.** The compositor currently
   uses the winit backend (nested). Standalone mode needs DRM/libinput/udev/
   seat (Phase 6).

---

## Reporting issues

When reporting a build or test failure, include:

- `cargo build` output (the full error, not just the last line)
- `rustc --version` and `cargo --version`
- `uname -a`
- The commit hash (`git rev-parse HEAD`)