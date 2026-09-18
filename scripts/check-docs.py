#!/usr/bin/env python3
"""Repository hygiene checks that a compiler cannot make.

Run by `scripts/check.sh` and by the `docs` job in CI. It answers three
questions with no third-party dependency:

1. Do the relative links in every Markdown file resolve to a real path?
2. Does any file still mention a name or artefact this project abandoned?
3. Do the machine-readable files (TOML, JSON, YAML) parse?

All three read the same file set, and that set is the *repository's content*:
`git ls-files --cached --others --exclude-standard`. Content git ignores is not
part of the repository, so scanning it makes this checker report on something
the repository does not contain, and pass or fail depending on what unrelated
tooling happens to have left on disk. It used to walk the whole directory tree
and skip a hand-written list of names (`.git target dist node_modules`), which
is always one `.gitignore` line behind: a scratch note under the ignored
`.openclaw/` directory was enough to fail the documentation gate.

The rule needs a git work tree. Outside one, the checker says so and exits 1;
it never falls back to a wider scan.

Exit code is 0 when nothing is wrong, 1 when at least one check failed.
"""

from __future__ import annotations

import json
import os
import re
import subprocess
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))

# The checker is excluded from the text scan because it necessarily contains
# the very strings it looks for.
SELF = os.path.relpath(os.path.abspath(__file__), ROOT).replace(os.sep, "/")

TEXT_EXTENSIONS = (
    ".md",
    ".toml",
    ".yml",
    ".yaml",
    ".json",
    ".rs",
    ".nudo",
    ".ebnf",
    ".sh",
    ".ps1",
    ".py",
    ".txt",
    ".svg",
)

# The file set every check below reads, which is git's own answer to "what is in
# the repository". `--cached` is every tracked file; `--others
# --exclude-standard` adds every untracked file that is not ignored, so a new
# page is checked before it is staged as well as after. See the module docstring
# for why this is not a list of directory names.
REPOSITORY = (
    "git",
    "ls-files",
    "--cached",
    "--others",
    "--exclude-standard",
    "-z",
)

# Translation trees are checked by their own tools, not by this one.
#
# A translation in progress is a legitimate state: its table of contents lands
# before the chapters do. This checker assumes completeness, so running it over a
# half-finished translation reports work that is simply not finished yet.
# Instead:
#
#   * book/scripts/check_translations.py reports what is missing and what is
#     behind the English source, which is the question that matters there;
#   * book/scripts/check_links.py resolves every reference in the *built* site,
#     which finds a broken link in a finished translation more reliably than
#     reading the Markdown does.
#
# Residue scanning still covers these files: an abandoned project name in a
# translation is exactly as unwelcome as one in the English source.
TRANSLATION_SOURCE = "book/i18n/"

# Names and artefacts this project abandoned. Any occurrence is a leftover and
# fails the check.
FORBIDDEN = (
    "Kavra",
    "kavra",
    "Elyrn",
    "elyrn",
    "SmoujLang",
    "Lorem ipsum",
    "TODO TODO",
    "example.com",
    "placeholder text",
)

# Words to report but not fail on: `.nu` is a legitimate *mention* (NUDO
# deliberately does not use it) but never a legitimate source extension.
INFORMATIONAL = (".nu`", ".nu\"")


def is_translation_source(relative: str) -> bool:
    return relative.startswith(TRANSLATION_SOURCE) and "/src/" in relative


def repository_files():
    """Every file the repository contains, relative to ROOT, in git's order.

    One definition of the file set, for every check here, instead of each check
    guessing with its own list of directory names.
    """
    try:
        result = subprocess.run(
            REPOSITORY, cwd=ROOT, capture_output=True, check=False
        )
    except OSError as error:
        print(f"cannot run git to list the repository's files: {error}", file=sys.stderr)
        raise SystemExit(1) from error
    if result.returncode != 0:
        detail = result.stderr.decode("utf-8", "surrogateescape").strip()
        print("cannot list the repository's files with git:", file=sys.stderr)
        if detail:
            print(f"  {detail}", file=sys.stderr)
        print(
            "this checker scans the repository's content, so it must run inside "
            "a git work tree",
            file=sys.stderr,
        )
        raise SystemExit(1)
    names = result.stdout.decode("utf-8", "surrogateescape").split("\0")
    return [
        name
        for name in names
        if name and name != SELF and os.path.isfile(os.path.join(ROOT, name))
    ]


LINK = re.compile(r"\[[^\]]*\]\(([^)\s]+)(?:\s+\"[^\"]*\")?\)")


def check_links(errors):
    checked = 0
    for relative in repository_files():
        if not relative.endswith(".md") or is_translation_source(relative):
            continue
        absolute = os.path.join(ROOT, relative)
        with open(absolute, encoding="utf-8") as handle:
            text = handle.read()
        for target in LINK.findall(text):
            if target.startswith(("http://", "https://", "mailto:", "#")):
                continue
            path = target.split("#", 1)[0]
            if not path:
                continue
            checked += 1
            base = ROOT if path.startswith("/") else os.path.dirname(absolute)
            resolved = os.path.normpath(os.path.join(base, path.lstrip("/")))
            if not os.path.exists(resolved):
                errors.append(f"{relative}: link `{target}` does not resolve")
    print(f"links: {checked} relative links checked")


def check_residue(errors):
    informational = 0
    scanned = 0
    for relative in repository_files():
        if not relative.endswith(TEXT_EXTENSIONS):
            continue
        scanned += 1
        try:
            with open(os.path.join(ROOT, relative), encoding="utf-8") as handle:
                text = handle.read()
        except UnicodeDecodeError:
            errors.append(f"{relative}: not valid UTF-8")
            continue
        for needle in FORBIDDEN:
            if needle in text:
                errors.append(f"{relative}: contains abandoned name or artefact `{needle}`")
        for needle in INFORMATIONAL:
            if needle in text:
                informational += 1
    print(f"text files scanned: {scanned}")
    if informational:
        print(
            f"info: {informational} file(s) mention `.nu`; that is expected only "
            "when explaining why NUDO does not use it"
        )


def check_machine_readable(errors):
    toml_count = json_count = yaml_count = 0
    for relative in repository_files():
        path = os.path.join(ROOT, relative)
        if relative.endswith(".json"):
            json_count += 1
            try:
                with open(path, encoding="utf-8") as handle:
                    json.load(handle)
            except Exception as error:  # noqa: BLE001 - reported, not raised
                errors.append(f"{relative}: invalid JSON ({error})")
        elif relative.endswith(".toml"):
            try:
                import tomllib  # Python 3.11+
            except ImportError:
                continue
            toml_count += 1
            try:
                with open(path, "rb") as handle:
                    tomllib.load(handle)
            except Exception as error:  # noqa: BLE001 - reported, not raised
                errors.append(f"{relative}: invalid TOML ({error})")
        elif relative.endswith((".yml", ".yaml")):
            try:
                import yaml  # type: ignore
            except ImportError:
                continue
            yaml_count += 1
            try:
                with open(path, encoding="utf-8") as handle:
                    yaml.safe_load(handle)
            except Exception as error:  # noqa: BLE001 - reported, not raised
                errors.append(f"{relative}: invalid YAML ({error})")
    print(f"machine-readable files: {toml_count} TOML, {json_count} JSON, {yaml_count} YAML (best effort)")


def main() -> int:
    print(f"checking documentation under {ROOT}\n")
    errors: list[str] = []
    check_links(errors)
    check_residue(errors)
    check_machine_readable(errors)

    if errors:
        print("\ndocumentation problems:")
        for error in errors:
            print(f"  - {error}")
        return 1

    print("\ndocumentation checks passed")
    return 0


if __name__ == "__main__":
    sys.exit(main())
