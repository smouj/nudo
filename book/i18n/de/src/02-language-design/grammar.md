# 02.1 — Grammatik und Syntaxdisziplin

> **Status:** Vorgeschlagen  
> **Summary:** Die Grammatik ist ein Vertrag zwischen Quelltext und Parser; vorläufige Agenten-Syntax muss sichtbar vorläufig bleiben, bis sie akzeptiert ist.

## Kleine Syntax, starke Semantik

NUDOs Entwurfsziel ist nicht, für jedes KI-Konzept ein Schlüsselwort zu erfinden.
Syntax ist teuer, weil jede neue Form das Parsen, die Werkzeuge, die Formatierung,
das Lernen und die Kompatibilität betrifft.

## Parser-Druck

Ein Recursive-Descent-Parser arbeitet am besten, wenn die Ausdrucksgrammatik
Präzedenz explizit macht und Linksrekursion vermeidet. Eine robuste
Ausdruckshierarchie sollte konzeptionell so aufgebaut sein:

```text
logical-or
  → logical-and
  → equality
  → comparison
  → additive
  → multiplicative
  → unary
  → postfix
  → primary
```

Postfix-Parsing ist der Ort, an dem sich Aufrufe, Feldzugriff und Indexierung
natürlich zusammensetzen.

## Kontextabhängige versus reservierte Wörter

Nur Wörter, die tatsächlich als reservierte Tokens akzeptiert werden, sollten vom
Lexer als reserviert behandelt werden. Künftige Syntax wie `agent`, `task`, `ask`
und `verify` braucht eine Entscheidung auf NEP-Ebene, bevor Werkzeuge sich darauf
verlassen.
