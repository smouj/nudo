# 13.2 — Modellausgabe und Verifikationsablauf

> **Status:** Vorgeschlagene Syntax (PROPOSED SYNTAX)  
> **Summary:** Das kanonische Agentenbeispiel zeigt, warum generierte Ausgabe und verifizierte Ausgabe absichtlich getrennt sind.

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

Die wichtige Eigenschaft ist nicht die genaue Oberflächensyntax. Die Invariante ist,
dass `publish` nicht versehentlich einen unverifizierten generierten Wert erhalten
kann, wenn seine Signatur `Verified<Article>` verlangt — und dass die Ablehnung ein
Wert ist, den der Aufrufer behandelt, und keine Panik.
