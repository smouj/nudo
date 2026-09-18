# Diagnostics and error codes

**State: the registry is in use** (milestones M1–M2). Codes marked *implemented*
are emitted today; codes marked *reserved* are allocated so that nobody invents
a competing one, and are not emitted by anything yet.

## Why codes are stable

A person reads the message. A tool — an editor, a CI job, a repair agent — reads
the code. Both matter, and they have different requirements: the message should
improve over time, and the code must not change meaning once it is published.

A code is therefore **permanent**. If a rule is removed, its code is retired and
never reused. If a rule changes meaning, it gets a new code, and the old one is
documented as superseded with a pointer.

## Families

The first digit after `NDO` is the family:

| Family | Area | Status |
| ------ | ---- | ------ |
| `NDO1xxx` | lexical structure and syntax | partially implemented |
| `NDO2xxx` | names and types | partially implemented |
| `NDO3xxx` | effects | reserved |
| `NDO4xxx` | agents, tasks and tools | reserved |
| `NDO5xxx` | capabilities and policies | reserved |
| `NDO6xxx` | runtime | reserved |
| `NDO7xxx` | interoperability | reserved |
| `NDO8xxx` | packages and files | partially implemented |
| `NDO9xxx` | internal compiler invariants | reserved |

`NDO9xxx` is for "this cannot happen" conditions. Emitting one means the compiler
has a bug, and the diagnostic is required to say so rather than to blame the
program.

## Registered codes

| Code | Name | Severity | State |
| ---- | ---- | -------- | ----- |
| `NDO1001` | `UNEXPECTED_TOKEN` | error | **implemented** (parser, M2) |
| `NDO2001` | `UNRESOLVED_NAME` | error | **implemented** (resolution, M3.1) |
| `NDO2002` | `DUPLICATE_NAME` | error | **implemented** (resolution, M3.1) |
| `NDO2003` | `WRONG_NAMESPACE` | error | **implemented** (resolution, M3.1) |
| `NDO2004` | `TYPE_MISMATCH` | error | **implemented** (type checker, M3.2) |
| `NDO2005` | `WRONG_ARGUMENT_COUNT` | error | **implemented** (type checker, M3.2) |
| `NDO2006` | `NOT_CALLABLE` | error | **implemented** (type checker, M3.2) |
| `NDO1002` | `UNKNOWN_CHARACTER` | error | **implemented** |
| `NDO1003` | `UNTERMINATED_STRING` | error | **implemented** |
| `NDO1004` | `UNTERMINATED_BLOCK_COMMENT` | error | **implemented** |
| `NDO1005` | `INVALID_NUMBER_LITERAL` | error | **implemented** |
| `NDO1006` | `INVALID_ESCAPE_SEQUENCE` | error | **implemented** |
| `NDO8001` | `UNEXPECTED_FILE_EXTENSION` | warning | **implemented** |

The single source of truth in code is
[`../compiler/nudo-diagnostics/src/lib.rs`](../compiler/nudo-diagnostics/src/lib.rs):
`codes::ALL` lists every registered code and `codes::IMPLEMENTED` lists the ones
the toolchain can actually emit. A test asserts that the second is a subset of
the first, and that identifiers are well formed and unique.

**The registry may not claim a capability the toolchain lacks.** A code that no
stage can emit is `reserved`, and the specification says so.

## Anatomy of a diagnostic

Every diagnostic carries:

* a **code** — stable, from the registry;
* a **severity** — `error` rejects the program, `warning` does not, `note` is
  attached context;
* a **message** — one line, lowercase, no trailing full stop, describing what is
  wrong rather than scolding;
* a **span**, whenever the problem is in the source. A diagnostic without a span
  is a tool error, not a language error;
* optional **notes** (context) and at most one **help** (the action to take).

Rendered:

```text
error[NDO1002]: unknown character `@`
  --> examples/00-hello-world/main.nudo:3:5
  |
3 |     @
  |     ^
  = help: the pre-alpha lexer recognises only the minimal token set
```

Rules for messages:

* say what is wrong, and name the offending text with backticks;
* do not blame the author, and do not be cute;
* never suggest a fix that is not valid NUDO;
* `help` is an instruction, `note` is context. Do not put an instruction in a
  note.

## Recovering

A stage reports as many real problems as it can, then continues:

* the lexer reports an unlexable character and keeps it as a token, so that no
  byte of the file is unaccounted for;
* a malformed literal still produces a token;
* the parser reports the token it could not place, puts it in an `Error` node and
  resumes at the next `;`, `}` or item;
* a stage never stops at the first error, and never loops forever on bad input.

Recovery rules exist so that one typo does not produce one message followed by
fifty cascading ones. Cascading diagnostics are treated as a bug in the stage
that emitted them. Three rules make that concrete:

* **Every recovery step consumes a token.** A recovery that loops is a hang, and
  a hang on user input is a bug.
* **One position, one diagnostic.** If the lexer already rejected a byte, the
  parser does not report the construct that byte made unreadable. The reader
  sees one mistake, and an agent repairing the file fixes one byte.
* **A cap, not a cascade.** The parser reports at most 24 syntax errors per file.
  A file of nonsense stops producing messages long before it stops being read.

## Rendering

Rendering lives in [`../compiler/nudo-diagnostics`](../compiler/nudo-diagnostics)
and nowhere else. Stages construct diagnostics; only the CLI prints them.

* The gutter follows the convention people already read in Rust and C: the
  `-->` line is indented by the width of the line number, and every `|` lines up
  with it.
* Columns are counted in **characters**, not bytes, so a caret lands where an
  editor's cursor would.
* A tab in the offending line is rendered as a single space, so the caret does
  not drift.
* Colour is off unless the output is a terminal, and can be forced with
  `--color always|never`.
* A multi-line span underlines the first line and stops; it does not draw a
  second caret block.

## Chains, not just positions

Some diagnostics are about a single token. The interesting ones are about a path
through the program:

```text
error[NDO3001]: `fetch` needs the `Network` capability
  --> app.nudo:12:5
   |
12 |     let body = fetch(url)
   |                ^^^^^ this call
   |
   = note: `fetch` declares `with Network`
   = note: reached from `render` (line 8), which has no `with` clause
   = help: declare `fn render(...) -> Text with Network`, or call a function that
           does not need it
```

A diagnostic that names only the final call site makes the reader reconstruct the
path themselves. Where a requirement propagates, the diagnostic **must** show the
chain that introduced it — the call, the declaration that demanded the capability
or the type, and each step between them. This is a requirement on the stages that
produce these diagnostics (M3 onward), not a style suggestion:

* trust errors (`Generated<T>` where `Verified<T>` is required) show the value's
  origin and the verification that is missing;
* capability and effect errors show the declaration that demanded it and the path
  that reached the call;
* budget errors show what was spent and where.

## Structured output

Human-readable output is not a machine interface. A tool — an editor, a CI job, a
repair agent — needs the same information as data: code, severity, message, file,
span, labels, notes and help.

`nudo check --json` is **PLANNED, not implemented**, and deliberately not
implemented in M1: a schema published before the parser exists would have to
change as soon as real syntax errors appear. When it arrives it is specified here
first, with a version field, and the human renderer and the JSON renderer are
required to describe the same diagnostics — a mismatch between them is a bug in
whichever is wrong.

## The diagnostic contract for agents

Diagnostics are the interface through which an agent repairs a program, so:

* the code is machine-matchable and stable;
* the position is exact enough to rewrite the right bytes;
* the text is deterministic for the same input;
* the toolchain never emits a diagnostic it cannot justify.

`nudo check` prints the same content to stderr as a human sees, and the exit code
distinguishes "clean" (`0`), "diagnostics found" (`1`) and "the tool could not
run" (`2`). A caller must never have to parse English to decide what happened.
