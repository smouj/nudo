# 15.1 — Meilensteine und Belege

> **Status:** Historisch / Lebend  
> **Summary:** NUDO verwendet belegbasierte Meilensteine statt Kalenderversprechen; ein Meilenstein ist abgeschlossen, wenn seine Exit-Kriterien und Konformitätsbelege vollständig sind.

Die Roadmap beginnt mit der Repository-Grundlage und der lexikalischen Pipeline und
führt dann über Parser/AST, Typsystem, Interpreter, Effekte, Vertrauenstypen,
Agenten-Laufzeit, Capabilities/Policies, Interoperabilität, Tooling und WebAssembly.

Diese Reihenfolge spiegelt Abhängigkeiten wider, nicht Marketingpriorität. Eine
Agenten-Laufzeit, die vor den Fundamenten für Parser, Typen und Effekte gebaut würde,
würde semantische Entscheidungen in Laufzeitcode zwingen und spätere
Compiler-Prüfungen schwerer sauber definierbar machen.
