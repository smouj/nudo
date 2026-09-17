# `Generated<T>`

**State: proposed.** The type exists in the design, not in the compiler.

## What it means

`Generated<T>` is a value of type `T` that a model produced. It means:

* the value exists, and can be read, displayed, stored and passed around;
* it was produced by a model, at a known time, under known parameters;
* it has **not** been checked against anything;
* it is not interchangeable with `T`, and not interchangeable with
  `Verified<T>`.

## Why a distinct type

Because the alternative is a convention: "remember that this string came from
the model". Conventions are not enforced, are not visible in a signature, and
are not carried across a function boundary. A reviewer reading

```nudo
fn publish(article: Article) -> Unit
```

cannot tell whether `article` came from a model or from a parser.

With the type, they can:

```nudo
fn publish(article: Verified<Article>) -> Unit
```

The signature now states the precondition, and the compiler enforces it.

## Rules

1. **No implicit conversion** to `T` or to `Verified<T>`. Every crossing is
   explicit in the source.
2. **Provenance is attached.** A `Generated<T>` records which model, which
   version, which parameters and which inputs produced it. See
   [`provenance.md`](provenance.md).
3. **It can be used safely without verification.** Displaying it to a human,
   logging it, comparing it, storing it: all fine. What it cannot do is satisfy
   a context that requires trust.
4. **Verification is a step, not a cast.** See [`verified.md`](verified.md).
5. **Failure is representable.** A model call that fails does not produce a
   `Generated<T>` with wrong content; it fails, and the caller handles it.

## What it deliberately cannot express

`Generated<T>` does not say *how* confident anyone is, or *how good* the value
is. Confidence scores are claims by the thing being judged, and treating them as
a trust signal is how a language ends up trusting a model's opinion of itself.

If a program wants graded trust, that is a domain type the program defines, and
it is checked by a verifier that a reviewer can read.

## Open questions

* Whether `Generated<T>` and `Verified<T>` share a common trait, and what that
  would be good for.
* How provenance is represented when a value is copied, serialised or sent to
  another agent.
* Whether a `Generated<T>` can be edited by a person and what that does to its
  provenance.
