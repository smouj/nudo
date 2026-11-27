# 04.1 — Fundamentos del sistema de tipos

> **Status:** Implementado (IMPLEMENTED) / Especificado (SPECIFIED) / Planificado (PLANNED)
>
> **Summary:** NUDO tiene un comprobador parcial implementado para reglas de tipos aceptadas; el sistema de tipos completo sigue en desarrollo y aún no forma parte del comando <code>check</code> de la CLI.

El sistema de tipos está especificado en los capítulos del lenguaje y en las NEP aceptadas. El crate de Rust <code>nudo-typeck</code> implementa un subconjunto creciente y consume HIR, no texto fuente. Sus pruebas y casos de conformidad ejercitan las reglas implementadas. La CLI aún no invoca el comprobador.

## Reglas implementadas por el comprobador actual

| Regla | Comportamiento actual |
| --- | --- |
| Tipos primitivos y literales | Se representan <code>Int</code>, <code>Float</code>, <code>Bool</code>, <code>Text</code> y <code>Unit</code> |
| Anotaciones y firmas de función | Los valores se comparan con los tipos escritos; se comprueban los tipos de función y las llamadas |
| Llamadas | La aridad usa <code>NDO2005</code>, los argumentos incompatibles <code>NDO2004</code> y los valores no invocables <code>NDO2006</code> |
| Llamadas genéricas | Los parámetros de tipo se infieren a partir de los argumentos y de los tipos de resultado esperados que ya se conocen, en orden de origen |
| Tipos con nombre | La aridad de argumentos de tipo usa <code>NDO2017</code>; se comprueban la identidad nominal y el acceso a campos admitido |
| Aritmética | Los operandos numéricos coincidentes admiten <code>+</code>, <code>-</code>, <code>*</code> y <code>/</code>; <code>%</code> solo admite Int y el <code>-</code> unario conserva el tipo numérico |
| Condiciones | La condición de <code>if</code> debe ser <code>Bool</code>; se siguen visitando las expresiones de las ramas |
| Asignación de confianza | El <code>Verified&lt;T&gt;</code> integrado puede usarse donde se espera <code>T</code> en las posiciones admitidas |

La inferencia genérica tiene límites deliberados. No resuelve tipos para todo el programa, no generaliza una variable local ni adivina un parámetro sin resolver. Los argumentos genéricos siguen siendo invariantes y las conversiones numéricas no son implícitas.

## Reglas aún incompletas

El comprobador todavía no define un tipo de resultado para <code>if</code>, la concordancia entre valores de ramas, todos los operadores, la indexación, la construcción nominal, <code>match</code> ni la correspondencia con <code>Result&lt;T, E&gt;</code>. También está pendiente un diagnóstico estable para un parámetro genérico que siga sin resolverse. [NEP-0015](https://github.com/smouj/nudo/blob/main/neps/0015-uninferable-generic-parameters.md) propone ese contrato de diagnóstico, pero sigue en Draft y todavía no se ha aceptado ni implementado. La comprobación del rango de literales enteros, agentes, tareas y herramientas aún no está implementada en este comprobador.

Estas omisiones son deliberadas. Una regla solo se implementa cuando se acepta su contrato semántico y diagnóstico. La hoja de ruta y la especificación indican qué trabajo está listo y qué decisiones siguen abiertas.

## Qué significa una comprobación limpia de la CLI

<code>nudo check</code> ejecuta las fases léxica, sintáctica y de resolución de nombres. Como el comprobador parcial de tipos no está conectado a ese comando, un resultado limpio de la CLI no demuestra que se hayan superado todas las reglas de tipos. El comprobador se ejercita actualmente mediante sus propias pruebas y el corpus de conformidad de tipos.
