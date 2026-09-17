# Models

**State: proposed.** No model interface exists (milestone M7).

## Principle

NUDO does not privilege any model provider. The language does not know what
OpenAI, Anthropic, Google or a local runtime is; it knows what a **model
capability** is, and a provider is one implementation of it.

Two consequences:

* a program cannot depend on provider-specific behaviour and still be a NUDO
  program, so that behaviour must be expressed explicitly;
* local models are not a degraded mode. A model that runs on the user's machine
  is the same kind of thing as a hosted one, and the specification treats it
  that way.

## Declaration

```nudo
model LocalLlama {
    provider: "local"
    // …
}
```

A model declaration names an implementation and its configuration. The
configuration is provider-specific by nature; the interface a program uses is
not.

## The interface

Provisional, but constrained:

* a call takes a prompt, optional context, and constraints (budget, deadline);
* it returns `Generated<T>` — never `T`, never `Verified<T>`;
* it may fail: unavailable, over budget, refused, cancelled;
* it reports what it spent;
* it produces provenance: which model, which version, which parameters.

## Rules

* **Everything a model returns is `Generated<T>`.** There is no call that
  bypasses the trust types
  ([`../trust/generated.md`](../trust/generated.md)).
* **Model output is data.** It is never interpreted as an instruction by the
  runtime, and never as code.
* **Model selection is not a capability.** Which model is used does not change
  what the program is allowed to do; capabilities are separate and unaffected.
* **A model call is an effect.** It requires a capability, consumes budget, and
  appears in the trace.
* **No prompt is privileged.** Nothing in the language executes a "system
  prompt" specially. A program that wants trust boundaries in text must express
  them in data, and know that text is not a security boundary — which is why
  capabilities are.

## Open questions

* How structured output is requested and validated, given that validation is a
  verification step and therefore part of the trust model.
* Whether a model declaration can be resolved at runtime (a registry) or is
  fixed at compile time.
* How determinism is reported when a provider offers a seed and when it does not.
* Whether streaming responses are part of the language's model interface or
  purely a runtime detail.
