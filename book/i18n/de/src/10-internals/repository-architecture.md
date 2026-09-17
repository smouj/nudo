# 10.1 — Repository-Architektur

> **Status:** Implementiertes Fundament  
> **Summary:** Das Monorepo hält die Entwicklung von Spezifikation, Compiler, Laufzeit und Tooling in einer prüfbaren Historie, während konzeptionelle Crate-Grenzen erhalten bleiben.

Der Workspace trennt Compiler-Stufen, Laufzeitdienste, gemeinsame Fundament-Crates,
CLI, Tooling und Backends. Das macht Verantwortlichkeiten sichtbar, aber
Crate-Grenzen in der Prä-Alpha-Phase sollten überarbeitbar bleiben.

Ein Platzhalter-Crate ist ein Roadmap-Marker, kein Beleg dafür, dass seine
öffentliche API fertig ist. Der Implementierung sollte erlaubt sein, interne
Einheiten zusammenzulegen oder zu trennen, wenn die Erfahrung eine bessere Grenze
zeigt.
