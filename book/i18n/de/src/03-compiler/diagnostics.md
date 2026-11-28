# 03.4 — Diagnostik als Produktgestaltung

> **Status:** Implementiert (IMPLEMENTED) / Geplant (PLANNED)
> **Summary:** NUDO-Diagnostik verwendet stabile Codes und Quell-Spans; die implementierten Gruppen decken lexikalische, syntaktische, Namensauflösungs- und teilweise Typfehler ab.

Diagnostik gehört zum Spracherlebnis und ist kein Debug-Dump. Das Crate <code>nudo-diagnostics</code> stellt strukturierte Diagnosen mit Schweregrad, Code, Quellposition, Hinweisen und Hilfetext bereit. Der Renderer wird gemeinsam genutzt, damit jede Compilerstufe Fakten melden kann, ohne ihre Ausgabe selbst zu formatieren.

## Implementierte Diagnosegruppen

- <code>NDO1xxx</code> umfasst lexikalische und Syntaxfehler. Lexer und Parser erholen sich, soweit möglich, und melden Quell-Spans.
- <code>NDO2001</code>–<code>NDO2003</code> umfasst Namensauflösungsfehler, die <code>nudo check</code> ausgibt.
- <code>NDO2004</code>–<code>NDO2019</code> betreffen implementierte Typprüfregeln. <code>NDO2010</code> markiert eine Ganzzahl-Magnitude, die keinen <code>Int</code> ergeben kann; genau <code>2^63</code> ist nur direkt unter unärem <code>-</code> für <code>Int::MIN</code> zulässig. Gemäß NEP-0020 decken <code>NDO2011</code>–<code>NDO2016</code> Enum- und <code>Result</code>-Muster, Payload-Arity, Vollständigkeit, Erreichbarkeit und Armtypen ab. Die Tests legen strukturierte Hinweise und Quellpositionen fest.

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
