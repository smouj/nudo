# 03.1 — コンパイラパイプライン

> **ステータス:** 実装済み (IMPLEMENTED) / 仕様化済み (SPECIFIED)  
> **概要:** NUDO は明示的なコンパイラ段階として構成され、各変換がテスト可能な責務を持ちます。


{{#include ../diagrams/pipeline.svg}}

*パイプライン。アクセントは現在存在するものを示します。ソースの読み込み、字句解析器、無損失構文木、パーサー、型付き AST、診断。*

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

最初の四つの段階は実装済みで、適合性テストケースに覆われています。字句解析、無損失構文木、解析、型付き AST です。`nudo check` はそれらすべてを実行するため、クリーンな実行は今や「字句解析と解析に成功した」ことを意味します。

明確な段階の境界はデバッグを改善し、フォーマッタ、言語サーバー、ドキュメントツールが、独立した前提でテキストを再解析するのではなく、安定した表現を共有できるようにします。

pre-alpha の間はアーキテクチャを柔軟に保つべきです。プレースホルダーのクレートは、実装がこのプロジェクトに正しい境界を教える前に API を凍結する理由にはなりません。
