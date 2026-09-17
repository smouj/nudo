#!/usr/bin/env python3
"""Checks the GitHub Actions workflows against the repository's own policy.

The policy is stated in `.github/workflows/security.yml` and in
[`SECURITY.md`](../SECURITY.md):

1. every workflow declares explicit permissions (so the default is not "write
   everything the token can write");
2. no workflow requests `write-all`;
3. every action is pinned to a full commit SHA, because a tag can be moved and
   a commit cannot;
4. no workflow reaches for a secret other than the automatic `GITHUB_TOKEN`,
   because a pull request workflow must not be able to receive one.

This is a script rather than an inline shell step on purpose. An inline check
that greps the workflow directory for a forbidden string also matches the text
of the check itself, which is a bug this repository already shipped once.

Run by `scripts/check.sh` and by the `Security` workflow. Exit code 0 when the
policy holds, 1 otherwise.
"""

from __future__ import annotations

import os
import re
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
WORKFLOW_DIR = os.path.join(ROOT, ".github", "workflows")

# `owner/repo@<40 hex>` — the only accepted form for a remote action.
PINNED = re.compile(r"^[A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+@[0-9a-f]{40}$")
# `owner/repo@v4` — a movable reference, which is what we refuse.
MOVABLE = re.compile(r"^[A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+@[^0-9a-f]")

ALLOWED_SECRETS = {"GITHUB_TOKEN"}
SECRET_USE = re.compile(r"secrets\.([A-Za-z0-9_]+)")


def iter_uses(node):
    """Yields every `uses:` value in a workflow, at any depth."""
    if isinstance(node, dict):
        for key, value in node.items():
            if key == "uses" and isinstance(value, str):
                yield value
            else:
                yield from iter_uses(value)
    elif isinstance(node, list):
        for item in node:
            yield from iter_uses(item)


def iter_permissions(node, path="top"):
    """Yields (path, value) for every `permissions:` mapping."""
    if isinstance(node, dict):
        for key, value in node.items():
            if key == "permissions":
                yield path, value
            else:
                yield from iter_permissions(value, key if path == "top" else f"{path}.{key}")
    elif isinstance(node, list):
        for index, item in enumerate(node):
            yield from iter_permissions(item, f"{path}[{index}]")


def iter_strings(node):
    if isinstance(node, str):
        yield node
    elif isinstance(node, dict):
        for value in node.values():
            yield from iter_strings(value)
    elif isinstance(node, list):
        for item in node:
            yield from iter_strings(item)


def main() -> int:
    try:
        import yaml  # type: ignore
    except ImportError:
        print(
            "the workflow check needs PyYAML to read the workflows; install it "
            "rather than skipping the check (`pip install pyyaml`)"
        )
        return 1

    if not os.path.isdir(WORKFLOW_DIR):
        print(f"{WORKFLOW_DIR} does not exist")
        return 1

    files = sorted(
        os.path.join(WORKFLOW_DIR, name)
        for name in os.listdir(WORKFLOW_DIR)
        if name.endswith((".yml", ".yaml"))
    )
    if not files:
        print("no workflows to check")
        return 1

    problems: list[str] = []
    actions_checked = 0

    for path in files:
        relative = os.path.relpath(path, ROOT)
        with open(path, encoding="utf-8") as handle:
            try:
                document = yaml.safe_load(handle)
            except yaml.YAMLError as error:  # type: ignore[attr-defined]
                problems.append(f"{relative}: invalid YAML ({error})")
                continue

        if not isinstance(document, dict):
            problems.append(f"{relative}: not a workflow mapping")
            continue

        if "permissions" not in document:
            problems.append(
                f"{relative}: no top-level `permissions:`; the default is broader than this repository allows"
            )

        for scope, value in iter_permissions(document):
            if value == "write-all":
                problems.append(f"{relative}: `permissions` at {scope} requests write-all")

        for use in iter_uses(document):
            # Local composite actions and container references are not pinned.
            if use.startswith("./") or use.startswith("docker://"):
                continue
            actions_checked += 1
            if PINNED.match(use):
                continue
            if MOVABLE.match(use):
                problems.append(
                    f"{relative}: `uses: {use}` is a movable reference; pin the full commit SHA"
                )
            else:
                problems.append(f"{relative}: `uses: {use}` is not a recognised action reference")

        # Only the automatic token may be referenced. Its presence in a pull
        # request workflow is not a leak; any other secret would be.
        for text in iter_strings(document):
            for name in SECRET_USE.findall(text):
                if name not in ALLOWED_SECRETS:
                    problems.append(
                        f"{relative}: references `secrets.{name}`; pull request workflows must not receive secrets"
                    )

    print(f"workflows: {len(files)} files, {actions_checked} action references")

    if problems:
        print("\nworkflow policy violations:")
        for problem in problems:
            print(f"  - {problem}")
        return 1

    print("workflow policy: ok")
    return 0


if __name__ == "__main__":
    sys.exit(main())
