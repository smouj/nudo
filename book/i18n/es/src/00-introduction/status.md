# 00.3 — Estado actual del proyecto

> **Status:** Implementado (IMPLEMENTED) / Planificado (PLANNED)
> **Summary:** El repositorio cuenta con un front end funcional y soporte parcial de comprobación de tipos; la ejecución y la comprobación de efectos siguen siendo hitos futuros.

El estado siguiente refleja la hoja de ruta del repositorio. «Implementado» significa que existen código y pruebas para el alcance indicado. No significa que el lenguaje completo ni un flujo de trabajo para usuarios estén terminados.

| Hito | Estado | Qué está disponible |
| --- | --- | --- |
| M0 — Base del repositorio | Completo | Workspace, reglas de contribución, CI e infraestructura documental |
| M1 — Modelo de fuentes y lexer | Completo | Archivos fuente, spans, tokens, diagnósticos léxicos y recuperación |
| M2 — Parser y árbol sintáctico | Completo | Sintaxis sin pérdidas, parser, AST tipado y diagnósticos sintácticos |
| M3.0 — Decisiones semánticas | Completo | Decisiones aceptadas registradas en la especificación y en las NEP |
| M3.1 — HIR y resolución de nombres | Completo | Reducción a HIR, ámbitos, resolución de nombres y NDO2001–NDO2003 |
| M3.2 — Sistema de tipos | En curso | Comprobador parcial para reglas aceptadas, incluido el <code>match</code> tipado de enumeraciones y <code>Result</code>; aún no lo invoca la CLI |
| M3.3 — Tipos de confianza | Parcial | Identidad de tipos y regla aceptada de asignación de Verified a su tipo interno |
| M4 — Intérprete | Planificado | Primer programa de extremo a extremo y <code>nudo run</code> |
| M5 — Sistema de efectos | Planificado | Inferencia, comprobación y diagnósticos de efectos; la semántica sigue propuesta |

El comando <code>nudo check</code> carga el código fuente, lo analiza léxica y sintácticamente y resuelve nombres. Puede mostrar tokens, árboles sintácticos y resoluciones. Una ejecución limpia significa que esas fases no notificaron errores; no demuestra que el programa tenga tipos correctos, sea ejecutable o resulte seguro durante la ejecución.

El crate de comprobación de tipos cubre tipos primitivos y literales, anotaciones, llamadas, inferencia genérica acotada, acceso a campos, ciertas operaciones aritméticas, condiciones Bool, el <code>match</code> tipado de enumeraciones y <code>Result</code>, y la asignación aceptada de tipos de confianza. Su corpus de conformidad prueba estas reglas independientemente de la CLI. Las reglas no admitidas siguen deliberadamente incompletas hasta que sus contratos semánticos estén listos.

M4 y M5 tienen criterios de dependencia. M4 requiere completar el trabajo restante de tipos y confianza, además de una ruta de ejecución real. M5 necesita esas bases y un diseño aceptado del sistema de efectos. La hoja de ruta define la finalización mediante criterios funcionales y evidencia de conformidad, no mediante fechas.

## Por qué importan las etiquetas de estado

Un ejemplo pulido puede parecer una función publicada. Por eso cada capítulo indica si describe comportamiento implementado, diseño aceptado, una propuesta o trabajo planificado. La <a href="https://github.com/smouj/nudo/blob/main/ROADMAP.md">hoja de ruta</a> es el registro cambiante y autoritativo del estado.
