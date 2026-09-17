#!/usr/bin/env python3
"""Builds every language of the manual into one site.

Languages that are still being translated, or that do not build, are left out and
reported rather than taking the whole site down with them: the finished languages
should be online while the others are being written. Pass --strict to require
every language, which is what a release gate should do.


Layout of the result, which mirrors how GitHub Pages serves a project site:

    book/output/site/            English (canonical), at the site root
    book/output/site/es/         Español
    book/output/site/zh/         简体中文
    book/output/site/ja/         日本語
    book/output/site/de/         Deutsch

English lives at the root rather than under /en/ so that the canonical language
has the shortest URL, and so that an existing link to the book keeps working.

Diagrams are shared: they are copied from the canonical tree into each language
tree before building, so every edition includes them with the same relative path
and there is exactly one copy in the repository.
"""

from __future__ import annotations

import json
import os
import shutil
import subprocess
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
BOOK = os.path.join(ROOT, "book")
OUTPUT = os.path.join(BOOK, "output", "site")

# (code, book root relative to the repository root, output path relative to the site root)
LANGUAGES = [
    ("en", "book", ""),
    ("es", "book/i18n/es", "es"),
    ("zh", "book/i18n/zh", "zh"),
    ("ja", "book/i18n/ja", "ja"),
    ("de", "book/i18n/de", "de"),
]


def sync_shared(book_root: str) -> tuple[int, int]:
    """Copies the canonical theme and diagrams into a language tree.

    mdBook copies `additional-css` and `additional-js` into the output, and it
    does not copy files from outside the book root. Referencing `../../theme`
    therefore links a stylesheet that is never written to the site, and the
    language renders with no design at all — silently, because mdBook does not
    consider it an error.

    Copying the shared directories into each book root before building avoids that
    entirely, and keeps book/theme as the single place a design change is made.
    """
    theme_source = os.path.join(BOOK, "theme")
    theme_target = os.path.join(ROOT, book_root, "theme")
    diagrams_source = os.path.join(BOOK, "src", "diagrams")
    diagrams_target = os.path.join(ROOT, book_root, "src", "diagrams")

    if os.path.abspath(book_root) != os.path.abspath("book"):
        if os.path.isdir(theme_source):
            shutil.rmtree(theme_target, ignore_errors=True)
            shutil.copytree(theme_source, theme_target)
        if os.path.isdir(diagrams_source):
            shutil.rmtree(diagrams_target, ignore_errors=True)
            shutil.copytree(diagrams_source, diagrams_target)

    themes = len(os.listdir(theme_target)) if os.path.isdir(theme_target) else 0
    diagrams = len(os.listdir(diagrams_target)) if os.path.isdir(diagrams_target) else 0
    return themes, diagrams


def sync_diagrams(book_root: str) -> int:
    """Copies the canonical diagrams into a language tree, or does nothing for English."""
    source = os.path.join(BOOK, "src", "diagrams")
    target = os.path.join(ROOT, book_root, "src", "diagrams")
    if os.path.abspath(os.path.dirname(source)) == os.path.abspath(os.path.dirname(target)):
        return 0
    if not os.path.isdir(source):
        return 0
    shutil.rmtree(target, ignore_errors=True)
    shutil.copytree(source, target)
    return len(os.listdir(target))


def main() -> int:
    arguments = [argument for argument in sys.argv[1:] if argument != "--strict"]
    strict = "--strict" in sys.argv[1:]
    languages = arguments
    selected = [entry for entry in LANGUAGES if not languages or entry[0] in languages]
    if not selected:
        print(f"no such language: {', '.join(languages)}", file=sys.stderr)
        return 2

    shutil.rmtree(OUTPUT, ignore_errors=True)
    os.makedirs(OUTPUT, exist_ok=True)

    failures = 0
    skipped: list[str] = []
    published: list[str] = []
    for code, book_root, destination in selected:
        book_dir = os.path.join(ROOT, book_root)
        if not os.path.isfile(os.path.join(book_dir, "book.toml")):
            print(f"{code}: {book_root}/book.toml is missing", file=sys.stderr)
            failures += 1
            continue

        # A language whose translation has not started is skipped rather than
        # treated as a failure: the site must deploy from the first day, and each
        # language should appear as it is finished. A language that has started
        # and does not build *is* a failure.
        if not os.path.isfile(os.path.join(book_dir, "src", "SUMMARY.md")):
            print(f"=== {code}: not translated yet — skipped")
            skipped.append(code)
            continue

        themes, diagrams = sync_shared(book_root)
        synced = f" ({themes} theme files, {diagrams} diagrams synced)" if themes or diagrams else ""
        print(f"=== {code}: mdbook build {book_root}{synced}")

        result = subprocess.run(
            ["mdbook", "build", book_root],
            cwd=ROOT,
            check=False,
            text=True,
            capture_output=True,
        )
        if result.returncode != 0:
            # A language that does not build is left out of the site rather than
            # taking the site down with it. The reason is printed, and --strict
            # turns any omission into a failure, so a release can require every
            # language to be present.
            print(result.stdout, end="")
            print(result.stderr, end="", file=sys.stderr)
            print(f"=== {code}: build failed — left out of the site", file=sys.stderr)
            skipped.append(f"{code} (build failed)")
            if strict:
                failures += 1
            continue

        built = os.path.join(book_dir, "output", "html")
        target = os.path.join(OUTPUT, destination) if destination else OUTPUT
        shutil.copytree(built, target, dirs_exist_ok=True)
        pages = sum(1 for _, _, files in os.walk(target) for name in files if name.endswith(".html"))
        published.append(code)
        print(f"    {code}: {pages} pages → book/output/site/{destination or ''}")

    if failures:
        print(f"\n{failures} language(s) failed", file=sys.stderr)
        return 1

    # The switcher reads this, so it never offers a language the site does not
    # have. Built from what actually landed on disk, not from what was requested.
    with open(os.path.join(OUTPUT, "languages.json"), "w", encoding="utf-8") as handle:
        json.dump({"languages": published, "default": LANGUAGES[0][0]}, handle, indent=2)
        handle.write("\n")

    # Resolve every local reference in the built site before declaring success:
    # a broken link renders perfectly and fails silently.
    links = subprocess.run(
        ["python3", os.path.join(BOOK, "scripts", "check_links.py")],
        cwd=ROOT,
        check=False,
        text=True,
        capture_output=True,
    )
    print(links.stdout, end="")
    if links.returncode != 0:
        print(links.stderr, end="", file=sys.stderr)
        failures += 1

    summary = f"\nsite built: {len(published)} language(s) in {os.path.relpath(OUTPUT, ROOT)}"
    if skipped:
        summary += f"\nnot translated yet, skipped: {', '.join(skipped)}"
    print(summary)
    print(f"languages.json: {published}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
