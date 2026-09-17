# 03.1 — El pipeline del compilador

> **Status:** Especificado  
> **Summary:** NUDO se estructura en fases explícitas del compilador para que cada transformación tenga una responsabilidad comprobable.


{{#include ../diagrams/pipeline.svg}}

*El pipeline, con el acento marcando lo que existe hoy.*

```text
SourceFile
   ↓
Lexer → Tokens + lexical diagnostics
   ↓
Lossless Syntax Tree
   ↓
Typed AST
   ↓
HIR + name resolution
   ↓
Type checking
   ↓
Effect checking
   ↓
MIR
   ↓
Interpreter / code generation
```

Unos límites nítidos entre fases mejoran la depuración y permiten que el
formateador, el servidor de lenguaje y las herramientas de documentación compartan
representaciones estables en lugar de reanalizar el texto con supuestos
independientes.

La arquitectura debería seguir siendo flexible durante la fase pre-alpha. Los crates
de relleno no son motivo para congelar una API antes de que la implementación enseñe
al proyecto dónde está el límite correcto.
