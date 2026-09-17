# 02.2 — Ausdrücke und Präzedenz

> **Status:** Offen  
> **Summary:** Präzedenz und Assoziativität von Ausdrücken müssen eindeutig sein, bevor M2 als stabil genug für nachgelagerte Werkzeuge gelten kann.

Präzedenz ist keine Formatierungs-Nebensache. `a + b * c` muss in jeder konformen
Implementierung eine einzige Bedeutung haben.

Ein Parser-Entwurf sollte diese Eigenschaften explizit machen:

- Postfix-Operationen binden stärker als unäre Operatoren;
- Multiplikation/Division binden stärker als Addition/Subtraktion;
- Vergleich und Gleichheit haben definiertes Verkettungsverhalten;
- Boolesche Operatoren haben feste Präzedenz;
- eine Zuweisung muss, falls sie eingeführt wird, ihre Assoziativität definieren;
- künftige Formen wie `ask`, `verify` oder `delegate` müssen angeben, ob sie
  Primärausdrücke, Präfixausdrücke oder Deklarationen sind.

Der Formatter sollte genug Syntax ausgeben, um die Bedeutung zu erhalten, ohne auf
menschliche Intuition angewiesen zu sein.
