#!/usr/bin/env bash
#
# NoxL ISO build script.
#
# Produces a real, bootable x86_64 NoxL Live/Install ISO using archiso.
# Must be run on an Arch Linux host (or the archlinux WSL2 distro / a VM)
# with `archiso` installed and root privileges.
#
# Usage:
#   sudo ./scripts/build-iso.sh
#
# Output:
#   build/noxl-*.iso
#
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
PROFILE_DIR="${REPO_ROOT}/system/live"
BUILD_DIR="${REPO_ROOT}/build"
OUT_DIR="${BUILD_DIR}/out"

log()  { printf '\033[1;35m[noxl]\033[0m %s\n' "$*"; }
err()  { printf '\033[1;31m[noxl]\033[0m %s\n' "$*" >&2; }
die()  { err "$*"; exit 1; }

# --- sanity checks -----------------------------------------------------------
[ "$(id -u)" -eq 0 ] || die "Must run as root (use sudo)."
command -v mkarchiso >/dev/null 2>&1 || die \
  "mkarchiso not found. Install archiso: sudo pacman -S archiso"
[ -f "${PROFILE_DIR}/profiledef.sh" ] || die "Missing profiledef.sh in ${PROFILE_DIR}"

# --- regenerate the OBSIDIAN wallpaper so the ISO always has a fresh one -----
if command -v python3 >/dev/null 2>&1; then
  log "Generating OBSIDIAN wallpaper..."
  python3 "${REPO_ROOT}/scripts/gen-wallpaper.py" \
    "${PROFILE_DIR}/airootfs/usr/share/noxl/wallpapers/obsidian-default.png"
else
  log "python3 not found; using existing wallpaper (if present)."
fi

# --- build -------------------------------------------------------------------
mkdir -p "${BUILD_DIR}" "${OUT_DIR}"

log "Building NoxL ISO (this downloads packages and may take a while)..."
log "Profile: ${PROFILE_DIR}"
log "Output:  ${OUT_DIR}"

mkarchiso \
  -v \
  -w "${BUILD_DIR}/work" \
  -o "${OUT_DIR}" \
  "${PROFILE_DIR}"

# --- result ------------------------------------------------------------------
ISO_PATH="$(ls -t "${OUT_DIR}"/noxl-*.iso 2>/dev/null | head -n1 || true)"
if [ -n "${ISO_PATH}" ] && [ -f "${ISO_PATH}" ]; then
  log "ISO built successfully:"
  ls -lh "${ISO_PATH}"
  echo
  log "Boot test with QEMU:"
  echo "  qemu-system-x86_64 -m 4G -smp 4 -cdrom '${ISO_PATH}' -boot d"
else
  die "Build finished but no ISO was produced. Check output above."
fi