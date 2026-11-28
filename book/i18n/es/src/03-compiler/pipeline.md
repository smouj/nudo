# 03.1 — Pipeline del compilador

> **Status:** Implementado (IMPLEMENTED) / Planificado (PLANNED)
>
> **Summary:** El front end de NUDO se divide en fases explícitas; HIR y la resolución de nombres están implementados, mientras que la comprobación y la ejecución posteriores siguen en desarrollo.

{{#include ../diagrams/pipeline.svg}}

*La ruta de acento marca el trabajo del front end utilizado por la CLI. La
comprobación de tipos es parcial; los efectos, MIR y la ejecución están
planificados.*

~~~text
SourceFile → Lexer → Lossless syntax tree → AST → HIR / name resolution
                                                        ↓
                            comprobador parcial de reglas de tipos aceptadas
                                                        ↓
                                    efectos → MIR → intérprete / backend
~~~

El comando <code>nudo check</code> carga el código fuente, ejecuta el análisis léxico y sintáctico, reduce a HIR/resuelve nombres y aplica las reglas de tipos aceptadas que están implementadas. Puede volcar tokens, el árbol sintáctico y las resoluciones de nombres. No ejecuta el código.

El crate <code>nudo-hir</code> reduce el AST tipado a identificadores y definiciones resueltas. El crate <code>nudo-typeck</code> consume ese HIR e implementa un conjunto parcial de reglas de tipos aceptadas. La CLI lo invoca tras resolver los nombres sin errores; las pruebas unitarias y el corpus de conformidad de typeck también lo ejercitan directamente. Las reglas Draft siguen sin implementarse.

El comprobador de efectos planificado, MIR, el intérprete y el backend WebAssembly no están disponibles. Sus bloques en el diagrama muestran la arquitectura prevista y el orden de la hoja de ruta; no prometen que hoy un programa pueda recorrer esas fases.

Los límites explícitos entre fases permiten probar cada transformación y compartir representaciones con herramientas futuras, en vez de volver a analizar el código con supuestos independientes. Durante la etapa pre-alpha, los límites entre crates todavía pueden cambiar a medida que se acumula evidencia de implementación.
