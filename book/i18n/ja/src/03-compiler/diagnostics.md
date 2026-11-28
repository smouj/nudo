# 03.4 — 製品設計としての診断

> **Status:** 実装済み (IMPLEMENTED) / 計画中 (PLANNED)
> **Summary:** NUDO の診断は安定したコードとソース span を使用します。実装済みの分類は、字句・構文・名前解決・部分的な型検査のエラーを扱います。

診断は言語体験の一部であり、debug dump ではありません。<code>nudo-diagnostics</code> crate は、severity、code、ソース位置、note、help を持つ構造化診断を提供します。コンパイラーの各段階が独自に出力を整形せずに事実を報告できるよう、renderer を共有しています。

## 実装済みの診断分類

- <code>NDO1xxx</code> は字句エラーと構文エラーを扱います。lexer と parser は可能な場合に回復し、ソース span を報告します。
- <code>NDO2001</code>–<code>NDO2003</code> は名前解決エラーで、<code>nudo check</code> が出力します。
- <code>NDO2004</code>–<code>NDO2019</code> は実装済みの型検査規則を扱います。<code>NDO2010</code> は <code>Int</code> で表せない整数の大きさを指します。正確に <code>2^63</code> の値は <code>Int::MIN</code> を表す単項 <code>-</code> の直接のオペランドとしてのみ使用できます。NEP-0020 に基づき、<code>NDO2011</code>–<code>NDO2016</code> は enum と <code>Result</code> のパターン、payload の個数、網羅性、到達可能性、match の各腕の型を扱います。テストは構造化された note とソース位置を固定します。

型チェッカーは部分的な実装で、CLI からはまだ呼び出されません。そのため、エラーのない <code>nudo check</code> はソースが字句解析、構文解析、名前解決を通過したことを示すだけで、型検査を通過したという意味ではありません。

## 計画中の診断分類

エフェクトシステムの <code>NDO3xxx</code> 分類は M5 で計画されています。現在これらの診断を出すエフェクトチェッカーはありません。以下は目標を示す例であり、現在の toolchain の出力ではありません。

~~~text
error[NDO3xxx]: capability Network が不足しています

  --> src/main.nudo:14:5
   |
14 |     fetch(url)
   |     ^^^^^^^^^^ Network が必要です
   |
   = note: 呼び出し経路が Network を必要とします
~~~

採用済み診断仕様は構造とソース位置の要件を定義します。エフェクト構文と検査規則は、提案が採用されることを引き続き前提としています。

## 診断の品質

役に立つ診断は、どの規則に違反したか、ソースのどこで違反したか、どの要件がエラーを生んだかを説明します。信頼型の不一致では、<code>Generated&lt;T&gt;</code> と <code>Verified&lt;T&gt;</code> が別の型であることと、保証に違反する値を示すべきです。安全でない cast を提案してはいけません。
