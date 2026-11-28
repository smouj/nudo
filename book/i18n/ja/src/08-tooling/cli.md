# 08.1 — Toolchain と CLI

> **Status:** 実装済み (IMPLEMENTED) / 計画中 (PLANNED)
> **Summary:** 現在の <code>nudo</code> コマンドはソース、構文、名前と実装済みの採用済み型規則を検査します。実行、エフェクト、その他の toolchain は今後の作業です。

## 実装済みコマンド

~~~sh
nudo check path/to/file.nudo
nudo check --dump-tokens path/to/file.nudo
nudo check --dump-tree path/to/file.nudo
nudo check --dump-resolutions path/to/file.nudo
nudo --version
nudo --help
~~~

check コマンドは1つ以上のソースファイルを読み込み、字句解析と構文解析を行い、採用済み構文を HIR に lowering し、名前を解決して、実装済みの採用済み型規則を検査します。通常形式または CI 向けの report も出力できます。

型検査の範囲は部分的です。未対応または Draft の規則は検査されず、このコマンドはプログラムを実行しません。<code>nudo run</code>、<code>nudo build</code>、REPL、formatter、language server、package manager はまだありません。

終了コードは安定しています。0 は実装済みの段階で error 診断がなかったこと、1 は error が1つ以上報告されたこと、2 は使用法または入力のエラー、あるいは未実装コマンドを示します。

## フロントエンドの測定

独立した <code>nudo-bench</code> program は、4種類の決定論的な合成入力を、lex、parse、HIR lowering、型検査の4段階で測定します。結果は16行です。後続段階に使うソースと HIR の準備は計測区間の外で行います。公開 parser が token を受け取るため、parse には token 化が含まれます。

型チェッカーが報告するのは expression-type table の長さです。この数には型なしの slot も含まれ、実行された意味論規則数ではありません。数値が等しくても、二つの型チェッカー revision が同じ処理をしたとは証明できません。

{{#include ../diagrams/benchmark-portability.svg}}

*観測時間は、記録された構成全体に依存します。CPU clock や core 数
だけでは、別のマシンにおける性能を確実に推定できません。*

実行結果には commit、変更の有無、Rust version、OS、architecture、取得できる場合は CPU model、入力サイズ、iteration 数、時間統計が記録されます。プロジェクト共通の基準値や CI performance gate はありません。

有用な比較のために、両方の revision を release mode で build し、同一の workload と iteration 設定を使い、同じ静かなマシンで記録してください。複数回実行します。しきい値を超えたら調査の手掛かりとし、単一の測定を回帰の証拠とみなしてはいけません。

あるコンピューターの結果は、そのコンピューターと workload を説明します。計画のための概算には使えますが、別のコンピューターの時間を推定するには、同じ commit、toolchain、workload、設定で実測する必要があります。CPU、memory system、scheduler、持続周波数は段階ごとに違う影響を与えるため、マシン間で一つの数値を使う確実な換算方法はありません。

この suite が測定するのはコンパイラーのフロントエンド段階だけです。これらのシステムはまだ toolchain にないため、インタープリターの起動、プログラム全体の実行、agent/model の呼び出し、最大 memory 使用量は報告できません。
