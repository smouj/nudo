# 08.2 — フロントエンドのベンチマーク

> **Status:** 実装済み (IMPLEMENTED) / 計画中 (PLANNED)
>
> **Summary:** NUDO には小規模で再現可能なフロントエンド・ベンチマーク一式があります。結果が示すのは記録した workload と host の測定値であり、普遍的な速度評価ではありません。

ベンチマークプログラムは現在存在するコンパイラー段階を測定します。インタープリターと <code>nudo run</code> がまだないため、NUDO プログラムの実行時間は測定しません。

## Workload と段階の境界

この suite は、関数宣言、let 束縛、テキストリテラル、コメントが多いソースの4種類の決定論的な合成入力を生成します。それぞれを4段階で測定し、合計16行の結果になります。

| 段階 | 測定する処理 |
| --- | --- |
| Lex | 登録済みソースファイルを token 化 |
| Parse | token 化し、公開 parser で parse |
| HIR | 事前に準備した構文木を lowering |
| 型チェッカー | 事前に準備した HIR を検査 |

parse には token 化が含まれます。parser が token stream を必要とするためです。HIR lowering と型検査は計測前に入力を準備するため、報告値はそれぞれの段階を分離して測定します。workload は意図的に小さな合成データであり、すべての言語機能や完全なアプリケーションを表すものではありません。

型チェッカーの単位は、型がない slot を含む expression-type table の長さです。実行された規則数や正常に検査された式の数ではありません。単位数が等しくても、同じ意味論上の作業を行った証拠にはなりません。

## 結果に記録する情報

実行結果には、ソース revision と変更の有無、Rust compiler、OS、architecture、取得可能な場合は CPU の説明、iteration 数、入力 byte 数、出力 unit 数、時間統計が記録されます。report には最小値、中央値、95 パーセンタイル、平均値が含まれ、lexer と parser の行には throughput も含まれます。プロジェクト共通の基準値や CI の自動 performance gate はありません。

## 結果の比較

release build を使います。同じ workload、段階の境界、iteration 設定、compiler toolchain を比較します。両方の revision を同じマシン上で近い静穏条件のもと実行し、繰り返して、JSON report をレビュー対象の変更とともに保存します。一度の比較は調査の手掛かりです。時間差をコードに帰属させる前に再計測します。

PR #31 では型チェッカーの意味論上の対象範囲が変わりました。その変更前後の型検査測定は、ソースと expression table の数が同じでも、異なる処理を行う二つの実装を示すことがあります。同等の作業での性能低下や向上を、それだけで立証することはできません。

## 異なるマシンでの測定

あるコンピューターの結果は、そのコンピューターと構成における測定点です。概算の計画には役立つことがありますが、CPU の clock、core 数、model 名だけでは別の CPU の結果へ確実に換算できません。microarchitecture、memory と cache、持続電力制限、OS の scheduling、compiler version、バックグラウンド負荷が段階ごとに異なる影響を与えます。

対象マシンで見積もるには、同じ commit と workload をそこで実行し、metadata を記録して測定を繰り返します。それができない場合は概算の外挿であることを明示し、前提と不確実性を記載してください。benchmark 結果として提示してはいけません。

{{#include ../diagrams/benchmark-portability.svg}}

*この図は、測定値にその構成を添える理由と、別のハードウェアでは
個別の実行が必要な理由を示します。*

現在の CLI オプションと JSON report 形式は<a href="https://github.com/smouj/nudo/blob/main/benchmarks/README.md">ベンチマーク README</a>を参照してください。
