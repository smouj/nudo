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

* **M3.1, the syntax half.** `generic-parameter-list` on `fn`, `struct`, `enum`,
  `task` and `tool` ([NEP-0010](neps/0010-declaration-site-generics.md)), so a
  program can declare a parameterised type of its own; and `verify`'s operand
  restricted to a named value ([NEP-0002](neps/0002-generated-verified.md)),
  which removes the one-token limitation M2 documented. Each has a conformance
  case: `0014-declaration-generics`, `0015-verify-named-operand` and
  `0016-verify-parenthesised`.
* `NDO2001`, `NDO2002` and `NDO2003` in the registry, and in use.
* **M3.1: `nudo-hir` and name resolution.** A semantic representation that is
  not an AST copy — parentheses are gone, a name is a `DefId` or a marked
  unresolved — with two namespaces, hoisted items, shadowing versus duplicates,
  and an initialiser lowered before its own name is in scope. `nudo check`
  resolves names after a clean parse, `--dump-resolutions` prints the result, and
  `tests/conformance/resolve` holds four cases. The design and its deliberately
  missing pieces are in [`docs/internals/hir-design.md`](docs/internals/hir-design.md).

* **M3.0 — the semantic gates.** Six NEPs decide the semantics the type checker
  will be written against, each with the alternatives it rejected:
  [NEP-0007](neps/0007-integer-semantics.md) (`Int` is signed 64-bit, arithmetic
  **traps** on overflow and division by zero, a literal out of range is a compile
  error), [NEP-0008](neps/0008-error-model.md) (`Result<T, E>` is intrinsic,
  failure is a value, a trap is not, there is no `?`),
  [NEP-0002](neps/0002-generated-verified.md) (`verify` yields
  `Result<Verified<T>, VerificationError>` and its operand is a named value),
  [NEP-0009](neps/0009-mutability.md) (bindings are immutable, and no borrow
  checker is introduced),
  [NEP-0010](neps/0010-declaration-site-generics.md) (declarations declare their
  type parameters; `Result` is intrinsic) and
  [NEP-0011](neps/0011-path-and-list-spelling.md) (a path uses `::`, a list is
  comma-separated).
* `M3.0`, `M3.1`, `M3.2` and `M3.3` in [`ROADMAP.md`](ROADMAP.md), with the
  closure test M3 is judged by: `publish(draft)` must fail to compile, and the
  handled `Result` of a verification must be accepted.
* **[NEP-0012](neps/0012-early-exit.md) — early exit.** There is no `return` in
  this edition: a block's value is its final expression, and `if` and `match` are
  expressions, so a conditional result is written where the value is.
  `return` stays an ordinary identifier, and
  `tests/conformance/parser/0017-no-return` pins that a stray one is `NDO1001`.

### Changed

* **`verify` now yields a `Result`.** It was specified as producing
  `Verified<T>` directly, which made a rejected verification look like a panic.
  It yields `Result<Verified<T>, VerificationError>`, so a `Verified<T>` cannot be
  obtained without handling the failure (`spec/expressions.md`,
  `spec/trust/verified.md`, the README and the manual).
* **The parser's one-token limitation on `verify` is gone**, removed by
  restricting the operand to a named value rather than by reserving the word
  ([NEP-0002](neps/0002-generated-verified.md)). `verify (expr) with V` is no
  longer a limitation: it is not in the grammar.
* **The grammar gained `generic-parameter-list`** and the restricted
  `verify-expression` operand. `nudo-parser` accepts both in M3.1; until then the
  restriction is recorded in `spec/grammar.md` and `ROADMAP.md` rather than left
  to be discovered.
* `spec/types.md`, `spec/expressions.md`, `spec/declarations.md`,
  `spec/generics.md`, `spec/grammar.md` and `spec/trust/verified.md` state the
  decided semantics instead of naming them as open questions.
* **`examples/04-results` is read by the toolchain.** With early exit settled
  away ([NEP-0012](neps/0012-early-exit.md)), the example writes its failure as
  the value of an `if` instead of returning it, and it moves from the design
  previews to the readable set in `examples/README.md` and the three example
  tests.

### Fixed

* **The documentation gate reads repository content, not the working tree.**
  `scripts/check-docs.py` walked every directory except a hand-written list of
  names, so git-ignored scratch and generated output were scanned too, and a
  scratch note with an invented link failed the gate on a path the repository
  does not contain. The file set is now git's own answer to "what is in the
  repository" — every tracked file, plus every untracked file git does not
  ignore — and [`scripts/test-check-docs.py`](scripts/test-check-docs.py) pins
  that boundary. Outside a git work tree the checker stops and says so instead
  of scanning wider.

* **The console-output gate reads repository content too.** The sibling of the
  defect above: [`scripts/check-console.py`](scripts/check-console.py) built its
  Markdown corpus by walking the working tree, so a git-ignored scratch note
  quoting an old message satisfied "every fragment appears in a document" and a
  page that had fallen behind passed the check the gate exists for. Its corpus is
  now the repository's content — git's
  `ls-files --cached --others --exclude-standard` — with the same fail-closed
  rule outside a git work tree, and
  [`scripts/test-check-console.py`](scripts/test-check-console.py) pins it.

* **The specification no longer describes a `return` the grammar never had.**
  `spec/declarations.md`, `spec/expressions.md`, `spec/grammar.md`,
  `docs/language/tour.md` and the manual's parser chapter now state the decision
  ([NEP-0012](neps/0012-early-exit.md)) instead of claiming a statement the EBNF
  never had; the four book translations were brought to the same revision.

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

* **One position, one parser diagnostic.** Two constructs meeting at the same
  token reported the same mistake twice. The parser now records what it has
  already reported and stays quiet the second time.
* **A missing `;` does not poison the next statement.** A statement whose
  terminator is missing recovers to the next `;` or `}`, instead of leaving its
  junk for whatever construct comes next to report again.
* **No empty error nodes.** A recovery with nothing to skip was creating an
  `Error` node with no children — a tree that claims the parser could not place
  something it had placed.

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

