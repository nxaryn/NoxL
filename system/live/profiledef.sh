#!/usr/bin/env bash
#
# NoxL archiso profile definition.
# Drives `mkarchiso` to build the NoxL Live/Install ISO.
#
# This produces a REAL bootable x86_64 ISO: systemd-boot (UEFI) + syslinux (BIOS),
# upstream Linux kernel, systemd, and a Wayland-capable userland.

edition="noxl"
iso_name="noxl"
iso_label="NOXL_$(date +%Y%m)"
iso_publisher="NoxL Project <https://github.com/Noxaryn/NoxL>"
iso_application="NoxL Live/Install ISO"
iso_version="$(date +%Y.%m.%d)"
install_dir="noxl"
buildmodes=('iso')
bootmodes=('bios.syslinux.mbr' 'bios.syslinux.uefi' 'uefi.systemd-boot')
arch="x86_64"
pacman_conf="pacman.conf"
airootfs_image_type="squashfs"
airootfs_image_tool_options=('-comp' 'xz' '-Xbcj' 'x86' '-b' '1M' '-Xdict-size' '1M')
file_permissions=(
  ["/etc/shadow"]="0:0:0400"
  ["/etc/gshadow"]="0:0:0400"
  ["/root"]="0:0:0700"
  ["/root/.automated_script.sh"]="0:0:0755"
  ["/usr/local/bin/noxl-autologin"]="0:0:0755"
  ["/usr/local/bin/noxl-live-setup"]="0:0:0755"
)