# Phase 5 — Installation: Status

**Phase:** 5 of 6 (Installation)
**Goal:** Graphical installer, Live ISO, disk management, bootloader setup.

**Status:** ✅ Installer compiles on real Linux (Arch WSL2).

---

## What Phase 5 delivers

| Component | Status |
|---|---|
| **Disk detection** | ✅ Detects disks via `/sys/block` (path, model, size, removable) |
| **Partitioning** | ✅ `sgdisk` (UEFI + root, GPT) |
| **Formatting** | ✅ `mkfs.fat` (EFI), `mkfs.ext4`/`mkfs.btrfs` (root) |
| **Mounting** | ✅ Mount root + EFI |
| **Base install** | ✅ `pacstrap` (base, linux, systemd, NetworkManager, PipeWire, Mesa) |
| **System config** | ✅ Timezone, hostname, keyboard, NetworkManager |
| **User creation** | ✅ `useradd`, password, sudo |
| **Bootloader** | ✅ `systemd-boot` + loader entry with root UUID |
| **Graphical UI** | ✅ Layer-shell surface, disk selection, OBSIDIAN theme, real fonts |

---

## Usage

### Non-interactive (scriptable)

```bash
sudo noxl-installer \
  --disk /dev/sda \
  --fs ext4 \
  --username noxl \
  --timezone UTC \
  --keyboard us
```

### Interactive (graphical)

```bash
sudo noxl-installer
```

Opens a layer-shell window with disk selection and step progress.

---

## Verified on Arch Linux (WSL2)

```
cargo build   → clean, binary produced
```

---

## Known limitations

1. **Not yet tested on real hardware.** The installer compiles but has not
   been run against a real disk. The partitioning/formatting commands are
   real (`sgdisk`, `mkfs.*`, `pacstrap`) but unverified end-to-end.
2. **No LUKS encryption yet.** The `--encrypt` flag is parsed but the LUKS
   setup (`cryptsetup`) is not implemented.
3. **No Btrfs subvolumes/snapshots.** Btrfs formatting works but subvolume
   layout and snapshot config are not set up.
4. **Graphical UI is basic.** Disk selection works but there's no keyboard
   navigation, no confirmation dialog, and no progress bar.
5. **No error recovery.** If pacstrap fails midway, the installer doesn't
   clean up or offer a retry.

---

## Next: Phase 6 — Stability

Phase 6 covers hardware testing, performance testing, security testing,
update testing, recovery testing, multi-monitor testing, and clean
installation testing. See `docs/PHASE6.md`.