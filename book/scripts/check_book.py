#!/usr/bin/env python3
from pathlib import Path
import re, sys
ROOT = Path(__file__).resolve().parents[1]
SRC = ROOT / "src"
summary = (SRC / "SUMMARY.md").read_text(encoding="utf-8")
links = re.findall(r"\[[^\]]+\]\(([^)#]+\.md)\)", summary)
errors=[]
seen=set()
for rel in links:
    p=(SRC/rel).resolve()
    if not p.exists(): errors.append(f"missing SUMMARY target: {rel}"); continue
    if p in seen: errors.append(f"duplicate SUMMARY target: {rel}")
    seen.add(p)
    txt=p.read_text(encoding='utf-8')
    heads=re.findall(r'^(#+)\s+(.+)$',txt,re.M)
    if not heads or len(heads[0][0])!=1: errors.append(f"{rel}: must start with one H1")
    if '> **Status:**' not in txt: errors.append(f"{rel}: missing status")
    if '> **Summary:**' not in txt: errors.append(f"{rel}: missing summary")
    prev=0
    for hashes,title in heads:
        level=len(hashes)
        if prev and level>prev+1: errors.append(f"{rel}: heading jump H{prev}->H{level} at {title}")
        prev=level
    if re.search(r'\b(TODO|TBD|FIXME)\b', txt): errors.append(f"{rel}: unresolved placeholder")
all_md={p.resolve() for p in SRC.rglob('*.md') if p.name!='SUMMARY.md'}
orphans=sorted(str(p.relative_to(SRC)) for p in all_md-seen)
if orphans: errors.append('orphan pages not in SUMMARY: '+', '.join(orphans))
if errors:
    print('NUDO book validation failed:')
    for e in errors: print(' -',e)
    sys.exit(1)
print(f'NUDO book validation: OK ({len(seen)} pages, no orphans)')
