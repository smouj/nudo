# 13.2 — モデル出力と検証のフロー

> **ステータス:** 提案中の構文 (PROPOSED SYNTAX)  
> **概要:** 正典となるエージェントの例は、生成された出力と検証された出力が意図的に分離されている理由を示します。

```nudo
let draft: Generated<Article> =
    ask Writer {
        "Create an article from the supplied research."
    }

let checked: Result<Verified<Article>, VerificationError> =
    verify draft with ArticleVerifier

match checked {
    Ok(article) => publish(article)
    Err(reason) => report(reason)
}
```

重要な性質は、正確な表面構文ではありません。不変条件は、`publish` のシグネチャが `Verified<Article>` を要求するとき、`publish` が未検証の生成値を誤って受け取ることができないという点です。さらに、その拒否は呼び出し側が扱う値であり、パニックではないという点です。
