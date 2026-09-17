# 13.2 — Model output and verification flow

> **Status:** Proposed syntax  
> **Summary:** The canonical agent example shows why generated output and verified output are intentionally separate.

```nudo
let draft: Generated<Article> =
    ask Writer {
        "Create an article from the supplied research."
    }

let article: Verified<Article> =
    verify draft with ArticleVerifier

publish(article)
```

The important property is not the exact surface syntax. The invariant is that
`publish` cannot accidentally receive an unverified generated value when its
signature requires `Verified<Article>`.
