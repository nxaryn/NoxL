# Build the NoxL desktop and install into the live image root.
#
# This script is referenced by docs/PHASE2.md. It builds the Rust workspace
# and copies the binaries into the live image's airootfs so they are present
# on the ISO.
#
# Usage (on a Linux host with Rust + archiso):
#   ./scripts/install-desktop-to-live.sh
#
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DESKTOP_DIR="${REPO_ROOT}/desktop"
LIVE_BIN="${REPO_ROOT}/system/live/airootfs/usr/local/bin"

log() { printf '\033[1;35m[noxl]\033[0m %s\n' "$*"; }
die() { printf '\033[1;31m[noxl]\033[0m %s\n' "$*" >&2; exit 1; }

command -v cargo >/dev/null 2>&1 || die "cargo not found. Install Rust first."

log "Building NoxL desktop..."
cd "${DESKTOP_DIR}"
cargo build --release

mkdir -p "${LIVE_BIN}"

log "Installing binaries to live image..."
for bin in noxl-compositor noxl-panel noxl-launcher noxl-notifications noxl-shell; do
  if [ -f "target/release/${bin}" ]; then
    cp "target/release/${bin}" "${LIVE_BIN}/"
    log "  installed ${bin}"
  else
    die "expected binary not found: target/release/${bin}"
  fi
done

log "Desktop installed into live image."