#!/usr/bin/env bash
#
# NoxL Desktop build script.
#
# Builds all NoxL desktop crates in release mode.
# Must be run on a Linux host with Rust, Wayland, and D-Bus development files.
#
# Usage:
#   ./scripts/build-desktop.sh
#
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DESKTOP_DIR="${REPO_ROOT}/desktop"

log() { printf '\033[1;35m[noxl]\033[0m %s\n' "$*"; }
die() { printf '\033[1;31m[noxl]\033[0m %s\n' "$*" >&2; exit 1; }

command -v cargo >/dev/null 2>&1 || die \
  "cargo not found. Install Rust: https://rustup.rs"

log "Building NoxL desktop (release)..."
cd "${DESKTOP_DIR}"
cargo build --release

log "Build complete. Binaries in ${DESKTOP_DIR}/target/release/:"
ls -1 "${DESKTOP_DIR}"/target/release/noxl-* 2>/dev/null | sed 's|.*/||' || true

echo
log "Install to /usr/local/bin (optional):"
echo "  sudo cp target/release/noxl-* /usr/local/bin/"