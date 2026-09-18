# 13.2 — Salida del modelo y flujo de verificación

> **Status:** Sintaxis propuesta (PROPOSED SYNTAX)  
> **Summary:** El ejemplo canónico de agente muestra por qué la salida generada y la salida verificada son intencionadamente distintas.

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

La propiedad importante no es la sintaxis superficial exacta. El invariante es que
`publish` no puede recibir por accidente un valor generado sin verificar cuando su
firma exige `Verified<Article>` — y que el rechazo es un valor que maneja quien
llama, no un pánico.
