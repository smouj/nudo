# 13.2 — Model output and verification flow

> **Status:** Proposed syntax  
> **Summary:** The canonical agent example shows why generated output and verified output are intentionally separate.

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

The important property is not the exact surface syntax. The invariant is that
`publish` cannot accidentally receive an unverified generated value when its
signature requires `Verified<Article>` — and that the rejection is a value the
caller handles, not a panic.
