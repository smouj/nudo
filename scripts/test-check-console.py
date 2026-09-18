#!/usr/bin/env python3
"""Pins what `scripts/check-console.py` reads its documents from.

The checker's second direction — every documented console fragment must appear
in at least one Markdown file — is only worth anything against the *repository's*
documents. Both halves of that matter, and each is easy to lose silently:

* a git-ignored file is not read, so a scratch note quoting the tool cannot
  stand in for a document that has fallen behind;
* a versioned file is read, so one that falls behind still fails the check;
* a file that is new but not yet staged is read too, because a contributor
  should see the failure before `git add`, not first in CI;
* outside a git work tree the checker refuses to run, because "wider" is never
  the fallback for a scope it cannot determine.

Each test builds a throwaway git repository and copies the real checker into it.
The checker shells out to the built CLI to learn what the tool prints; a scope
test must not depend on that build, so each test injects a one-line `python -c`
command as the case's output and calls the checker's `main()` in process. With
the output fixed to contain the fragment, the exit code isolates the one thing
under test: whether the fragment is found in the repository's documents.

Run by `scripts/check.sh` and by CI. Standard library only.

Exit code is 0 when every test passes, 1 otherwise.
"""

from __future__ import annotations

import contextlib
import importlib.util
import io
import os
import shutil
import subprocess
import sys
import tempfile
import unittest

HERE = os.path.dirname(os.path.abspath(__file__))
CHECKER = os.path.join(HERE, "check-console.py")

# A fragment from the checker's own case list, so the fixture reads like a
# document that quotes the tool. Each test injects it as the command's output.
FRAGMENT = "checked 1 file: 0 errors and 0 warnings"

# The environment variables that keep the host's git configuration, and any
# repository above the fixture, out of a test.
GIT_VARIABLES = (
    "GIT_CONFIG_GLOBAL",
    "GIT_CONFIG_NOSYSTEM",
    "GIT_CONFIG_COUNT",
    "GIT_CEILING_DIRECTORIES",
    "GIT_TERMINAL_PROMPT",
)


def load_checker(path, name):
    """The checker at `path` as a module, so a test can run it in process."""
    spec = importlib.util.spec_from_file_location(name, path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


@unittest.skipUnless(
    shutil.which("git"), "git is required to determine repository content"
)
class Scope(unittest.TestCase):
    """What the checker reads, and what it refuses to read."""

    def setUp(self):
        self.work = tempfile.TemporaryDirectory(prefix="nudo-check-console-")
        self.addCleanup(self.work.cleanup)
        self.root = os.path.realpath(self.work.name)

        # Keep the host's git configuration out of the fixture: a global
        # excludes file would make these tests depend on the machine.
        descriptor, self.git_config = tempfile.mkstemp(prefix="nudo-gitconfig-")
        os.close(descriptor)
        self.addCleanup(os.unlink, self.git_config)

        os.makedirs(os.path.join(self.root, "scripts"))
        shutil.copyfile(CHECKER, os.path.join(self.root, "scripts", "check-console.py"))
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

    @contextlib.contextmanager
    def git_variables(self):
        """Give the in-process checker the same git environment as `git()`."""
        saved = {name: os.environ.get(name) for name in GIT_VARIABLES}
        overrides = self.environment()
        os.environ.update({name: overrides[name] for name in GIT_VARIABLES})
        try:
            yield
        finally:
            for name, value in saved.items():
                if value is None:
                    os.environ.pop(name, None)
                else:
                    os.environ[name] = value

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

    def check(self, fragment, root=None):
        """Run the checker in process, with `fragment` as the tool's output.

        Returns the exit code and what the checker printed.
        """
        directory = root or self.root
        module = load_checker(
            os.path.join(directory, "scripts", "check-console.py"),
            "nudo_check_console_fixture",
        )
        module.BINARY = sys.executable
        module.CASES = [(["-c", f"print({fragment!r})"], "stdout", [fragment])]
        stdout, stderr = io.StringIO(), io.StringIO()
        with self.git_variables(), contextlib.redirect_stdout(
            stdout
        ), contextlib.redirect_stderr(stderr):
            try:
                code = module.main()
            except SystemExit as exit_:
                code = exit_.code
        return code, stdout.getvalue(), stderr.getvalue()

    def test_a_clean_repository_passes_and_its_note_is_read(self):
        self.write("note.md", FRAGMENT + "\n")
        self.track("note.md")

        code, stdout, stderr = self.check(FRAGMENT)

        self.assertEqual(code, 0, stdout + stderr)
        # The count guards the other direction: a checker that read nothing
        # would report the fragment as missing for every test below.
        self.assertIn("documented console fragments: 1 checked", stdout)

    def test_an_ignored_note_is_not_scanned(self):
        self.write(".gitignore", "/scratch/\n")
        self.write("scratch/note.md", FRAGMENT + "\n")
        self.track(".gitignore")

        code, stdout, stderr = self.check(FRAGMENT)

        self.assertEqual(code, 1, stdout + stderr)
        self.assertIn("which no document shows", stdout)
        self.assertNotIn("scratch", stdout)

    def test_a_versioned_note_is_scanned(self):
        self.write("note.md", FRAGMENT + "\n")
        self.track("note.md")

        code, stdout, stderr = self.check(FRAGMENT)

        self.assertEqual(code, 0, stdout + stderr)
        self.assertNotIn("which no document shows", stdout)

    def test_an_untracked_note_is_scanned(self):
        # Not staged yet is still repository content: the fragment must be found
        # before `git add`, not first in CI.
        self.write("note.md", FRAGMENT + "\n")

        code, stdout, stderr = self.check(FRAGMENT)

        self.assertEqual(code, 0, stdout + stderr)

    def test_an_ignored_note_does_not_mask_a_versioned_miss(self):
        # The regression this pins: scratch used to be read, so a fragment that
        # no document carries was "found" in a file the repository does not
        # contain, and a document that had fallen behind passed.
        self.write(".gitignore", "/scratch/\n")
        self.write("scratch/note.md", FRAGMENT + "\n")
        self.write("page.md", "# Page\n")
        self.track(".gitignore", "page.md")

        code, stdout, stderr = self.check(FRAGMENT)

        self.assertEqual(code, 1, stdout + stderr)
        self.assertIn("which no document shows", stdout)
        self.assertNotIn("scratch", stdout)

    def test_outside_a_work_tree_the_checker_fails_closed(self):
        # A wider scan is never the fallback: outside a work tree there is no
        # repository content to define the scope, and reading anyway would let
        # an undefined tree look green.
        lonely = tempfile.TemporaryDirectory(prefix="nudo-check-console-")
        self.addCleanup(lonely.cleanup)
        outside = os.path.realpath(lonely.name)
        os.makedirs(os.path.join(outside, "scripts"))
        shutil.copyfile(CHECKER, os.path.join(outside, "scripts", "check-console.py"))

        code, stdout, stderr = self.check(FRAGMENT, root=outside)

        self.assertEqual(code, 1, stdout + stderr)
        self.assertIn("git work tree", stderr)


if __name__ == "__main__":
    unittest.main(verbosity=2)
