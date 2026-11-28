# 04.2 — Valores Generated y Verified

> **Status:** Implementado (IMPLEMENTED) / Especificado (SPECIFIED) / Planificado (PLANNED)
>
> **Summary:** El comprobador aplica la distinción entre los tipos generado y verificado; ejecutar la verificación y registrar la procedencia siguen siendo trabajo futuro del runtime.

{{#include ../diagrams/trust-flow.svg}}

*Las reglas de tipos estáticas están implementadas en parte; la operación
verify y la ruta de procedencia son una vista previa del diseño y aún no se
pueden ejecutar.*

## La distinción de confianza

<code>Generated&lt;T&gt;</code> y <code>Verified&lt;T&gt;</code> son tipos distintos. El comprobador no los convierte implícitamente y sus argumentos genéricos siguen siendo invariantes. Por tanto, un resultado generado no puede satisfacer un parámetro que exija explícitamente el tipo verificado.

Las reglas aceptadas también permiten usar el <code>Verified&lt;T&gt;</code> integrado donde se espera <code>T</code> en posiciones admitidas. Esto se basa en la identidad resuelta del tipo integrado; un tipo definido por el usuario con la misma grafía no obtiene esa regla. El comportamiento está cubierto por casos de conformidad del comprobador, pero <code>nudo check</code> todavía no ejecuta la comprobación de tipos.

## La operación de verificación planificada

El parser acepta la sintaxis <code>verify value with Verifier</code> y la representa en HIR. El contrato del verificador, la ejecución, el tratamiento de errores y la procedencia aún no están implementados. El diseño del verificador sigue siendo una propuesta; por eso la forma siguiente ilustra el flujo previsto y no es un programa ejecutable:

~~~nudo
let checked: Result<Verified<Article>, VerificationError> =
    verify draft with ArticleVerifier
~~~

El diseño hace visibles dos hechos distintos: la verificación puede fallar y un valor correcto lleva un tipo de confianza diferente. No vuelve determinista la generación probabilística; hace revisable el límite de confianza.

## Por qué no representar la confianza con un booleano

~~~text
ModelOutput<T> { value: T, verified: Bool }
~~~

Quien llama puede olvidarse de inspeccionar una bandera. Los tipos distintos permiten que el comprobador rechace un valor en un límite de comprobación compatible, en vez de depender de una convención en tiempo de ejecución.

## Qué queda pendiente

El comprobador admite <code>match</code> exhaustivo sobre el <code>Result&lt;T, E&gt;</code> integrado, con <code>Ok(T)</code> y <code>Err(E)</code> igual que las variantes de una enumeración de usuario. La operación <code>verify</code> aún no puede producir este tipo: su ejecución en runtime, las declaraciones de verificadores comprobables estáticamente, un tipo estable de error, la procedencia y las trazas siguen planificados o propuestos.
