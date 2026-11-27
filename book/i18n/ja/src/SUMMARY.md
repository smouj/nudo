# Summary

[はじめに](index.md)

# 00 — はじめに
- [NUDO とは何か](00-introduction/what-is-nudo.md)
- [NUDO が存在する理由](00-introduction/why-exists.md)
- [現在のプロジェクト状況](00-introduction/status.md)

# 01 — 哲学
- [決定論的なソフトウェアを第一に](01-philosophy/deterministic-first.md)
- [明示的な信頼](01-philosophy/explicit-trust.md)
- [Local-first と provider agnostic](01-philosophy/local-first.md)

# 02 — 言語設計
- [文法と構文規律](02-language-design/grammar.md)
- [式と優先順位](02-language-design/expressions.md)
- [module とプログラムの境界](02-language-design/modules.md)

# 03 — コンパイラー
- [コンパイラーパイプライン](03-compiler/pipeline.md)
- [ソースモデルと lexer](03-compiler/lexer.md)
- [Parser、損失のない構文、AST](03-compiler/parser.md)
- [製品設計としての診断](03-compiler/diagnostics.md)

# 04 — 型システム
- [型システムの基礎](04-type-system/fundamentals.md)
- [<code>Generated&lt;T&gt;</code> と <code>Verified&lt;T&gt;</code>](04-type-system/generated-verified.md)

# 05 — エージェントと AI
- [制限付き実行者としてのエージェント](05-agents-and-ai/agents.md)
- [タスク、ツール、モデル](05-agents-and-ai/tasks-tools-models.md)

# 06 — セキュリティ
- [Capability、エフェクト、権限](06-security/capabilities-effects.md)
- [脅威モデル](06-security/threat-model.md)

# 07 — Runtime
- [trace と provenance](07-runtime/trace-provenance.md)
- [予算と承認](07-runtime/budgets-approvals.md)

# 08 — ツール
- [Toolchain と CLI](08-tooling/cli.md)
- [フロントエンドのベンチマーク](08-tooling/benchmarks.md)

# 09 — 相互運用性
- [MCP、A2A、WebAssembly](09-interoperability/mcp-a2a-wasm.md)

# 10 — 内部構造
- [リポジトリアーキテクチャ](10-internals/repository-architecture.md)

# 11 — 設計の根拠
- [Rust を選ぶ理由と独自コンパイラーの理由](11-rationale/why-rust-own-compiler.md)
- [信頼が型の説明に含まれる理由](11-rationale/why-trust-types.md)

# 12 — 却下した選択肢
- [意図的に却下した選択肢](12-alternatives/rejected.md)

# 13 — 例
- [一般的な決定論的プログラム](13-examples/ordinary-program.md)
- [モデル出力と検証の流れ](13-examples/trust-flow.md)

# 14 — コントリビューション
- [仕様、NEP、ADR](14-contributing/spec-nep-adr.md)

# 15 — 歴史
- [マイルストーンと証拠](15-history/milestones.md)
