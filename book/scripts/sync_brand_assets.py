#!/usr/bin/env python3
from pathlib import Path
import argparse, shutil, sys
OFFICIAL = [
    'nudo-symbol-primary.png',
    'nudo-symbol-light.png',
    'nudo-symbol-dark.png',
    'nudo-horizontal-dark.png',
]
parser=argparse.ArgumentParser()
parser.add_argument('--repo-root', default='.')
a=parser.parse_args()
repo=Path(a.repo_root).resolve()
src=repo/'assets'/'logo'
book=Path(__file__).resolve().parents[1]
dst=book/'src'/'assets'/'brand'
typdst=book/'typst'/'assets'/'brand'
dst.mkdir(parents=True,exist_ok=True)
typdst.mkdir(parents=True,exist_ok=True)
missing=[]
for name in OFFICIAL:
    p=src/name
    if p.exists():
        shutil.copy2(p,dst/name)
        shutil.copy2(p,typdst/name)
    else: missing.append(name)
if missing:
    print('Official logo assets not available; text fallback remains active:')
    for n in missing: print(' -',n)
else:
    (book/'typst'/'brand.typ').write_text('#let brand-mark() = image("assets/brand/nudo-symbol-primary.png", width: 15mm)\n', encoding='utf-8')
    print('Official NUDO brand assets synchronized without modification; Typst cover now uses the official primary symbol.')
