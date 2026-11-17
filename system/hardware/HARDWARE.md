# NoxL hardware support

NoxL uses the upstream Linux kernel and Arch's `linux-firmware`, so it supports
the same hardware as current Arch Linux. This document states what is supported,
what is detected automatically, and what is not promised.

## Graphics

| Vendor | Driver | Status |
|---|---|---|
| Intel (integrated) | `i915` / `xe` | Supported (Mesa, upstream) |
| AMD (integrated + discrete) | `amdgpu` | Supported (Mesa, upstream) |
| NVIDIA | `nvidia` (proprietary) / `nouveau` (open) | Supported with guidance; proprietary driver installed via package manager |

NoxL does not bundle proprietary firmware. The installer detects the GPU and
offers the appropriate driver package. Unsupported or very new hardware is
reported, not silently ignored.

## Networking

| Type | Driver / service | Status |
|---|---|---|
| Ethernet (Intel/Realtek/Atheros) | kernel drivers + NetworkManager | Supported |
| Wi-Fi (Intel, Atheros, Broadcom, Realtek) | kernel drivers + NetworkManager | Supported; Broadcom may need `broadcom-wl` |
| Bluetooth | `btusb` + BlueZ | Supported |
| VPN | NetworkManager + `networkmanager-openvpn` / `wireguard-tools` | Supported |

## Audio

PipeWire + WirePlumber. Works with the vast majority of HDA/USB/Bluetooth audio
devices via the upstream kernel and ALSA.

## Input

libinput (Wayland). Touchpads, mice, keyboards, and touchscreens are supported.
Gesture configuration is exposed in NoxL Settings (Phase 3+).

## Storage

ext4, Btrfs, XFS, FAT32/exFAT, NTFS (via `ntfs-3g`), LUKS full-disk encryption,
LVM. NVMe, SATA, and USB storage are supported by the upstream kernel.

## Suspend / resume

ACPI suspend-to-RAM (suspend) and hibernation (optional, requires swap ≥ RAM).
Configured via systemd and `systemd-boot`/`GRUB` resume parameters.

## What is NOT promised

- Universal hardware compatibility. NoxL does not guarantee support for
  every device, especially very new, very old, or niche hardware.
- Proprietary firmware is not shipped. Some Wi-Fi/Bluetooth/GPU devices need
  firmware packages installed separately.
- NoxL does not replace the kernel's hardware support with custom drivers.