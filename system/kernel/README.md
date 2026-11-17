# NoxL kernel policy
#
# NoxL uses the UPSTREAM Linux kernel. It does NOT fork or fake the kernel.
# On the live ISO and installed systems, the kernel is provided by the Arch
# `linux` package (the official upstream stable kernel, built with the Arch
# config). An LTS option is available via the `linux-lts` package.
#
# This file documents the kernel configuration requirements and the packages
# that satisfy them. It is consumed by the installer and the build scripts.
#
# ---------------------------------------------------------------------------
# Kernel packages
# ---------------------------------------------------------------------------
# Standard (stable):   linux
# Long-term support:   linux-lts
# Headers (for DKMS):  linux-headers  /  linux-lts-headers
# Firmware:            linux-firmware
#
# ---------------------------------------------------------------------------
# Required kernel features (provided by the Arch linux/linux-lts packages)
# ---------------------------------------------------------------------------
# [x] Hardware detection           (udev + sysfs, upstream)
# [x] CPU scheduling               (CFS / EEVDF, upstream)
# [x] Memory management            (upstream)
# [x] Filesystem support           ext4, btrfs, xfs, vfat, ntfs3, exfat
# [x] USB support                  xHCI, EHCI, storage, HID
# [x] PCI device support           upstream
# [x] GPU driver compatibility     amdgpu, i915, xe, nvidia (open + proprietary)
# [x] Wi-Fi / Ethernet             cfg80211, mac80211, e1000e, r8169, iwlwifi, ath...
# [x] Bluetooth                    btusb, btintel, btrtl, ...
# [x] Power management             ACPI, cpufreq, suspend/resume (hibernation optional)
# [x] ACPI support                 upstream
#
# ---------------------------------------------------------------------------
# Kernel cmdline (installed system)
# ---------------------------------------------------------------------------
# Set by the bootloader. See system/boot/loader/entries/noxl.conf and
# system/boot/grub/grub.cfg. Keep it minimal:
#   quiet splash rw
#
# ---------------------------------------------------------------------------
# Building a custom kernel (optional, advanced)
# ---------------------------------------------------------------------------
# NoxL does not require a custom kernel build. If you want to build one:
#   1. Install base-devel and the ncurses/bison/flex/elfutils toolchain.
#   2. Use the upstream source from https://kernel.org or the Arch PKGBUILDs.
#   3. Use `make localmodconfig` or `make olddefconfig` as a starting point.
#   4. Install with `make modules_install` and `make install`, then regenerate
#      the initramfs with `mkinitcpio -P`.
#
# The default NoxL experience uses the packaged Arch kernels above.