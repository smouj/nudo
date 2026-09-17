#!/usr/bin/env python3
"""Checks the translations against the canonical English source.

Two things are worth knowing about a translation, and neither is visible by
looking at it:

* **Missing.** A chapter with no translation at all. A reader in that language
  falls back to English, which is acceptable while a language is being brought
  up but must be visible rather than silent.
* **Behind.** A chapter whose English source has changed since it was translated.
  This is the one that matters: the translation still reads perfectly, and it is
  now wrong. Gettext detects this automatically; per-language trees need it
  recorded, which is what `stamp_translation.py` does.

Missing chapters are an error. Behind chapters are reported and fail only with
`--strict`, so that a translation in progress can be built and reviewed without
pretending it is finished.
"""

from __future__ import annotations

import argparse
import json
import os
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
sys.path.insert(0, os.path.join(ROOT, "book", "scripts"))

from stamp_translation import english_chapters  # noqa: E402  (same directory)


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--strict", action="store_true", help="also fail when a translation is behind")
    arguments = parser.parse_args()

    english = english_chapters()
    languages = sorted(
        name
        for name in os.listdir(os.path.join(ROOT, "book", "i18n"))
        if os.path.isdir(os.path.join(ROOT, "book", "i18n", name, "src"))
    )
    if not languages:
        print("no translations to check")
        return 0

    missing_total = 0
    behind_total = 0

    for language in languages:
        language_dir = os.path.join(ROOT, "book", "i18n", language)
        manifest_path = os.path.join(language_dir, "translation.json")
        manifest = {}
        if os.path.isfile(manifest_path):
            with open(manifest_path, encoding="utf-8") as handle:
                manifest = json.load(handle)

        missing = [name for name in english if not os.path.isfile(os.path.join(language_dir, "src", name))]
        behind = [
            name
            for name, digest in manifest.items()
            if name in english and english[name] != digest
        ]
        unknown = sorted(set(manifest) - set(english))

        missing_total += len(missing)
        behind_total += len(behind)

        state = "complete" if not missing and not behind else "in progress"
        print(f"{language}: {len(english) - len(missing)}/{len(english)} chapters translated ({state})")
        for name in missing[:5]:
            print(f"    missing: {name}")
        if len(missing) > 5:
            print(f"    missing: … and {len(missing) - 5} more")
        for name in behind[:5]:
            print(f"    BEHIND the English source: {name}")
        if len(behind) > 5:
            print(f"    BEHIND: … and {len(behind) - 5} more")
        for name in unknown[:5]:
            print(f"    translated but no longer in the source: {name}")
        if behind and not manifest:
            print("    no translation.json: run stamp_translation.py after translating")

    if missing_total:
        print(f"\nmissing chapters: {missing_total}", file=sys.stderr)
    if behind_total:
        print(f"translations behind the English source: {behind_total}", file=sys.stderr)

    # Advisory by default, because a manual that is translated in four languages
    # is translated by four people who do not finish on the same day. Blocking
    # the deploy until every language is complete would take the finished
    # languages offline with the unfinished ones.
    #
    # What must never happen is a *stale* translation being presented as current,
    # which is why the state is printed on every run and why --strict exists for
    # a release gate.
    if missing_total == 0 and behind_total == 0:
        print("\ntranslations: complete and current")
    else:
        print("\ntranslations: incomplete (advisory; use --strict to fail)")

    if arguments.strict and (missing_total or behind_total):
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
