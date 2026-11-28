# Type checker design (M3.2–M3.3)

**Status: first slice, bounded generic inference with NDO2008/NDO2009 recovery,
and the M3.3 trust-type assignment rule implemented.**
`nudo-typeck` is exercised by unit tests and by `tests/conformance/typeck`.
Deterministic robustness cases also check recovered HIR, diagnostic span bounds
and repeatability. This document records what the checker does and the one
architectural rule that makes "is HIR enough?" a question with a test.

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

Named-type arguments and function components live in immutable shared storage.
Cloning a type therefore shares its nested type tree rather than allocating and
copying each generic argument or function parameter at every use site.

Three decisions are visible in that shape, and each is there so that later slices
fill in cases instead of redesigning the centre:

* **`Error` is compatible with everything.** One mistake is one diagnostic
  instead of a cascade of follow-up complaints about the same value.
* **`Named` carries its definition, and compares by identity.** Two named types
  are the same type when they mean the same definition. Structural comparison —
  fields, variants — is a later slice; identity is already the right answer for
  everything that exists today, and it is the answer that cannot be wrong.
* **There is no type variable.** Call-local inference (NEP-0013) substitutes
  declaration parameters directly from argument types. A variable would add
  machinery the bounded rule does not need.

## What the first slice checks

| Rule | Example |
| ---- | ------- |
| Literal typing | Decimal magnitudes through `Int::MAX` are `Int`; `2^63` is accepted only directly under unary `-` for `Int::MIN`; other out-of-range magnitudes use `NDO2010`; `1.5` is `Float`, `true` is `Bool`, `"nudo"` is `Text` |
| Annotations | `let x: Int = 1;` types both sides and compares them |
| Named type arity | Resolved structs, enums, type parameters and intrinsic types require their declared type-argument count; `NDO2017` points at the type use |
| Blocks | a block's type is its final expression's, or `Unit` when it has none |
| Names | a path takes the type of the definition it resolved to |
| Functions | signatures are known before bodies; an omitted result means `Unit`, and the body is checked against that result (`NDO2004` on mismatch) |
| `if` | the condition is `Bool`; an expected result checks both branches, otherwise branch types must match exactly; an `if` without `else` is `Unit` |
| Field access | `article.title` takes the field's type after substituting the receiver's generic arguments; a built-in `Verified<T>` receiver uses the supported view of `T`; `NDO2007` points at a missing field's name |
| Calls | arity (`NDO2005`, reported at the argument list), each argument against its parameter (`NDO2004`, reported at the argument), and calling something that is not a function (`NDO2006`, reported at the callee); a built-in `Verified<Fn(...) -> ...>` can be called through its supported `Fn` view; malformed calls still check matched arguments against their parameters and excess argument expressions for nested errors |
| Function types | `Fn(A, B) -> C` annotations keep parameter and result types as a function type, so function-valued parameters can be called and participate in structural comparison and bounded inference |
| Generic calls | Direct function-item parameters are inferred from arguments in source order, then a known expected result; unresolved parameters use `NDO2008`, signature-absent parameters use `NDO2009`, and dependent cascades are suppressed |
| Operators | Arithmetic requires matching numeric types, `%` is `Int`-only, Boolean operators require `Bool`, equality accepts matching primitive views, and ordering accepts matching `Int` or `Float`; the supported `Verified<T>`-to-`T` view applies; invalid domains report `NDO2018` at the operator |
| Conditionals | Branches use a known expected type or must have exactly matching types; an `if` without `else` requires a `Unit` then-branch; an uncontextualised mismatch reports `NDO2019` |
| Trust assignment | The resolved built-in `Verified<T>` is usable where `T` is expected; `Generated<T>` is not, and the relation does not widen nested generic arguments |
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
wrong: unsupported expression results type as `Error`, and `Error` reports
nothing. Indexing, `match`, and agentic expressions still recursively check
their child expressions, so a supported call or annotation error inside them is
not hidden.

| Not yet | Arrives with |
| ------- | ------------ |
| Sequence and index typing, including out-of-range behavior | a specification decision |
| Arithmetic evaluation | the interpreter; overflow and division traps are specified by NEP-0007 |
| Enum pattern resolution, payload binding, exhaustiveness and arm result types | NEP-0020 (**Draft**); do not implement before acceptance |
| Building a nominal value: **the grammar has no construction syntax at all**, so it needs a NEP before it can be implemented | when the language decides |
| `Result<T, E>` and exhaustive `match` behavior | after the match rules are accepted and implemented |
| Effects | M5 |
| `verify`, provenance tracking, and `Result<T, E>` exhaustiveness for trust values | M3.3 and later slices |

Two rules are worth stating as *absences*, because they are decisions rather than
gaps:

* **No implicit numeric conversion.** `let x: Float = 1;` is `NDO2004`. NEP-0007
  says the conversion is written down, and this is what "written down" means.
* **No `Result`-specific rule for the trust types** (NEP-0014). They are
  ordinary `Named` types; the accepted `Verified<T>`-to-`T` assignment relation
  is checked by resolved built-in identity, and generic arguments stay invariant.

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
