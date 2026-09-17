# 04.1 — Fundamentos del sistema de tipos

> **Status:** Propuesto  
> **Summary:** El sistema de tipos favorece las firmas públicas explícitas, las relaciones nominales y la información de confianza y de efectos visible para el compilador.

Entre las restricciones de diseño actuales están los tipos escalares primitivos, los
structs, los enums, las secuencias, las funciones, los genéricos y las formas de tipo
opcional y de tipo resultado.

Las interfaces públicas deberían ir anotadas. La inferencia local puede reducir el
ruido, pero los tipos de la API pública no deberían depender en secreto de detalles
de implementación.

Las cuestiones abiertas, como la anchura de los enteros, la semántica del
desbordamiento, la varianza y la representación exacta de las capacidades que
proporciona el runtime, deben resolverse de forma deliberada.
