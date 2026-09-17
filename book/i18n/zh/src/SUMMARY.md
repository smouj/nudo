# 目录

[NUDO 技术手册](index.md)

# 00 — 引言
- [什么是 NUDO？](00-introduction/what-is-nudo.md)
- [NUDO 为什么存在](00-introduction/why-exists.md)
- [当前项目状态](00-introduction/status.md)

# 01 — 设计哲学
- [确定性软件优先](01-philosophy/deterministic-first.md)
- [显式信任](01-philosophy/explicit-trust.md)
- [本地优先与供应商无关](01-philosophy/local-first.md)

# 02 — 语言设计
- [文法与语法纪律](02-language-design/grammar.md)
- [表达式与优先级](02-language-design/expressions.md)
- [模块与程序边界](02-language-design/modules.md)

# 03 — 编译器
- [编译器流水线](03-compiler/pipeline.md)
- [源模型与词法分析器](03-compiler/lexer.md)
- [语法分析器、无损语法与 AST](03-compiler/parser.md)
- [作为产品设计的诊断](03-compiler/diagnostics.md)

# 04 — 类型系统
- [类型系统基础](04-type-system/fundamentals.md)
- [`Generated<T>` 与 `Verified<T>`](04-type-system/generated-verified.md)

# 05 — 智能体与 AI
- [作为有界执行器的智能体](05-agents-and-ai/agents.md)
- [任务、工具与模型](05-agents-and-ai/tasks-tools-models.md)

# 06 — 安全
- [能力、效果与授权](06-security/capabilities-effects.md)
- [威胁模型](06-security/threat-model.md)

# 07 — 运行时
- [追踪与来源](07-runtime/trace-provenance.md)
- [预算与审批](07-runtime/budgets-approvals.md)

# 08 — 工具链
- [工具链与 CLI](08-tooling/cli.md)

# 09 — 互操作性
- [MCP、A2A 与 WebAssembly](09-interoperability/mcp-a2a-wasm.md)

# 10 — 内部实现
- [仓库架构](10-internals/repository-architecture.md)

# 11 — 设计理由
- [为什么选择 Rust，为什么自建编译器？](11-rationale/why-rust-own-compiler.md)
- [为什么信任属于类型体系](11-rationale/why-trust-types.md)

# 12 — 被拒绝的替代方案
- [被有意拒绝的替代方案](12-alternatives/rejected.md)

# 13 — 示例
- [普通确定性程序](13-examples/ordinary-program.md)
- [模型输出与验证流程](13-examples/trust-flow.md)

# 14 — 贡献
- [规范、NEP 与 ADR](14-contributing/spec-nep-adr.md)

# 15 — 历史
- [里程碑与证据](15-history/milestones.md)
