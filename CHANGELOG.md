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

