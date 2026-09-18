# The compiler pipeline

The pipeline is a sequence of stages with explicit boundaries. Each stage is a
crate, consumes one kind of data and produces another, and never reaches back
into an earlier stage.

```text
Source
  ↓
Lexer            compiler/nudo-lexer        ← implemented (M1)
  ↓
Tokens
  ↓
Parser           compiler/nudo-parser       ← implemented (M2)
  ↓
Syntax tree      compiler/nudo-syntax       ← implemented (M2)
AST              compiler/nudo-ast          ← implemented (M2)
  ↓
HIR              compiler/nudo-hir          ← implemented (M3)
  ↓
Type checking    compiler/nudo-typeck       ← planned (M3)
  ↓
Effect checking  compiler/nudo-effects      ← planned (M5)
  ↓
MIR
  ↓
Backend
  ├── Interpreter   backends/interpreter    ← planned (M4)
  └── WebAssembly   backends/wasm           ← planned (M11)
```

## Why stages, and not a monolith

Because the interesting decisions in this language are semantic, and semantics
need a place to be decided. A stage boundary is where one question stops and the
next begins:

| Stage | The question it answers |
| ----- | ----------------------- |
| Lexer | What are the tokens? |
| Parser | What is the shape of the program? |
| HIR | What does each name refer to? |
| Type checking | Is this program well-typed? |
| Effect checking | What can this program touch? |
| MIR | What does this program compute, with control flow made explicit? |
| Backend | How does this machine run it? |

Each boundary is also a place where a diagnostic gets a stable code from the
right family — `1xxx` for syntax, `2xxx` for types, `3xxx` for effects. See
[`../../spec/errors.md`](../../spec/errors.md).

## Rule: disagreement is a compiler bug

If an implementation and the specification disagree, the implementation is wrong
([`../../spec/README.md`](../../spec/README.md)). That applies stage by stage: a
stage may not quietly accept something the grammar does not describe, and may not
quietly reject something the specification permits.

## Rule: no stage prints

Stages produce `nudo-diagnostics` values. Rendering happens once, in the CLI.
A stage that formats its own output produces diagnostics that no tool can match
on and that no other front end can reuse.

## What exists today

The implemented slice is deliberately small and complete:

```text
.nudo → SourceFile → Lexer → Tokens → Diagnostics
```

* `crates/nudo-source` — a file, its identity and its line index.
* `crates/nudo-span` — byte positions, spans, and 1-based character columns.
* `compiler/nudo-lexer` — tokenisation with recovery, no panics, deterministic
  output.
* `compiler/nudo-diagnostics` — the code registry and the caret renderer.
* `cli/nudo-cli` — `check`, argument handling and exit codes.

Everything else is an empty crate with a documented contract, in
[`../../ARCHITECTURE.md`](../../ARCHITECTURE.md).

## Testing a stage

* **Unit tests** live next to the code in `#[cfg(test)]` modules and in the
  crate's `tests/` directory.
* **Fixtures** (`../../fixtures`) pin one rule with declared expectations — see
  [`../../fixtures/README.md`](../../fixtures/README.md).
* **Conformance cases** (`../../tests/conformance`) pin language-visible
  behaviour that any implementation must reproduce.
* **Robustness properties** cover what must never happen: a panic, a hang, a
  span outside the source, a non-deterministic result. The lexer's are in
  `compiler/nudo-lexer/tests/robustness.rs`.

## Measuring

`benchmarks/nudo-bench` measures the front end reproducibly, with no third-party
harness. Measurements exist to compare a change against the previous one; the
project has no accepted performance target, and pre-alpha is the wrong time to
acquire one.
