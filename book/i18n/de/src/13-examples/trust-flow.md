# 13.2 — Modellausgabe und Verifikationsablauf

> **Status:** Vorgeschlagene Syntax  
> **Summary:** Das kanonische Agentenbeispiel zeigt, warum generierte Ausgabe und verifizierte Ausgabe absichtlich getrennt sind.

```nudo
let draft: Generated<Article> =
    ask Writer {
        "Create an article from the supplied research."
    }

let article: Verified<Article> =
    verify draft with ArticleVerifier

publish(article)
```

Die wichtige Eigenschaft ist nicht die genaue Oberflächensyntax. Die Invariante ist,
dass `publish` nicht versehentlich einen unverifizierten generierten Wert erhalten
kann, wenn seine Signatur `Verified<Article>` verlangt.
