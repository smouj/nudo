# 03.4 — Diagnostics as product design

> **Status:** Implemented / Planned  
> **Summary:** Compiler errors explain the rule, the source location and the chain that introduced the requirement.

Diagnostics are part of the language experience, not a debug dump. The lexical
and syntax families (`NDO1xxx`) are implemented: a file is rejected with a stable
code, an exact span and an expected/found pair. The type, effect and capability
families are planned, and the chain they will have to show is specified in
[`spec/errors.md`](../../spec/errors.md).

A high-quality capability error should identify both the call site and the reason:

```text
error[NDO3xxx]: missing capability `Network`

  ┌─ src/main.nudo:14:5
  │
14│     fetch(url)
  │     ^^^^^^^^^^ requires Network
  │
  └─ current task was not granted Network
```

For trust types, errors should explain that `Generated<T>` and `Verified<T>` are
intentionally distinct, and point to the explicit verification boundary rather
than recommending an unsafe cast.
