# 04.1 — Grundlagen des Typsystems

> **Status:** Implementiert (IMPLEMENTED) / Spezifiziert (SPECIFIED) / Geplant (PLANNED)
>
> **Summary:** NUDO besitzt einen teilweise implementierten Typechecker für angenommene Regeln; das vollständige Typsystem ist weiterhin in Arbeit und noch nicht Teil des CLI-Befehls <code>check</code>.

Das Typsystem ist in den Sprachkapiteln und angenommenen NEPs spezifiziert. Das Rust-Crate <code>nudo-typeck</code> implementiert eine wachsende Teilmenge und verarbeitet HIR statt Quelltext. Seine Tests und Konformitätsfälle prüfen die implementierten Regeln. Die CLI ruft den Typechecker noch nicht auf.

## Vom aktuellen Typechecker implementierte Regeln

| Regel | Aktuelles Verhalten |
| --- | --- |
| Primitive und Literaltypen | <code>Int</code>, <code>Float</code>, <code>Bool</code>, <code>Text</code> und <code>Unit</code> werden repräsentiert |
| Annotationen und Funktionssignaturen | Werte werden mit angegebenen Typen verglichen; Funktionstypen und Aufrufe werden geprüft |
| Aufrufe | Arität verwendet <code>NDO2005</code>, inkompatible Argumente <code>NDO2004</code>, nicht aufrufbare Werte <code>NDO2006</code> |
| Generische Aufrufe | Typparameter werden aus Argumenten und bereits bekannten erwarteten Ergebnistypen in Quellreihenfolge hergeleitet |
| Benannte Typen | Die Typargument-Arität verwendet <code>NDO2017</code>; nominale Identität und unterstützter Feldzugriff werden geprüft |
| Arithmetik | Übereinstimmende numerische Operanden unterstützen <code>+</code>, <code>-</code>, <code>*</code> und <code>/</code>; <code>%</code> gilt nur für Int, unäres <code>-</code> erhält den Zahlentyp |
| Bedingungen | Die Bedingung von <code>if</code> muss <code>Bool</code> sein; die Zweigausdrücke werden weiterhin besucht |
| Vertrauenszuweisung | Das eingebaute <code>Verified&lt;T&gt;</code> kann an unterstützten Stellen verwendet werden, an denen <code>T</code> erwartet wird |

Die generische Inferenz ist bewusst begrenzt. Sie löst keine Typen für das gesamte Programm, generalisiert keine lokale Bindung und errät keinen ungelösten Parameter. Generische Argumente bleiben invariant und numerische Konvertierungen erfolgen nicht implizit.

## Noch unvollständige Regeln

Der Typechecker definiert noch keinen Ergebnistyp für <code>if</code>, keine Übereinstimmung zwischen Zweigwerten, nicht alle Operatoren, keine Indizierung, nominale Konstruktion, <code>match</code> oder <code>Result&lt;T, E&gt;</code>-Matching. Auch eine stabile Diagnose für einen ungelösten generischen Parameter steht noch aus. [NEP-0015](https://github.com/smouj/nudo/blob/main/neps/0015-uninferable-generic-parameters.md) schlägt diesen Diagnosevertrag vor; die NEP ist jedoch noch ein Draft und weder angenommen noch implementiert. Ganzzahl-Literalbereiche sowie Agenten, Tasks und Tools werden in diesem Typechecker noch nicht implementiert.

Diese Lücken sind beabsichtigt. Eine Regel gilt erst dann als implementiert, wenn ihr semantischer und diagnostischer Vertrag angenommen wurde. Roadmap und Spezifikation halten fest, welche Arbeit bereit ist und welche Entscheidungen noch offen sind.

## Was eine erfolgreiche CLI-Prüfung bedeutet

<code>nudo check</code> führt Lexing, Syntaxanalyse und Namensauflösung aus. Da der teilweise Typechecker nicht in diesen Befehl eingebunden ist, belegt ein erfolgreicher CLI-Lauf nicht, dass alle Typregeln erfüllt sind. Der Typechecker wird derzeit mit eigenen Tests und dem Typkonformitätskorpus geprüft.
