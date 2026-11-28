# 03.1 — Compiler-Pipeline

> **Status:** Implementiert (IMPLEMENTED) / Geplant (PLANNED)
>
> **Summary:** Das NUDO-Frontend besteht aus klar getrennten Stufen; HIR und Namensauflösung sind implementiert, spätere Prüf- und Ausführungsstufen befinden sich noch in Arbeit.

{{#include ../diagrams/pipeline.svg}}

*Der hervorgehobene Pfad zeigt die Frontend-Arbeit, die von der CLI verwendet
wird. Die Typprüfung ist teilweise implementiert; Effektprüfung, MIR und
Ausführung sind geplant.*

~~~text
SourceFile → Lexer → Lossless syntax tree → AST → HIR / name resolution
                                                        ↓
                              partieller Typechecker für angenommene Regeln
                                                        ↓
                                    Effekte → MIR → Interpreter / Backend
~~~

Der CLI-Befehl <code>nudo check</code> lädt den Quellcode, führt Lexing und Parsing aus, senkt nach HIR ab beziehungsweise löst Namen auf und prüft die bisher implementierten angenommenen Typregeln. Er kann Tokens, den Syntaxbaum und Namensauflösungen ausgeben. Er führt den Quellcode nicht aus.

Das Crate <code>nudo-hir</code> senkt den typisierten AST auf Bezeichner und aufgelöste Definitionen ab. Das Crate <code>nudo-typeck</code> verarbeitet dieses HIR und implementiert einen Teil der angenommenen Typregeln. Die CLI ruft es nach erfolgreicher Namensauflösung auf; Unit-Tests und das Typeck-Konformitätskorpus prüfen es ebenfalls direkt. Draft-Regeln bleiben unimplementiert.

Der geplante Effektprüfer, MIR, Interpreter und das WebAssembly-Backend sind nicht verfügbar. Ihre Blöcke im Diagramm zeigen die vorgesehene Architektur und die Reihenfolge der Roadmap; sie versprechen nicht, dass ein Programm diese Stufen heute durchlaufen kann.

Explizite Stufengrenzen machen jede Transformation testbar und erlauben es künftigen Tools, Repräsentationen gemeinsam zu nutzen, statt Quellcode mit unabhängigen Annahmen erneut zu parsen. In der Prä-Alpha-Phase können sich Crate-Grenzen noch ändern, wenn weitere Implementierungsbelege vorliegen.
