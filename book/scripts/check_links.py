#!/usr/bin/env python3
"""Resolves every local reference in the built site against the files on disk.

The failure this catches is silent. A link to a repository document outside
`book/src` is rewritten by mdBook into `<name>.html` relative to the book root,
where nothing is ever written: the page renders, the link 404s, and no tool
complains. The published site had exactly that defect.

Run after building. `book/scripts/build_site.py` and the `Book` workflow both do.
"""

from __future__ import annotations

import os
import re
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
SITE = os.path.join(ROOT, "book", "output", "site")
REFERENCES = re.compile(r'(?:href|src)="([^"#?]+)"')
SKIP = ("http://", "https://", "//", "mailto:", "data:")


def main() -> int:
    if not os.path.isdir(SITE):
        print(f"{os.path.relpath(SITE, ROOT)} does not exist; build the site first", file=sys.stderr)
        return 2

    checked = 0
    missing: dict[str, str] = {}

    for directory, _, filenames in os.walk(SITE):
        for filename in filenames:
            if not filename.endswith(".html"):
                continue
            page = os.path.join(directory, filename)
            with open(page, encoding="utf-8", errors="replace") as handle:
                html = handle.read()
            for reference in REFERENCES.findall(html):
                if reference.startswith(SKIP):
                    continue
                checked += 1
                target = os.path.normpath(os.path.join(directory, reference))
                if not os.path.exists(target):
                    relative = os.path.relpath(page, SITE)
                    missing.setdefault(reference, relative)

    pages = sum(1 for _, _, files in os.walk(SITE) for name in files if name.endswith(".html"))
    print(f"site references: {checked} checked across {pages} pages")

    if missing:
        print("\nbroken references:")
        for reference, page in sorted(missing.items()):
            print(f"  {reference}   (first seen in {page})")
        return 1

    print("every local reference resolves")
    return 0


if __name__ == "__main__":
    sys.exit(main())
