# Einführung

> **Status:** Historisch (HISTORICAL) / Fortlaufend
> **Summary:** Ein beleggestützter Überblick über NUDOs Design, den heute vorhandenen Compiler und die Runtime, auf die das Projekt hinarbeitet.

<div class="status-key">
DOCUMENT: NUDO-BOOK-001 · EDITION: ENGINEERING PAPER · REVISION: R0 · STATUS: LIVING
</div>

NUDO ist ein allgemeines Programmiersprachenprojekt. Sein Design soll gewöhnliche deterministische Programme um begrenzte probabilistische Arbeit ergänzen und Datentrust, benötigte Autorität und Verifikation im Programm sichtbar halten. Das Projekt ist Prä-Alpha: Es besitzt ein funktionsfähiges Frontend und ein teilweise implementiertes Typechecker-Crate, führt NUDO-Programme aber noch nicht aus.

Dieses Buch erläutert Motivation, angenommene Sprachregeln, Compilerdesign, implementiertes Verhalten, offene Vorschläge und geplante Runtime. Es ist umfassender als die normative Spezifikation; jedes Beispiel muss daher mit Blick auf seinen Status gelesen werden. Die Spezifikation definiert die Sprache, dieses Buch fügt keine Regeln hinzu.

## Wie dieses Buch zu lesen ist

Für das Design beginne mit Philosophie und Vertrauen. Zum implementierten Compiler lies Sprachdesign → Compiler → Typsystem. Für die künftige Agenten-Runtime lies Agenten und KI → Sicherheit → Runtime. Die Meilensteintabelle unterscheidet funktionierenden Code, angenommene Regeln und Vorschläge.

## Was heute funktioniert

- <code>nudo check</code> lädt <code>.nudo</code>-Dateien, lexikalisiert und parst sie, löst Namen auf und meldet lexikalische, Syntax- und Namensauflösungsdiagnosen.
- <code>--dump-tokens</code>, <code>--dump-tree</code> und <code>--dump-resolutions</code> zeigen stabile Ansichten dieser Stufen.
- Die Rust-Crates enthalten einen teilweise implementierten Typechecker. Seine implementierten Regeln werden mit Unit-Tests und dem Typkonformitätskorpus geprüft, der Checker ist aber noch nicht in <code>nudo check</code> eingebunden.
- Die Benchmarksuite misst synthetische Arbeitslasten für Lexer, Parser, HIR-Absenkung und Typprüfung. Sie misst keine Ausführung von NUDO-Programmen.

<code>nudo run</code>, <code>nudo build</code>, Interpreter, Effektprüfer, Agenten-Runtime, Tool-Aufruf, Policy-Durchsetzung, Provenienzspeicher und WebAssembly-Backend gibt es noch nicht.

## Aktueller Projektstatus

{{#include diagrams/status.svg}}

*Meilensteinstatus und Abhängigkeiten bis M5. Die Hervorhebung kennzeichnet
abgeschlossene oder aktive Implementierung; abgedunkelte Zeilen sind geplant.
Die <a href="https://github.com/smouj/nudo/blob/main/ROADMAP.md">Roadmap</a>
ist maßgeblich, falls diese Übersicht und das Repository voneinander abweichen.*

M3 ist in Arbeit: HIR und Namensauflösung sind implementiert, während Typprüfung und Vertrauens-Typen nur angenommene Teilregeln abdecken. M4 (das erste ausführbare Programm) und M5 (Effektprüfung) sind weiterhin geplant. M5 hängt vom Abschluss früherer Meilensteine und der Annahme der Effektsemantik ab.

## Autorität

Die Spezifikation definiert die Sprache. Angenommene NEPs ändern sie. Konformitätstests legen beobachtbares Verhalten fest. Dieses Buch erläutert diese Regeln und kennzeichnet künftige Arbeit; es stuft einen Vorschlag nie zu einer implementierten Fähigkeit hoch.
