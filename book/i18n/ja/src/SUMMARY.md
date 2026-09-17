# 目次

[はじめに](index.md)

# 00 — はじめに
- [NUDO とは何か](00-introduction/what-is-nudo.md)
- [NUDO が存在する理由](00-introduction/why-exists.md)
- [現在のプロジェクト状況](00-introduction/status.md)

# 01 — 哲学
- [決定論的ソフトウェアを第一に](01-philosophy/deterministic-first.md)
- [明示的なトラスト](01-philosophy/explicit-trust.md)
- [ローカルファーストとプロバイダー非依存](01-philosophy/local-first.md)

# 02 — 言語設計
- [文法と構文の規律](02-language-design/grammar.md)
- [式と優先順位](02-language-design/expressions.md)
- [モジュールとプログラムの境界](02-language-design/modules.md)

# 03 — コンパイラ
- [コンパイラパイプライン](03-compiler/pipeline.md)
- [ソースモデルと字句解析器](03-compiler/lexer.md)
- [パーサー、無損失構文、AST](03-compiler/parser.md)
- [プロダクト設計としての診断](03-compiler/diagnostics.md)

# 04 — 型システム
- [型システムの基礎](04-type-system/fundamentals.md)
- [`Generated<T>` と `Verified<T>`](04-type-system/generated-verified.md)

# 05 — エージェントと AI
- [境界付けられた実行者としてのエージェント](05-agents-and-ai/agents.md)
- [タスク、ツール、モデル](05-agents-and-ai/tasks-tools-models.md)

# 06 — セキュリティ
- [ケイパビリティ、エフェクト、権威](06-security/capabilities-effects.md)
- [脅威モデル](06-security/threat-model.md)

# 07 — ランタイム
- [トレースと来歴](07-runtime/trace-provenance.md)
- [予算と承認](07-runtime/budgets-approvals.md)

# 08 — ツール群
- [ツールチェーンと CLI](08-tooling/cli.md)

# 09 — 相互運用性
- [MCP、A2A、WebAssembly](09-interoperability/mcp-a2a-wasm.md)

# 10 — 内部構造
- [リポジトリのアーキテクチャ](10-internals/repository-architecture.md)

# 11 — 設計の根拠
- [なぜ Rust なのか、なぜ独自のコンパイラなのか](11-rationale/why-rust-own-compiler.md)
- [なぜトラストが型の話に属するのか](11-rationale/why-trust-types.md)

# 12 — 却下された代替案
- [意図的に却下された代替案](12-alternatives/rejected.md)

# 13 — 例
- [通常の決定論的プログラム](13-examples/ordinary-program.md)
- [モデル出力と検証のフロー](13-examples/trust-flow.md)

# 14 — コントリビュート
- [仕様、NEP、ADR](14-contributing/spec-nep-adr.md)

# 15 — 歴史
- [マイルストーンとエビデンス](15-history/milestones.md)
