# HIR design (M3.1)

**Status: implemented.** `nudo-hir` exists, is covered by unit tests and by
`tests/conformance/resolve`, and `nudo check` runs it. This document records why
it has the shape it has, and — more usefully — what it deliberately leaves out.

Read [`../../ROADMAP.md`](../../ROADMAP.md) (M3.1) and
[`parser-design.md`](parser-design.md) first; this document assumes the tree and
the parser that feed it.

## Why a separate layer

The obvious shortcut is to resolve names *in the AST*: add a field to the
`PathExpression` node, walk the tree twice, done. It is a shortcut because it
makes the AST two things at once — a description of the source, which tools and
errors need, and a resolved program, which the type checker needs — and the two
disagree about what to keep.

HIR is the second thing, alone:

| | AST (`nudo-ast`) | HIR (`nudo-hir`) |
| --- | --- | --- |
| Parentheses | a node | **gone** |
| Trivia | gone already | gone |
| A name | a string in a place | **a `DefId`, or a marked unresolved** |
| Error nodes | preserved | skipped, because they carry no meaning |
| Purpose | tools, formatter, editor | every later stage |

The parentheses line is the clearest test of whether a layer is real: `(a)` and
`a` are different ASTs and the *same* HIR, because they mean the same thing. A
lowering that keeps a `Parenthesized` node is a copy of the AST with different
struct names.

## Shape

```text
Hir
├── defs:       Vec<Def>        every definition, with the scope it lives in
├── exprs:      Vec<Expr>       the expression arena
├── stmts:      Vec<Stmt>       let-bindings and expression statements
├── scopes:     Vec<Scope>      a tree, by parent
├── references: Vec<Reference>  every name use and what it resolved to
└── root:       ScopeId
```

* **Arenas, not boxes.** A `DefId` is an index, stable for the compilation, so
  anything that wants to talk about a definition — a diagnostic now, a type and
  a provenance record later — says `DefId` instead of repeating a name and
  hoping it means the same thing.
* **Spans are preserved everywhere.** Every `Def`, `Expr` and `Stmt` carries the
  span it came from, and every `Reference` carries the span of the name. A
  diagnostic written at M3 is still renderable at M8.
* **A definition carries its structure.** A `struct`'s fields and an `enum`'s
  variants are recorded with their types, lowered in the declaration's own scope
  so that a declaration's type parameter is visible inside its own fields. They
  are records, not definitions: a field is reached through a value
  (`article.title`), never on its own, so it belongs to its owner rather than to
  a scope.
* **A `match` arm carries its pattern.** The head as written, the definitions
  the pattern introduces, and its body — because whether that head is a variant of
  the scrutinee's enum or a new binding is a question only the scrutinee's *type*
  can answer, and the type checker is the layer that has it.
* **Names are kept for diagnostics only.** `ExprKind::Path` holds both the name
  as written and its `Resolution`, because an error message needs the text and a
  later stage needs the target. Keeping the text is not a failure to resolve.
* **An unresolved name is a state, not an absence.** `Resolution::Unresolved`
  keeps the node in the HIR, so one mistake does not become a cascade and a
  consumer can still ask about the rest of the file.

## The rules it implements

| Rule | Why it is the rule |
| ---- | ------------------ |
| **Two namespaces** (`type`, `value`) | A struct and a binding may share a name; nothing is both a type and a value. Using one where the other belongs is `NDO2003`, which is a better message than "not found". |
| **Items are hoisted** | A function may call one declared below it. That is why lowering has two passes: declare everything, then resolve bodies. |
| **Shadowing in a nested scope is allowed; a duplicate in one scope is `NDO2002`** | The scope is what makes the difference, not the name. |
| **An initialiser is lowered before its own name is in scope** | So `let x = x;` names the *outer* `x`, which is what a reader expects. Getting this backwards is a classic subtle bug. |
| **A declaration's parameters belong to the declaration** | Not to the file. They are declared in the declaration's own scope, with their annotations lowered where the declaration's type parameters are already visible. |
| **Type parameters are visible inside their declaration only** | `fn identity<T>(value: T) -> T` sees `T`; `fn other()` does not. |

## What it deliberately does not do

* **No types.** `Int` resolves to a built-in definition and is not checked;
  nothing is inferred; arity is not checked. That is M3.2. Nine built-in names
  (`Int`, `Float`, `Bool`, `Text`, `Unit`, `Result`, `Generated`, `Verified`,
  `Fn`) are predeclared in the file scope so a program can name them without a
  declaration, and a program cannot redefine them by accident.
* **No pattern resolution.** A pattern head — `Pending` in `match x { Pending
  => … }` — cannot be resolved without the scrutinee's type: a bare name there is
  either a variant or a new binding, and the spelling does not say which. Guessing
  from capitalisation would be inventing a language rule, so the head is left for
  M3.2 and sub-patterns become bindings. The head is now *recorded* rather than
  discarded — as written, with the bindings beside it — so that the checker can
  decide it with the scrutinee's type in hand. A case in
  `tests/conformance/resolve` pins the part that is still unresolved.
* **No modules.** A path written with `::` is reported unresolved with a note,
  because `::` reaches into a module and modules are M10.
* **No diagnostics beyond `NDO2001`–`NDO2003`.** The `2xxx` family grows with the
  type checker.

## Where resolution integrates

`nudo check` parses, and then resolves **only if the parser reported no error**.
A file that is not a program has no names to resolve, and running resolution on
one produces a cascade of messages about a tree the parser was already unhappy
with. That rule is what keeps `NDO1001` and `NDO2001` from arriving together for
one mistake.

The resolution result is dumped by `--dump-resolutions` in the stable
`nudo-hir v1` format, which the `resolve` conformance corpus compares:

```text
# nudo-hir v1
def 0009 function `main` 1:4
def 0010 binding `value` 2:9
ref 3:5 value `value` -> def 0010 binding `value`
```

A path that resolved to nothing says `-> unresolved`, so the case shows the
mistake rather than hiding it.

## Alternatives considered

| Alternative | Why not |
| --------- | ------- |
| Resolve in the AST | Makes one structure serve two purposes with conflicting needs, and leaves every later stage re-deriving what a name meant |
| A tree of boxes (`Box<Expr>`) instead of arenas | Simple to write, painful to extend: adding a field to `Expr` invalidates every pattern match, and two stages cannot hold a reference to the same expression |
| Store types on the defs now | Types are M3.2, and a placeholder type here would be a second source of truth about what a name means |
| Drop unresolved names from the HIR | Turns one mistake into a cascade, and takes away the thing a diagnostic needs to point at |
| Intern names and store only `Symbol` | Removes the text a diagnostic must print, and buys memory the pre-alpha does not need yet |

## Where this document goes next

M3.2 adds the type layer: a type arena, `TypeId`, instantiation of generic
declarations, `Result<T, E>` as an intrinsic type, exhaustiveness of `match`, and
the arity and namespace checks that need types. The shape above is designed for
that — a type arena beside the expression arena, and `DefId` as the thing types
attach to — and if it is wrong, this document is the place to record why.
