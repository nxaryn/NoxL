#!/usr/bin/env bash
#
# NoxL QEMU boot-test helper.
#
# Boots the built ISO in QEMU/KVM for verification. Checks for KVM availability
# and falls back to TCG (slower) if unavailable.
#
# Usage:
#   ./scripts/boot-test-qemu.sh [path-to-iso]
#
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
OUT_DIR="${REPO_ROOT}/build/out"

ISO="${1:-}"
if [ -z "${ISO}" ]; then
  ISO="$(ls -t "${OUT_DIR}"/noxl-*.iso 2>/dev/null | head -n1 || true)"
fi
[ -n "${ISO}" ] && [ -f "${ISO}" ] || {
  echo "usage: $0 [path-to-iso]" >&2
  echo "error: no ISO found in ${OUT_DIR}" >&2
  exit 1
}

command -v qemu-system-x86_64 >/dev/null 2>&1 || {
  echo "error: qemu-system-x86_64 not found. Install qemu-desktop." >&2
  exit 1
}

ACCEL=""
if [ -e /dev/kvm ] && [ -r /dev/kvm ] && [ -w /dev/kvm ]; then
  ACCEL="-enable-kvm -cpu host"
  echo "[noxl] Using KVM acceleration."
else
  echo "[noxl] /dev/kvm unavailable; using TCG (software emulation, slower)."
fi

echo "[noxl] Booting ${ISO} (4 GB RAM, 4 vCPUs, UEFI via OVMF if present)..."

OVMF_CODE=""
OVMF_VARS=""
if [ -f /usr/share/edk2/ovmf/OVMF_CODE.fd ]; then
  OVMF_CODE="-drive if=pflash,format=raw,readonly=on,file=/usr/share/edk2/ovmf/OVMF_CODE.fd"
  cp -n /usr/share/edk2/ovmf/OVMF_VARS.fd /tmp/noxl-OVMF_VARS.fd 2>/dev/null || true
  [ -f /tmp/noxl-OVMF_VARS.fd ] && \
    OVMF_VARS="-drive if=pflash,format=raw,file=/tmp/noxl-OVMF_VARS.fd"
fi

# shellcheck disable=SC2086
exec qemu-system-x86_64 \
  -m 4G -smp 4 \
  ${ACCEL} \
  ${OVMF_CODE} ${OVMF_VARS} \
  -cdrom "${ISO}" \
  -boot d \
  -vga virtio \
  -device virtio-net-pci,netdev=n0 -netdev user,id=n0 \
  -device virtio-ramfb