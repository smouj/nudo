# 03.1 — Compiler-Pipeline

> **Status:** Implementiert / Spezifiziert (IMPLEMENTED / SPECIFIED)  
> **Summary:** NUDO ist in explizite Compiler-Stufen gegliedert, damit jede Transformation eine testbare Verantwortung hat.


{{#include ../diagrams/pipeline.svg}}

*Die Pipeline, wobei die Akzentfarbe markiert, was heute existiert: das Laden von
Quelltext, den Lexer, den verlustfreien Syntaxbaum, den Parser, den typisierten AST
und Diagnostik.*

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

Die ersten vier Stufen sind implementiert und durch Konformitätsfälle abgedeckt:
das Lexing, der verlustfreie Syntaxbaum, das Parsen und der typisierte AST. `nudo
check` durchläuft sie alle, weshalb ein sauberer Lauf jetzt „es lexikalisiert und es
parst" bedeutet.

Scharfe Stufengrenzen verbessern das Debugging und erlauben es Formatter, Language
Server und Dokumentationswerkzeugen, stabile Repräsentationen zu teilen, statt Text
mit unabhängigen Annahmen neu zu parsen.

Die Architektur sollte während der Prä-Alpha-Phase flexibel bleiben.
Platzhalter-Crates sind kein Grund, eine API einzufrieren, bevor die Implementierung
dem Projekt zeigt, wo die richtige Grenze liegt.
