#!/usr/bin/env bash
#
# Prepares a machine to build NUDO from source.
#
# Installs the Rust toolchain through rustup when it is missing, adds the
# components the pipeline needs, then builds the workspace. Nothing is
# installed system-wide and no elevation is requested.
#
set -euo pipefail

cd "$(dirname "$0")/.."

if ! command -v cargo >/dev/null 2>&1; then
    if ! command -v rustup >/dev/null 2>&1; then
        echo "installing rustup (user-local, https://rustup.rs)"
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs -o /tmp/nudo-rustup-init.sh
        sh /tmp/nudo-rustup-init.sh -y --profile minimal --default-toolchain stable --no-modify-path
        rm -f /tmp/nudo-rustup-init.sh
        export PATH="$HOME/.cargo/bin:$PATH"
    fi
fi

if ! command -v cargo >/dev/null 2>&1; then
    echo "cargo is still unavailable; add \$HOME/.cargo/bin to PATH and re-run" >&2
    exit 1
fi

echo "toolchain: $(cargo --version), $(rustc --version)"

if command -v rustup >/dev/null 2>&1; then
    rustup component add rustfmt clippy
fi

echo "building the workspace"
cargo build --workspace

echo
echo "ready. Next:"
echo "  cargo run --package nudo-cli -- check examples/00-hello-world/main.nudo"
echo "  scripts/check.sh"
