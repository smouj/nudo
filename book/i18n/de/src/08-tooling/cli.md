# 08.1 — Toolchain und CLI

> **Status:** Implementiert / Geplant  
> **Summary:** Das Kommando `nudo` soll einen zusammenhängenden Einstiegspunkt bieten: heute für lexikalische Prüfungen, später für Build-, Run-, Format-, Test-, Audit- und Trace-Abläufe.

Derzeit implementierte Kommandos umfassen die lexikalische Prüfung und grundlegende
CLI-Metadaten. Künftige Roadmap-Kommandos umfassen build, run, REPL, formatter,
documentation, testing, evaluation, doctor/audit und tracing.

Die Kommandooberfläche sollte vorhersagbar bleiben: stabile Exit-Codes,
maschinenlesbare Ausgabe, wo sie nötig ist, Diagnosecodes und keine stille
Netzwerkaktivität.
