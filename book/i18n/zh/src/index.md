# 简介

> **状态（Status）：** 历史 / 持续更新（Historical / Living）  
> **概述（Summary）：** 对 NUDO 是什么、为什么存在，以及它的编译器、信任模型和面向智能体的运行时如何被预期协同工作的结构化说明。

<div class="status-key">
文档（DOCUMENT）：NUDO-BOOK-001 · 版本类型（EDITION）：工程论文（ENGINEERING PAPER） · 修订（REVISION）：R0 · 状态（STATUS）：持续更新（LIVING）
</div>

NUDO 是一个通用编程语言项目，其设计把智能体（agent）、工具（tool）、模型（model）、
权限（permission）、预算（budget）、验证（verification）和来源（provenance）
当作语言与运行时应当能够显式推理的对象。

本书是说明性的。它有意比规范性规范（specification）更宽泛：
它记录动机、权衡、架构、被拒绝的替代方案和实践示例。
它绝不可以把一项提案中的功能悄然提升为已实现的功能。

## 如何阅读本书

如果你想理解**理念**，请从“设计哲学”和“信任”开始。如果你想参与编译器工作，
请按“语言设计 → 编译器 → 类型系统”的顺序阅读。如果你想理解智能体安全，
请阅读“智能体与 AI → 安全 → 运行时”。

## 当前现实

{{#include diagrams/status.svg}}

*工具链今天能做什么，以及哪些只是被写下来的。同一张表也维护在
[`ROADMAP.md`](https://github.com/smouj/nudo/blob/main/ROADMAP.md) 中；若两者不一致，以该文件为准。*


在修订版本 R0，NUDO 处于 pre-alpha（前 alpha）阶段。仓库基础与词法流水线
（lexical pipeline）是成熟的实现层。语法分析器、带类型的 AST、类型检查器、解释器、
效果检查器、智能体运行时、策略系统和 WASM 后端属于设计/路线图工作，
而不是可用于生产的承诺。

## 权威性

规范定义语言。被接受的 NEP 对规范进行修订。一致性测试证明可观察的一致行为。
本书解释这些规则；它不凌驾于它们之上。
