# 02.1 — 文法与语法纪律

> **状态（Status）：** 已实现 / 提案中（IMPLEMENTED / PROPOSED）  
> **概述（Summary）：** 文法是源文本与解析器之间的契约，由解析器实现并由检查器强制执行；临时的智能体语法在被接受之前，必须保持明显是临时的。

## 小语法，强语义

NUDO 的设计目标不是为每个 AI 概念发明一个关键字。语法是昂贵的，
因为每一种新形式都会影响解析、工具、格式化、学习和兼容性。

## 语法分析器的压力

递归下降解析器（recursive-descent parser）在表达式文法对优先级（precedence）表述明确
并避免左递归（left recursion）时工作得最好。一个稳健的表达式层级在概念上应当组织为：

```text
logical-or
  → logical-and
  → equality
  → comparison
  → additive
  → multiplicative
  → unary
  → postfix
  → primary
```

后缀解析（postfix parsing）是调用、字段访问和索引自然组合的位置。

## 上下文关键字与保留字

只有实际上被接受为保留记号（reserved token）的词，才应被词法分析器视为保留字。
诸如 `agent`、`task`、`ask` 和 `verify` 这样的未来语法，在工具依赖它们之前，
需要一个 NEP 级别的决定。
