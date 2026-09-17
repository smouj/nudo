# 03.3 — Parser, verlustfreie Syntax und AST

> **Status:** Geplant  
> **Summary:** M2 soll den Token-Strom in eine verlustfreie Syntaxstruktur überführen, sich von fehlerhafter Eingabe erholen und einen typisierten AST für spätere semantische Stufen bereitstellen.

## Warum zuerst verlustfreie Syntax

Ein verlustfreier Baum behält Trivia und jedes Quellbyte. Diese Eigenschaft ist
wichtig für einen Formatter, Editor-Werkzeuge und automatisierte Agenten, die Code
ändern müssen, ohne Kommentare oder Layout stillschweigend zu zerstören.

## Fehlererholung

Ein brauchbarer Parser hört nicht beim ersten fehlenden Token auf. Erholungspunkte
sollten Elementgrenzen, Semikolons und schließende Begrenzer umfassen, damit ein
Fehler nicht den Rest einer Datei in Rauschen verwandelt.

## Typisierter AST

Der AST sollte nachgelagerten Durchläufen semantische Formen (`Function`, `Call`,
`Type`) geben, ohne dass diese rohe Details von Syntaxknoten kennen müssen.
Konvertierungen von Syntax zu AST müssen Span-erhaltend bleiben.
