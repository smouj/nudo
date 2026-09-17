# 03.4 — 作为产品设计的诊断

> **状态（Status）：** 规范已定义 / 计划中（SPECIFIED / PLANNED）  
> **概述（Summary）：** 编译器错误应当解释规则、源码位置，以及引入该要求的链条。

诊断（diagnostic）是语言体验的一部分，而不是调试转储。

一条高质量的能力（capability）错误应当同时指出调用点和原因：

```text
error[NDO3xxx]: missing capability `Network`

  ┌─ src/main.nudo:14:5
  │
14│     fetch(url)
  │     ^^^^^^^^^^ requires Network
  │
  └─ current task was not granted Network
```

对于信任类型（trust type），错误应当解释 `Generated<T>` 与 `Verified<T>` 是
有意区分的，并指向显式的验证边界，而不是建议一次不安全的转换。
