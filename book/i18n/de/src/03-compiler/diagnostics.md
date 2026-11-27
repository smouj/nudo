# 03.4 — Diagnostik als Produktgestaltung

> **Status:** Implementiert (IMPLEMENTED) / Geplant (PLANNED)
> **Summary:** NUDO-Diagnostik verwendet stabile Codes und Quell-Spans; die implementierten Gruppen decken lexikalische, syntaktische, Namensauflösungs- und teilweise Typfehler ab.

Diagnostik gehört zum Spracherlebnis und ist kein Debug-Dump. Das Crate <code>nudo-diagnostics</code> stellt strukturierte Diagnosen mit Schweregrad, Code, Quellposition, Hinweisen und Hilfetext bereit. Der Renderer wird gemeinsam genutzt, damit jede Compilerstufe Fakten melden kann, ohne ihre Ausgabe selbst zu formatieren.

## Implementierte Diagnosegruppen

- <code>NDO1xxx</code> umfasst lexikalische und Syntaxfehler. Lexer und Parser erholen sich, soweit möglich, und melden Quell-Spans.
- <code>NDO2001</code>–<code>NDO2003</code> umfasst Namensauflösungsfehler, die <code>nudo check</code> ausgibt.
- <code>NDO2004</code>–<code>NDO2007</code> und <code>NDO2017</code> betreffen implementierte Typprüfregeln. Die Typechecker-Tests legen strukturierte Erwartet/Gefunden-Hinweise und Quellpositionen für diese Regeln fest.

Der Typechecker ist teilweise implementiert und wird noch nicht von der CLI aufgerufen. Ein fehlerfreies <code>nudo check</code> besagt daher, dass der Quellcode Lexing, Parsing und Namensauflösung bestanden hat; es bedeutet nicht, dass er die Typprüfung bestanden hat.

## Geplante Diagnosegruppen

Die Gruppe des Effektsystems <code>NDO3xxx</code> ist für M5 geplant. Heute gibt es keinen Effektprüfer, der diese Diagnosen ausgibt. Das folgende Beispiel ist ein veranschaulichendes Ziel und keine Ausgabe der aktuellen Toolchain:

~~~text
error[NDO3xxx]: capability Network fehlt

  --> src/main.nudo:14:5
   |
14 |     fetch(url)
   |     ^^^^^^^^^^ benötigt Network
   |
   = note: der Aufrufpfad benötigt Network
~~~

Die angenommene Diagnosespezifikation legt Struktur und Quellanforderungen fest. Syntax und Prüfregeln des Effektsystems hängen weiterhin davon ab, dass ein Vorschlag angenommen wird.

## Qualität der Diagnostik

Eine hilfreiche Diagnose erklärt, welche Regel verletzt wurde, wo der Quellcode sie verletzt hat und welche Anforderung zum Fehler führte. Bei Vertrauens-Typ-Konflikten sollte sie erklären, dass <code>Generated&lt;T&gt;</code> und <code>Verified&lt;T&gt;</code> verschieden sind, und den Wert benennen, der die Zusicherung verletzt. Sie sollte keine unsichere Umwandlung vorschlagen.
