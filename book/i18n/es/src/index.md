# Introducción

> **Status:** Histórico (HISTORICAL) / Vivo
> **Summary:** Un recorrido basado en evidencia por el diseño de NUDO, el compilador que existe hoy y el runtime hacia el que avanza el proyecto.

<div class="status-key">
DOCUMENT: NUDO-BOOK-001 · EDITION: ENGINEERING PAPER · REVISION: R0 · STATUS: LIVING
</div>

NUDO es un proyecto de lenguaje de programación de propósito general. Su diseño pretende que programas deterministas ordinarios puedan incluir trabajo probabilístico acotado y que la confianza de los datos, la autoridad necesaria y la verificación sean visibles en el programa. El proyecto está en fase pre-alpha: tiene un front end funcional y un crate parcial de comprobación de tipos, pero aún no ejecuta programas NUDO.

Este libro explica la motivación, las reglas aceptadas del lenguaje, el diseño del compilador, el comportamiento implementado, las propuestas abiertas y el runtime planificado. Es más amplio que la especificación normativa, por lo que cada ejemplo debe leerse teniendo en cuenta su estado. La especificación define el lenguaje; este libro no añade reglas.

## Cómo leer este libro

Para conocer el diseño, empieza por Filosofía y Confianza. Para el compilador implementado, lee Diseño del lenguaje → Compilador → Sistema de tipos. Para el futuro runtime de agentes, lee Agentes e IA → Seguridad → Runtime. La tabla de hitos distingue el código funcional de las reglas aceptadas y las propuestas.

## Qué funciona hoy

- <code>nudo check</code> carga archivos <code>.nudo</code>, los analiza léxica y sintácticamente, resuelve nombres y aplica las reglas de tipos aceptadas que ya están implementadas.
- <code>--dump-tokens</code>, <code>--dump-tree</code> y <code>--dump-resolutions</code> muestran vistas estables de esas fases.
- Los crates de Rust contienen un comprobador parcial de tipos. Sus reglas implementadas se ejercitan con pruebas unitarias y el corpus de conformidad de tipos, y <code>nudo check</code> las aplica tras analizar y resolver los nombres correctamente.
- La suite de benchmarks mide cargas sintéticas en el lexer, el parser, la reducción a HIR y la comprobación de tipos. No mide la ejecución de programas NUDO.

Todavía no hay <code>nudo run</code>, <code>nudo build</code>, intérprete, comprobador de efectos, runtime de agentes, despacho de herramientas, aplicación de policies, registrador de procedencia ni backend WebAssembly.

## Estado actual del proyecto

{{#include diagrams/status.svg}}

*El estado de los hitos y sus dependencias hasta M5. El acento identifica la
implementación completada o activa; las filas atenuadas están planificadas.
La <a href="https://github.com/smouj/nudo/blob/main/ROADMAP.md">hoja de ruta</a>
es la fuente de verdad si este resumen y el repositorio llegan a divergir.*

M3 está en curso: HIR y la resolución de nombres están implementados, mientras que la comprobación de tipos y los tipos de confianza solo cubren una parte de las reglas aceptadas. M4 (el primer programa ejecutable) y M5 (la comprobación de efectos) siguen planificados. M5 depende de completar los hitos anteriores y aceptar la semántica del sistema de efectos.

## Autoridad

La especificación define el lenguaje. Las NEP aceptadas la modifican. Las pruebas de conformidad fijan el comportamiento observable. Este libro explica esas reglas y etiqueta el trabajo futuro; nunca convierte una propuesta en una capacidad implementada.
