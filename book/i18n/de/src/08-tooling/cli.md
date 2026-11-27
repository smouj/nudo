# 08.1 — Toolchain und CLI

> **Status:** Implementiert (IMPLEMENTED) / Geplant (PLANNED)
> **Summary:** Der aktuelle Befehl <code>nudo</code> prüft Quellcode, Syntax und Namen; Typprüfung, Ausführung, Effekte und die übrige Toolchain bleiben künftige Arbeit.

## Implementierte Befehle

~~~sh
nudo check path/to/file.nudo
nudo check --dump-tokens path/to/file.nudo
nudo check --dump-tree path/to/file.nudo
nudo check --dump-resolutions path/to/file.nudo
nudo --version
nudo --help
~~~

Der Befehl check lädt eine oder mehrere Quelldateien, führt Lexing und Parsing aus, senkt akzeptierte Syntax auf HIR ab, löst Namen auf und meldet lexikalische, Syntax- und Namensauflösungsdiagnosen. Außerdem kann er einen normalen oder CI-orientierten Bericht ausgeben.

Sein Umfang endet bei der Namensauflösung. Ein fehlerfreies Ergebnis garantiert weder Typkorrektheit noch Ausführbarkeit des Programms. Der teilweise Typechecker ist ein separates Crate mit Unit-Tests und Konformitätsfällen; die CLI ruft ihn nicht auf. <code>nudo run</code>, <code>nudo build</code>, REPL, Formatter, Language Server und Paketmanager existieren noch nicht.

Die Exit-Codes sind stabil: 0 bedeutet, dass die implementierten Stufen keine Fehlerdiagnose gemeldet haben; 1 bedeutet, dass mindestens ein Fehler gemeldet wurde; 2 bedeutet einen Nutzungs- oder Eingabefehler oder einen nicht implementierten Befehl.

## Frontend-Messungen

Das separate Programm <code>nudo-bench</code> misst vier deterministische synthetische Eingaben in vier Stufen: Lexing, Parsing, HIR-Absenkung und Typprüfung. Das ergibt sechzehn Zeilen. Quell- und HIR-Eingaben für spätere Stufen werden außerhalb ihrer Messbereiche vorbereitet; Parse enthält das Tokenisieren, da der öffentliche Parser Tokens verarbeitet.

Der Typechecker meldet die Länge seiner Ausdruckstyp-Tabelle. Diese Zahl enthält Einträge ohne Typ und zählt nicht die ausgeführten semantischen Regeln. Gleiche Zahlen beweisen nicht, dass zwei Typechecker-Versionen gleich viel Arbeit geleistet haben.

{{#include ../diagrams/benchmark-portability.svg}}

*Die gemessene Zeit hängt vom gesamten erfassten Aufbau ab. Die Skalierung
zwischen Rechnern lässt sich nicht zuverlässig allein aus CPU-Takt oder
Kernzahl ableiten.*

Ein Lauf erfasst Commit, Änderungsstatus, Rust-Version, Betriebssystem, Architektur, nach Möglichkeit das CPU-Modell, Eingabegröße, Iterationen und Timing-Statistiken. Es gibt weder eine projektweite Referenzbasis noch eine Performance-Schranke in CI.

Für brauchbare Vergleiche baue beide Revisionen im Release-Modus, verwende identische Arbeitslasten und Iterationseinstellungen und miss sie auf demselben möglichst unbelasteten Rechner. Wiederhole die Läufe. Behandle überschrittene Schwellen als Anlass zur Untersuchung; eine einzelne Stichprobe beweist keine Regression.

Ergebnisse von einem Rechner beschreiben diesen Rechner und diese Arbeitslast. Sie können bei der Planung als grober Anhaltspunkt dienen; für eine Schätzung auf einem anderen Rechner sind dort Messungen mit demselben Commit, derselben Toolchain, Arbeitslast und denselben Einstellungen nötig. Verschiedene CPUs, Speichersysteme, Scheduler und Dauertaktraten können die Stufen unterschiedlich verändern. Daher gibt es keine verlässliche Ein-Zahl-Umrechnung zwischen Rechnern.

Die Suite misst nur Compiler-Frontendstufen. Sie kann weder Interpreterstart noch durchgängige Programmausführung, Agenten- oder Modellaufrufe oder Spitzen-Speicherverbrauch angeben, weil diese Systeme noch nicht Teil der Toolchain sind.
