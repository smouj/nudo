# 04.1 — Fundamentos del sistema de tipos

> **Status:** Implementado (IMPLEMENTED) / Especificado (SPECIFIED) / Planificado (PLANNED)
>
> **Summary:** NUDO tiene un comprobador parcial implementado para reglas de tipos aceptadas; el sistema completo sigue en desarrollo y la CLI ya ejecuta el subconjunto implementado.

El sistema de tipos está especificado en los capítulos del lenguaje y en las NEP aceptadas. El crate de Rust <code>nudo-typeck</code> implementa un subconjunto creciente y consume HIR, no texto fuente. Sus pruebas y casos de conformidad ejercitan las reglas implementadas. La CLI lo invoca tras resolver los nombres correctamente.

## Reglas implementadas por el comprobador actual

| Regla | Comportamiento actual |
| --- | --- |
| Tipos primitivos y literales | Se representan <code>Int</code>, <code>Float</code>, <code>Bool</code>, <code>Text</code> y <code>Unit</code> |
| Rango de literales enteros | Se acepta <code>Int::MAX</code>; la magnitud <code>2^63</code> solo se acepta directamente bajo el unario <code>-</code> para escribir <code>Int::MIN</code>; magnitudes mayores o inválidas usan <code>NDO2010</code> |
| Anotaciones y firmas de función | Los valores se comparan con los tipos escritos; se comprueban los tipos de función y las llamadas |
| Resultados de función | Si se omite el tipo de resultado, es <code>Unit</code>; las funciones que devuelven un valor declaran su tipo y los desacuerdos del cuerpo usan <code>NDO2004</code> |
| Llamadas | La aridad usa <code>NDO2005</code>, los argumentos incompatibles <code>NDO2004</code> y los valores no invocables <code>NDO2006</code> |
| Llamadas genéricas | Los parámetros de tipo se infieren a partir de los argumentos y de los tipos de resultado esperados que ya se conocen, en orden de origen; los parámetros genéricos no inferibles usan <code>NDO2008</code> y los parámetros de función ausentes de la firma usan <code>NDO2009</code> |
| Tipos con nombre | La aridad de argumentos de tipo usa <code>NDO2017</code>; se comprueban la identidad nominal y el acceso a campos admitido |
| Operadores | La aritmética usa vistas numéricas coincidentes; <code>%</code> solo admite Int; los operadores booleanos usan <code>Bool</code>; igualdad admite vistas primitivas coincidentes y el orden admite <code>Int</code> o <code>Float</code> coincidentes; los dominios inválidos usan <code>NDO2018</code> |
| Condiciones | La condición de <code>if</code> es <code>Bool</code>; un tipo esperado comprueba ambas ramas y, si no existe, deben coincidir exactamente; sin <code>else</code>, el resultado y la rama <code>then</code> son <code>Unit</code>; un desacuerdo sin contexto usa <code>NDO2019</code> |
| Coincidencia de enumeraciones | Las enumeraciones de usuario y <code>Result&lt;T, E&gt;</code> admiten patrones de variantes explícitos, enlaces planos y tipados de payload, cobertura con comodín, exhaustividad y reglas exactas para los tipos de las ramas; los diagnósticos usan <code>NDO2011</code>–<code>NDO2016</code> |
| Asignación de confianza | El <code>Verified&lt;T&gt;</code> integrado puede usarse donde se espera <code>T</code> en las posiciones admitidas |

La inferencia genérica tiene límites deliberados. No resuelve tipos para todo el programa, no generaliza una variable local ni adivina un parámetro sin resolver. Los argumentos genéricos siguen siendo invariantes y las conversiones numéricas no son implícitas. [NEP-0015](https://github.com/smouj/nudo/blob/main/neps/0015-uninferable-generic-parameters.md) hace visible este límite: <code>NDO2008</code> señala una llamada genérica sin suficiente información, mientras que <code>NDO2009</code> señala un parámetro de función ausente de su firma. Los parámetros de tipo externos permanecen fijos al invocar un valor de función. La recuperación suprime solo los diagnósticos que dependen de un error anterior.

## Reglas aún incompletas

La indexación de secuencias y sus límites en ejecución, y la construcción nominal siguen pendientes. La ejecución de verificadores y la procedencia, además de agentes, tareas y herramientas, aún no están implementadas en este comprobador.

Estas omisiones son deliberadas. Una regla solo se implementa cuando se acepta su contrato semántico y diagnóstico. La hoja de ruta y la especificación indican qué trabajo está listo y qué decisiones siguen abiertas.

## Qué significa una comprobación limpia de la CLI

<code>nudo check</code> ejecuta las fases léxica, sintáctica, de resolución de nombres y de comprobación parcial de tipos. Un resultado limpio significa que las reglas aceptadas implementadas no notificaron errores; las reglas no admitidas o sujetas a una NEP Draft siguen sin comprobarse. El comprobador también se ejercita mediante sus propias pruebas y el corpus de conformidad de tipos.
