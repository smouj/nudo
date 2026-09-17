#!/usr/bin/env python3
from pathlib import Path
import re, subprocess, shutil, sys
ROOT=Path(__file__).resolve().parents[1]
SRC=ROOT/'src'; GEN=ROOT/'typst'/'generated'; GEN.mkdir(parents=True,exist_ok=True)
summary=(SRC/'SUMMARY.md').read_text(encoding='utf-8')
links=re.findall(r"\[[^\]]+\]\(([^)#]+\.md)\)", summary)
out=[]
prev_parent=None
for idx,rel in enumerate(links):
    src=SRC/rel
    parent=Path(rel).parent.as_posix()
    cmd=['pandoc','-f','gfm+raw_html','-t','typst',str(src)]
    try: txt=subprocess.check_output(cmd,text=True)
    except FileNotFoundError:
        print('pandoc is required to prepare Typst content',file=sys.stderr); sys.exit(2)
    # Raw HTML helper blocks have no PDF value; keep their text only.
    txt=txt.replace('#raw("<div class=\\"status-key\\">")','').replace('#raw("</div>")','')
    prefix = '#pagebreak()\n' if prev_parent is not None and parent != prev_parent else ''
    out.append(prefix + f'// ---- {rel} ----\n'+txt+'\n')
    prev_parent=parent
(GEN/'book-content.typ').write_text('\n'.join(out),encoding='utf-8')
print(f'Prepared {len(links)} chapters -> {GEN/"book-content.typ"}')
