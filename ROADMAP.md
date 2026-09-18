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

## M2 — Parser and syntax tree — **done**

### M2.0 — Grammar freeze — **done**

The syntactic grammar is decided before anything parses it, and its two
load-bearing rules are enforced mechanically rather than by reviewer attention:

* [x] **No left recursion.** Recursive descent cannot parse it, and the grammar
      had four such productions (`call-expression`, `field-expression`,
      `binary-expression`, `optional-type`) when this was checked.
* [x] **Precedence stated once**, as a chain of grammar levels that the
      productions implement, copied identically into the two documents that
      describe it.
* [x] `scripts/check-grammar.py` fails the pipeline if either rule breaks, or if
      the three copies of the precedence chain disagree.
* [x] The M2 design is written down before the code:
      [`docs/internals/parser-design.md`](docs/internals/parser-design.md).

**Gate.** Four decisions blocked a *correct* parser, not a compiling one. Two were
taken as part of M2.1 and are implemented: the keyword policy
([NEP-0005](neps/0005-keyword-policy.md), which also resolved the real parse
conflict between `true`/`false` and identifiers) and generic syntax
([NEP-0006](neps/0006-generic-syntax.md)). The other two — the `verify` form and
mutability — remain open, and their absence is recorded in
[`docs/internals/parser-design.md`](docs/internals/parser-design.md) rather than
hidden.

### M2.1 — Parser and syntax tree — **done**

* [x] `nudo-syntax`: lossless tree, `SyntaxKind`, error nodes, and the property
      test that the tree's token texts re-concatenate to the original file
* [x] `nudo-parser`: recursive descent with precedence climbing on the declared
      chain, and recovery with a progress guarantee (no hangs, no cascades)
* [x] `nudo-ast`: typed wrappers for consumers that want structure, not text
* [x] `NDO1001` and the rest of the `1xxx` family in use, with expected/found
* [x] `nudo check` parses, so a clean run stops meaning "lexically correct"
* [x] A tree dump in a stable format, matching what `--dump-tokens` gives tokens
* [x] Conformance cases under `tests/conformance/parser/` (13 cases)

**Exit criteria:** met. Every `fixtures/valid` program parses to a tree with no
error nodes, every `fixtures/invalid` one is rejected with the declared
diagnostics, the losslessness property holds across the corpus, and the tree
reprints its source byte for byte (`SyntaxTree::reprint`, checked by every
conformance case).

**The gate was taken first.** The two cheap decisions that blocked a correct
parser were settled in this milestone:

* [NEP-0005](neps/0005-keyword-policy.md) — keyword policy: a ten-word reserved
  core, everything else contextual. Implemented in `nudo-lexer` and
  `nudo-parser`.
* [NEP-0006](neps/0006-generic-syntax.md) — generic syntax: angle brackets,
  invariant type arguments, no bounds, no declaration-site parameters.

The other two decisions the design document listed are still open, and do not
block a parser for the frozen grammar: the `verify` form
([NEP-0002](neps/0002-generated-verified.md)) and mutability. Both are recorded
in [`docs/internals/parser-design.md`](docs/internals/parser-design.md), with the
one visible consequence of the first: `verify (expr) with V` is rejected.

**Known gaps, deliberately accepted:**

* declaration-site generic parameters (`enum Outcome<T, E>`) are not in the
  grammar, so they are rejected — NEP-0006 says why;
* the grammar's paths use `::` and its lists are comma-separated, while several
  examples write `web.search` one per line. The parser follows the grammar, the
  examples say they are previews, and the spelling needs a NEP to change;
* `nudo-ast` exposes items and expressions, not every possible shape; it grows
  when a consumer needs it.

**The formatter is still M10.** M2 owes it a tree it can reprint, and pays that
debt: `reprint` is the identity for every fixture and every conformance case.

## M3 — Type system — **in progress**

### M3.0 — Semantic gates — **done**

The semantics are decided before the checker is written, because they change what
the checker *does*, not how it is written. Six NEPs, all accepted and all
recorded with the alternatives that lost:

| Gate | Decision | NEP |
| ---- | -------- | --- |
| `Int` | Signed 64-bit, **traps** on overflow and division by zero, truncating division, a literal out of range is a compile error | [NEP-0007](neps/0007-integer-semantics.md) |
| Error model | `Result<T, E>` is **intrinsic**, failure is a value, a trap is not, and there is **no `?`** | [NEP-0008](neps/0008-error-model.md) |
| `verify` | Yields `Result<Verified<T>, VerificationError>`; its operand is a *named* value; a verifier is deterministic | [NEP-0002](neps/0002-generated-verified.md) |
| Mutability | Bindings are immutable, and stay immutable: no `mut`, no assignment, **no borrow checker** | [NEP-0009](neps/0009-mutability.md) |
| Generics | Declarations declare their parameters; `Result` is intrinsic | [NEP-0010](neps/0010-declaration-site-generics.md) |
| Spelling | A path uses `::`; a list inside a declaration is comma-separated | [NEP-0011](neps/0011-path-and-list-spelling.md) |

**Exit criteria:** met, in the sense that matters — every question a type checker
needs answered is answered, in `spec/`, with the reasoning in `neps/`. Nothing is
implemented yet, and the grammar changes two of these decisions require
(`generic-parameter-list`, the restricted `verify` operand) are recorded as M3.1
work rather than left as a surprise.

### M3.1 — HIR and name resolution

* [ ] `nudo-hir`: `DefId`, `ItemId`, `ExprId`, `LocalId`, `ScopeId`, `TypeRefId`,
      with spans preserved and no dependence on syntactic text
* [ ] Lowering from AST to HIR
* [ ] Name resolution: namespaces, shadowing, duplicates, types versus values,
      paths
* [ ] `NDO2001` (`unresolved name`) and the rest of the `2xxx` family in use
* [ ] `nudo-parser`: accept `generic-parameter-list` and the restricted `verify`
      operand that M3.0's NEPs added to the grammar

### M3.2 — Basic type system

* [ ] Primitive types: `Int` (with NEP-0007's rules), `Float`, `Bool`, `Text`,
      `Unit`
* [ ] Structs, enums, generics and instantiation, arity checking
* [ ] `Result<T, E>` as an intrinsic type, and exhaustiveness of `match` on it
* [ ] Function types and calls; no implicit numeric conversion
* [ ] Agent, task and tool type declarations
* [ ] Type errors with stable `NDO2xxx` codes

### M3.3 — The trust types

* [ ] `Generated<T>` and `Verified<T>` as distinct types, with no conversion
      between them
* [ ] `verify` yielding `Result<Verified<T>, VerificationError>`
* [ ] Conformance cases: conversion rejection, verification failure, provenance

**Exit criteria for M3:** the program below is *rejected*, and the one after it is
accepted, both pinned by conformance cases:

```nudo
fn publish(article: Verified<Article>) { }

let draft: Generated<Article> = ask Writer { "Create an article." };
publish(draft);                                  // NDO2xxx: Generated is not Verified
```

```nudo
let checked: Result<Verified<Article>, VerificationError> =
    verify draft with ArticleVerifier;

match checked {
    Ok(article) => publish(article)
    Err(reason) => report(reason)
}
```

That pair is the whole point of the language: it turns "the compiler can tell
what a model produced from what has been checked" into a property a build can
demonstrate.

**Not touched in M3:** the interpreter, the effect system, agents, MCP, the
sandbox, the language server and WebAssembly.

## M4 — Interpreter, as one vertical slice — **planned**

M4 is deliberately not "an interpreter". It is **one path from source to output
that really runs**, because a project with nine well-documented subsystems and no
executable program is still a specification. The smallest useful program is the
target:

```nudo
fn add(a: Int, b: Int) -> Int {
    a + b
}

fn main() {
    let result = add(20, 22);
    print(result);
}
```

Everything that requires, and nothing beyond it:

* [ ] Functions: parameters, return values, calls
* [ ] `let`, `Int`, `Bool`, `Text`
* [ ] Arithmetic and comparison operators, `if`
* [ ] MIR lowering for the above, and a tree-walking interpreter
* [ ] `print`, which is the first `std` API and the first effect — so the
      pre-alpha standard library is one function behind one capability, not a
      directory tree
* [ ] `nudo run`, and the runtime diagnostics that a real program needs
      (`NDO6xxx`)
* [ ] Runtime startup benchmark

**Exit criteria:** the program above prints `42`, and the examples in
`examples/00-` through `examples/02-` run and produce their documented output.
The later examples are design previews and are excluded until the syntax they use
is accepted.

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
* [ ] **The 30-second demonstration**, end to end and runnable: `ask` produces a
      `Generated<T>` → using it where `Verified<T>` is required fails to compile
      → `verify` produces a `Verified<T>` → a tool needs `Network` → the compiler
      refuses the call without a grant → with the grant, the run leaves a trace
      and provenance. One program that runs demonstrates this better than twenty
      pages that describe it.

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
