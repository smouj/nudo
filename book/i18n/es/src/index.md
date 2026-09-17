# El Libro Técnico de NUDO

> **Status:** Histórico / Vivo (HISTORICAL / LIVING)  
> **Summary:** Una explicación estructurada de qué es NUDO, por qué existe y cómo se pretende que encajen su compilador, su modelo de confianza y su runtime orientado a agentes.

<div class="status-key">
DOCUMENT: NUDO-BOOK-001 · EDITION: ENGINEERING PAPER · REVISION: R0 · STATUS: LIVING
</div>

NUDO es un proyecto de lenguaje de programación de propósito general cuyo diseño
trata los agentes, las herramientas, los modelos, los permisos, los presupuestos,
la verificación y la procedencia como algo sobre lo que el lenguaje y el runtime
deberían poder razonar de forma explícita.

Este libro es explicativo. Es deliberadamente más amplio que la especificación
normativa: recoge la motivación, los compromisos de diseño, la arquitectura, las
alternativas descartadas y ejemplos prácticos. Nunca debe convertir en silencio una
característica propuesta en una implementada.

## Cómo leer este libro

Si se quiere entender la **idea**, conviene empezar por Filosofía y Confianza. Si
se quiere trabajar en el compilador, seguir Diseño del lenguaje → Compilador →
Sistema de tipos. Si se quiere entender la seguridad de los agentes, leer Agentes
e IA → Seguridad → Runtime.

## La realidad actual

{{#include diagrams/status.svg}}

*Lo que la cadena de herramientas hace hoy y lo que solo está escrito. La misma
tabla se mantiene en [`ROADMAP.md`](https://github.com/smouj/nudo/blob/main/ROADMAP.md), que prevalece si las
dos no coinciden.*


En la revisión R0, NUDO está en fase pre-alpha. La base del repositorio y el
pipeline léxico son la capa de implementación madura. El parser, el AST tipado, el
comprobador de tipos, el intérprete, el comprobador de efectos, el runtime de
agentes, el sistema de políticas y el backend WASM son trabajo de diseño y de hoja
de ruta, no afirmaciones de producción.

## Autoridad

La especificación define el lenguaje. Los NEP aceptados la modifican. Las pruebas de
conformidad demuestran la concordancia observable. Este libro explica esas reglas;
no las anula.
