# 02.2 — Ausdrücke und Präzedenz

> **Status:** Spezifiziert (SPECIFIED)  
> **Summary:** Präzedenz, Assoziativität und das Verbot von Linksrekursion sind entschieden und mechanisch geprüft. Der Parser, der sie umsetzt, ist M2.

Präzedenz ist keine Formatierungsfrage. `a + b * c` muss in jeder konformen
Implementierung genau eine Bedeutung haben.

Diese Fragen sind nicht mehr offen. Sie stehen genau einmal als Kette von
Grammatik-Ebenen in
[`grammar/nudo.ebnf`](https://github.com/smouj/nudo/blob/main/grammar/nudo.ebnf),
wortgleich in die beiden Dokumente kopiert, die sie beschreiben, und mechanisch
geprüft von
[`scripts/check-grammar.py`](https://github.com/smouj/nudo/blob/main/scripts/check-grammar.py):

- Postfix-Operationen binden stärker als unäre Operatoren und sie verketten, sodass
  `a.b(c)[d]` ein einziger Ausdruck ist;
- Multiplikation und Division binden stärker als Addition und Subtraktion, beide
  linksassoziativ;
- Vergleiche verketten **nicht**: `a < b < c` ist ein Syntaxfehler und nicht
  `(a < b) < c`; Vergleiche werden mit `&&` kombiniert;
- boolesche Operatoren haben feste Präzedenz, `||` am schwächsten;
- es gibt keine Zuweisungsebene, weil noch offen ist, ob NUDO überhaupt veränderliche
  Bindungen hat. Die Ebene jetzt zu schreiben hieße, eine Sprachfunktion zu erfinden,
  um eine Lücke in einem Diagramm zu füllen;
- `ask`, `verify` und `delegate` sind primäre Ausdrücke, keine Operatoren — genau das
  hält `verify draft with V` parsebar: `with` ist kein Operator, und keine
  Postfix-Form beginnt damit.

Die Regel, die das prüfbar statt erstrebenswert macht, ist **keine Linksrekursion**.
Ein Recursive-Descent-Parser kann keine Produktion verarbeiten, die eine Form
ableitet, die mit sich selbst beginnt — die erste Fassung dieser Grammatik hatte vier
davon. Das ist jetzt ein Build-Fehler und keine Entdeckung, die noch bevorsteht.

Der Formatter sollte genug Syntax ausgeben, um Bedeutung zu erhalten, ohne sich auf
menschliche Intuition zu verlassen.
