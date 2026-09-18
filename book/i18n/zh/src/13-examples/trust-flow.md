# 13.2 — 模型输出与验证流程

> **状态（Status）：** 提案语法（PROPOSED syntax）  
> **概述（Summary）：** 这个经典的智能体示例说明为什么生成的输出与已验证的输出被有意分开。

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

重要的性质不是确切的表层语法。不变量在于：当 `publish` 的签名要求
`Verified<Article>` 时，它不可能意外收到一个未验证的生成值——而且这种拒绝是调用方
处理的一个值，而不是一次 panic。
