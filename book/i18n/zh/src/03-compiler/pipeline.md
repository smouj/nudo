# 03.1 — 编译器流水线

> **状态（Status）：** 已实现 / 规范已定义（IMPLEMENTED / SPECIFIED）  
> **概述（Summary）：** NUDO 被组织为显式的编译器阶段，使每一次变换都有一项可测试的职责。


{{#include ../diagrams/pipeline.svg}}

*流水线，其中强调色标记的是今天已经存在的部分：源加载、词法分析器、无损语法树、
解析器、带类型的 AST 以及诊断。*

```text
SourceFile
   ↓
Lexer → Tokens + lexical diagnostics
   ↓
Lossless Syntax Tree
   ↓
Typed AST
   ↓
HIR + name resolution
   ↓
Type checking
   ↓
Effect checking
   ↓
MIR
   ↓
Interpreter / code generation
```

前四个阶段已经实现，并由一致性用例覆盖：词法分析、无损语法树、解析以及带类型的 AST。
`nudo check` 会运行它们全部，这正是为什么一次干净的运行现在意味着“它能通过词法分析，
也能通过语法分析”。

清晰的阶段边界改善调试，并让格式化器、语言服务器和文档工具共享稳定的表示，
而不是带着各自的假设去重新解析文本。

在 pre-alpha 期间，架构应当保持灵活。占位（placeholder）crate 不是
在实现教会项目正确边界之前就冻结 API 的理由。
