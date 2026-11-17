#!/usr/bin/env bash
#
# NoxL desktop test suite.
#
# Runs the full compile + unit test + lint pipeline. Works on any Linux host
# with Rust; ideal for WSL2 (fast feedback) or CI.
#
# Usage:
#   ./scripts/test-desktop.sh
#
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DESKTOP_DIR="${REPO_ROOT}/desktop"

log()  { printf '\033[1;35m[noxl]\033[0m %s\n' "$*"; }
pass() { printf '\033[1;32m[noxl]\033[0m %s\n' "$*"; }
fail() { printf '\033[1;31m[noxl]\033[0m %s\n' "$*" >&2; }

command -v cargo >/dev/null 2>&1 || {
    fail "cargo not found. Install Rust: https://rustup.rs"
    exit 1
}

cd "${DESKTOP_DIR}"

FAILED=0

# --- 1. Build ----------------------------------------------------------------
log "==> cargo build"
if cargo build 2>&1 | tail -5; then
    pass "build OK"
else
    fail "build FAILED"
    FAILED=1
fi

# --- 2. Tests ----------------------------------------------------------------
log "==> cargo test"
TEST_OUTPUT="$(cargo test 2>&1)"
echo "${TEST_OUTPUT}" | grep "test result"

PASSED="$(echo "${TEST_OUTPUT}" | grep -oE '[0-9]+ passed' | grep -oE '[0-9]+' | awk '{s+=$1} END {print s}')"
FAILED_TESTS="$(echo "${TEST_OUTPUT}" | grep -oE '[0-9]+ failed' | grep -oE '[0-9]+' | awk '{s+=$1} END {print s}')"

if [ "${FAILED_TESTS:-0}" -eq 0 ]; then
    pass "${PASSED} tests passed, 0 failed"
else
    fail "${FAILED_TESTS} tests FAILED"
    FAILED=1
fi

# --- 3. Clippy ---------------------------------------------------------------
log "==> cargo clippy"
if cargo clippy --all-targets 2>&1 | grep -q "^error"; then
    fail "clippy found errors"
    FAILED=1
else
    pass "clippy OK"
fi

# --- Summary -----------------------------------------------------------------
echo
if [ "${FAILED}" -eq 0 ]; then
    pass "All checks passed."
    exit 0
else
    fail "Some checks failed."
    exit 1
fi