# 03.1 — Compiler-Pipeline

> **Status:** Spezifiziert  
> **Summary:** NUDO ist in explizite Compiler-Stufen gegliedert, damit jede Transformation eine testbare Verantwortung hat.


{{#include ../diagrams/pipeline.svg}}

*Die Pipeline, wobei die Akzentfarbe markiert, was heute existiert.*

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

Scharfe Stufengrenzen verbessern das Debugging und erlauben es Formatter, Language
Server und Dokumentationswerkzeugen, stabile Repräsentationen zu teilen, statt Text
mit unabhängigen Annahmen neu zu parsen.

Die Architektur sollte während der Prä-Alpha-Phase flexibel bleiben.
Platzhalter-Crates sind kein Grund, eine API einzufrieren, bevor die Implementierung
dem Projekt zeigt, wo die richtige Grenze liegt.
