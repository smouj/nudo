# 00.3 — 当前项目状态

> **状态（Status）：** 已实现 / 计划中（IMPLEMENTED / PLANNED）  
> **概述（Summary）：** 本仓库有意如实说明“已实现的编译器阶段”与“已设计的未来阶段”之间的差距。

## 已实现的基础

当前可用的路径集中在源加载、跨度（span）、词法分析（lexing）、记号（token）、
诊断（diagnostic）、无损语法树（lossless syntax tree）、解析器（parser）、带类型的 AST，
以及覆盖这一切的 `nudo check`。项目还具备仓库、CI、安全、治理、
一致性（conformance）和文档基础设施。

## 计划中的编译器路径

```text
Source
  ↓
Lexer             implemented
  ↓
Lossless syntax   implemented
  ↓
Parser / AST      implemented
  ↓
HIR / typecheck   planned
  ↓
Effect checking   planned
  ↓
MIR
  ↓
Interpreter / WASM
```

“Implemented”（已实现）意味着某个阶段今天已经具备行为和测试。一次干净的 `nudo check`
运行意味着文件通过了词法分析和语法分析；它并不意味着程序是正确的，也不意味着它能运行。

## 状态标签为什么重要

一本关于 pre-alpha 语言的书很容易变得具有误导性，因为精致的示例看起来像是真的。
因此每一章都带有状态，未来语法的示例在适当之处被描述为提案（proposed）。
