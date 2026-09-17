#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
python scripts/check_book.py
if command -v markdownlint-cli2 >/dev/null 2>&1; then markdownlint-cli2 "src/**/*.md" "*.md" --config quality/.markdownlint.json; else echo "markdownlint-cli2: skipped (not installed)"; fi
if command -v vale >/dev/null 2>&1; then vale --config quality/.vale.ini src; else echo "Vale: skipped (not installed)"; fi
if command -v lychee >/dev/null 2>&1; then lychee --config quality/lychee.toml "src/**/*.md"; else echo "lychee: skipped (not installed)"; fi
