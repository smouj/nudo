# AGENTS.md — working in the NUDO repository as an agent

This file is a contract, not advice. It applies to autonomous agents and to
humans working with them. If you are an agent and you have not read it, stop and
read it before you change anything.

**The rule that governs all the others:**

> An agent may implement an accepted design. It must not silently redefine the
> language.

## Read before you write

Before changing anything, read, in this order:

1. [`DESIGN.md`](DESIGN.md) — what belongs in the language and what does not.
2. [`ARCHITECTURE.md`](ARCHITECTURE.md) — which crate owns what, and which
   dependency edges are allowed.
3. [`spec/README.md`](spec/README.md) and the chapter you are about to affect.
4. [`ROADMAP.md`](ROADMAP.md) — the milestone you are working in, and the
   milestones you are not.

If a change to the language, its semantics or its surface syntax is not already
described by an accepted specification chapter or an accepted NEP, you are not
implementing it. Stop and write the specification change first.

## Hard rules

* **Do not redefine language semantics silently.** No "reasonable" widening of a
  rule, no behaviour that the specification does not describe, no feature that
  exists only in the implementation.
* **Do not edit `spec/` to make tests pass.** The specification is the source of
  truth. If the implementation and the specification disagree, the
  implementation contains the bug (`spec/README.md`, "Authority").
* **Do not update expectations to silence a regression.** `tokens.txt`,
  `diagnostics.txt`, fixtures and snapshots are reviewed artefacts. Regenerate
  them only when the change is intended, and read the diff line by line.
* **Do not disable, skip or weaken a test** to make a change land. `#[ignore]`,
  `#[allow]` and commented-out assertions are all the same thing.
* **Do not add a dependency without justification.** The pre-alpha workspace has
  zero third-party dependencies on purpose. A new dependency needs a reason in
  the pull request that a reviewer can disagree with.
* **Do not introduce secrets.** No tokens, keys, credentials or personal data in
  code, tests, fixtures, workflow files or commit messages. Ever.
* **Do not force-push to `main`,** and do not disable branch protection.
* **Do not edit sensitive workflows without need.** `.github/workflows/**` and
  `CODEOWNERS` are security-relevant; a change there is reviewed like code.
* **Do not mix unrelated changes.** One pull request, one idea. If you notice an
  unrelated problem, leave it and mention it in the pull request.
* **Keep commits and pull requests small and auditable.** Conventional Commits,
  see [CONTRIBUTING.md](CONTRIBUTING.md).

## Required validation before declaring a task complete

Run the real thing; do not describe what you would run.

```sh
scripts/check.sh
```

That is the whole pipeline, and it is the same one CI runs:

| Step | Command |
| ---- | ------- |
| Formatting | `cargo fmt --all --check` |
| Lints | `cargo clippy --workspace --all-targets --all-features -- -D warnings` |
| Build | `cargo build --workspace --all-targets` |
| Tests | `cargo test --workspace` |
| Conformance | `scripts/conformance.sh` |
| Rustdoc | `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps` |
| Documentation | `python3 scripts/check-docs.py` |
| Workflow policy | `python3 scripts/check-workflows.py` |
| Documented console output | `python3 scripts/check-console.py` |

A task is not complete until all of these pass on the commit you are reporting.
If something fails and you cannot fix it, say so plainly, name the blocker and
leave the work in a state another person can pick up. "Should pass" is not a
result. "It passed when I ran it" is.

## Reporting

Report what you did, what you verified, and what you did not. State explicitly:

* the commands you ran and their real outcomes;
* what is implemented and what is still planned;
* any place where the specification was ambiguous and you had to choose;
* anything you deliberately left untouched.

Never report a feature as working because the code compiles. Never describe a
capability the toolchain does not have.

## When the specification is wrong or incomplete

That happens, and it is a legitimate finding. The response is a NEP
([`neps/README.md`](neps/README.md)), not a quiet implementation. Open the
proposal, describe the problem and the alternatives, and leave the code alone
until it is accepted.
