# Changelog

All notable changes to NUDO are recorded here.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and
versioning follows [Semantic Versioning](https://semver.org/spec/v2.0.0.html).
While the project is pre-alpha, every release may contain breaking changes, and
each one is described in this file with what a reader must do about it.

**No release has been published yet.** The versions below name states of the
`main` branch; `0.0.1` is the version declared in [`Cargo.toml`](Cargo.toml) and
is not a download. Release workflows are prepared and unexercised — see
[`SECURITY.md`](SECURITY.md) for why offering a pre-alpha toolchain as a binary
would be a promise the project cannot keep.

## [Unreleased]

### Added

* **The parser, the lossless syntax tree and the typed AST.** `nudo-syntax`
  holds a tree in which every byte of the file is reachable — whitespace and
  comments included — so it can reprint its source byte for byte;
  `nudo-parser` is a recursive-descent parser with precedence climbing on the
  frozen chain; `nudo-ast` wraps the part of a tree that is unambiguously well
  formed as typed values for consumers that want structure rather than text.
* `nudo check --dump-tree`, in the stable `nudo-tree v1` format, alongside
  `--dump-tokens`.
* 13 conformance cases under [`tests/conformance/parser/`](tests/conformance/parser),
  and their reference driver. The corpus is implementation-neutral: a tree dump,
  a diagnostics dump, and the property that the tree reprints the file.
* [NEP-0006](neps/0006-generic-syntax.md) — generic syntax: angle brackets,
  invariant type arguments, no bounds and no declaration-site parameters, with
  the parser cost of the decision stated as part of it.
* Property tests for the parser: losslessness, never hangs, always produces a
  tree, bounded diagnostics, and one diagnostic per mistake.
* `fixtures/invalid/syntax.nudo`, which pins the case M2 exists for: a file that
  lexes cleanly and is not a program.

### Changed

* **`nudo check` parses.** A clean run used to mean "no lexical diagnostics"; it
  now means "no lexical or syntax diagnostics". Exit codes are unchanged: `1`
  still means the program is wrong and `2` that the tool could not run.
* **[NEP-0005](neps/0005-keyword-policy.md) is accepted and implemented.**
  `nudo-lexer` reserves a ten-word core (`fn`, `let`, `struct`, `enum`, `if`,
  `else`, `match`, `const`, `true`, `false`); `agent`, `task`, `tool`, `model`,
  `role`, `tools`, `allow`, `budget`, `with`, `verify`, `ask` and `delegate` stay
  contextual, so a program may still name a binding `agent`. Reserving `true` and
  `false` settles the literal/path conflict the grammar checker reported.
* **The token set is the one the grammar declares.** `[`, `]`, `::`, `.`, `?`,
  `=>`, `==`, `!=`, `<`, `>`, `<=`, `>=`, `&&`, `||` and `!` are tokens now. `&`
  and `|` on their own are not, and are reported: there are no bitwise operators.
* `NDO1001` moved from *reserved* to **implemented** in the diagnostics registry.
* The documented regeneration command for the token corpus was wrong: it kept the
  summary line `check` prints after a dump, which is not part of the format.
  Fixed, and documented for the tree corpus too.

### Fixed

* **Recovery no longer skips silently.** An intermediate parser accepted
  `examples/05-agent` — exit code 0 — by discarding the tokens it could not
  place. Every recovery path now reports once before skipping. This was found by
  running the examples through the new parser, not by a test that already knew
  the answer.
* **One byte, one diagnostic.** A character the lexer cannot tokenise is
  reported once, not again by the parser as the statement around it becomes
  unreadable.
* **A node's span starts at its first token.** Trivia is placed in the enclosing
  node, so a type's `text()` is `Int` and not ` Int`, and a comment written above
  an item belongs to the file rather than to the item.
* **A character that starts no token is no longer dropped.** It is reported as
  `NDO1002` and kept as an `Unknown` token, which is what makes "every byte is in
  the tree" true for files with errors — the case a formatter or a repair tool
  most needs.

### Known gaps

Recorded here rather than left implicit, because a changelog that only lists
additions is a marketing document:

* declaration-site generic parameters (`enum Outcome<T, E>`) are not in the
  grammar and are rejected; [NEP-0006](neps/0006-generic-syntax.md) says why and
  what decides it;
* `verify (expr) with V` is rejected, because `verify` is contextual and the
  parser has one token of lookahead. The limitation is pinned by a test, and
  disappears when [NEP-0002](neps/0002-generated-verified.md) decides what
  `verify` is;
* the grammar's paths use `::` and its lists are comma-separated, while several
  examples write `web.search` one per line. The parser follows the grammar;
  the examples are previews and say which syntax stops them. Which spelling the
  language should have is a NEP, not a parser bug;
* the type checker and the interpreter do not exist
  ([`ROADMAP.md`](ROADMAP.md), M3–M4);
* `std/` and `protocols/` contain documentation and no code.

### Changed

* **The expression grammar is frozen and parseable.** It no longer contains left
  recursion, which a recursive-descent parser cannot handle; precedence is stated
  once as a chain of grammar levels that the productions implement, instead of a
  prose table that could drift. `scripts/check-grammar.py` enforces both rules,
  and fails if the three copies of the precedence chain disagree. Comparison no
  longer chains silently, `?` is a type operator only, and the deliberate absence
  of an assignment level is documented with the decision it waits on.
* `spec/expressions.md`, `spec/grammar.md` and `grammar/syntax-reference.md` now
  state exactly the grammar's precedence, associativity and lookahead
  requirements, and say which parts are implemented.
* `spec/errors.md` requires diagnostics to show the *chain* that introduced a
  requirement, and records `--json` as planned rather than pretended.

### Fixed

* The EBNF header embedded a comment delimiter inside a comment, which breaks any
  EBNF reader. Found by the new grammar checker while it was being written.

### Added

* `scripts/check-grammar.py` — left recursion, undefined and unreachable
  productions, and the precedence chain against its two other copies.
* [`docs/internals/parser-design.md`](docs/internals/parser-design.md) — the M2
  design: strategy, tree representation with the rowan trade-off, recovery
  contract, testing, and the four decisions that block implementation.
* **M2.1: the parser, the syntax tree and the AST.** See the unreleased section
  above for the details; the milestone's exit criteria are met and its two gate
  decisions are recorded as NEPs 0005 and 0006.
* [NEP-0005](neps/0005-keyword-policy.md) — keyword policy: a small reserved core
  and contextual recognition elsewhere, which resolved a real parse conflict
  between `true`/`false` and identifiers.
* The NUDO language specification: authority order, lexical structure, grammar,
  types, effects, errors, and the agent/trust chapters
  (see [`spec/`](spec/README.md)).
* The NEP process, with a template and four seed proposals
  (see [`neps/`](neps/README.md)).
* Repository governance, security policy, contribution guide and the agent
  contract ([`AGENTS.md`](AGENTS.md)).
* Brand assets and [`assets/brand/BRAND.md`](assets/brand/BRAND.md).
* CI running the same pipeline as `scripts/check.sh` on Linux, Windows and
  macOS, plus security and documentation workflows.
* `cargo-deny` and `cargo-audit` policy, with the workspace's zero-dependency
  state recorded in [`deny.toml`](deny.toml).

### Known gaps

Recorded here rather than left implicit, because a changelog that only lists
additions is a marketing document:

* the parser, type checker and interpreter do not exist
  ([`ROADMAP.md`](ROADMAP.md), M2–M4);
* `nudo check` performs lexical analysis only;
* the examples after `examples/04-results/` illustrate proposed syntax and are
  not accepted by the current toolchain;
* `std/` and `protocols/` contain documentation and no code.

## [0.0.1] — pre-alpha

The first published state of the repository. Milestones M0 and M1 of
[`ROADMAP.md`](ROADMAP.md).

### Added

* Monorepo workspace with 35 crates, of which six have behaviour:
  `nudo-common`, `nudo-span`, `nudo-source`, `nudo-diagnostics`, `nudo-lexer`
  and `nudo-cli` (plus the `nudo-bench` measurement binary).
* `nudo check`: reads `.nudo` sources, lexes them and reports diagnostics with
  rendered source snippets, exit code `0`, `1` or `2` according to the outcome.
* `nudo --version` and `nudo --help`, with the help listing which commands are
  implemented and which are planned.
* Lexer: identifiers, integer and floating-point literals with `_` separators,
  text literals with `\n \r \t \\ \" \0`, line comments, nested block comments,
  the punctuation of the pre-alpha token set, and `fn` / `let`.
* Error recovery: lexing reports, skips and continues; it always terminates and
  never panics, including on adversarial input.
* Diagnostics with stable codes (`NDO1002`–`NDO1006`, `NDO8001`), notes, help
  text and a caret renderer that counts columns in characters.
* Conformance corpus in an implementation-neutral format, with five cases and a
  documented driver ([`tests/conformance/README.md`](tests/conformance/README.md)).
* Fixture corpus with declared expectations
  ([`fixtures/README.md`](fixtures/README.md)).
* Adversarial robustness tests over generated inputs; reproducible measurements
  through `nudo-bench`.
* Dual licence: Apache-2.0 OR MIT.

