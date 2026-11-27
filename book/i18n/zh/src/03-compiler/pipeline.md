# 03.1 — 编译器流水线

> **状态（Status）：** 已实现 / 计划中（IMPLEMENTED / PLANNED）
>
> **概述（Summary）：** NUDO 前端被划分为显式阶段；HIR 和名称解析已实现，后续检查与执行阶段仍在推进中。

{{#include ../diagrams/pipeline.svg}}

*强调色标记当前 CLI 使用的前端工作。类型检查只有部分实现，并单独通过测试；
效果检查、MIR 和执行仍在计划中。*

~~~text
SourceFile → Lexer → Lossless syntax tree → AST → HIR / name resolution
                                                        ↓
                                     partial type checker (not in nudo check)
                                                        ↓
                                    effects → MIR → interpreter / backend
~~~

CLI 命令 <code>nudo check</code> 会进行源文件加载、词法分析、解析以及 HIR 降低/名称解析。
它可以导出记号、语法树和名称解析结果。它不会调用 <code>nudo-typeck</code>，
也不会执行源码。

<code>nudo-hir</code> crate 将带类型的 AST 降低为标识符和已解析定义。
<code>nudo-typeck</code> crate 消费 HIR，并实现一部分已接受的类型规则。
单元测试和 typeck 一致性语料会直接测试它；其余规则必须先确定规范语义，才能实现。

计划中的效果检查器、MIR、解释器和 WebAssembly 后端目前都不是可用能力。
图中的方框表示预期架构和路线图，不代表程序今天已经可以通过这些阶段。

显式的阶段边界让每次变换都可测试，并使未来工具可以共享表示，而不必使用各自的假设重复解析源码。
在 pre-alpha 阶段，随着实现积累证据，crate 边界仍可能调整。
