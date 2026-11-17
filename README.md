# NoxL

**A custom, independent Linux distribution with its own desktop environment.**

NoxL is built on a stable Linux foundation (Arch Linux base, official Linux kernel,
systemd, Wayland, PipeWire) and adds an original desktop environment, a complete
customisation system, and a native application ecosystem under the **OBSIDIAN**
design language.

> **Status:** Phase 3 (Customisation) — source complete and compiling on
> real Linux. 49 unit tests passing. See [docs/PHASE1.md](docs/PHASE1.md),
> [docs/PHASE2.md](docs/PHASE2.md), [docs/PHASE3.md](docs/PHASE3.md), and
> [docs/TESTING.md](docs/TESTING.md) for details.

---

## Design identity

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

Philosophy: minimalistic, futuristic, modular, powerful, elegant. No generic
dashboard aesthetics, no decorative glow, no oversized rounded corners.

---

## Repository layout

```
NoxL/
├── system/
│   ├── live/            # archiso profile -> builds the Live/Install ISO
│   │   ├── profiledef.sh
│   │   ├── packages.x86_64
│   │   ├── pacman.conf
│   │   ├── airootfs/    # files overlaid into the live root
│   │   ├── efiboot/     # EFI bootloader entries
│   │   └── syslinux/    # BIOS bootloader entries
│   ├── kernel/          # kernel config + build helper (uses upstream Arch kernel)
│   ├── boot/            # bootloader config (systemd-boot / GRUB)
│   ├── services/        # systemd units (display manager, noxl-session, etc.)
│   ├── networking/      # NetworkManager config
│   ├── security/        # PAM, polkit, firewall defaults
│   └── hardware/        # firmware/driver guidance
├── desktop/             # NoxL Desktop (Rust / Smithay Wayland compositor)
│   ├── crates/
│   │   ├── noxl-compositor/   # Wayland compositor, WM, input, IPC
│   │   ├── noxl-panel/        # taskbar / panel (Bottom layer)
│   │   ├── noxl-launcher/     # application launcher (Top layer)
│   │   ├── noxl-notifications/# notification centre (Overlay layer)
│   │   ├── noxl-lockscreen/   # lock screen (Overlay layer, PAM)
│   │   ├── noxl-settings/     # Control Centre, presets, wallpaper
│   │   ├── noxl-widgets/      # widget engine (sandboxed)
│   │   ├── noxl-shell/        # session entry point
│   │   ├── noxl-config/       # shared config model (OBSIDIAN defaults)
│   │   ├── noxl-ipc/          # control socket protocol
│   │   └── noxl-render/       # shared software renderer (wl_shm)
├── applications/        # (Phase 4+) native apps
├── themes/              # OBSIDIAN theme tokens + definitions
├── installer/           # (Phase 5) graphical installer
├── scripts/             # build + dev helper scripts
├── docs/                # architecture, phase status, known issues
└── README.md
```

Directories that do not yet contain real content are intentionally omitted until
the phase that needs them (per the "no empty directories" rule).

---

## Building (on a Linux host)

Requirements: an **Arch Linux** host (or the `archlinux` WSL2 distro / a VM) with
`archiso`, `git`, and ~15 GB free.

```bash
git clone <this repo>
cd NoxL
sudo ./scripts/build-iso.sh
# Result: build/noxl-*.iso
```

Build the NoxL desktop (on a Linux host with Rust + Wayland + D-Bus):

```bash
./scripts/build-desktop.sh
# Binaries in desktop/target/release/
```

Run the test suite (compile + unit tests + lint):

```bash
./scripts/test-desktop.sh
# 49 tests passing
```

Boot test:

```bash
qemu-system-x86_64 -m 4G -smp 4 -cdrom build/noxl-*.iso -boot d
```

Full instructions, including how to verify the ISO boots to a systemd + Wayland
session, are in [docs/PHASE1.md](docs/PHASE1.md).

---

## Legal / upstream

NoxL is an independent distribution. It is not affiliated with or endorsed by
Arch Linux. It uses upstream Arch Linux packages, the upstream Linux kernel,
systemd, Wayland (wlroots/Qt), PipeWire, NetworkManager, Mesa, and GRUB/systemd-boot
under their respective licenses.