#!/usr/bin/env python3
"""Records which English revision each translated chapter was made from.

Run this after translating, and only after translating: it writes the hash of the
English source next to the translated file, so that a later English change is
visible as "this translation is behind" instead of being invisible.

    python3 book/scripts/stamp_translation.py --lang es

Recording the hash is a claim that the translation was made from that revision.
Stamping without translating is exactly the kind of dishonesty the book's status
labels exist to prevent.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
SOURCE = os.path.join(ROOT, "book", "src")
MANIFEST_NAME = "translation.json"


def english_chapters() -> dict[str, str]:
    """Maps a chapter's path relative to book/src to the hash of its English text."""
    chapters = {}
    for directory, subdirectories, filenames in os.walk(SOURCE):
        subdirectories[:] = sorted(name for name in subdirectories if name != "diagrams")
        for filename in sorted(filenames):
            if not filename.endswith(".md"):
                continue
            absolute = os.path.join(directory, filename)
            relative = os.path.relpath(absolute, SOURCE)
            with open(absolute, "rb") as handle:
                chapters[relative] = hashlib.sha256(handle.read()).hexdigest()
    return chapters


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--lang", required=True, help="language code, for example es")
    arguments = parser.parse_args()

    language_dir = os.path.join(ROOT, "book", "i18n", arguments.lang)
    if not os.path.isdir(language_dir):
        print(f"no such language directory: {os.path.relpath(language_dir, ROOT)}", file=sys.stderr)
        return 2

    manifest = {}
    for relative, digest in english_chapters().items():
        if os.path.isfile(os.path.join(language_dir, "src", relative)):
            manifest[relative] = digest

    if not manifest:
        print("nothing translated yet: no stamps written", file=sys.stderr)
        return 1

    path = os.path.join(language_dir, MANIFEST_NAME)
    with open(path, "w", encoding="utf-8") as handle:
        json.dump(manifest, handle, indent=2, sort_keys=True)
        handle.write("\n")

    total = len(english_chapters())
    print(f"{arguments.lang}: stamped {len(manifest)}/{total} chapters from the current English revision")
    print(f"  {os.path.relpath(path, ROOT)}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
