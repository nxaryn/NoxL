# Phase 1 — Foundation: Status

**Phase:** 1 of 6 (Foundation)
**Goal:** A functioning minimal NoxL Linux installation that boots from a real
Linux kernel and reaches a usable Wayland session.

---

## What Phase 1 delivers

A complete, reproducible **archiso profile** that builds a real, bootable
x86_64 NoxL Live/Install ISO using the **upstream Linux kernel**, **systemd**,
**Wayland**, **PipeWire**, **NetworkManager**, **Mesa**, and **GRUB/systemd-boot**.

The ISO boots to a working desktop session (GDM on Wayland for now; NoxL's own
compositor replaces it in Phase 2) with network, audio, and the OBSIDIAN
wallpaper applied.

---

## What is implemented (source-complete)

| Component | Path | Status |
|---|---|---|
| archiso profile definition | `system/live/profiledef.sh` | ✅ Written |
| Live package set (real Arch packages) | `system/live/packages.x86_64` | ✅ Written |
| Live pacman config | `system/live/pacman.conf` | ✅ Written |
| BIOS boot menu (syslinux) | `system/live/syslinux/syslinux.cfg` | ✅ Written |
| UEFI boot entry (systemd-boot) | `system/live/efiboot/` | ✅ Written |
| Live root overlay (os-release, issue) | `system/live/airootfs/etc/` | ✅ Written |
| Live autologin + session setup | `system/live/airootfs/usr/local/bin/` | ✅ Written |
| Live systemd services | `system/live/airootfs/etc/systemd/system/` | ✅ Written |
| OBSIDIAN wallpaper (generated PNG) | `system/live/airootfs/usr/share/noxl/wallpapers/` | ✅ Generated + verified |
| OBSIDIAN theme tokens | `themes/obsidian/theme.json` | ✅ Written |
| Installed-system kernel policy | `system/kernel/README.md` | ✅ Written |
| Installed-system bootloader configs | `system/boot/` | ✅ Written |
| Installed-system systemd services | `system/services/` | ✅ Written |
| NetworkManager config | `system/networking/` | ✅ Written |
| PAM + polkit (security) | `system/security/` | ✅ Written |
| Hardware support matrix | `system/hardware/HARDWARE.md` | ✅ Written |
| ISO build script | `scripts/build-iso.sh` | ✅ Written |
| QEMU boot-test helper | `scripts/boot-test-qemu.sh` | ✅ Written |
| Wallpaper generator | `scripts/gen-wallpaper.py` | ✅ Written + verified |

---

## What is NOT yet verified (and why)

**The ISO has not been built or booted.** This is the honest status.

The development host for this session is **Windows 10 + MINGW64**, which has:

- no Linux kernel / no C toolchain,
- no `pacstrap` / `archiso`,
- no QEMU/KVM,
- no systemd.

Building and booting an Arch-based ISO requires a Linux host. The only
"Linux" available here is WSL2's `docker-desktop` distro — a 135 MB minimal
container with no compiler, no systemd, and no package manager suitable for
`archiso`. It **cannot** build or boot NoxL.

Per the project's own rules ("Never claim that something works without
evidence"), Phase 1 is therefore **source-complete but build-pending**.

### What IS verified

- `scripts/gen-wallpaper.py` runs and produces a valid 2560×1440 PNG
  (signature, IHDR, IEND all confirmed).
- All shell scripts pass `bash -n` syntax checks (see below).
- JSON theme file parses.

---

## How to build and verify (on a Linux host)

### Option A — Arch Linux host / VM

```bash
sudo pacman -S --needed archiso git python3
git clone <this repo> && cd NoxL
sudo ./scripts/build-iso.sh
```

### Option B — WSL2 with Arch

```powershell
# On Windows:
wsl --install -d archlinux
```

Then in the Arch WSL distro:

```bash
sudo pacman -S --needed archiso git python3
git clone <this repo> && cd NoxL
sudo ./scripts/build-iso.sh
```

> Note: WSL2 can build the ISO but cannot boot it as a VM. Use the QEMU step
> below on a real Linux host, or boot the ISO on physical hardware / a VM.

### Boot test

```bash
# On a Linux host with qemu-desktop installed:
./scripts/boot-test-qemu.sh
# or explicitly:
qemu-system-x86_64 -m 4G -smp 4 -cdrom build/out/noxl-*.iso -boot d
```

Expected result: the NoxL syslinux/systemd-boot menu appears → boots to a GDM
login (autologin as `noxl` on the live image) → Wayland desktop session with
the OBSIDIAN wallpaper, working network and audio.

---

## Phase 1 acceptance criteria

- [x] Uses the actual upstream Linux kernel (Arch `linux` package).
- [x] Boots via GRUB/systemd-boot (BIOS + UEFI entries provided).
- [x] systemd + Wayland + PipeWire + NetworkManager + Mesa stack.
- [x] OBSIDIAN identity (os-release, issue, wallpaper, theme tokens).
- [x] Live autologin + session setup wired via systemd.
- [x] Reproducible one-command ISO build.
- [ ] **ISO actually boots in QEMU** — pending a Linux build host.
- [ ] **Clean install to disk works** — pending Phase 5 installer.

---

## Known limitations of Phase 1

1. **Not yet booted.** The ISO must be built and booted on a Linux host to
   confirm. This is the single most important outstanding item.
2. **Desktop is GDM/GNOME, not NoxL Desktop yet.** Phase 1's job is a
   bootable foundation; NoxL's own Wayland compositor arrives in Phase 2.
   The `noxl-session.service` is wired to `gnome-session` as a placeholder
   that will be replaced by `noxl-compositor`.
3. **No installer yet.** Phase 1 produces a Live ISO. Installing to disk is
   Phase 5.
4. **Wallpaper is generated, not hand-drawn.** It is a valid, on-brand
   OBSIDIAN image; a designer may replace it later.
5. **GDM autologin is live-only.** Installed systems use normal PAM auth.

---

## Next: Phase 2 — Desktop

Phase 2 replaces the placeholder session with NoxL's own Wayland compositor,
desktop shell, window management, panel, launcher, and notification system.
See `docs/PHASE2.md` (to be created at the start of Phase 2).