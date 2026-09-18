# 08.1 — Toolchain und CLI

> **Status:** Implementiert / Geplant (IMPLEMENTED / PLANNED)  
> **Summary:** Das Kommando `nudo` bietet einen zusammenhängenden Einstiegspunkt: heute vom Lexing und Parsen bis später zu Build-, Run-, Format-, Test-, Audit- und Trace-Abläufen.

Derzeit implementierte Kommandos umfassen `check` — das lexikalisiert und parst, mit
`--dump-tokens` und `--dump-tree` — sowie grundlegende CLI-Metadaten. Künftige
Roadmap-Kommandos umfassen build, run, REPL, formatter, documentation, testing,
evaluation, doctor/audit und tracing.

Die Kommandooberfläche sollte vorhersagbar bleiben: stabile Exit-Codes,
maschinenlesbare Ausgabe, wo sie nötig ist, Diagnosecodes und keine stille
Netzwerkaktivität.
