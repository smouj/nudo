# 08.2 — Benchmarking del front end

> **Status:** Implementado (IMPLEMENTED) / Planificado (PLANNED)
>
> **Summary:** NUDO cuenta con una pequeña suite reproducible de benchmarks del front end; sus resultados describen la carga y el equipo registrados, no una clasificación universal de velocidad.

El programa de benchmarks mide fases del compilador que existen hoy. No mide la ejecución de programas NUDO, porque todavía no existen el intérprete ni <code>nudo run</code>.

## Cargas de trabajo y límites de las fases

La suite genera cuatro entradas sintéticas y deterministas: declaraciones de funciones, bindings let, literales de texto y código con muchos comentarios. Cada una se mide en cuatro fases, para un total de dieciséis filas:

| Fase | Trabajo medido |
| --- | --- |
| Lex | Tokenizar un archivo fuente registrado |
| Parse | Tokenizar y analizar mediante el parser público |
| HIR | Reducir un árbol sintáctico ya preparado |
| Comprobador de tipos | Comprobar un HIR ya preparado |

La medición del parser incluye la tokenización porque el análisis necesita un flujo de tokens. La reducción a HIR y la comprobación de tipos preparan la entrada antes de medir, de modo que los valores informados aíslan esas fases. Las cargas son sintéticas y deliberadamente pequeñas; no representan todas las funciones del lenguaje ni una aplicación completa.

La unidad del comprobador de tipos es la longitud de la tabla de tipos de expresiones, incluidos los espacios que no tienen tipo. No es un recuento de reglas ejecutadas ni de expresiones comprobadas correctamente. Que dos ejecuciones tengan el mismo recuento no demuestra que hayan realizado el mismo trabajo semántico.

## Qué registra cada resultado

Una ejecución registra la revisión del código y si el árbol está modificado, el compilador de Rust, el sistema operativo, la arquitectura, la descripción de CPU cuando está disponible, el número de iteraciones, los bytes de entrada, las unidades de salida y estadísticas de tiempo. El informe incluye mínimo, mediana, percentil 95 y media; las filas del lexer y del parser también informan del rendimiento. No hay una referencia global del proyecto ni una barrera automática de rendimiento en CI.

## Cómo comparar resultados

Usa builds de release. Compara las mismas cargas de trabajo, límites de fase, configuración de iteraciones y toolchain del compilador. Ejecuta ambas revisiones en el mismo equipo, en condiciones parecidas y sin carga de fondo; repítelas y conserva los informes JSON junto al cambio que se revisa. Una comparación aislada es una señal para investigar; repítela antes de atribuir una diferencia de tiempo al código.

El comprobador cambió su cobertura semántica en el PR #31. Una medición de comprobación de tipos anterior y posterior a ese cambio puede describir dos implementaciones que realizan trabajos distintos, incluso si coinciden el código fuente y el recuento de expresiones. Por sí sola no demuestra una regresión ni una mejora con trabajo equivalente.

## Mediciones entre equipos

El resultado de un equipo es un punto medido para ese equipo y esa configuración. Puede servir para planificar de forma aproximada, pero la frecuencia, el número de núcleos o el nombre del modelo de CPU no permiten convertirlo de manera fiable a otra CPU. La microarquitectura, la memoria y la caché, los límites de potencia sostenida, la planificación del sistema operativo, la versión del compilador y la carga de fondo pueden afectar a cada fase de forma diferente.

Para estimar el rendimiento en el equipo de destino, ejecuta allí el mismo commit y las mismas cargas de trabajo, registra los metadatos y repite las mediciones. Si no es posible, etiqueta cualquier estimación como extrapolación aproximada e incluye sus supuestos e incertidumbre. No la presentes como un resultado de benchmark.

{{#include ../diagrams/benchmark-portability.svg}}

*El diagrama muestra por qué las mediciones deben conservar su configuración y
por qué hace falta una ejecución propia en un segundo equipo.*

Consulta el <a href="https://github.com/smouj/nudo/blob/main/benchmarks/README.md">README de benchmarks</a> para ver las opciones actuales de la línea de comandos y el formato del informe JSON.
