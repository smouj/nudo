#!/usr/bin/env python3
"""Checks that documented console output is output the toolchain really prints.

Documentation drifts in a specific, embarrassing way: a message is reworded in
the code, the document keeps the old text, and a reader copies a command whose
output does not match what they see. This repository shipped exactly that bug —
four documents showed `0 errors, 0 warnings` while the tool printed
`0 errors and 0 warnings`.

So the check runs both ways:

1. every fragment below must appear in the tool's real output, which proves the
   documentation is not describing something that does not exist;
2. every fragment must appear in at least one Markdown file, which proves the
   documentation has not fallen behind the tool.

Fragments are deliberately the stable parts: version strings, summary lines and
diagnostic headers. Timings, absolute paths and caret rows are not checked,
because they legitimately vary between machines.

The second direction is only meaningful against the *repository's* documents.
The corpus is therefore the repository's content — `git ls-files --cached
--others --exclude-standard` — and not the working tree, because content git
ignores is not part of the repository: a scratch note quoting the old text would
satisfy the check and hide a document that has fallen behind, which is the very
drift this checker exists to catch. It used to walk the whole directory tree and
skip a hand-written list of names (`.git target dist node_modules`), which is
always one `.gitignore` line behind.

The file set is pinned by `scripts/test-check-console.py`. The rule needs a git
work tree: outside one the checker says so and exits 1, and it never falls back
to a wider scan.

Run by `scripts/check.sh` and by the `Docs` workflow, after a build of the CLI.
"""

from __future__ import annotations

import os
import subprocess
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
BINARY = os.path.join(ROOT, "target", "debug", "nudo")

# The checker is excluded from the corpus because it necessarily contains the
# very fragments it looks for.
SELF = os.path.relpath(os.path.abspath(__file__), ROOT).replace(os.sep, "/")

# (arguments, stream, fragments that must appear in the output and in the docs)
CASES = [
    (["--version"], "stdout", ["nudo 0.0.1 (pre-alpha)"]),
    (
        ["check", "examples/00-hello-world/main.nudo"],
        "stdout",
        ["checked 1 file: 0 errors and 0 warnings"],
    ),
    (
        ["check", "fixtures/invalid/unterminated-text.nudo"],
        "stderr",
        [
            "error[NDO1003]: unterminated text literal",
            "nudo: 1 error and 0 warnings",
        ],
    ),
    (
        ["check", "--dump-tokens", "examples/00-hello-world/main.nudo"],
        "stdout",
        ["# nudo-tokens v1"],
    ),
    (
        ["check", "--dump-tree", "examples/03-types/main.nudo"],
        "stdout",
        ["# nudo-tree v1"],
    ),
    (
        ["check", "--dump-resolutions", "examples/00-hello-world/main.nudo"],
        "stdout",
        ["# nudo-hir v1"],
    ),
]

# The file set the corpus is built from, which is git's own answer to "what is
# in the repository": `--cached` is every tracked file, `--others
# --exclude-standard` adds every untracked file git does not ignore, so a new
# page is read before it is staged as well as after. See the module docstring
# for why this is not a list of directory names.
REPOSITORY = (
    "git",
    "ls-files",
    "--cached",
    "--others",
    "--exclude-standard",
    "-z",
)


def repository_files():
    """Every file the repository contains, relative to ROOT, in git's order.

    One definition of the file set, taken from git, instead of a hand-written
    list of directory names to skip.
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
            "this checker reads the repository's content, so it must run inside "
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


def markdown_corpus() -> str:
    """Everything the repository's documentation says, as one string."""
    chunks = []
    for relative in repository_files():
        if not relative.endswith(".md"):
            continue
        with open(os.path.join(ROOT, relative), encoding="utf-8") as handle:
            chunks.append(handle.read())
    return "\n".join(chunks)


def main() -> int:
    if not os.path.isfile(BINARY):
        print(
            f"{os.path.relpath(BINARY, ROOT)} does not exist; build the CLI first "
            "(cargo build --package nudo-cli)"
        )
        return 1

    corpus = markdown_corpus()
    problems: list[str] = []
    checked = 0

    for args, stream, fragments in CASES:
        result = subprocess.run(
            [BINARY, *args],
            cwd=ROOT,
            capture_output=True,
            text=True,
            check=False,
        )
        output = result.stdout if stream == "stdout" else result.stderr
        command = "nudo " + " ".join(args)

        for fragment in fragments:
            checked += 1
            if fragment not in output:
                problems.append(
                    f"`{command}` no longer prints {fragment!r}; the documentation still does"
                )
            elif fragment not in corpus:
                problems.append(
                    f"`{command}` prints {fragment!r}, which no document shows"
                )

    print(f"documented console fragments: {checked} checked against {len(CASES)} commands")

    if problems:
        print("\nconsole output documentation problems:")
        for problem in problems:
            print(f"  - {problem}")
        return 1

    print("documented console output: ok")
    return 0


if __name__ == "__main__":
    sys.exit(main())
