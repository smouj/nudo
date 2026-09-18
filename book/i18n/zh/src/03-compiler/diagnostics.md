# 03.4 — 作为产品设计的诊断

> **状态（Status）：** 已实现 / 计划中（IMPLEMENTED / PLANNED）  
> **概述（Summary）：** 编译器错误解释规则、源码位置，以及引入该要求的链条。

诊断（diagnostic）是语言体验的一部分，而不是调试转储。词法与语法两个族（`NDO1xxx`）
已经实现：文件会以一个稳定的码、一个精确的跨度和一对“预期/实际”被拒绝。
类型、效应（effect）与能力（capability）三个族是计划中的，它们届时必须展示的链条
已在 `spec/errors.md` 中规范定义。

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
