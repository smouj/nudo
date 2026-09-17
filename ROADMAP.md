# NUDO — roadmap

Milestones, not dates. A milestone ends when its exit criteria are met and its
conformance cases pass, not when a calendar is empty. Dates are deliberately
absent: a language that ships on a schedule instead of on evidence is a
language that lies about its stability.

Status legend: **done**, **in progress**, **planned**.

## M0 — Repository foundation — **done**

The repository can be built, checked and reviewed by a stranger without asking
anyone anything.

* [x] Monorepo layout, workspace manifest and pinned toolchain
* [x] Dual licence (Apache-2.0 OR MIT)
* [x] CI running the same pipeline as `scripts/check.sh`, on Linux, Windows and macOS
* [x] Issue and pull request templates, discussion templates, CODEOWNERS
* [x] Governance, security policy, contribution guide and the agent contract
* [x] Brand assets, and a brand document that says what may not be done with them

**Exit criteria:** a fresh clone builds, `scripts/check.sh` passes, and no
document describes behaviour the toolchain does not have.

## M1 — Source model and lexer — **done**

```text
.nudo → SourceFile → Lexer → Tokens → Diagnostics
```

* [x] `SourceFile`, `SourceId`, `SourceMap`, UTF-8 loading
* [x] `Span`, `BytePos`, line/column mapping with Unicode columns
* [x] Lexical structure: identifiers, numbers, text literals, comments, punctuation
* [x] Error recovery: lexing never stops at the first error and never panics
* [x] `NDO` diagnostic codes with a rendered source snippet
* [x] `nudo check` (lexical only), `nudo --version`, `nudo --help`
* [x] Conformance corpus with a documented, implementation-neutral format
* [x] Adversarial robustness properties (2 000 random inputs, 300 structured inputs)

**Exit criteria:** the token stream of every conformance case is reproducible by
a second implementation from the corpus alone. Met.

**Known gaps, deliberately accepted:** keywords other than `fn` and `let` are
not reserved; Unicode identifiers are an open question
([`DESIGN.md`](DESIGN.md#open-questions)); a literal's *value* is not computed
until the parser needs it.

## M2 — Parser and AST — **planned**

* [ ] Lossless syntax tree (`nudo-syntax`): every byte of input is reachable
* [ ] Recursive-descent parser with error recovery
* [ ] Typed AST and visitors (`nudo-ast`)
* [ ] `NDO1001` and the rest of the `1xxx` family in use
* [ ] Formatter able to round-trip without losing a byte
* [ ] Conformance cases for valid and invalid programs

**Exit criteria:** the parser accepts every `fixtures/valid` program, rejects
every `fixtures/invalid` one with the declared diagnostics, and the formatter is
idempotent on the conformance corpus.

## M3 — Type system — **planned**

* [ ] Primitive types, structs, enums, generics
* [ ] `Result` and `Generated<T>` / `Verified<T>` as distinct types
* [ ] Agent, task and tool type declarations
* [ ] Type errors with stable `NDO2xxx` codes
* [ ] Name resolution in HIR

**Exit criteria:** no `Generated<T>` value is usable as a `Verified<T>` without
an explicit verification step, proven by a conformance case.

## M4 — Interpreter — **planned**

* [ ] MIR lowering and a tree-walking interpreter
* [ ] Deterministic execution of ordinary programs
* [ ] `nudo run`, `nudo build`, `nudo repl`
* [ ] Runtime diagnostics with stable `NDO6xxx` codes
* [ ] Runtime startup benchmark

**Exit criteria:** the examples in `examples/00-` through `examples/02-` run and
produce their documented output. The later examples are design previews and are
excluded from this criterion until the syntax they use is accepted.

## M5 — Effect system — **planned**

* [ ] Effect inference and checking (`nudo-effects`)
* [ ] Effects reported in signatures and callable through explicit handlers
* [ ] `NDO3xxx` codes

**Exit criteria:** an effectful call from an effect-free function is a compile
error, with a diagnostic that names the effect and the path that introduced it.

## M6 — `Generated<T>` and `Verified<T>` — **planned**

* [ ] Verifier protocol, and `verify` in the grammar
* [ ] Provenance records attached to verified values (`nudo-provenance`)
* [ ] Execution traces (`nudo-trace`)
* [ ] `nudo trace`

**Exit criteria:** every `Verified<T>` value in a program can be traced back to
the inputs and the verification step that produced it.

## M7 — Agent runtime — **planned**

* [ ] Agent declarations, lifecycle and delegation (`nudo-agent`)
* [ ] Provider-agnostic model interface (`nudo-model`)
* [ ] Tool registration and dispatch (`nudo-tool`)
* [ ] Tasks, acceptance criteria and budgets (`nudo-task`)
* [ ] Memory stores (`nudo-memory`)

**Exit criteria:** a multi-agent example runs end to end with a complete trace,
against a local model, with no network access required.

## M8 — Capabilities and policies — **planned**

* [ ] Capability declarations, grants and checks (`nudo-capability`)
* [ ] Policies and approvals (`nudo-policy`)
* [ ] Sandbox boundary for tools (`nudo-sandbox`)
* [ ] `NDO5xxx` codes
* [ ] Threat model reviewed against a working implementation

**Exit criteria:** a program that attempts an ungranted effect fails before the
effect happens, and the failure names the capability and the request site.

## M9 — Interoperability — **planned**

* [ ] MCP client (consume external tools) and server (expose NUDO tools)
* [ ] A2A delegation between implementations
* [ ] `NDO7xxx` codes
* [ ] Untrusted-input handling reviewed against the threat model

**Exit criteria:** a NUDO program calls an external MCP tool without the
adapter being able to widen its own capabilities.

## M10 — Tooling — **planned**

* [ ] Language server with diagnostics, go-to-definition and formatting
* [ ] Documentation generator
* [ ] Test runner and evaluation harness
* [ ] `nudo fmt`, `nudo doc`, `nudo test`, `nudo eval`, `nudo doctor`, `nudo audit`

**Exit criteria:** an editor integration works against the language server with
no compiler-specific client code.

## M11 — WebAssembly — **planned**

* [ ] WASI target producing runnable modules
* [ ] Capability checks preserved across the boundary
* [ ] Release artefacts for every supported target, with checksums and an SBOM

**Exit criteria:** a NUDO program compiled to WASI runs in a host with only the
capabilities its declarations asked for.

## After 1.0

Not defined here, on purpose. Stabilisation criteria belong in
[`GOVERNANCE.md`](GOVERNANCE.md), and the compatibility story has to be designed
against a language that actually exists.
