# Documentation

Two layers, for two audiences:

| Layer | For | Where |
| ----- | --- | ----- |
| **Specification** | What NUDO means. Normative, reviewable, and the thing the compiler must obey | [`../spec`](../spec/README.md) |
| **Documentation** | How to use it, and how it works. Explanatory, and allowed to be incomplete | this directory |

If the two disagree, the specification wins — see the authority order in
[`../spec/README.md`](../spec/README.md).

Documentation also exists for contributors:

* [`../ARCHITECTURE.md`](../ARCHITECTURE.md) — the shape of the code
* [`../DESIGN.md`](../DESIGN.md) — why the language is the way it is
* [`../CONTRIBUTING.md`](../CONTRIBUTING.md) and [`../AGENTS.md`](../AGENTS.md) —
  how to change it
* [`../ROADMAP.md`](../ROADMAP.md) — what exists and what does not

## Sections

| Section | Contents |
| ------- | -------- |
| [`getting-started/`](getting-started/installing.md) | Installing the toolchain, and the first program |
| [`language/`](language/tour.md) | A tour of the syntax, with the provisional parts clearly marked |
| [`agents/`](agents/README.md) | Agents, tasks and trust, explained for a reader |
| [`security/`](security/README.md) | The threat model, and the security posture |
| [`compiler/`](compiler/pipeline.md) | How the compiler is put together |
| [`tooling/`](tooling/cli.md) | The `nudo` command line |
| [`interoperability/`](interoperability/README.md) | MCP, A2A and WASM |
| [`internals/`](internals/README.md) | Notes for people working on the compiler |

## How this tree is checked

[`scripts/check-docs.py`](../scripts/check-docs.py) resolves every relative link
in every Markdown file, looks for names this project abandoned, and parses the
machine-readable files. It runs in [`scripts/check.sh`](../scripts/check.sh) and
in CI, and the files it reads are the repository's content — git's
`ls-files --cached --others --exclude-standard`, nothing else.

That boundary is deliberate. A file git ignores is not part of the repository,
so the checker may not report on it: build output under `book/output/`, assets a
build copies into place, and local scratch are none of its business, and a
scratch note with an invented link must not fail a gate on a path a reader of
the repository cannot open. Link *targets* are still resolved against the
working tree, so a page may point at a file a build generates before that build
has run. The scope is pinned by
[`scripts/test-check-docs.py`](../scripts/test-check-docs.py), and the checker
needs a git work tree: outside one it stops and says so rather than scanning
something wider.

## A warning about completeness

NUDO is pre-alpha. Most of the language documented here does not run yet, and
each page says so where it matters. A page that describes unimplemented
behaviour as though it worked is a bug — report it as one.
