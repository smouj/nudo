# 03.1 — Compiler-Pipeline

> **Status:** Implementiert (IMPLEMENTED) / Geplant (PLANNED)
>
> **Summary:** Das NUDO-Frontend besteht aus klar getrennten Stufen; HIR und Namensauflösung sind implementiert, spätere Prüf- und Ausführungsstufen befinden sich noch in Arbeit.

{{#include ../diagrams/pipeline.svg}}

*Der hervorgehobene Pfad zeigt die implementierte Frontend-Arbeit. Die Typprüfung
ist teilweise implementiert und wird separat getestet; Effektprüfung, MIR und
Ausführung sind geplant.*

~~~text
SourceFile → Lexer → Lossless syntax tree → AST → HIR / name resolution
                                                        ↓
                                 Teil-Typechecker (nicht in nudo check)
                                                        ↓
                                    Effekte → MIR → Interpreter / Backend
~~~

Der CLI-Befehl <code>nudo check</code> lädt den Quellcode, führt Lexing und Parsing aus und senkt nach HIR ab beziehungsweise löst Namen auf. Er kann Tokens, den Syntaxbaum und Namensauflösungen ausgeben. Er ruft <code>nudo-typeck</code> nicht auf und führt den Quellcode nicht aus.

Das Crate <code>nudo-hir</code> senkt den typisierten AST auf Bezeichner und aufgelöste Definitionen ab. Das Crate <code>nudo-typeck</code> verarbeitet dieses HIR und implementiert einen Teil der angenommenen Typregeln. Unit-Tests und das Typeck-Konformitätskorpus prüfen es direkt. Die übrigen Regeln müssen spezifiziert werden, bevor sie implementiert werden.

Der geplante Effektprüfer, MIR, Interpreter und das WebAssembly-Backend sind nicht verfügbar. Ihre Blöcke im Diagramm zeigen die vorgesehene Architektur und die Reihenfolge der Roadmap; sie versprechen nicht, dass ein Programm diese Stufen heute durchlaufen kann.

Explizite Stufengrenzen machen jede Transformation testbar und erlauben es künftigen Tools, Repräsentationen gemeinsam zu nutzen, statt Quellcode mit unabhängigen Annahmen erneut zu parsen. In der Prä-Alpha-Phase können sich Crate-Grenzen noch ändern, wenn weitere Implementierungsbelege vorliegen.
