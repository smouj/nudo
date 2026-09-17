#!/usr/bin/env bash
#
# The canonical check for the NUDO repository.
#
# This script is the single definition of "this repository is correct". CI runs
# the same steps through the same entry point, so a contributor can reproduce a
# CI failure locally, and CI can never drift from the documented pipeline.
#
# Usage:
#   scripts/check.sh
#
# Environment:
#   NUDO_CI=1   add --locked to cargo invocations (CI sets this)
#
set -euo pipefail

cd "$(dirname "$0")/.."

LOCKED=""
if [ "${NUDO_CI:-0}" = "1" ]; then
    LOCKED="--locked"
    echo "NUDO_CI=1: cargo will refuse to change Cargo.lock"
fi

step() {
    printf '\n=== %s ===\n' "$1"
}

step "cargo fmt --all --check"
cargo fmt --all --check

step "cargo clippy (warnings denied)"
cargo clippy --workspace --all-targets --all-features $LOCKED -- -D warnings

step "cargo build --workspace --all-targets"
cargo build --workspace --all-targets $LOCKED

step "cargo test --workspace"
cargo test --workspace $LOCKED

step "conformance corpus"
./scripts/conformance.sh

step "rustdoc (broken links denied)"
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --quiet $LOCKED

step "documentation checks"
python3 scripts/check-docs.py

step "workflow policy"
python3 scripts/check-workflows.py

step "documented console output"
python3 scripts/check-console.py

printf '\nall checks passed\n'
