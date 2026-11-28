# 06.1 — Capability、エフェクト、権限

> **Status:** 提案中 (PROPOSED) / 計画中 (PLANNED)
>
> **Summary:** NUDO は静的なエフェクト追跡と明示的な capability 権限を提案していますが、現在の toolchain にはエフェクト検査も runtime での capability 強制もありません。

{{#include ../diagrams/authority.svg}}

*これは提案中の権限モデルであり、現在の runtime が強制する
セキュリティ境界ではありません。*

## 関連する三つの概念

- **Effect:** ネットワーク利用など、計算が実行し得る操作の静的な記述。
- **Capability grant:** プログラムや実行コンテキストに与えられる権限。
- **Policy:** 権限を行使できる状況や方法を制限する規則。

この設計は、関数が必要とする effect と呼び出し地点で利用できる権限を結びつけます。既定で拒否し、委譲先のツールやエージェントが呼び出し元の権限を広げられないようにすることを目指します。

## 現在の実装状況

M5 は計画中です。NEP-0004 は Discussion 段階で、エフェクトの章も引き続き提案段階です。effect inference、effect checker、<code>NDO3xxx</code> の実装、runtime での capability 強制はまだありません。<code>nudo-effects</code> crate はロードマップ上の境界を示すもので、完成したサブシステムではありません。

次の例は Discussion 中の提案を示しています。利用者が依存できる採用済み構文ではありません。

~~~nudo
fn fetch(url: Text) -> Text with Network {
    // 提案: fetch を呼び出すには Network が必要。
}
~~~

ロードマップにおける M5 の終了条件は、エフェクトを持つ関数呼び出しをエフェクトなしの関数から拒否し、その effect と導入元の呼び出し経路を示す診断を出すことです。チェッカーを実装するには、設計の採用が先に必要です。runtime policy の強制は別の後続マイルストーンです。
