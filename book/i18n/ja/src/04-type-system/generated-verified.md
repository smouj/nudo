# 04.2 — Generated と Verified の値

> **Status:** 実装済み (IMPLEMENTED) / 仕様策定済み (SPECIFIED) / 計画中 (PLANNED)
>
> **Summary:** チェッカーは生成型と検証済み型の区別を強制します。検証の実行と provenance の記録は、今後の runtime 作業です。

{{#include ../diagrams/trust-flow.svg}}

*静的な型規則は部分的に実装済みです。verify 操作と provenance の経路は
設計のプレビューであり、現時点では実行できません。*

## 信頼の区別

<code>Generated&lt;T&gt;</code> と <code>Verified&lt;T&gt;</code> は別の型です。チェッカーは暗黙に変換せず、ジェネリック引数も invariant のままです。そのため、生成結果は検証済み型を明示的に要求するパラメーターを満たせません。

採用済み規則では、対応する位置で <code>T</code> が期待される場合に、組み込みの <code>Verified&lt;T&gt;</code> を使用することも認めています。これは解決済みの組み込み型 identity に基づき、同じ綴りのユーザー定義型にその規則は適用されません。この動作は型チェッカーの適合性ケースでカバーされていますが、<code>nudo check</code> はまだ型検査を実行しません。

## 計画中の検証操作

parser は <code>verify value with Verifier</code> という構文を受け入れ、HIR に表現します。Verifier の契約、実行、失敗処理、provenance は未実装です。Verifier の設計は提案段階なので、次の形は想定フローを示すものであり、実行可能なプログラムではありません。

~~~nudo
let checked: Result<Verified<Article>, VerificationError> =
    verify draft with ArticleVerifier
~~~

この設計では二つの事実を別々に示します。検証は失敗し得ること、成功した値は異なる信頼型を持つことです。確率的な生成を決定論的にするのではなく、境界をレビュー可能にします。

## なぜ trust を Boolean で表さないのか

~~~text
ModelOutput<T> { value: T, verified: Bool }
~~~

呼び出し側が flag の確認を忘れる可能性があります。型を分けると、runtime の慣習に頼らず、対応する型検査境界でチェッカーが値を拒否できます。

## 今後の作業

runtime 操作、静的に検査可能な Verifier 宣言、安定した失敗型、provenance 記録、trace は計画中または提案中です。<code>Result&lt;T, E&gt;</code> は必要な型引数を付けて名前を使えますが、<code>match</code> の型検査と網羅的な処理は未実装です。
