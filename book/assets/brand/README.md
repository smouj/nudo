# Brand asset integration

This package does not redraw or recolour the NUDO logo. Run
`scripts/sync_brand_assets.py` inside the real NUDO repository to copy the
official delivered files from `assets/logo/` into the book source for web output.

Expected delivered assets:

- `nudo-symbol-primary.png`
- `nudo-symbol-light.png`
- `nudo-symbol-dark.png`
- `nudo-horizontal-dark.png`

The pending horizontal-primary/light variants are intentionally not synthesized.
