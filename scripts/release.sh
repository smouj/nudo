#!/usr/bin/env bash
#
# Builds release artefacts for the supported targets.
#
# Prepared for milestone M0 and exercised by .github/workflows/release.yml.
# No release is published from this script: it only produces files and
# checksums, so that a human can inspect them before tagging.
#
# Usage:
#   scripts/release.sh [--target <triple>]...
#
# Without --target, the host target is built. Cross targets need their Rust
# standard library installed (for example
# `rustup target add x86_64-unknown-linux-musl`).
#
set -euo pipefail

cd "$(dirname "$0")/.."

VERSION="$(cargo metadata --no-deps --format-version 1 | python3 -c 'import json,sys; print(json.load(sys.stdin)["packages"][0]["version"])')"
DIST="dist"
TARGETS=()

while [ $# -gt 0 ]; do
    case "$1" in
        --target)
            shift
            [ $# -gt 0 ] || { echo "--target needs a triple" >&2; exit 2; }
            TARGETS+=("$1")
            ;;
        -h|--help)
            sed -n '2,16p' "$0"
            exit 0
            ;;
        *)
            echo "unknown argument: $1" >&2
            exit 2
            ;;
    esac
    shift
done

if [ ${#TARGETS[@]} -eq 0 ]; then
    TARGETS=("$(rustc -vV | awk '/^host:/ {print $2}')")
fi

mkdir -p "$DIST"

for target in "${TARGETS[@]}"; do
    echo "=== building $target ==="
    cargo build --release --locked --target "$target" --package nudo-cli

    case "$target" in
        *windows*) archive="nudo-$VERSION-$target.zip" ;;
        *)         archive="nudo-$VERSION-$target.tar.gz" ;;
    esac

    staging="$(mktemp -d)"
    binary="target/$target/release/nudo"
    [ -f "$binary" ] || binary="$binary.exe"
    cp "$binary" "$staging/"
    cp LICENSE-MIT LICENSE-APACHE README.md "$staging/"

    if [ "${archive##*.}" = "zip" ]; then
        (cd "$staging" && zip -q -r "$OLDPWD/$DIST/$archive" .)
    else
        tar -czf "$DIST/$archive" -C "$staging" .
    fi
    rm -rf "$staging"
done

echo "=== checksums ==="
(cd "$DIST" && sha256sum ./* > SHA256SUMS && cat SHA256SUMS)

echo
echo "age: artefacts are in $DIST/. No release was published."
echo "The release workflow attaches these files, a checksum file and an SBOM."
