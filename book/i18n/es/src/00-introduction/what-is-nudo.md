# 00.1 — ¿Qué es NUDO?

> **Status:** Especificado (SPECIFIED) / Implementado (IMPLEMENTED)
>
> **Summary:** NUDO es un proyecto de lenguaje de propósito general cuyo diseño hace explícitas la confianza y la autoridad; su implementación actual es un compilador pre-alpha con un front end funcional.

## El modelo en una frase

NUDO pretende conectar código determinista, personas, agentes, herramientas y modelos mediante un sistema común de tipos, permisos y verificación.

## El problema que aborda

En muchas aplicaciones con agentes, un prompt es una cadena, el esquema de una herramienta es JSON, los permisos residen en la configuración, los presupuestos en la consola del proveedor y la verificación en el código de la aplicación. Cada parte puede funcionar, pero el compilador no ve cómo se relacionan.

NUDO explora cómo hacer visibles esos límites en el lenguaje y el runtime. Un tipo puede distinguir datos generados de datos verificados. Una firma de efectos podría expresar qué autoridad necesita una función. Un runtime podría registrar qué entradas y comprobaciones produjeron un resultado. Son objetivos de diseño; aún no son todas funciones disponibles.

## Qué existe hoy

La CLI puede cargar archivos fuente, tokenizarlos, analizarlos en un árbol sintáctico sin pérdidas, resolver nombres y aplicar las reglas de tipos aceptadas que están implementadas. Los crates de HIR, resolución de nombres y comprobación parcial de tipos están implementados. El comprobador también tiene sus propias pruebas y casos de conformidad.

La toolchain actual no ejecuta programas NUDO. Todavía no hay intérprete, comprobador de efectos, proveedor de modelos, runtime de agentes, sandbox de herramientas, almacén de procedencia ni backend WASM. El <a href="status.md">estado del proyecto</a> y la <a href="https://github.com/smouj/nudo/blob/main/ROADMAP.md">hoja de ruta</a> distinguen el front end funcional de los subsistemas planificados.

## Qué hace distinto al proyecto

NUDO trata el software determinista como la vía normal y el trabajo probabilístico como una operación acotada con un límite de confianza visible. Una respuesta de un modelo no debería convertirse en datos fiables de la aplicación solo porque tenga la forma adecuada. Una herramienta no debería obtener autoridad solo porque el proceso anfitrión la tenga.

El compilador y el runtime pretenden que esas afirmaciones puedan revisarse y, cuando exista la implementación, hacerlas cumplir.

## Qué no es

NUDO no es un SDK de modelos, un framework de prompts, un DSL específico de un proveedor ni una nueva capa sobre Python. Los formatos de petición de cada proveedor corresponden a adaptadores del runtime; la composición de prompts suele corresponder a bibliotecas. NUDO tampoco está listo para producción: el lenguaje, el compilador y el runtime están incompletos y siguen sujetos a cambios mediante el proceso de especificación.

## Fuente de verdad

La especificación define el lenguaje. Las NEP aceptadas la modifican. Las pruebas fijan el comportamiento observable de la implementación. El libro explica motivaciones y estado, pero no resuelve de forma implícita una cuestión del lenguaje que siga abierta.
