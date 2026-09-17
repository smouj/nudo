# Conformance corpus

This corpus is the executable part of the language specification. It exists so
that a NUDO implementation other than the reference one can be checked against
the same cases, in the same format, without reading the reference source.

The rule that gives the corpus its authority: **if the implementation and the
corpus disagree, the implementation is wrong.** Fixing a case is a
specification change and follows the NEP process; regenerating expectations to
make a red test green is forbidden. See [`../../AGENTS.md`](../../AGENTS.md).

## Layout

```text
tests/conformance/
└── lexer/
    └── 0001-hello-world/
        ├── main.nudo          # input, always this file name
        ├── tokens.txt         # expected token dump
        └── diagnostics.txt    # expected diagnostics (optional)
```

* The directory name is `<4-digit index>-<kebab-case-slug>`.
* The input file is always named `main.nudo`, so that expectations never
  encode a machine-specific path.
* Cases are grouped by pipeline stage. `lexer/` is the only stage that exists
  today; see [`../README.md`](../README.md) for the planned areas.

## Format: `tokens.txt`

The token dump produced by `nudo check --dump-tokens`, format `nudo-tokens v1`:

```text
# nudo-tokens v1
0000 1:1-1:3 KeywordFn "fn"
0001 1:4-1:7 Ident "add"
```

* line 1 is the format header and never changes;
* one line per token, in stream order;
* `NNNN` — zero-padded token index;
* `line:column-line:column` — 1-based start and **exclusive** end positions,
  counted in characters, not bytes;
* the token kind name, exactly as listed in `TokenKind` in
  `compiler/nudo-lexer/src/lib.rs`;
* the lexeme in Rust debug form, so whitespace and quotes are unambiguous;
* the stream always ends with one `Eof` token whose position is the end of the
  file.

## Format: `diagnostics.txt`

```text
# nudo-diagnostics v1
error NDO1002 2:13
error NDO1003 3:13
```

* line 1 is the format header;
* one line per diagnostic, in emission order;
* `<severity> <code> <line>:<column>` where severity is `error`, `warning` or
  `note`, the code is a stable `NDO` code (see `spec/errors.md`) and the
  position is the 1-based start of the offending span.

A case with no diagnostics omits the file, or contains only the header.

## Regenerating expectations

Expectations are reviewed artefacts, not build output. When a case legitimately
changes, regenerate deliberately and read the diff:

```sh
cargo run --package nudo-cli -- check --dump-tokens \
    tests/conformance/lexer/0001-hello-world/main.nudo \
    | tail -n +1 > tests/conformance/lexer/0001-hello-world/tokens.txt
```

If the diff contains anything you did not intend to change, you have found a
regression, not a snapshot to update.

## Running the corpus

```sh
scripts/conformance.sh
```

The reference driver is `compiler/nudo-lexer/tests/conformance.rs`. It is only
one consumer of the corpus; the corpus itself is implementation-neutral.
