#!/usr/bin/env python3
from pathlib import Path
import re, subprocess, html, shutil
ROOT=Path(__file__).resolve().parents[1]; SRC=ROOT/'src'; OUT=ROOT/'output'/'html-preview'
if OUT.exists(): shutil.rmtree(OUT)
(OUT/'assets').mkdir(parents=True)
shutil.copy2(ROOT/'theme'/'css'/'tokens.css', OUT/'assets'/'tokens.css')
shutil.copy2(ROOT/'theme'/'css'/'nudo-book.css', OUT/'assets'/'nudo-book.css')
shutil.copy2(ROOT/'theme'/'images'/'paper-noise.png', OUT/'assets'/'paper-noise.png')
summary=(SRC/'SUMMARY.md').read_text(encoding='utf-8')
entries=[]
for line in summary.splitlines():
    m=re.match(r'\s*-?\s*\[([^]]+)\]\(([^)]+\.md)\)',line)
    if m: entries.append((m.group(1),m.group(2)))
nav=''.join(f'<a href="{rel[:-3]}.html">{html.escape(title)}</a>' for title,rel in entries)
css='<link rel="stylesheet" href="'+('../'*0)+'assets/tokens.css"><link rel="stylesheet" href="assets/nudo-book.css">'
for title,rel in entries:
    src=SRC/rel
    frag=subprocess.check_output(['pandoc','-f','gfm+raw_html','-t','html5',str(src)],text=True)
    depth=len(Path(rel).parts)-1
    prefix='../'*depth
    nav2=''.join(f'<a href="{prefix+rr[:-3]}.html">{html.escape(tt)}</a>' for tt,rr in entries)
    page=f"""<!doctype html><html><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>{html.escape(title)} · NUDO</title><link rel="stylesheet" href="{prefix}assets/tokens.css"><link rel="stylesheet" href="{prefix}assets/nudo-book.css"><style>.preview-shell{{display:grid;grid-template-columns:280px 1fr;min-height:100vh}}.preview-nav{{padding:28px 20px;background:#e6dcc9;border-right:1px solid #B8AB93;position:sticky;top:0;height:100vh;overflow:auto;box-sizing:border-box;font-family:Arial,sans-serif}}.preview-nav b{{display:block;font:800 22px Arial;margin-bottom:18px;letter-spacing:.08em}}.preview-nav b span{{color:#3F85FF}}.preview-nav a{{display:block;padding:5px 8px;text-decoration:none;color:#444;font-size:13px}}.preview-main{{padding:45px 6vw;max-width:900px}}@media(max-width:800px){{.preview-shell{{display:block}}.preview-nav{{position:relative;height:auto;max-height:300px}}}}</style></head><body><div class="preview-shell"><nav class="preview-nav"><b>NUD<span>O</span></b>{nav2}</nav><main class="preview-main">{frag}</main></div></body></html>"""
    dst=OUT/(rel[:-3]+'.html'); dst.parent.mkdir(parents=True,exist_ok=True); dst.write_text(page,encoding='utf-8')
# landing alias
(OUT/'index.html').write_text((OUT/'index.html').read_text(encoding='utf-8'),encoding='utf-8')
print(f'Built standalone HTML preview: {len(entries)} pages')
