# 06.1 — Capabilities, Effekte und Autorität

> **Status:** Vorgeschlagen (PROPOSED) / Geplant (PLANNED)
>
> **Summary:** NUDO schlägt statische Effektverfolgung und explizite Capability-Autorität vor; die aktuelle Toolchain implementiert weder Effektprüfung noch Runtime-Durchsetzung von Capabilities.

{{#include ../diagrams/authority.svg}}

*Dies ist das vorgeschlagene Autoritätsmodell und keine Sicherheitsgrenze,
die von der heutigen Runtime durchgesetzt wird.*

## Drei verwandte Konzepte

- **Effekt:** Statische Beschreibung dessen, was eine Berechnung tun kann, etwa das Netzwerk nutzen.
- **Capability-Erteilung:** Autorität, die einem Programm oder Ausführungskontext zur Verfügung steht.
- **Policy:** Regeln, die einschränken, wann oder wie Autorität ausgeübt werden darf.

Das Design verknüpft den von einer Funktion benötigten Effekt mit der Autorität, die am Aufrufort verfügbar ist. Es soll standardmäßig verweigern und verhindern, dass ein delegiertes Tool oder ein Agent die Autorität des Aufrufers erweitert.

## Aktueller Implementierungsstand

M5 ist geplant. NEP-0004 befindet sich in der Diskussion; das Effektkapitel bleibt ein Vorschlag. Es gibt heute weder Effektinferenz noch Effektprüfung, eine Implementierung von <code>NDO3xxx</code> oder Runtime-Durchsetzung von Capabilities. Das Crate <code>nudo-effects</code> markiert eine Grenze in der Roadmap, kein fertiges Subsystem.

Das folgende Beispiel veranschaulicht den diskutierten Vorschlag. Es ist keine angenommene Syntax, auf die sich Nutzer verlassen können:

~~~nudo
fn fetch(url: Text) -> Text with Network {
    // Vorgeschlagen: Ein Aufruf von fetch benötigt Network.
}
~~~

Das Exit-Kriterium von M5 in der Roadmap ist, einen Aufruf mit Effekt aus einer effektfreien Funktion abzulehnen und eine Diagnose auszugeben, die sowohl den Effekt als auch den verursachenden Aufrufpfad nennt. Das Design muss angenommen werden, bevor dieser Prüfer implementiert werden kann. Runtime-Policy-Durchsetzung ist ein separater späterer Meilenstein.
