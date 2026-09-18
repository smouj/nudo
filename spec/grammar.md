# Grammar

**State: the lexical and syntactic layers are both implemented.**

| Layer | State | Checked by |
| ----- | ----- | ---------- |
| Lexical grammar | **IMPLEMENTED** (M1) | `compiler/nudo-lexer`, `tests/conformance/lexer` |
| Syntactic grammar | **IMPLEMENTED** (M2) | `compiler/nudo-parser`, `tests/conformance/parser` |

"Implemented" means the reference parser accepts exactly the productions below
and rejects the rest, with `NDO1001` and the expected/found pair. It does not
mean the language is finished: the productions marked provisional can still
change, and each change needs a NEP. Nothing is type-checked (M3) and nothing
runs (M4).

## Where the grammar lives

* [`../grammar/nudo.ebnf`](../grammar/nudo.ebnf) — the normative EBNF, split into
  a lexical and a syntactic section.
* [`../grammar/syntax-reference.md`](../grammar/syntax-reference.md) — the same
  grammar in a form a person can read, with examples.

Both are specification artefacts. A change that alters what a program means needs
a [NEP](../neps/README.md); a change that only clarifies is a normal pull
request.

## Two rules the grammar must satisfy

### 1. No left recursion

`nudo-parser` is a recursive-descent parser
([`../docs/internals/parser-design.md`](../docs/internals/parser-design.md)).
Such a parser cannot handle a production that derives a form starting with
itself, directly or indirectly:

```ebnf
(* This is not parseable by recursive descent, and must never appear. *)
call-expression = expression, argument-list ;
```

The first revision of this grammar had four such productions, which is why the
rule is now stated here, in the EBNF header, and enforced by
`scripts/check-grammar.py` — a grammar that drifts back into left recursion fails
CI rather than being discovered by whoever writes the parser.

Left-recursive repetition is expressed with `{ … }` instead:

```ebnf
postfix-expression = primary-expression, { postfix-suffix } ;
```

### 2. No unstated precedence

Operator precedence is not described in prose anywhere, because prose and
productions drift apart silently. The grammar declares a precedence chain:

```text
expression > logical-or > logical-and > equality > comparison
           > additive > multiplicative > unary-expression
           > postfix-expression > primary-expression
```

Each level is *defined in terms of the next*, so the grammar and the precedence
cannot disagree. The identical marker appears in
[`syntax-reference.md`](../grammar/syntax-reference.md) and
[`expressions.md`](expressions.md), and the checker fails if the three copies
differ or if a production stops implementing the declared chain.

## How to read the EBNF

```ebnf
(* a comment *)
rule    = expression ;
```

| Notation | Meaning |
| -------- | ------- |
| `rule = … ;` | A production |
| `a , b` | `a` followed by `b` |
| `a \| b` | `a` or `b` |
| `[ a ]` | `a` is optional |
| `{ a }` | zero or more repetitions of `a` |
| `"text"` | A literal token |
| `? … ?` | A special sequence: prose, or an input this grammar does not spell out |
| `comment` | Text between the comment delimiters |

The grammar describes accepted syntax, not meaning. [`types.md`](types.md)
defines what the accepted forms mean, and a grammar that accepts something the
type system rejects is normal.

## What is decided, and what is not

Decided, and implemented by the productions:

| Area | Decision |
| ---- | -------- |
| Expression precedence and associativity | The chain above; comparison does not chain |
| Postfix forms | Calls, field access and indexing chain left |
| Blocks | Statements terminate with `;`; the optional final expression is the value |
| Types | At most one `?` per type level; `(T?)?` via parentheses |
| `match` | Unambiguous without lookahead: a block is never a postfix suffix |
| The agentic expressions | `ask`, `verify` and `delegate` are primaries, not operators |

Not decided. Each item is provisional in the EBNF, and each needs a NEP before
its syntax is frozen:

| Production | Open question |
| ---------- | ------------- |
| `agent-decl` | Whether `role:`, `tools:` and `allow:` stay clauses or become fields |
| `task-decl` | Whether `verify:` is a clause or a value |
| `pattern` | Whether patterns grow beyond enum variants and bindings |

Decided since, each by a NEP:

| Production | Decision |
| ---------- | -------- |
| `ask-expression` | A primary expression that yields `Generated<T>`; `ask` stays contextual ([NEP-0002](../neps/0002-generated-verified.md)) |
| `verify-expression` | Its operand is a **named value** (a path, optionally called), and it yields `Result<Verified<T>, VerificationError>` ([NEP-0002](../neps/0002-generated-verified.md)) |
| Early exit | There is no `return`; a block's value is its final expression, and `if` and `match` are expressions ([NEP-0012](../neps/0012-early-exit.md)) |
| `generic-type` | Angle brackets, invariant arguments ([NEP-0006](../neps/0006-generic-syntax.md)) |
| `generic-parameter-list` | Declarations declare their parameters ([NEP-0010](../neps/0010-declaration-site-generics.md)) |
| `binding` | Immutable; no assignment, ever, in this edition ([NEP-0009](../neps/0009-mutability.md)) |
| Tool names and lists | A path uses `::`; lists are comma-separated ([NEP-0011](../neps/0011-path-and-list-spelling.md)) |

## What the grammar may not do

* It may not accept a program the specification cannot describe. An unexplained
  production is an unfinished design, not a flexible one.
* It may not collide with the reserved-word list in
  [`lexical-structure.md`](lexical-structure.md). Adding a keyword is a breaking
  change; [NEP-0005](../neps/0005-keyword-policy.md) proposes how future
  reserved words are chosen.
* It may not be the only place a rule is written down. If the grammar is the only
  description of a feature, the feature is underspecified.

## Relationship to the parser

The parser exists, and it agrees with this grammar:

* **Disagreement is a bug in the parser**, never in the grammar, unless a NEP
  changed the grammar in the same pull request.
* A production the parser accepts but the grammar does not is a silent language
  extension, which is exactly what [`../AGENTS.md`](../AGENTS.md) forbids.
* A production the grammar describes but the parser rejects is a silent
  restriction, and is reported the same way. There is none open right now: the
  two the grammar gained with [NEP-0010](../neps/0010-declaration-site-generics.md)
  (`generic-parameter-list`) and [NEP-0002](../neps/0002-generated-verified.md)
  (the restricted `verify-expression` operand) are implemented, and
  `tests/conformance/parser/0014-declaration-generics` and `0016-verify-parenthesised`
  pin them.
* Every accepted production has a case under
  [`../tests/conformance/parser`](../tests/conformance/parser), and every
  rejected one a diagnostic case with its `NDO` code.

Two places where the parser follows the grammar against the run-able examples are
worth knowing about, because the examples are the thing a reader tries first:
`tool-name` is a path, so a tool is named `web::search` and not `web.search`; and
a list inside a declaration is comma-separated, so `tools: a::b, c::d` and not one
name per line. The examples in [`../examples`](../examples) use the dotted,
one-per-line spelling and are marked as previews for that reason. Whether the
spelling should change is a NEP, not a parser bug.
