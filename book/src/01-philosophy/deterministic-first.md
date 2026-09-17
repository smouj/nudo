# 01.1 — Deterministic software first

> **Status:** Specified  
> **Summary:** Ordinary deterministic code must remain the easy, predictable case even though NUDO is designed for AI-era systems.

A language for agents that makes simple software awkward is solving the wrong
problem. NUDO therefore treats arithmetic, control flow, functions, data and
modules as the foundation. Agentic constructs sit on top of that foundation.

## Determinism around probability

The intended shape is:

```text
validated deterministic input
        ↓
probabilistic operation
        ↓
Generated<T>
        ↓
explicit validation / policy / approval
        ↓
deterministic continuation
```

The model is allowed to be uncertain. The program must not pretend that this
uncertainty disappeared merely because a response parsed successfully.
