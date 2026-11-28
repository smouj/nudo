# 04.1 — Grundlagen des Typsystems

> **Status:** Implementiert (IMPLEMENTED) / Spezifiziert (SPECIFIED) / Geplant (PLANNED)
>
> **Summary:** NUDO besitzt einen teilweise implementierten Typechecker für angenommene Regeln; das vollständige Typsystem ist weiterhin in Arbeit und noch nicht Teil des CLI-Befehls <code>check</code>.

Das Typsystem ist in den Sprachkapiteln und angenommenen NEPs spezifiziert. Das Rust-Crate <code>nudo-typeck</code> implementiert eine wachsende Teilmenge und verarbeitet HIR statt Quelltext. Seine Tests und Konformitätsfälle prüfen die implementierten Regeln. Die CLI ruft den Typechecker noch nicht auf.

## Vom aktuellen Typechecker implementierte Regeln

| Regel | Aktuelles Verhalten |
| --- | --- |
| Primitive und Literaltypen | <code>Int</code>, <code>Float</code>, <code>Bool</code>, <code>Text</code> und <code>Unit</code> werden repräsentiert |
| Ganzzahl-Literalgrenzen | <code>Int::MAX</code> ist gültig; die Magnitude <code>2^63</code> ist nur direkt unter unärem <code>-</code> für <code>Int::MIN</code> zulässig; größere oder anderweitig ungültige Magnituden verwenden <code>NDO2010</code> |
| Annotationen und Funktionssignaturen | Werte werden mit angegebenen Typen verglichen; Funktionstypen und Aufrufe werden geprüft |
| Funktionsergebnisse | Ohne Ergebnisannotation ist das Ergebnis <code>Unit</code>; Funktionen mit Rückgabewert schreiben ihren Ergebnistyp, Abweichungen im Rumpf verwenden <code>NDO2004</code> |
| Aufrufe | Arität verwendet <code>NDO2005</code>, inkompatible Argumente <code>NDO2004</code>, nicht aufrufbare Werte <code>NDO2006</code> |
| Generische Aufrufe | Typparameter werden aus Argumenten und bereits bekannten erwarteten Ergebnistypen in Quellreihenfolge hergeleitet; nicht herleitbare Typparameter eines Aufrufs verwenden <code>NDO2008</code>, Signaturparameter ohne Verwendung <code>NDO2009</code> |
| Benannte Typen | Die Typargument-Arität verwendet <code>NDO2017</code>; nominale Identität und unterstützter Feldzugriff werden geprüft |
| Operatoren | Arithmetik verwendet übereinstimmende numerische Ansichten, <code>%</code> nur Int; boolesche Operatoren verwenden <code>Bool</code>, Gleichheit gleiche primitive Ansichten und Ordnung gleiches <code>Int</code> oder <code>Float</code>; ungültige Domänen melden <code>NDO2018</code> |
| Bedingungen | Die Bedingung von <code>if</code> ist <code>Bool</code>; ein erwarteter Typ prüft beide Zweige, andernfalls müssen sie genau übereinstimmen; ohne <code>else</code> sind Ergebnis und <code>then</code>-Zweig <code>Unit</code>; ein nicht kontextualisierter Konflikt meldet <code>NDO2019</code> |
| Vertrauenszuweisung | Das eingebaute <code>Verified&lt;T&gt;</code> kann an unterstützten Stellen verwendet werden, an denen <code>T</code> erwartet wird |

Die generische Inferenz ist bewusst begrenzt. Sie löst keine Typen für das gesamte Programm, generalisiert keine lokale Bindung und errät keinen ungelösten Parameter. Generische Argumente bleiben invariant und numerische Konvertierungen erfolgen nicht implizit. [NEP-0015](https://github.com/smouj/nudo/blob/main/neps/0015-uninferable-generic-parameters.md) macht diese Grenze sichtbar: <code>NDO2008</code> verweist auf einen generischen Aufruf ohne ausreichende Informationen, <code>NDO2009</code> auf einen Funktionsparameter, der in der Signatur fehlt. Äußere Typparameter bleiben beim Aufruf eines Funktionswerts fest. Die Fehlerbehandlung unterdrückt nur Diagnosen, die von einem vorherigen Fehler abhängen.

## Noch unvollständige Regeln

Sequenzindizierung und deren Laufzeitverhalten bei ungültigen Grenzen, nominale Konstruktion, <code>match</code> und <code>Result&lt;T, E&gt;</code>-Matching stehen noch aus. Die Ausführung von Verifizierern und Provenienz sowie Agenten, Tasks und Tools sind in diesem Typechecker noch nicht implementiert.

Diese Lücken sind beabsichtigt. Eine Regel gilt erst dann als implementiert, wenn ihr semantischer und diagnostischer Vertrag angenommen wurde. Roadmap und Spezifikation halten fest, welche Arbeit bereit ist und welche Entscheidungen noch offen sind.

## Was eine erfolgreiche CLI-Prüfung bedeutet

<code>nudo check</code> führt Lexing, Syntaxanalyse und Namensauflösung aus. Da der teilweise Typechecker nicht in diesen Befehl eingebunden ist, belegt ein erfolgreicher CLI-Lauf nicht, dass alle Typregeln erfüllt sind. Der Typechecker wird derzeit mit eigenen Tests und dem Typkonformitätskorpus geprüft.
