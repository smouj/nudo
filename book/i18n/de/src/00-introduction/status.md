# 00.3 — Aktueller Projektstatus

> **Status:** Implementiert (IMPLEMENTED) / Geplant (PLANNED)
> **Summary:** Das Repository verfügt über ein funktionsfähiges Compiler-Frontend und eine teilweise Typprüfung; Programmausführung und Effektprüfung sind weiterhin künftige Meilensteine.

Der folgende Status spiegelt die Roadmap des Repositorys wider. „Implementiert“ bedeutet, dass es für den genannten Umfang Code und Tests gibt. Es bedeutet nicht, dass die gesamte Sprache oder ein benutzerorientierter Ablauf fertig ist.

| Meilenstein | Stand | Verfügbar |
| --- | --- | --- |
| M0 — Repository-Grundlage | Abgeschlossen | Workspace, Beitragsregeln, CI und Dokumentationsinfrastruktur |
| M1 — Quellmodell und Lexer | Abgeschlossen | Quelldateien, Spans, Tokens, lexikalische Diagnostik und Fehlerbehandlung |
| M2 — Parser und Syntaxbaum | Abgeschlossen | Verlustfreie Syntax, Parser, typisierter AST und Syntaxdiagnostik |
| M3.0 — Semantische Weichenstellungen | Abgeschlossen | Angenommene Entscheidungen in Spezifikation und NEPs festgehalten |
| M3.1 — HIR und Namensauflösung | Abgeschlossen | HIR-Absenkung, Scopes, Namensauflösung und NDO2001–NDO2003 |
| M3.2 — Typsystem | In Arbeit | Teilweiser Typechecker für angenommene Regeln, einschließlich typisiertem Enum- und <code>Result</code>-Matching; von der CLI aufgerufen |
| M3.3 — Vertrauens-Typen | Teilweise | Typidentität und die angenommene Zuweisungsregel von Verified zum inneren Typ |
| M4 — Interpreter | Geplant | Erstes durchgängiges Programm und <code>nudo run</code> |
| M5 — Effektsystem | Geplant | Effektinferenz, -prüfung und Diagnostik; die Semantik ist weiterhin vorgeschlagen |

Der CLI-Befehl <code>nudo check</code> lädt Quellcode, analysiert ihn lexikalisch und syntaktisch, löst Namen auf und prüft die bisher implementierten angenommenen Typregeln. Er kann Tokens, Syntaxbäume und Namensauflösungen anzeigen. Ein fehlerfreier Lauf bedeutet, dass die implementierten Stufen keine Fehler gemeldet haben; nicht unterstützte Regeln, Ausführbarkeit und Runtime-Sicherheit sind damit nicht nachgewiesen.

Das Typechecker-Crate deckt primitive und Literaltypen, Annotationen, Aufrufe, begrenzte generische Inferenz, Feldzugriffe, ausgewählte Arithmetik, Bool-Bedingungen, typisiertes Enum- und <code>Result</code>-Matching sowie die angenommene Zuweisung von Vertrauens-Typen ab. Sein Konformitätskorpus prüft diese Regeln zusätzlich unabhängig von der CLI. Nicht unterstützte Regeln bleiben bewusst unvollständig, bis ihre semantischen Verträge bereit sind.

M4 und M5 haben Abhängigkeiten. M4 benötigt die noch ausstehende Typ- und Vertrauensarbeit sowie einen echten Ausführungspfad. M5 benötigt diese Grundlagen und ein angenommenes Design des Effektsystems. Die Roadmap definiert den Abschluss über funktionierende Exit-Kriterien und Konformitätsbelege, nicht über ein Kalenderdatum.

## Warum Statuslabels wichtig sind

Ein ausgefeiltes Beispiel kann wie eine ausgelieferte Funktion wirken. Deshalb kennzeichnet jedes Kapitel, ob es implementiertes Verhalten, angenommenes Design, einen Vorschlag oder geplante Arbeit beschreibt. Die <a href="https://github.com/smouj/nudo/blob/main/ROADMAP.md">Roadmap</a> ist das maßgebliche, fortlaufend gepflegte Statusregister.
