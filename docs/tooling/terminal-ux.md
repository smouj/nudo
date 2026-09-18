# Terminal UX

**State: implemented for `nudo check`** (Phases A–D of the plan below). This
document is the single source of truth for how NUDO speaks in a terminal. It is
presentation only: nothing here may change what a command does.

The rule that decides every argument in this file:

> Improving how NUDO *shows* its state is in scope. Changing how NUDO *works* is
> not. The interface adapts to the compiler and the roadmap; never the other way
> round.

## The boundary

```text
compiler / runtime / tooling
        │  structured data and diagnostics
        ▼
       CLI                      cli/nudo-cli
        │
        ▼
  terminal renderer            cli/nudo-cli/src/terminal
```

A compiler stage must never know about colour, ANSI, spinners, terminals,
progress bars, TTYs, console width or animation. Stages produce data and
diagnostics; the CLI decides how to present them. `nudo-diagnostics` remains the
source of truth for codes, severities and positions, and is rendered, never
re-derived.

Two consequences worth stating because they are the mistakes this design exists
to prevent:

* **No stage prints.** `println!` in the lexer, parser, HIR or any later stage is
  a bug, not a shortcut.
* **No fake stages.** The pipeline shows the stages this command actually
  intended to run for this file, and what each one did. A stage the command would
  have reached but did not is `pending` with its reason (`not reached`); a stage
  the command never runs is simply absent, so there is no `TYPE` row before the
  type checker exists. Stage lists are built from real data, never hardcoded.

## States and symbols

Five states, centralised in one place. These symbols are never written by hand in
a command.

| State | Unicode | ASCII | Meaning |
| ----- | ------- | ----- | ------- |
| pending | `○` | `.` | not started |
| active | `●` | `>` | running now |
| success | `✓` | `+` | finished, nothing to report |
| warning | `!` | `!` | finished, with something to look at |
| error | `×` | `x` | failed |

The ASCII set is used when the terminal cannot represent the Unicode one, or when
the user asked for it. **State is never carried by the symbol alone**: the label
is always there, so a reader who cannot see the glyph still knows what happened.

## Colour

Colour is secondary to structure, and optional everywhere.

| Role | Used for |
| ---- | -------- |
| `nudo` (blue) | the active stage, the current thing |
| `ok` (green) | success |
| `warn` (yellow) | warnings |
| `err` (red) | errors |
| `dim` | metadata: counts, paths, timing |

Rules: never colour a whole block; colour the state symbol, the `NDOxxxx` code
and the active item, and little else; every distinction must survive with no
colour at all.

`NO_COLOR` (any value, per <https://no-color.org>), `TERM=dumb`, a redirected
stream or `--color never` all turn colour off. `--color always` turns it on where
the stream would otherwise not have it.

## Modes

One renderer, three modes, chosen from the environment — never from the command's
meaning.

| Mode | When | What it may use |
| ---- | ---- | --------------- |
| `Interactive` | stdout is a TTY, no CI, no `--plain` | colour, in-place updates, the pulse, a live pipeline |
| `Plain` | a pipe, a redirect, `--plain`, `NO_COLOR` + non-TTY | one line per event, no colour, no animation |
| `Ci` | `--ci`, or `CI` in the environment with a TTY | deterministic full block, no animation, ends with `PASS`/`FAIL` |

`--json` is reserved for the structured form and is **not implemented**; when it
arrives it must never mix with ANSI or animation.

### Interactive

```text
NUDO CHECK

src/main.nudo
────────────────────────────────────────

✓ SOURCE ── ✓ LEX 12 tokens ── ✓ PARSE ── ✓ HIR 4 definitions

SUMMARY
  files     1
  errors    0
  warnings  0
────────────────────────────────────────
checked 1 file: 0 errors and 0 warnings
```

A stage says what it did when that is worth a reader's attention. When the line
would not fit, the stages stack instead — same information, smaller window:

```text
  ✓ source
  ✓ lex    12 tokens
  ✓ parse
  ✓ hir    4 definitions
```

### Plain

```text
checking src/main.nudo
source: ok
lex: ok
parse: ok
hir: ok
checked 1 file: 0 errors and 0 warnings
```

### Ci

```text
NUDO CHECK

[1/4] source ok
[2/4] lex    ok
[3/4] parse  ok
[4/4] hir    ok

files       1
errors      0
warnings    0

checked 1 file: 0 errors and 0 warnings

PASS
```

The last line of every mode is the stable, greppable contract. `check` keeps
`checked 1 file: 0 errors and 0 warnings` in all three modes, because scripts and
documentation already depend on it: structure is added *around* the contract,
never instead of it.

## `NudoPulse`

The indeterminate-progress animation. It is NUDO's, not a generic spinner: a node
travelling along a connection.

```text
●────    ─●───    ──●──    ───●─    ────●    ───●─    ──●──    ─●───
```

Rules, all of them non-negotiable:

* It animates **only** when the output is a TTY, colour/motion is allowed, and no
  CI or `--plain`/`--quiet`/`NO_COLOR` applies.
* It runs at a **moderate rate** (about 10 frames per second), never 60.
* It is a single line, redrawn in place.
* **Its absence never changes a result.** The same command, piped, produces the
  same exit code, the same diagnostics and the same summary.
* It is only used for genuinely indeterminate operations. A stage that knows its
  progress shows the stage, not an animation.

## Diagnostics

The existing model is rendered, not replaced.

```text
NDO2003  UNRESOLVED NAME

src/main.nudo:6:5

  5 │ fn make() {
  6 │     User
    │     ──── not a value
  7 │ }

`User` is a type, not a value.

1 error
```

Not negotiable: no code is renumbered, no severity changes for visual reasons, no
second renderer, and no stage emits a diagnostic itself.

## Width

From 40 columns to very wide. The same information, recomposed:

```text
wide                                  narrow
✓ SOURCE ── ✓ LEX ── ● PARSE          PIPELINE
                                        ✓ source
                                        ✓ lex
                                        ● parse
```

One rule: no fixed column assumptions. Labels are padded to the widest label in
their own block, and each block's own width decides whether it may sit on one
line.

## Accessibility

Never depend on colour, animation or Unicode alone. Every state has a word, every
mode works without colour, the ASCII symbol set is a first-class alternative, and
motion is opt-out. `NO_COLOR`, a dumb terminal, a pipe and a narrow window are
normal cases, not degraded ones.

## What this layer deliberately does not do yet

`nudo run`, the REPL, tasks, agents, budgets and traces are **not implemented**,
and the terminal layer must not pretend otherwise: no `TYPE` row before the type
checker exists, no `PASS` for a stage that did not run, no fake budget line.

The shapes are reserved so they can be added without redesign, and nothing else:

* `nudo run` will render `TASK`, `STATUS`, `STEPS`, `BUDGET` as further blocks of
  the same key/value and pipeline primitives.
* The REPL prompt is reserved as `nudo›`.
* `--json` is reserved for machine-readable output.

## Testing

| Level | Covers |
| ----- | ------ |
| Unit | capability detection, symbol fallback, layout and width handling, state transitions, the plain renderer, pulse frames and their gating |
| Snapshot | rendered strings for: a clean check, a lexical error, a syntax error, a resolution error, a warning, a narrow terminal, `NO_COLOR`, a non-TTY, and CI |
| Regression | exit codes, diagnostic codes and counts, and compiler results unchanged; the conformance corpora still green |

Snapshots are always taken with animation off, so a pipeline that happens to be
fast or slow cannot make a test flaky.

## Implementation plan

| Phase | Content | State |
| ----- | ------- | ----- |
| A | This specification | **done** |
| B | `TerminalCapabilities`, `Theme`, `Symbols`, `Status`, layout primitives, with unit tests | **done** |
| C | Applied to `nudo check`'s static output, with snapshots and no behaviour change | **done** |
| D | Pipeline rendering from the stages that actually ran | **done** |
| E | `NudoPulse`, TTY-only, after the rest is stable | implemented, gated off outside a TTY |
| F | Responsive width, plain, CI and machine-readable fallbacks | planned, narrowed by `--json` |

Everything lands in `cli/nudo-cli/src/terminal`. It is **not** a crate: there is
no architectural boundary here that a crate would express, and NUDO does not add
crates for convenience. It has **no dependencies**: capability detection, symbols,
layout and the pulse are a few hundred lines of standard library, and adding
`indicatif`, `console`, `crossterm` or `owo-colors` to draw eight characters would
be the first third-party dependency in the workspace, bought for a spinner.
