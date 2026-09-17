# 03.4 — Diagnostics as product design

> **Status:** Specified / Planned  
> **Summary:** Compiler errors should explain the rule, the source location and the chain that introduced the requirement.

Diagnostics are part of the language experience, not a debug dump.

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
