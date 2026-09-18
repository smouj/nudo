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
├── lexer/
│   └── 0001-hello-world/
│       ├── main.nudo          # input, always this file name
│       ├── tokens.txt         # expected token dump
│       └── diagnostics.txt    # expected diagnostics (optional)
├── parser/
│   └── 0001-function/
│       ├── main.nudo          # input, always this file name
│       ├── tree.txt           # expected syntax tree dump
│       └── diagnostics.txt    # expected diagnostics (optional)
└── resolve/
    └── 0001-resolves-a-program/
        ├── main.nudo          # input, always this file name
        ├── resolutions.txt    # expected resolutions dump
        └── diagnostics.txt    # expected diagnostics (optional)
```

* The directory name is `<4-digit index>-<kebab-case-slug>`.
* The input file is always named `main.nudo`, so that expectations never
  encode a machine-specific path.
* Cases are grouped by pipeline stage. `lexer/`, `parser/`, `resolve/` and `typeck/`
exist today; see
  [`../README.md`](../README.md) for the planned areas.

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

## Line endings

Expectations are compared line by line, and a carriage return is treated as a
checkout artefact rather than content: the reference driver normalises CRLF to LF
before comparing. `.gitattributes` normalises the corpus to LF in the repository
and in the working tree, so this should never matter — it is handled anyway,
because a Windows contributor regenerating an expectation with a shell redirect
writes CRLF, and a corpus that only works on the author's machine is not a
corpus.

The token dump itself always ends its lines with LF, on every platform.

## Format: `tree.txt`

The syntax tree dump produced by `nudo check --dump-tree`, format
`nudo-tree v1`:

```text
# nudo-tree v1
SourceFile 0..13
  FunctionDecl 0..12
    KeywordFn "fn" 0..2
    Ident "main" 3..7
    ParameterList 7..9
      LParen "(" 7..8
      RParen ")" 8..9
```

* line 1 is the format header and never changes;
* one line per element, in tree order, so a parent is followed by its children;
* two spaces of indentation per level of nesting;
* the element's name, exactly as listed in `SyntaxKind` in
  `compiler/nudo-syntax/src/lib.rs`;
* for a token, the lexeme in Rust debug form;
* every line ends with the byte range the element covers, so a reader can find
  the same element in the source. Byte offsets, not line/column: the tree is
  compared byte for byte, and the token corpus already covers positions;
* trivia — whitespace and comments — appears in the tree. That is the point of
  a lossless tree, and it is why a dump is longer than the file it came from.

The same `diagnostics.txt` format is used for both stages, and the same `NDO`
codes: a parser case that is rejected says which code rejected it.

## Regenerating expectations

Expectations are reviewed artefacts, not build output. When a case legitimately
changes, regenerate deliberately and read the diff:

```sh
cargo run --package nudo-cli -- check --dump-tree \
    tests/conformance/parser/0001-function/main.nudo \
    | sed '/^checked /d' > tests/conformance/parser/0001-function/tree.txt
```

The `sed` drops the summary line, which `check` prints to stdout after a dump
and which is not part of the format:

```text
checked 1 file: 0 errors and 0 warnings
```

The `typeck` corpus uses `types.txt` for the `nudo-typeck v1` dump. The CLI does
not expose a type dump yet — the checker is deliberately not wired into `check`
until calls exist — so those expectations are written by their driver with
`NUDO_BLESS=1` and reviewed as a diff, exactly like every other expectation here.

For the token corpus, use the same command with `--dump-tokens`, `tokens.txt`
and the `lexer/` directory. For the resolution corpus, `--dump-resolutions`,
`resolutions.txt` and the `resolve/` directory.

If the diff contains anything you did not intend to change, you have found a
regression, not a snapshot to update.

## Running the corpus

```sh
scripts/conformance.sh
```

The reference drivers are `compiler/nudo-lexer/tests/conformance.rs` and
`compiler/nudo-parser/tests/conformance.rs`. They are only consumers of the
corpus; the corpus itself is implementation-neutral.
