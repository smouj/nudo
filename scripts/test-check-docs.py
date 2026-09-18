#!/usr/bin/env python3
"""Pins what `scripts/check-docs.py` scans, so its scope cannot drift.

The checker scans the repository's content — every file git would track — and
nothing else. Both halves of that sentence matter, and each is easy to lose
silently:

* a git-ignored file is not scanned, so a broken link in scratch cannot fail the
  gate on a path the repository does not contain;
* a versioned file is scanned, so a broken link in one still fails it;
* a file that is new but not yet staged is scanned too, because a contributor
  should see the failure before `git add`, not first in CI;
* outside a git work tree the checker refuses to run, because "wider" is never
  the fallback for a scope it cannot determine.

Each test builds a throwaway git repository, copies the real checker into it and
runs it there. The fixtures reuse the checker's own list of abandoned names
rather than spelling one out, so this file cannot trip the residue scan it is
testing.

Run by `scripts/check.sh` and by CI. Standard library only.

Exit code is 0 when every test passes, 1 otherwise.
"""

from __future__ import annotations

import importlib.util
import os
import shutil
import subprocess
import sys
import tempfile
import unittest

HERE = os.path.dirname(os.path.abspath(__file__))
CHECKER = os.path.join(HERE, "check-docs.py")


def load_checker():
    """The checker as a module, to reuse the artefacts it looks for."""
    spec = importlib.util.spec_from_file_location("nudo_check_docs", CHECKER)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


@unittest.skipUnless(
    shutil.which("git"), "git is required to determine repository content"
)
class Scope(unittest.TestCase):
    """What the checker reads, and what it refuses to read."""

    def setUp(self):
        self.work = tempfile.TemporaryDirectory(prefix="nudo-check-docs-")
        self.addCleanup(self.work.cleanup)
        self.root = os.path.realpath(self.work.name)

        # Keep the host's git configuration out of the fixture: a global
        # excludes file would make these tests depend on the machine.
        descriptor, self.git_config = tempfile.mkstemp(prefix="nudo-gitconfig-")
        os.close(descriptor)
        self.addCleanup(os.unlink, self.git_config)

        os.makedirs(os.path.join(self.root, "scripts"))
        shutil.copyfile(CHECKER, os.path.join(self.root, "scripts", "check-docs.py"))
        self.git("-c", "init.defaultBranch=main", "init", "--quiet")

    def environment(self):
        env = dict(os.environ)
        env["GIT_CONFIG_GLOBAL"] = self.git_config
        env["GIT_CONFIG_NOSYSTEM"] = "1"
        env["GIT_CONFIG_COUNT"] = "0"
        # Do not let the search for a repository escape the temporary directory:
        # "not a work tree" must mean that, whatever the machine looks like.
        env["GIT_CEILING_DIRECTORIES"] = os.path.dirname(self.root)
        env["GIT_TERMINAL_PROMPT"] = "0"
        return env

    def git(self, *arguments):
        result = subprocess.run(
            ["git", *arguments],
            cwd=self.root,
            env=self.environment(),
            capture_output=True,
            text=True,
            check=False,
        )
        self.assertEqual(
            result.returncode, 0, f"git {' '.join(arguments)}: {result.stderr}"
        )
        return result

    def write(self, relative, text):
        path = os.path.join(self.root, *relative.split("/"))
        os.makedirs(os.path.dirname(path), exist_ok=True)
        with open(path, "w", encoding="utf-8") as handle:
            handle.write(text)

    def track(self, *relatives):
        self.git("add", "--", *relatives)

    def link(self, label, target):
        return f"[{label}]({target})"

    def check(self):
        """Run the copy of the checker that lives in the fixture repository."""
        return subprocess.run(
            [sys.executable, os.path.join(self.root, "scripts", "check-docs.py")],
            cwd=self.root,
            env=self.environment(),
            capture_output=True,
            text=True,
            check=False,
        )

    def test_a_clean_repository_passes_and_its_pages_are_read(self):
        self.write("README.md", self.link("next", "next.md"))
        self.write("next.md", "# Next\n")
        self.track("README.md", "next.md")

        result = self.check()

        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        # The count guards the other direction: a checker that scans nothing
        # would pass every test below for the wrong reason.
        self.assertIn("links: 1 relative links checked", result.stdout)

    def test_ignored_file_with_a_broken_link_is_not_scanned(self):
        self.write(".gitignore", "/scratch/\n")
        self.write("scratch/note.md", self.link("gone", "nowhere-else.md"))
        self.track(".gitignore")

        result = self.check()

        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        self.assertNotIn("does not resolve", result.stdout)

    def test_versioned_file_with_a_broken_link_fails(self):
        self.write("README.md", self.link("gone", "nowhere-else.md"))
        self.track("README.md")

        result = self.check()

        self.assertEqual(result.returncode, 1, result.stdout + result.stderr)
        self.assertIn("README.md", result.stdout)
        self.assertIn("does not resolve", result.stdout)

    def test_untracked_file_with_a_broken_link_fails(self):
        # Not staged yet is still repository content: the failure must appear
        # before `git add`, not first in CI.
        self.write("page.md", self.link("gone", "nowhere-else.md"))

        result = self.check()

        self.assertEqual(result.returncode, 1, result.stdout + result.stderr)
        self.assertIn("page.md", result.stdout)

    def test_an_ignored_file_neither_fails_nor_masks_a_versioned_one(self):
        # The regression this pins: scratch used to be scanned, so this failed
        # with both paths listed, one of which the repository does not contain.
        self.write(".gitignore", "/scratch/\n")
        self.write("scratch/note.md", self.link("gone", "nowhere-else.md"))
        self.write("README.md", self.link("gone", "nowhere-else.md"))
        self.track(".gitignore", "README.md")

        result = self.check()

        self.assertEqual(result.returncode, 1, result.stdout + result.stderr)
        self.assertIn("README.md", result.stdout)
        self.assertNotIn("scratch", result.stdout)

    def test_ignored_residue_is_not_scanned(self):
        abandoned = load_checker().FORBIDDEN[0]
        self.write(".gitignore", "/scratch/\n")
        self.write("scratch/note.md", abandoned + "\n")
        self.track(".gitignore")

        result = self.check()

        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)

    def test_versioned_residue_is_scanned(self):
        abandoned = load_checker().FORBIDDEN[0]
        self.write("page.md", abandoned + "\n")
        self.track("page.md")

        result = self.check()

        self.assertEqual(result.returncode, 1, result.stdout + result.stderr)
        self.assertIn("page.md", result.stdout)

    def test_outside_a_work_tree_the_checker_fails_closed(self):
        # A wider scan is never the fallback: outside a work tree there is no
        # repository content to define the scope, and scanning anyway would let
        # an undefined tree look green.
        lonely = tempfile.TemporaryDirectory(prefix="nudo-check-docs-")
        self.addCleanup(lonely.cleanup)
        outside = os.path.realpath(lonely.name)
        os.makedirs(os.path.join(outside, "scripts"))
        shutil.copyfile(CHECKER, os.path.join(outside, "scripts", "check-docs.py"))

        result = subprocess.run(
            [sys.executable, os.path.join(outside, "scripts", "check-docs.py")],
            cwd=outside,
            env=self.environment(),
            capture_output=True,
            text=True,
            check=False,
        )

        self.assertEqual(result.returncode, 1, result.stdout + result.stderr)
        self.assertIn("git work tree", result.stderr)


if __name__ == "__main__":
    unittest.main(verbosity=2)
