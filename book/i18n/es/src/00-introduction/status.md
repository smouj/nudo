# 00.3 — Estado actual del proyecto

> **Status:** Implementado / Planificado  
> **Summary:** El repositorio es deliberadamente honesto sobre la distancia entre las fases del compilador ya implementadas y las fases futuras que solo están diseñadas.

## Base implementada

La ruta que funciona está centrada actualmente en la carga del código fuente, los
spans, el análisis léxico, los tokens, los diagnósticos y el comportamiento léxico
de `nudo check`. El proyecto cuenta además con infraestructura de repositorio, CI,
seguridad, gobernanza, conformidad y documentación.

## Ruta planificada del compilador

```text
Source
  ↓
Lexer             implemented
  ↓
Lossless syntax   planned
  ↓
Parser / AST      planned
  ↓
HIR / typecheck   planned
  ↓
Effect checking   planned
  ↓
MIR
  ↓
Interpreter / WASM
```

## Por qué importan las etiquetas de estado

Un libro sobre un lenguaje en fase pre-alpha puede volverse engañoso con facilidad,
porque los ejemplos cuidados parecen reales. Por eso cada capítulo lleva un estado,
y los ejemplos de sintaxis futura se describen como propuestas cuando corresponde.
