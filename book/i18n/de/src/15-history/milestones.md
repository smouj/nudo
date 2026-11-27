# 15.1 — Meilensteine und Nachweise

> **Status:** Historisch (HISTORICAL) / Fortlaufend
> **Summary:** NUDO misst Fortschritt anhand beleggestützter Meilensteine; M3 ist weiterhin in Arbeit, während der für M5 benötigte Interpreter und das Effektsystem geplant sind.

Die Roadmap misst Fortschritt anhand von Exit-Kriterien und Konformitätsbelegen statt anhand von Kalenderdaten. Diese Seite fasst den Weg bis M5 zusammen; maßgeblich bleibt die Roadmap des Repositorys.

{{#include ../diagrams/status.svg}}

| Meilenstein | Stand | Exit-Nachweis |
| --- | --- | --- |
| M0 — Repository-Grundlage | Abgeschlossen | Ein frischer Checkout lässt sich bauen, prüfen und begutachten |
| M1 — Quellmodell und Lexer | Abgeschlossen | Deterministische Tokens, Diagnostik und Fehlerbehandlung |
| M2 — Parser und Syntaxbaum | Abgeschlossen | Verlustfreies Parsing, Fehlerbehandlung und Konformitätsfälle |
| M3.0 — Semantische Weichenstellungen | Abgeschlossen | Grundlegende Entscheidungen des Typsystems vor der Implementierung dokumentiert |
| M3.1 — HIR und Namensauflösung | Abgeschlossen | Aufgelöstes HIR und stabile Namensdiagnostik |
| M3.2 — Grundlegendes Typsystem | In Arbeit | Angenommene Regeln kommen schrittweise mit Konformitätsbelegen hinzu |
| M3.3 — Vertrauens-Typen | Teilweise | Typidentität und Verified-Zuweisung existieren; verify und Provenienz fehlen |
| M4 — Interpreter | Geplant | Das Beispielprogramm berechnet und druckt 42 mit <code>nudo run</code> |
| M5 — Effektsystem | Geplant | Ein Aufruf mit Effekt aus einer effektfreien Funktion wird samt Effektpfad abgelehnt |

M4 und M5 sind keine aufeinanderfolgenden Bezeichnungen für austauschbare Arbeit. M4 erfordert einen tatsächlichen Ausführungspfad vom Quellcode bis zur Ausgabe. M5 baut auf Typ- und Vertrauensgrundlagen auf und erfordert angenommene Effektsemantik. Ein Draft-Vorschlag kann nicht als Implementierungsvertrag dienen.

## Was die Meilensteine absichern

Die Reihenfolge verhindert, dass eine ausgefeilte Demo Compiler-Garantien ersetzt. Bevor M5 als abgeschlossen gilt, müssen die vorangehenden Typ- und Vertrauenskriterien erfüllt sein, M4 muss das dokumentierte Beispiel ausführen, die Semantik des Effektsystems muss angenommen sein und Diagnostik sowie Konformitätsfälle müssen das Exit-Kriterium belegen.

Die Roadmap verspricht kein Kalenderdatum für M5. Der Fortschritt wird anhand dessen gemeldet, was sich bauen lässt und die festgelegten Tests besteht.
