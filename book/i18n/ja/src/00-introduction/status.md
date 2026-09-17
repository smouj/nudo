# 00.3 — 現在のプロジェクト状況

> **ステータス:** 実装済み (Implemented) / 計画中 (Planned)  
> **概要:** このリポジトリは、実装済みのコンパイラ段階と、設計された将来の段階との間の隔たりについて、意図的に正直であり続けます。

## 実装済みの基盤

現在動作する経路は、ソースの読み込み、スパン、字句解析、トークン、診断、そして字句レベルの `nudo check` の挙動を中心としています。このプロジェクトには、リポジトリ、CI、セキュリティ、ガバナンス、適合性テスト、ドキュメントの基盤も存在します。

## 計画中のコンパイラ経路

```text
Source
  ↓
Lexer             implemented
  ↓
Lossless syntax   planned
  ↓
Parser / AST      planned
  ↓
HIR / typecheck   planned
  ↓
Effect checking   planned
  ↓
MIR
  ↓
Interpreter / WASM
```

## なぜステータスラベルが重要か

pre-alpha の言語についての本は、整った例が本物に見えるため、容易に誤解を生むものになり得ます。そのため、すべての章はステータスを持ち、将来の構文の例は、適切な箇所で提案中 (proposed) として記述されます。
