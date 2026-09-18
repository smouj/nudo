# NUDO — architecture

This document describes the shape of the workspace: which crate owns what,
which dependency edges are allowed, and where the boundaries are. It is
normative for contributors and agents — see [`AGENTS.md`](AGENTS.md).

## Workspace

NUDO is a monorepo with one Rust workspace. The reason is not convenience: the
specification, the compiler, the runtime and the toolchain must change together
under one reviewable history, because a language change touches all four.

```text
compiler/     the compiler pipeline, stage by stage
runtime/      execution: agents, tools, models, policies, traces
  crates/     shared foundations with no dependencies on the layers above
cli/          the `nudo` binary
tooling/      formatter, language server, docs, tests, evaluation, REPL
backends/     execution strategies: tree-walking interpreter, WebAssembly
std/          the standard library (NUDO code, planned)
benchmarks/   reproducible measurements
```

Crate names are `nudo-<thing>`; directory names match the crate name except in
`backends/`, where the directory names the strategy and the crate is
`nudo-backend-<strategy>`.

## Compiler pipeline

Each stage has its own crate. Stages communicate through data, not through
shared mutable state, and a later stage never reaches back into an earlier one.

```text
Source text
    ↓
nudo-lexer          tokens + lexical diagnostics      implemented (M1)
    ↓
nudo-syntax         lossless syntax tree              implemented (M2)
nudo-parser         syntax tree                       implemented (M2)
nudo-ast            abstract syntax tree             implemented (M2)
    ↓
nudo-hir            name resolution, desugaring      implemented (M3)
    ↓
nudo-typeck         types, including trust types     planned (M3)
    ↓
nudo-effects        effects and capabilities         planned (M5)
    ↓
nudo-mir            mid-level IR                     planned (M4)
    ↓
nudo-codegen        backend-agnostic lowering        planned (M4)
    ↓
backends/interpreter | backends/wasm                 planned (M4 / M11)
```

Cross-cutting:

* `nudo-diagnostics` — the diagnostic model, the stable `NDO` code registry and
  the renderer. Every stage emits through it; no stage formats its own errors.
* `crates/nudo-span`, `crates/nudo-source` — positions, spans, files.
* `crates/nudo-common` — the vocabulary every crate agrees on.
* `crates/nudo-schema` — manifest and lockfile schemas.

## Crates

Every crate below is a real workspace member. "Implemented" means the crate has
behaviour and tests today; "planned" means it is an empty placeholder whose
contract is written down here, so that the shape of the workspace does not
change when work starts.

### Foundations — `crates/`

| Crate | Responsibility | Status |
| ----- | -------------- | ------ |
| `nudo-common` | Language name, toolchain version, file names, release channel. No dependencies. | Implemented |
| `nudo-span` | `BytePos`, `Span`, `LineIndex`, line/column mapping. | Implemented |
| `nudo-source` | `SourceId`, `SourceFile`, `SourceMap`, UTF-8 loading. | Implemented |
| `nudo-schema` | `nudo.toml` manifest and `nudo.lock` lockfile schemas. | Planned (M?) |

### Compiler — `compiler/`

| Crate | Responsibility | Status |
| ----- | -------------- | ------ |
| `nudo-lexer` | Source text → tokens, with lexical diagnostics and error recovery. | **Implemented (M1)** |
| `nudo-diagnostics` | Diagnostic model, `NDO` code registry, source-snippet renderer. | **Implemented (M1)** |
| `nudo-syntax` | Lossless syntax tree, preserved trivia, used by parser, formatter and LSP. | **Implemented (M2)** |
| `nudo-parser` | Recursive-descent parser producing the syntax tree. | **Implemented (M2)** |
| `nudo-ast` | Typed AST and visitors. | **Implemented (M2)** |
| `nudo-hir` | Name resolution and desugaring into a resolved IR. | **Implemented (M3.1)** |
| `nudo-typeck` | Type checking, including `Generated<T>`, `Verified<T>`, agent and task types. | Planned (M3) |
| `nudo-effects` | Effect and capability inference and checking. | Planned (M5) |
| `nudo-mir` | Mid-level IR: control flow made explicit, backends consume this. | Planned (M4) |
| `nudo-codegen` | Backend-agnostic lowering from MIR. | Planned (M4) |

### Runtime — `runtime/`

| Crate | Responsibility | Status |
| ----- | -------------- | ------ |
| `nudo-runtime` | The execution loop: scheduling, cancellation, resumption. | Planned (M4) |
| `nudo-agent` | Agent instantiation, lifecycle, delegation, supervision. | Planned (M7) |
| `nudo-model` | Provider-agnostic model interface. No provider is privileged. | Planned (M7) |
| `nudo-tool` | Tool registration, dispatch and result validation. | Planned (M7) |
| `nudo-task` | Task execution, acceptance criteria, budgets, retries. | Planned (M7) |
| `nudo-policy` | Policy evaluation, including approvals. | Planned (M8) |
| `nudo-capability` | Capability grants and checks. Deny by default. | Planned (M8) |
| `nudo-provenance` | Provenance records for values and artefacts. | Planned (M6) |
| `nudo-trace` | Execution traces for autonomous runs. | Planned (M6) |
| `nudo-memory` | Program and agent memory stores. | Planned (M7) |
| `nudo-sandbox` | Isolation boundary for tools and generated code. | Planned (M8) |

### Toolchain — `cli/`, `tooling/`, `backends/`

| Crate | Responsibility | Status |
| ----- | -------------- | ------ |
| `nudo-cli` (binary `nudo`) | Argument handling, command dispatch, exit codes. | **Implemented (M1–M2)** |
| `nudo-fmt` | Canonical formatter. | Planned (M10) |
| `nudo-lsp` | Language server. | Planned (M10) |
| `nudo-doc` | Documentation generator. | Planned (M10) |
| `nudo-test` | Test runner and conformance driver. | Planned (M10) |
| `nudo-eval` | Evaluation harness for agent, task and model behaviour. | Planned (M10) |
| `nudo-repl` | Interactive session. | Planned (M10) |
| `nudo-backend-interpreter` | Tree-walking interpreter: NUDO's first execution strategy. | Planned (M4) |
| `nudo-backend-wasm` | WebAssembly/WASI backend. | Planned (M11) |

## Boundary rules

These are enforced by review today and by tooling later:

1. **No cycles.** The crate graph is a DAG. A cycle is a design error, not a
   build problem.
2. **Foundations point down only.** Nothing in `crates/` may depend on
   `compiler/`, `runtime/`, `tooling/`, `cli/` or `backends/`.
3. **Compiler stages are ordered.** A stage may depend on earlier stages via
   the documented types, never on a later one.
4. **The runtime does not parse source.** It consumes compiler output. If the
   runtime needs to understand syntax, a compiler stage is missing.
5. **No crate depends on `nudo-cli`.** The binary is a leaf.
6. **Diagnostics are formatted in one place.** No stage prints; stages emit
   `nudo-diagnostics` values, and the CLI renders them.
7. **No dumping-ground crates.** There is no `nudo-utils`. A crate with "and"
   in its responsibility is two crates.
8. **Capability checks are not optional.** Any crate that can touch the world
   outside the process goes through `runtime/nudo-capability`.

## Runtime shape

The runtime is not implemented. Its intended shape, for review purposes:

* execution is a loop over tasks, not a thread per agent;
* every outbound effect (tool call, model call, filesystem, network) is checked
  against the capability set in force for that task before it happens;
* budgets are decremented as effects are requested, and exhaustion is a normal,
  checkable outcome rather than a panic;
* traces and provenance records are produced during execution, not
  reconstructed afterwards;
* cancellation is cooperative and observable.

## Standards and toolchain

| Concern | Decision |
| ------- | -------- |
| Implementation language | Rust, stable toolchain, MSRV declared in `Cargo.toml` |
| Third-party dependencies | none in the pre-alpha workspace; a new one needs a justification |
| Edition | workspace-wide, declared once |
| Formatting | `rustfmt`, enforced by CI |
| Lints | `clippy` with warnings denied, plus workspace lints in `Cargo.toml` |
| Portable targets | Windows x86_64, Linux x86_64, Linux ARM64, macOS ARM64, WASI |
| Specification format | Markdown, plus EBNF for the grammar |
| Error codes | stable `NDO` codes, allocated by family (`spec/errors.md`) |

## Where to add things

| You are adding | It goes in |
| -------------- | ---------- |
| A language rule | `spec/`, then conformance tests, then the compiler |
| A new error | `compiler/nudo-diagnostics` (registry) and `spec/errors.md` |
| A compiler stage behaviour | the stage's crate, with tests next to it |
| A runtime capability | `runtime/nudo-capability`, plus `spec/trust/capabilities.md` |
| A CLI command | `cli/nudo-cli`, plus `docs/tooling/` |
| A measurement | `benchmarks/` |
| A conformance case | `tests/conformance/<stage>/` |
| A unit-level input | `fixtures/` |
