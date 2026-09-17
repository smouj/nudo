# Effects

**State: proposed.** No effect checking exists (milestone M5).

## Why effects, and not a second capability system

Effects and capabilities answer the same question — "what is this code allowed
to do?" — asked at two times. Capabilities are about what a program *holds*;
effects are about what a piece of code *does*. A language with both, defined
separately, has two mechanisms that can disagree, and that disagreement is a
security bug.

So NUDO has one mechanism, described in one place:
[`trust/capabilities.md`](trust/capabilities.md) defines what exists, and this
chapter defines how it is tracked statically.

## Declaring an effect

```nudo
fn fetch(url: Text) -> Text with Network {
    // …
}
```

`with Network` says: calling this function requires the `Network` capability in
force at the call site. A function with no `with` clause performs no external
effect.

## Checking

* A call to an effectful function is an error unless the caller has the required
  capability.
* Effects propagate: a function that calls `fetch` needs `Network` too, and must
  say so. There is no silent propagation, because a caller that cannot see the
  effect cannot reason about the risk.
* The diagnostic names the effect and the call path that introduced it — not
  just the line, because the interesting information is *how* the effect got
  here.

## Effects are not exceptions

An effect is not an error channel. A failed tool call is a value
(`T?` / `Result`), not an effect. Effects describe what code touches, not what
went wrong.

## What is undecided

* The exact syntax for declaring several effects, and for handlers.
* Whether a caller can required-evaluate an effect to a value — that is, whether
  NUDO has an effect handler mechanism, and if so what it looks like.
* How effects interact with generic bounds ([`generics.md`](generics.md)), so
  that the two do not become two ways of saying the same thing.
* How the type system represents capabilities that are only known at runtime,
  such as an agent's tool set supplied by a host.

## What will not change

* **Deny by default.** An effect is permitted only where it was granted.
* **Explicit at the boundary.** Crossing from code that cannot perform a
  capability to code that can is visible in the source.
* **Not bypassable by convenience.** There is no "assume this is fine" form.
