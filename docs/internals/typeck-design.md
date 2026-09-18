# Type checker design (M3.2)

**Status: first slice implemented.** `nudo-typeck` exists and is exercised by
unit tests and by `tests/conformance/typeck`. This document records what it
checks, what it deliberately does not, and the one architectural rule that makes
"is HIR enough?" a question with a test.

Read [`hir-design.md`](hir-design.md) first; this layer consumes what that one
produces.

## The rule

> **The checker consumes HIR, and only HIR.** It never looks at a syntax tree, an
> AST, or a byte of source.

That is not tidiness. It is the difference between a semantic layer that is
complete and one that is *nearly* complete, and the failure mode is always the
same: a rule that needs to know where a parenthesis was, or how a name was
spelled, turns into a second reader of the same file. If a typing rule here needs
something HIR does not carry, the fix goes into [`nudo-hir`][hir], where every
later stage benefits from it.

Spans come from HIR, so a diagnostic points exactly where the resolver would have
pointed.

[hir]: hir-design.md

## Shape

```text
TypeckResult
├── definition_types: Vec<Option<Type>>   indexed by DefId
├── expression_types: Vec<Option<Type>>   indexed by ExprId
└── diagnostics
```

Two dense tables and the diagnostics. Definitions are typed in a **signature
pass** first — HIR hoists declarations, so a name's type must be known whatever
the order in the file — and then every body and value is checked.

`None` in a table means *never typed*, which is not the same as typed as
[`Type::Error`]: the one is an absence, the other is a value that already had a
diagnostic and must not produce another.

## The type representation

```text
Type
├── Int · Float · Bool · Text · Unit      the primitives
├── Named { name, definition, arguments } a resolved name, with arguments
├── Function { parameters, result }       Fn(A, B) -> C
└── Error                                 already reported; compatible with everything
```

Three decisions are visible in that shape, and each is there so that later slices
fill in cases instead of redesigning the centre:

* **`Error` is compatible with everything.** One mistake is one diagnostic
  instead of a cascade of follow-up complaints about the same value.
* **`Named` carries its definition, and compares by identity.** Two named types
  are the same type when they mean the same definition. Structural comparison —
  fields, variants — is a later slice; identity is already the right answer for
  everything that exists today, and it is the answer that cannot be wrong.
* **There is no type variable.** Inference is bounded and local (NEP-0013), and
  until generic instantiation exists there is nothing to solve. A type variable
  here would be a placeholder pretending to be a decision.

## What the first slice checks

| Rule | Example |
| ---- | ------- |
| Literal typing | `1` is `Int`, `1.5` is `Float`, `true` is `Bool`, `"nudo"` is `Text` |
| Annotations | `let x: Int = 1;` types both sides and compares them |
| Blocks | a block's type is its final expression's, or `Unit` when it has none |
| Names | a path takes the type of the definition it resolved to |
| Functions | a body is compared with the written return type; with none written, the body's type *is* the result type |
| Calls | arity (`NDO2005`), each argument against its parameter (`NDO2004`, reported at the argument), and calling something that is not a function (`NDO2006`) |
| One diagnostic shape | `expected` and `found` as structure, never as prose to re-parse |

The diagnostic points at the **value**, not at the name it was bound to: the
value is what breaks the promise, and therefore the place a reader has to change.

```text
error[NDO2004]: type mismatch
  --> src/main.nudo:1:18
note: expected: `Int`
note: found: `Bool`
```

## What it deliberately does not check

Every omission is a named next step, and each one is **silent** rather than
wrong: those expressions type as `Error`, and `Error` reports nothing.

| Not yet | Arrives with |
| ------- | ------------ |
| Generic instantiation and bounded inference (NEP-0013) — a call to a generic function is typed as unknown and says nothing, because comparing an argument against `T` would invent a rule | next slice |
| Struct and enum structure: fields, variants | after generics |
| `Result<T, E>` and exhaustiveness of `match` (NEP-0014) | after enums |
| Effects | M5 |
| `Generated<T>` / `Verified<T>` as distinct types with no conversion (NEP-0002) | M3.3 |

Two rules are worth stating as *absences*, because they are decisions rather than
gaps:

* **No implicit numeric conversion.** `let x: Float = 1;` is `NDO2004`. NEP-0007
  says the conversion is written down, and this is what "written down" means.
* **No special rule for the trust types** (NEP-0014). When they arrive they will
  be ordinary `Named` types with ordinary rules, and their conformance cases must
  not assert an error that is about anything else.

## Why the checker is not wired into `nudo check` yet

`check` currently lexes, parses and resolves, and the checker runs beside it in
its own tests and corpus but not inside it. Calls are checked now, so the
question is live; what still argues for waiting is that structs, enums and the
trust types type as unknown, and the examples in `examples/` are *about* those.
Wiring it in today would add a `TYPE` stage that reports on primitives and calls
and stays silent about the interesting half — so a clean run would mean less than
a reader would assume. It is wired when the structural rules land, and the
terminal layer gains its `TYPE` row at that moment, rendered from work actually
done.

Until then the checker is exercised by its own tests and by the `typeck`
conformance corpus, which is a corpus like any other: reviewed as a diff.

## Alternatives considered

| Alternative | Why not |
| --------- | ------- |
| Check while parsing, in one pass | Makes every later stage re-derive what a name meant, and puts type errors in a layer that cannot report them well |
| Type the AST | The AST is for tools and errors; types belong on definitions and expressions, and `DefId` is what types attach to |
| A type variable per generic parameter, solved by unification | NEP-0013 forbids unbounded inference; variables without a solver are decoration |
| Report a mismatch at the binding's name | The name is not what is wrong — the value is |
| Make `Error` compatible with nothing | Turns one mistake into a cascade, which is worse for the person fixing it |
