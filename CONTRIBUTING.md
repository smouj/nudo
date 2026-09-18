# Contributing to NUDO

Thank you for looking. NUDO is pre-alpha, which means the most valuable
contributions right now are design review, specification work and small,
verifiable changes — not new features built on unstable ground.

Before anything else: read [`AGENTS.md`](AGENTS.md). It applies to humans
working with agents too, and it is the shortest description of what this project
expects from a change.

## Setup

### Toolchain

Rust, stable channel. The exact requirements are declared in
[`rust-toolchain.toml`](rust-toolchain.toml) (channel and components) and
[`Cargo.toml`](Cargo.toml) (MSRV). You do not need a nightly toolchain and you
should not add one.

```sh
scripts/bootstrap.sh            # Linux, macOS, WSL
scripts\bootstrap.ps1           # Windows (PowerShell)
```

The scripts install the toolchain user-locally through rustup if it is missing,
add `rustfmt` and `clippy`, and build the workspace. Nothing is installed
system-wide.

### Build, test, lint

```sh
cargo build --workspace
cargo test --workspace
cargo fmt --all
cargo clippy --workspace --all-targets --all-features -- -D warnings
```

Or run everything the way CI does:

```sh
scripts/check.sh                # Linux, macOS, WSL
scripts\check.ps1               # Windows (PowerShell)
```

`scripts/check.sh` is the single definition of "this repository is correct". If
you add a check to CI, add it there in the same pull request.

### Run the toolchain

```sh
cargo run --package nudo-cli -- --version
cargo run --package nudo-cli -- check examples/00-hello-world/main.nudo
cargo run --package nudo-cli -- check --dump-tokens fixtures/valid/hello.nudo
cargo run --package nudo-cli -- check --dump-tree fixtures/valid/hello.nudo
```

Expect `check` to lex and parse. It is milestones M1 and M2; see
[`ROADMAP.md`](ROADMAP.md). Nothing is type-checked and nothing runs.

### Measure

```sh
cargo run --release --package nudo-bench
```

Measurements are for comparing changes against each other, not for setting
targets. NUDO is pre-alpha and has no accepted performance goal.

## Architecture

Read [`ARCHITECTURE.md`](ARCHITECTURE.md) before touching more than one crate.
The rules that matter most:

* a compiler stage may depend on earlier stages, never on later ones;
* nothing in `crates/` may depend on `compiler/`, `runtime/`, `tooling/`,
  `cli/` or `backends/`;
* no dependency cycles;
* no dumping-ground crate.

The workspace has **no third-party dependencies** today. If your change needs
one, explain in the pull request what it does, why the standard library cannot
do it, what it costs, and what the alternative was. Reviewers are expected to
push back.

## Language changes go through a NEP

A NEP — [NUDO Enhancement Proposal](neps/README.md) — is required for anything
that changes what a program means or how it is written:

* a new or changed syntax rule;
* a new type, and the rules for it;
* a new diagnostic code or a change to an existing one;
* a new capability, effect, or rule about trust;
* a change to the compiler pipeline's boundaries;
* a stabilisation decision.

Not required: bug fixes, documentation, tests, internal refactoring that does
not change behaviour, and tooling that does not affect the language.

The process:

1. Open a discussion using the *Language design* template, or an issue using the
   *Language proposal* template.
2. Write the NEP from `neps/0000-template.md` and open a pull request that adds
   it to `neps/`. The initial state is `Draft`.
3. Review moves it through `Discussion` to `Accepted` or `Rejected`. The
   maintainer decides, on the record, with the reasoning in the pull request.
4. Implementation starts only after `Accepted`. The NEP moves to
   `Implemented`, and later to `Stabilized`.

The specification change and the implementation are separate pull requests, in
that order. Never the other way around — see the authority order in
[`spec/README.md`](spec/README.md).

## Pull requests

Use the template. Keep one idea per pull request; if you find an unrelated
problem, leave it alone and say so.

A reviewer will check:

* does the change match the specification, or is the specification change part
  of this pull request?
* is the change tested, and does the test fail without the change?
* does `scripts/check.sh` pass?
* are the boundaries in `ARCHITECTURE.md` respected?
* does it add a dependency, a keyword, an error code, or a capability? If so,
  where is the NEP?
* is anything in the diff reported as done that is not done?

## Commit format

[Conventional Commits](https://www.conventionalcommits.org/), extended with two
project-specific types:

```text
feat(parser): add function declarations
fix(lexer): keep the caret aligned for tab indentation
spec(types): define Result semantics
nep: propose Generated and Verified
security(runtime): refuse to widen a grant during delegation
docs(contributing): document the NEP path
test(conformance): add a nested block comment case
refactor(span): replace the line scan with a binary search
perf(lexer): avoid re-scanning identifiers twice
build: raise the declared MSRV
ci: cache the cargo registry on Windows
chore: update the issue template config
```

Types: `feat`, `fix`, `perf`, `refactor`, `docs`, `test`, `build`, `ci`,
`chore`, `spec`, `nep`, `security`.

The scope is the crate or area. Commit subjects are imperative, lowercase, and
end without a full stop.

Do not use `update`, `changes`, `fix stuff`, `final`, `final2`, `misc`, or
anything else that fails to say what happened. If you cannot describe the
commit in one line, the commit is too big.

## Issues

Use the forms. A report that cannot be reproduced will be labelled
`status:needs-repro` and may be closed — not to dismiss it, but because an
unreproducible bug in a compiler cannot be fixed safely.

Labels are grouped in three families: `area:*` (what part of the project),
`type:*` (what kind of change) and `priority:p0`–`priority:p4` (how urgent),
plus `status:*` for workflow state. Apply the smallest set that is true; a
report with nine labels says nothing.

## Security reports

Never open a public issue for a vulnerability. See
[`SECURITY.md`](SECURITY.md).

## Conformance

If your change alters language-visible behaviour, it must be accompanied by
conformance cases under `tests/conformance/<stage>/`. The format is documented
in [`tests/conformance/README.md`](tests/conformance/README.md).

Regenerating expected output to make a test pass is not a fix. If the diff is
not what you intended, you have found a regression.
