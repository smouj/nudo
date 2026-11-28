# 08.1 — Toolchain y CLI

> **Status:** Implementado (IMPLEMENTED) / Planificado (PLANNED)
> **Summary:** El comando actual <code>nudo</code> comprueba el código fuente, la sintaxis, los nombres y el subconjunto implementado de reglas de tipos aceptadas; la ejecución, los efectos y el resto de la toolchain siguen siendo trabajo futuro.

## Comandos implementados

~~~sh
nudo check path/to/file.nudo
nudo check --dump-tokens path/to/file.nudo
nudo check --dump-tree path/to/file.nudo
nudo check --dump-resolutions path/to/file.nudo
nudo --version
nudo --help
~~~

El comando check carga uno o varios archivos, los analiza léxica y sintácticamente, reduce la sintaxis aceptada a HIR, resuelve nombres, aplica las reglas de tipos aceptadas que están implementadas e informa de diagnósticos. También puede emitir un informe normal o dirigido a CI.

La cobertura de tipos es parcial: las reglas no admitidas o sujetas a una NEP Draft siguen sin comprobarse, y el comando no ejecuta el programa. Todavía no existen <code>nudo run</code>, <code>nudo build</code>, REPL, formateador, servidor de lenguaje ni gestor de paquetes.

Los códigos de salida son estables: 0 significa que las fases implementadas no encontraron diagnósticos de error; 1 significa que se notificó al menos un error; 2 significa un error de uso o de entrada, o un comando no implementado.

## Mediciones del front end

El programa independiente <code>nudo-bench</code> mide cuatro entradas sintéticas y deterministas en cuatro fases: lex, parse, reducción a HIR y comprobación de tipos. Produce dieciséis filas. La preparación del código fuente y del HIR para las fases posteriores ocurre fuera de la región medida; parse incluye tokenización porque el parser público consume tokens.

El comprobador de tipos informa de la longitud de su tabla de tipos de expresiones. Ese recuento incluye los espacios que no tienen tipo y no cuenta las reglas semánticas ejecutadas. Que dos recuentos coincidan no prueba que dos revisiones del comprobador hayan realizado el mismo trabajo.

{{#include ../diagrams/benchmark-portability.svg}}

*El tiempo observado depende de toda la configuración registrada. No se puede
deducir de forma fiable la escala entre equipos solo por la frecuencia de CPU
o el número de núcleos.*

Una ejecución registra commit, estado modificado, versión de Rust, sistema operativo, arquitectura, modelo de CPU cuando está disponible, tamaño de entrada, iteraciones y estadísticas de tiempo. No hay una referencia global del proyecto ni una barrera de rendimiento en CI.

Para comparar de forma útil, compila ambas revisiones en modo release, usa cargas y cantidad de iteraciones idénticas y mídelo en el mismo equipo sin carga. Repite las ejecuciones. Trata los cruces de umbral como señales para investigar: una muestra aislada no demuestra una regresión.

Los resultados de un equipo describen ese equipo y esa carga. Pueden servir como referencia aproximada para planificar, pero para estimar otro equipo hay que medirlo con el mismo commit, toolchain, carga y configuración. Las CPU, los subsistemas de memoria, los planificadores y las frecuencias sostenidas pueden cambiar cada fase de manera distinta, así que no existe una conversión fiable de un solo número entre equipos.

La suite mide únicamente fases del front end del compilador. No puede informar del arranque del intérprete, de la ejecución completa de programas, de llamadas a agentes o modelos ni de la memoria máxima, porque esos sistemas aún no existen en la toolchain.
