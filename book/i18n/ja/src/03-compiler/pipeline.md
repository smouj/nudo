# 03.1 — コンパイラーパイプライン

> **Status:** 実装済み (IMPLEMENTED) / 計画中 (PLANNED)
>
> **Summary:** NUDO のフロントエンドは明確な段階に分かれています。HIR と名前解決は実装済みで、その後の検査と実行の段階は引き続き開発中です。

{{#include ../diagrams/pipeline.svg}}

*アクセント色の経路は実装済みのフロントエンド作業を示します。型検査は
CLI が使用するフロントエンドの段階を強調しています。型検査は部分的に
実装されています。エフェクト検査、MIR、実行は計画中です。*

~~~text
SourceFile → Lexer → Lossless syntax tree → AST → HIR / name resolution
                                                        ↓
                                採用済み型規則の部分的なチェッカー
                                                        ↓
                                       effects → MIR → interpreter / backend
~~~

CLI コマンド <code>nudo check</code> はソースを読み込み、字句解析と構文解析を行い、HIR への lowering と名前解決の後、実装済みの採用済み型規則を検査します。token、構文木、名前解決を dump できます。ソースは実行しません。

<code>nudo-hir</code> crate は型付き AST を識別子と解決済み定義に lowering します。<code>nudo-typeck</code> crate はその HIR を受け取り、採用済み型規則の一部を実装します。名前解決が成功した後に CLI が呼び出し、unit test と typeck 適合性コーパスも直接検証します。Draft の規則は未実装のままです。

計画中のエフェクトチェッカー、MIR、インタープリター、WebAssembly backend は利用できません。図中の箱は意図されたアーキテクチャとロードマップ上の順序を示すものであり、現在プログラムがそれらの段階を通過できるという約束ではありません。

段階の境界を明確にすることで、各変換をテストでき、将来のツールが異なる前提でソースを再 parse せずに表現を共有できます。pre-alpha の間は、実装の証拠が集まるにつれて crate の境界が変わる場合があります。
