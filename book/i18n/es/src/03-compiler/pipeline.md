# 03.1 — Pipeline del compilador

> **Status:** Implementado (IMPLEMENTED) / Planificado (PLANNED)
>
> **Summary:** El front end de NUDO se divide en fases explícitas; HIR y la resolución de nombres están implementados, mientras que la comprobación y la ejecución posteriores siguen en desarrollo.

{{#include ../diagrams/pipeline.svg}}

*La ruta de acento marca el trabajo del front end implementado. La comprobación
de tipos es parcial y se prueba por separado; la comprobación de efectos, MIR
y la ejecución están planificadas.*

~~~text
SourceFile → Lexer → Lossless syntax tree → AST → HIR / name resolution
                                                        ↓
                               comprobador parcial de tipos (no en nudo check)
                                                        ↓
                                    efectos → MIR → intérprete / backend
~~~

El comando <code>nudo check</code> carga el código fuente, ejecuta el análisis léxico y sintáctico y reduce a HIR/resuelve nombres. Puede volcar tokens, el árbol sintáctico y las resoluciones de nombres. No invoca <code>nudo-typeck</code> ni ejecuta el código.

El crate <code>nudo-hir</code> reduce el AST tipado a identificadores y definiciones resueltas. El crate <code>nudo-typeck</code> consume ese HIR e implementa un conjunto parcial de reglas de tipos aceptadas. Las pruebas unitarias y el corpus de conformidad de typeck lo ejercitan directamente. Sus reglas restantes deben especificarse antes de implementarse.

El comprobador de efectos planificado, MIR, el intérprete y el backend WebAssembly no están disponibles. Sus bloques en el diagrama muestran la arquitectura prevista y el orden de la hoja de ruta; no prometen que hoy un programa pueda recorrer esas fases.

Los límites explícitos entre fases permiten probar cada transformación y compartir representaciones con herramientas futuras, en vez de volver a analizar el código con supuestos independientes. Durante la etapa pre-alpha, los límites entre crates todavía pueden cambiar a medida que se acumula evidencia de implementación.
