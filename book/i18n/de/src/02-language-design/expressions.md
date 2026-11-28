# 02.2 — Ausdrücke und Präzedenz

> **Status:** Implementiert / Spezifiziert / Geplant (IMPLEMENTED / SPECIFIED / PLANNED)
> **Zusammenfassung:** Der Parser implementiert die Grammatik; der Typchecker prüft außerdem Operatoren, `if`-Zweige und typisiertes Enum-`match`. Für die Indizierung fehlt noch eine semantische Entscheidung.

Präzedenz ist keine Formatierungsfrage. `a + b * c` muss in jeder konformen
Implementierung genau eine Bedeutung haben, und der Parser erzeugt genau diese
Form: der Baum für `1 + 2 * 3` hat die Multiplikation als inneren Knoten und die
Addition darüber.

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

## Zentrale Regeln für Operatoren und `if`

[NEP-0017](https://github.com/smouj/nudo/blob/main/neps/0017-boolean-comparison-conditionals.md)
legt die zulässigen Operatortypen und das Ergebnis von Bedingungen fest:

- `!`, `&&` und `||` verwenden `Bool`; `&&` und `||` werten von links nach
  rechts mit Kurzschluss aus. Ungültige Operanden erzeugen `NDO2018` am Operator.
- Gleichheit vergleicht übereinstimmende primitive Ansichten von `Int`,
  `Float`, `Bool` oder `Text`. Ordnung ist nur für zwei `Int`- oder zwei
  `Float`-Werte definiert. Es gibt keine Zahlenumwandlung und keine
  benutzerdefinierten Überladungen. Die akzeptierte Ansicht `Verified<T>` als
  `T` gilt auch für Operanden.
- `Float`-Vergleiche folgen IEEE 754: NaN ist nicht gleich sich selbst und
  ungeordnet; positives und negatives Null sind gleich.
- Die Bedingung von `if` ist `Bool`. Mit `else` prüft ein erwarteter Typ beide
  Zweige; andernfalls müssen sie genau denselben Typ haben. Eine Abweichung
  erzeugt `NDO2019` am `else`-Zweig.
- Ein `if` ohne `else` hat den Typ `Unit`; auch der `then`-Zweig muss `Unit` sein.

Zur Laufzeit werden der übersprungene Operand einer Kurzschlussauswertung und
der nicht gewählte Zweig nicht ausgeführt. Die statische Effektprüfung
berücksichtigt sie dennoch alle, damit eine Deklaration für jede Eingabe gilt.

## Typisiertes Enum-Matching

[NEP-0020](https://github.com/smouj/nudo/blob/main/neps/0020-typed-enum-matching.md)
definiert `match` für benutzerdefinierte Enums und das eingebaute
`Result<T, E>`. Jeder Zweig nennt eine deklarierte Variante oder verwendet `_`;
Payloads werden nach Position mit ihren deklarierten Typen gebunden, nachdem die
generischen Argumente eingesetzt wurden. Alle Varianten müssen abgedeckt sein,
sofern kein erreichbarer Wildcard die übrigen Fälle abdeckt. Unbekannte Namen
sind Fehler und werden niemals zu Bindungen. Ohne erwarteten Ergebnistyp müssen
die Zweige exakt denselben Typ haben; mit einem erwarteten Typ gelten die normalen
Zuweisungsregeln für jeden Ausdruck. Der Typechecker meldet Muster-, Abdeckungs-
und Zweigfehler und prüft dennoch jeden Zweigkörper weiter.

```nudo
fn describe(value: Result<Int, Text>) -> Text {
    match value {
        Ok(number) => "success"
        Err(reason) => reason
    }
}
```

Parser und Typechecker haben Konformitätsfälle für generische Payloads,
`Result`, Wildcards, ungültige Muster, Fehlerbehandlung und Zweigtypen. Die CLI
wendet die implementierten angenommenen Regeln nach erfolgreicher
Namensauflösung an.
