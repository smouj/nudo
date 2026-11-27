# 08.2 — Frontend-Benchmarking

> **Status:** Implementiert (IMPLEMENTED) / Geplant (PLANNED)
>
> **Summary:** NUDO verfügt über eine kleine, reproduzierbare Frontend-Benchmarksuite; ihre Ergebnisse beschreiben die erfasste Arbeitslast und den Host, keine universelle Geschwindigkeitsrangliste.

Das Benchmarkprogramm misst Compilerstufen, die heute existieren. Es misst keine Ausführung von NUDO-Programmen, da Interpreter und <code>nudo run</code> noch fehlen.

## Arbeitslasten und Stufengrenzen

Die Suite erzeugt vier deterministische synthetische Eingaben: Funktionsdeklarationen, let-Bindungen, Textliterale und kommentarlastigen Quellcode. Jede wird in vier Stufen gemessen, insgesamt also sechzehn Zeilen:

| Stufe | Gemessene Arbeit |
| --- | --- |
| Lex | Eine registrierte Quelldatei tokenisieren |
| Parse | Tokenisieren und mit dem öffentlichen Parser parsen |
| HIR | Einen vorbereiteten Syntaxbaum absenken |
| Typechecker | Vorbereitetes HIR prüfen |

Die Parsermessung enthält die Tokenisierung, weil das Parsing einen Tokenstrom benötigt. HIR-Absenkung und Typprüfung bereiten ihre Eingabe vor der Messung vor, sodass die Werte diese Stufen isolieren. Die Arbeitslasten sind absichtlich klein und synthetisch; sie bilden weder alle Sprachfunktionen noch eine vollständige Anwendung ab.

Die Einheit des Typecheckers ist die Länge der Ausdruckstyp-Tabelle einschließlich Einträgen ohne Typ. Sie zählt weder ausgeführte Regeln noch erfolgreich geprüfte Ausdrücke. Gleiche Einheitenzahlen belegen keine gleiche semantische Arbeit.

## Was ein Ergebnis erfasst

Ein Lauf erfasst Quellrevision und Änderungsstatus, Rust-Compiler, Betriebssystem, Architektur, wenn verfügbar die CPU-Beschreibung, Iterationszahlen, Eingabebytes, Ausgabeeinheiten und Timing-Statistiken. Der Bericht enthält Minimum, Median, 95. Perzentil und Mittelwert; Lexer- und Parserzeilen enthalten zusätzlich den Durchsatz. Es gibt weder eine projektweite Referenzbasis noch eine automatische Performance-Schranke in CI.

## Ergebnisse vergleichen

Verwende Release-Builds. Vergleiche dieselben Arbeitslasten, Stufengrenzen, Iterationseinstellungen und dieselbe Compiler-Toolchain. Führe beide Revisionen unter ähnlich ruhigen Bedingungen auf demselben Rechner aus, wiederhole die Läufe und bewahre die JSON-Berichte bei der geprüften Änderung auf. Ein einzelner Vergleich ist ein Untersuchungssignal; wiederhole ihn, bevor du eine Zeitdifferenz dem Code zuschreibst.

Der Typechecker änderte in PR #31 seine semantische Abdeckung. Eine Typprüfmessung vor und nach dieser Änderung kann zwei Implementierungen beschreiben, die unterschiedliche Arbeit leisten, selbst wenn Quellcode und Ausdruckstabellenlänge übereinstimmen. Allein daraus lässt sich keine Regression oder Verbesserung bei gleichwertiger Arbeit ableiten.

## Messungen auf verschiedenen Rechnern

Ein Ergebnis von einem Rechner ist ein Messpunkt für genau diesen Rechner und Aufbau. Es kann grob bei der Planung helfen, lässt sich aber nicht zuverlässig anhand von Taktfrequenz, Kernzahl oder CPU-Modell auf eine andere CPU umrechnen. Mikroarchitektur, Speicher und Cache, dauerhafte Leistungsgrenzen, Betriebssystemplanung, Compiler-Version und Hintergrundlast können die Stufen unterschiedlich beeinflussen.

Für eine Schätzung auf dem Zielrechner führe denselben Commit und dieselben Arbeitslasten dort aus, erfasse die Metadaten und wiederhole die Messungen. Ist das nicht möglich, kennzeichne jede Schätzung als grobe Extrapolation und gib Annahmen und Unsicherheit an. Stelle sie nicht als Benchmark-Ergebnis dar.

{{#include ../diagrams/benchmark-portability.svg}}

*Das Diagramm zeigt, weshalb Messwerte ihren Aufbau mitführen und ein zweites
Hardwareziel einen eigenen Lauf benötigt.*

Die aktuellen Kommandozeilenoptionen und das JSON-Berichtsformat stehen in der <a href="https://github.com/smouj/nudo/blob/main/benchmarks/README.md">Benchmark-README</a>.
