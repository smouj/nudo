# 13.2 — 模型输出与验证流程

> **状态（Status）：** 提案语法（PROPOSED syntax）  
> **概述（Summary）：** 这个经典的智能体示例说明为什么生成的输出与已验证的输出被有意分开。

```nudo
let draft: Generated<Article> =
    ask Writer {
        "Create an article from the supplied research."
    }

let article: Verified<Article> =
    verify draft with ArticleVerifier

publish(article)
```

重要的性质不是确切的表层语法。不变量在于：当 `publish` 的签名要求
`Verified<Article>` 时，它不可能意外收到一个未验证的生成值。
