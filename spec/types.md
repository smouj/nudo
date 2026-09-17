# Types

**State: proposed.** No type checker exists (milestone M3). This chapter records
the decisions that are settled enough to design against, and marks the rest as
open.

## Design constraints

The type system is constrained by the rest of the language, not chosen freely:

1. It must distinguish a value a model produced from a value that has been
   checked, and it must make the conversion explicit.
2. It must carry capability information, so that "this program may reach the
   network" is a property the compiler can see.
3. It must be explainable. A type error that a person cannot trace to a rule in
   this chapter is a bad error.
4. It must not require the model provider to be known at compile time.

## Primitive types

| Type | Meaning |
| ---- | ------- |
| `Int` | A signed integer. Width is deliberately unspecified for now |
| `Float` | An IEEE-754 binary floating-point number |
| `Bool` | `true` or `false` |
| `Text` | A UTF-8 string |
| `Unit` | The type of an expression that produces nothing |

Whether `Int` has a defined width, and whether sized integers exist alongside
it, is an **open question**. It is a real decision with real consequences for
overflow behaviour, and it will not be settled by accident.

## Composite types

| Form | Meaning |
| ---- | ------- |
| `struct` | A product type with named fields |
| `enum` | A sum type with named variants |
| `[T]` | A sequence of `T` |
| `Fn(A, B) -> C` | A function value |
| `Agent`, `Task`, `Tool`, `Model` | Declared items used as types (see [`declarations.md`](declarations.md)) |
| `T?` | The absence of a value, or its presence |

## Trust types

The central decision of the type system:

```text
Generated<T>   a value produced by a model, not checked
Verified<T>    a value that passed an explicit verification step
```

Rules, which are the whole point:

* `Generated<T>` and `Verified<T>` are **different types**. They are not
  compatible, and there is no subtyping between them in either direction.
* There is **no implicit conversion** from `Generated<T>` to `Verified<T>`.
  Producing a `Verified<T>` requires a `verify` expression
  ([`trust/verified.md`](trust/verified.md)).
* There is no implicit conversion from `Generated<T>` to `T` either. Dropping
  the provenance of a model-produced value must be visible in the source,
  because it is the single most important thing a reviewer needs to see.
* `Verified<T>` carries the verifier and the inputs that produced it, as
  provenance ([`trust/provenance.md`](trust/provenance.md)).

Details in [`trust/generated.md`](trust/generated.md) and
[`trust/verified.md`](trust/verified.md).

## Capabilities in the type system

A function that performs an effect declares it:

```nudo
fn fetch(url: Text) -> Text with Network {
    // …
}
```

A call to `fetch` is only permitted in a context that has `Network`
([`effects.md`](effects.md), [`trust/capabilities.md`](trust/capabilities.md)).

Whether capability information is part of a function's *type* or tracked as an
*effect* is settled in favour of effects, because the two are the same question
asked at different times, and one mechanism is better than two.

## Inference

* Local inference: the type of a `let` binding may be omitted when it is
  determined by the initialiser.
* Signatures: function, agent, task and tool declarations **must** annotate
  their types. A public interface whose types are inferred from its body is a
  hidden dependency, and this language is not allowed to have those.
* Conversion between numeric types is always explicit. No implicit widening.
* `Generated<T>` and `Verified<T>` are never inferred into each other. If
  inference would have to guess which one you meant, it is an error and it says
  so.

## Subtyping

There is no subtyping between structs, and no structural typing. If two types
must be interchangeable, that is a named relationship the specification
describes, not an accident of having the same fields.

## Open questions

* Integer width and overflow semantics.
* Whether `Bool` is distinct from an enum with two variants.
* Whether `Text` is a primitive or a type over a sequence of bytes.
* Variance in generic positions ([`generics.md`](generics.md)).
* How the type system represents an agent's capability set when the agent's
  tools are supplied at runtime.
