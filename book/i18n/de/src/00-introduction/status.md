# 00.3 — Aktueller Projektstatus

> **Status:** Implementiert / Geplant  
> **Summary:** Das Repository ist bewusst ehrlich über die Lücke zwischen implementierten Compiler-Stufen und entworfenen künftigen Stufen.

## Implementierte Grundlage

Der funktionierende Pfad konzentriert sich derzeit auf das Laden von Quelltext,
Spans, Lexing, Tokens, Diagnostik und lexikalisches `nudo check`-Verhalten. Das
Projekt besitzt außerdem Repository-, CI-, Sicherheits-, Governance-,
Konformitäts- und Dokumentationsinfrastruktur.

## Geplanter Compiler-Pfad

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

## Warum Statuslabels wichtig sind

Ein Buch über eine Prä-Alpha-Sprache kann leicht irreführend werden, weil polierte
Beispiele echt aussehen. Jedes Kapitel trägt daher einen Status, und Beispiele
künftiger Syntax werden, wo angebracht, als vorgeschlagen beschrieben.
