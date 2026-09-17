# The `nudo` command line

```console
$ nudo --help
nudo 0.0.1 (pre-alpha) — the NUDO language toolchain

USAGE:
    nudo <COMMAND> [OPTIONS] [FILE]...
    nudo --version
    nudo --help
...
```

## What works today

| Command | Does |
| ------- | ---- |
| `nudo check <FILE>...` | Reads `.nudo` files, lexes them, reports lexical diagnostics |
| `nudo --version` | Prints the version and the release channel |
| `nudo --help` | Lists the implemented commands and the planned ones |

`check` options:

| Option | Meaning |
| ------ | ------- |
| `--dump-tokens` | Prints the token stream in the stable `nudo-tokens v1` format |
| `--color <WHEN>` | `auto` (default), `always`, `never` |
| `-h`, `--help` | Command help |

```console
$ nudo check examples/00-hello-world/main.nudo
checked 1 file: 0 errors, 0 warnings

$ nudo check --dump-tokens examples/00-hello-world/main.nudo
# nudo-tokens v1
0000 3:1-3:3 KeywordFn "fn"
...
```

## What is declared but not implemented

`nudo --help` lists these, and each one exits with code `2` and says plainly that
it is planned. It never exits successfully while doing nothing.

| Command | Will do | Milestone |
| ------- | ------- | --------- |
| `new` | Create a package | M10 |
| `init` | Add a manifest to a directory | M10 |
| `run` | Compile and run | M4 |
| `build` | Compile | M4 |
| `test` | Run a package's tests | M10 |
| `fmt` | Format sources | M10 |
| `doc` | Generate documentation | M10 |
| `repl` | Interactive session | M10 |
| `trace` | Inspect a run's trace | M6 |
| `eval` | Evaluate agent and task behaviour | M10 |
| `doctor` | Report toolchain health | M10 |
| `audit` | Audit capabilities, policies and provenance | M10 |

## Exit codes

| Code | Meaning |
| ---- | ------- |
| `0` | Success, no error diagnostics |
| `1` | At least one error diagnostic was reported |
| `2` | Usage error, unreadable input, or an unimplemented command |

These are stable, and a caller must never have to parse English to tell the
outcomes apart. `1` and `2` are different failures: `1` means your program has a
problem, `2` means the tool does not.

## Colour

`--color auto` colours only when stderr is a terminal, so piping to a file
produces clean text. `always` and `never` force the choice, for tests and for
users whose terminal is not detected correctly.

## Reading a diagnostic

```console
$ nudo check broken.nudo
error[NDO1003]: unterminated text literal
  --> broken.nudo:2:20
  |
2 |     let greeting = "never closed
  |                    ^^^^^^^^^^^^^
   = note: a text literal must be closed on the same line it starts on
   = help: add a closing `"`

nudo: 1 error and 0 warnings
```

The code is stable and matchable by a tool; the position is exact enough to
rewrite the right bytes; the text is deterministic for the same input. Details
in [`../../spec/errors.md`](../../spec/errors.md).

## What `check` does not do

It does not parse, type-check, or run the program. A clean result means "no
lexical diagnostics". The toolchain says so in its own help text, because a
command that overstates what it checked is worse than a command that does less.
