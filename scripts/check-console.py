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

Run by `scripts/check.sh` and by the `Docs` workflow, after a build of the CLI.
"""

from __future__ import annotations

import os
import subprocess
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
BINARY = os.path.join(ROOT, "target", "debug", "nudo")

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
]

SKIP_DIRECTORIES = {".git", "target", "dist", "node_modules"}


def markdown_corpus() -> str:
    """Everything the documentation says, as one string."""
    chunks = []
    for directory, subdirectories, filenames in os.walk(ROOT):
        subdirectories[:] = sorted(n for n in subdirectories if n not in SKIP_DIRECTORIES)
        for filename in sorted(filenames):
            if filename.endswith(".md"):
                with open(os.path.join(directory, filename), encoding="utf-8") as handle:
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
