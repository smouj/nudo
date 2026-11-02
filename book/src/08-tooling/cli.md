# 08.1 — Toolchain and CLI

> **Status:** Implemented / Planned  
> **Summary:** The current <code>nudo</code> command checks source, syntax and names; type checking, execution, effects and the wider toolchain remain future work.

## Implemented commands

~~~sh
nudo check path/to/file.nudo
nudo check --dump-tokens path/to/file.nudo
nudo check --dump-tree path/to/file.nudo
nudo check --dump-resolutions path/to/file.nudo
nudo --version
nudo --help
~~~

The check command loads one or more source files, lexes and parses them, lowers
accepted syntax to HIR, resolves names and reports lexical, syntax and
resolution diagnostics. It can also emit a plain or CI-oriented report.

Its scope ends at name resolution. A clean result does not promise that the
program is type-correct or executable. The partial type checker is a separate
crate with unit tests and conformance cases; the CLI does not call it. There is
no <code>nudo run</code>, <code>nudo build</code>, REPL, formatter, language
server or package manager.

Exit codes are stable: 0 means no error diagnostic in the implemented stages,
1 means at least one error was reported, and 2 means a usage or input error or
an unimplemented command.

## Front-end measurements

The separate <code>nudo-bench</code> program measures four deterministic,
synthetic inputs at each of four stages: lex, parse, HIR lowering and type
checking. That produces sixteen rows. The source and HIR setup for later stages
happens outside their timed regions; parse includes tokenization because the
public parser consumes tokens.

The type-checker reports the length of its expression-type table. That count
includes slots without a type and does not count semantic rules executed.
Equal counts are not proof that two type-checker revisions performed equal work.

{{#include ../diagrams/benchmark-portability.svg}}

*Observed time depends on the entire recorded setup. Hardware-to-hardware
scaling cannot be inferred reliably from CPU clock or core count alone.*

A run records commit, dirty state, Rust version, operating system, architecture,
CPU model when available, input size, iterations and timing statistics. There
is no project-wide reference baseline or performance gate in CI.

For useful comparisons, build both revisions in release mode, use identical
workloads and iteration settings, and record them on the same quiet machine.
Repeat the runs. Treat threshold crossings as investigation signals; a single
sample is not proof of a regression.

Results from one computer describe that computer and workload. They can provide
a rough point of reference for planning, but estimating another computer’s
timing needs a measurement on that computer with the same commit, toolchain,
workload and settings. Different CPUs, memory systems, schedulers and sustained
frequencies can change stages by different amounts, so there is no dependable
one-number conversion between machines.

The suite measures compiler front-end stages only. It cannot report interpreter
startup, end-to-end program execution, agent/model calls or peak memory because
those systems do not yet exist in the toolchain.

\n