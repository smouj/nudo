# Internals

Notes for people working on the compiler itself. If you are using NUDO rather
than changing it, start with [`../getting-started/installing.md`](../getting-started/installing.md).

## The short list

| Topic | Where |
| ----- | ----- |
| Crate boundaries and dependency rules | [`../../ARCHITECTURE.md`](../../ARCHITECTURE.md) |
| Pipeline, stage by stage | [`../compiler/pipeline.md`](../compiler/pipeline.md) |
| How to add a compiler stage | [`../../ARCHITECTURE.md`](../../ARCHITECTURE.md#where-to-add-things) |
| Robustness requirements | [`../compiler/pipeline.md`](../compiler/pipeline.md) |
| Error codes and rendering | [`../../spec/errors.md`](../../spec/errors.md) |

## Conventions that are easy to get wrong

**Positions are byte offsets; columns are characters.** `BytePos` and `Span` are
byte-based, because slicing needs bytes. Anything a human reads — a caret, a
line/column pair — counts characters. Mixing the two produces carets that drift
on the first non-ASCII line, which is why the renderer has a test for exactly
that.

**Spans are half-open.** `Span::new(start, end)` covers `start` and not `end`.
Every token's `end` is the byte after its last byte. `Eof` has an empty span at
the end of the file.

**Tabs are rendered as one space.** Not eight, not four. The column the
diagnostic reports is the column the caret points at, and a renderer that expands
tabs has to explain why the caret moved.

**Recovery is a requirement, not a nicety.** A stage reports what it can and
continues. A stage that stops at the first error, or that loops on malformed
input, is broken even if its happy path is perfect.

**Determinism is a requirement.** The same input produces the same output, byte
for byte. No iteration over an unordered map, no timing-dependent ordering, no
hash-seed-dependent output. The conformance corpus depends on it.

**Nothing panics on user input.** A panic is a bug in the compiler, not a
diagnostic. `nudo check` on a hostile file must always terminate with a
diagnostic or a clean result.

## Adding a stage

1. Write the specification chapter first
   ([`../../spec/README.md`](../../spec/README.md)).
2. If it changes the language, get a NEP accepted
   ([`../../neps/README.md`](../../neps/README.md)).
3. Add the crate with its contract in
   [`../../ARCHITECTURE.md`](../../ARCHITECTURE.md), and respect the dependency
   rules.
4. Emit `nudo-diagnostics` values with codes from the right family. Never print.
5. Add fixtures, conformance cases and robustness properties before declaring it
   done.
6. Run `scripts/check.sh`.

## Measuring before optimising

`benchmarks/nudo-bench` gives a reproducible number for the front end. Use it to
compare a change against the previous commit, and be suspicious of a speed-up
that no test notices. The project has no performance target yet; it has a
measurement, which is what you need before a target is worth setting.

## Known rough edges

Recorded so that they are visible rather than rediscovered:

* The lexer does not compute literal values. Numbers and text are tokens; their
  values arrive with the parser.
* The lexer's identifier alphabet is ASCII. Whether NUDO has Unicode identifiers
  is an open question in [`../../DESIGN.md`](../../DESIGN.md#open-questions).
* `nudo check` reads every file before reporting anything. For a few files that
  is not a problem; a real driver will want to interleave.
* The token dump's `LineCol` conversion is computed per token. Fine at the
  current scale, and the first thing to revisit if dumping ever costs more than
  lexing.
