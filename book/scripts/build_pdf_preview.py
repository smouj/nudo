#!/usr/bin/env python3
from pathlib import Path
import re, subprocess, shutil
ROOT=Path(__file__).resolve().parents[1]; SRC=ROOT/'src'; OUT=ROOT/'output'/'pdf'; OUT.mkdir(parents=True,exist_ok=True)
summary=(SRC/'SUMMARY.md').read_text(encoding='utf-8')
links=re.findall(r"\[[^\]]+\]\(([^)#]+\.md)\)", summary)
combined=ROOT/'output'/'_combined-preview.md'
cover="""<section class="cover"><div class="cover-mark">NUDO // TECHNICAL PUBLICATION</div><div class="cover-title">NUDO<small>The Technical Book</small></div><div class="cover-edition">Engineering Paper Edition<br>Programming language for humans and agents</div><div class="revbox"><div>DOCUMENT<br><b>NUDO-BOOK-001</b></div><div>REVISION<br><b>R0</b></div><div>STATUS<br><b>LIVING / PRE-ALPHA</b></div></div></section>\n"""
# Build a readable TOC from SUMMARY labels, not filenames.
entries=re.findall(r"\[([^\]]+)\]\(([^)#]+\.md)\)", summary)
toc_items=''.join(f'<li><span>{i+1:02d}</span> · {title}</li>' for i,(title,_) in enumerate(entries))
toc=f'<section class="toc"><h1>Contents</h1><ul class="toc-list">{toc_items}</ul></section>\n'
parts=[cover,toc]
prev_parent=None
for title,rel in entries:
    txt=(SRC/rel).read_text(encoding='utf-8')
    parent=Path(rel).parent.as_posix()
    major=(prev_parent is None or parent!=prev_parent)
    klass='chapter major' if major else 'chapter'
    parts.append(f'\n<div class="{klass}">\n\n'+txt+'\n\n</div>\n')
    prev_parent=parent
combined.write_text('\n'.join(parts),encoding='utf-8')
html=ROOT/'output'/'_combined-preview.html'
subprocess.check_call(['pandoc','-f','gfm+raw_html','-t','html5','--standalone','--metadata','title=The NUDO Technical Book',str(combined),'-o',str(html)])
# inject stylesheet absolute URI and paper class
s=html.read_text(encoding='utf-8')
css=(ROOT/'theme'/'css'/'nudo-print.css').resolve().as_uri()
s=s.replace('</head>',f'<link rel="stylesheet" href="{css}"></head>').replace('<body>','<body class="paper">')
html.write_text(s,encoding='utf-8')
try:
    subprocess.check_call(['weasyprint',str(html),str(OUT/'nudo-technical-book-preview.pdf')])
except FileNotFoundError:
    print('weasyprint not found; install it or use Typst production pipeline')
    raise
print('Built PDF preview:',OUT/'nudo-technical-book-preview.pdf')
