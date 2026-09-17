# Fuzzing

**Infrastructure is in place; a libFuzzer campaign is not.**

What exists today is a property-based robustness harness that runs in CI, needs
no external dependency and no nightly toolchain:

* `compiler/nudo-lexer/tests/robustness.rs` — 2 000 pseudo-random inputs and 300
  structured inputs (comments, literals and numbers opened and never closed),
  from a fixed-seed generator, asserting that the lexer

  1. never panics,
  2. always terminates,
  3. produces spans inside the source and on character boundaries,
  4. produces ordered, non-overlapping tokens,
  5. emits exactly one `Eof`, last,
  6. emits diagnostics whose spans are inside the source,
  7. produces the same output for the same input.

  A failure prints the seed, so it reproduces exactly.

## Why not libFuzzer yet

`cargo-fuzz` is the right tool for a campaign, and it is deliberately not wired
up yet:

* it requires a nightly toolchain, and the project builds on stable only;
* it adds a dependency surface to a workspace that currently has none;
* the property tests above already cover the guarantees the lexer makes, and
  cover them in CI on every platform.

The trade is honest: the property tests explore a smaller space than a
coverage-guided fuzzer, and they are weaker at finding deep state-machine bugs.
When the parser lands (M2), that changes: a coverage-guided fuzzer on the parser
is worth the cost, because a parser has state and depth in a way the lexer does
not.

## Planned layout

```text
fuzz/
├── Cargo.toml        # a separate workspace: fuzz targets must not be members
├── fuzz_targets/
│   ├── lexer.rs      # arbitrary bytes -> tokenize, assert the guarantees
│   └── parser.rs     # arbitrary bytes -> parse, then re-print and re-parse (M2)
└── corpus/           # seeds, reviewed like any other fixture
```

The target directory is created when the first target exists. An empty
`fuzz/lexer/` directory would be a promise, not infrastructure.

## Running a campaign, once it exists

```sh
cargo install cargo-fuzz
cargo +nightly fuzz run lexer
```

A campaign is expected to run nightly and for a few hours before a release, not
on every pull request: a fuzzer that runs for thirty seconds in CI finds nothing
and teaches reviewers to ignore it.
