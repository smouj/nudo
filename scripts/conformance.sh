#!/usr/bin/env bash
#
# Runs the language conformance corpus.
#
# The corpus lives in tests/conformance and is implementation-neutral; this
# script drives it with the reference implementation. A failure here means the
# implementation and the corpus disagree, which is a bug in the implementation
# unless the corpus was changed on purpose through the NEP process.
#
set -euo pipefail

cd "$(dirname "$0")/.."

cargo test --package nudo-lexer --test conformance -- --nocapture
cargo test --package nudo-parser --test conformance -- --nocapture

echo "conformance corpus: ok"
