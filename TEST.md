# Quick Test Guide

Run these commands in order once you're booted into Arch live.

---

## 1. Install the toolchain

```bash
pacman -Syu --noconfirm
pacman -S --noconfirm --needed \
    base-devel rust cargo \
    wayland wayland-protocols libxkbcommon \
    mesa dbus pkgconf git
```

---

## 2. Get the code

```bash
git clone https://github.com/nxaryn/NoxL.git ~/NoxL
cd ~/NoxL/desktop
```

---

## 3. Build and test

```bash
cargo build
cargo test
```

**Expected:** `49 passed; 0 failed`

---

## 4. See the desktop

Open 4 terminals. Run one command in each:

**Terminal 1 — compositor:**
```bash
./target/debug/noxl-compositor --nested
```

**Terminal 2 — panel:**
```bash
./target/debug/noxl-panel
```

**Terminal 3 — launcher:**
```bash
./target/debug/noxl-launcher
```

**Terminal 4 — notifications:**
```bash
./target/debug/noxl-notifications
```

**Expected:** A window opens. Inside it:
- OBSIDIAN panel bar at the bottom
- Press `Super` → launcher rectangle appears
- Notifications appear top-right

---

## 5. Build the ISO (optional, the real test)

```bash
sudo pacman -S --needed archiso
sudo ./scripts/build-iso.sh
```

**Expected:** `build/out/noxl-*.iso`

Boot it in a new VM or on another USB.

---

## Troubleshooting

| Problem | Fix |
|---|---|
| `cargo: command not found` | `pacman -S rust cargo` |
| `error: linker cc not found` | `pacman -S base-devel` |
| `wayland-client not found` | `pacman -S wayland wayland-protocols libxkbcommon` |
| Compositor window doesn't open | Make sure you're in a graphical session (not a pure TTY) |
| Panel doesn't appear | Run `noxl-panel` from a second terminal after the compositor starts |

---

## What to report back

If something breaks, paste:
1. The exact error message
2. Which step failed
3. `uname -a` output