# 04.2 — `Generated<T>` y `Verified<T>`

> **Status:** Propuesto  
> **Summary:** Los tipos de confianza pretenden impedir que una salida probabilística se convierta en datos de aplicación de confianza mediante una conversión implícita.


{{#include ../diagrams/trust-flow.svg}}

*Una salida de modelo solo se convierte en un valor verificado mediante un paso explícito.*

## ¿Por qué no `T`?

Si un modelo devuelve un `Article` directamente, la procedencia sobre cómo se produjo
ese valor desaparece de la frontera de tipos.

## ¿Por qué no `Result<T, E>`?

`Result` puede representar el éxito o el fallo de una operación. No representa el
estado de confianza de un valor correcto. Una respuesta de modelo perfectamente
analizada puede seguir sin estar verificada.

## ¿Por qué no un indicador booleano?

```text
ModelOutput<T> { value: T, verified: Bool }
```

Esto traslada la regla a una convención del runtime. Quien llama puede olvidarse de
inspeccionar el indicador. Los tipos distintos permiten que el comprobador haga
imposible ignorar la transición que falta.

## Cuestiones de diseño obligatorias

Antes de estabilizar, NUDO debe especificar los tipos de fallo del verificador, la
composición, la conservación de la procedencia, las fronteras de serialización y las
interacciones con los genéricos y la coincidencia de patrones.
