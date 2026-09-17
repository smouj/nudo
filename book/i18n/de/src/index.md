# Das NUDO Technical Book

> **Status:** Historisch / Lebend  
> **Summary:** Eine strukturierte Erklärung dessen, was NUDO ist, warum es existiert und wie Compiler, Vertrauensmodell und agentenorientierte Laufzeit zusammenwirken sollen.

<div class="status-key">
DOKUMENT: NUDO-BOOK-001 · AUSGABE: ENGINEERING PAPER · REVISION: R0 · STATUS: LEBEND
</div>

NUDO ist ein Projekt für eine universell einsetzbare Programmiersprache, deren
Entwurf Agenten, Tools, Modelle, Berechtigungen, Budgets, Verifikation und
Provenienz als Dinge behandelt, über die Sprache und Laufzeit ausdrücklich
nachdenken können sollen.

Dieses Buch ist erklärend. Es ist bewusst breiter angelegt als die normative
Spezifikation: Es hält Motivation, Abwägungen, Architektur, verworfene
Alternativen und praktische Beispiele fest. Es darf ein vorgeschlagenes Merkmal
niemals stillschweigend zu einem implementierten aufwerten.

## Wie dieses Buch zu lesen ist

Wer die **Idee** verstehen will, beginnt mit Philosophie und Vertrauen. Wer am
Compiler arbeiten will, folgt Sprachdesign → Compiler → Typsystem. Wer die
Sicherheit von Agenten verstehen will, liest Agenten und KI → Sicherheit → Laufzeit.

## Aktueller Stand

{{#include diagrams/status.svg}}

*Was die Toolchain heute tut und was nur aufgeschrieben ist. Dieselbe Tabelle wird
in [`ROADMAP.md`](https://github.com/smouj/nudo/blob/main/ROADMAP.md) gepflegt, die bei Abweichungen maßgeblich ist.*


Zum Revisionstand R0 ist NUDO prä-alpha. Die Repository-Grundlage und die
lexikalische Pipeline sind die reife Implementierungsschicht. Parser, typisierter
AST, Typechecker, Interpreter, Effektprüfung, Agenten-Laufzeit, Policy-System und
WASM-Backend sind Entwurfs- und Roadmap-Arbeit und keine Aussagen über
Produktionsreife.

## Autorität

Die Spezifikation definiert die Sprache. Akzeptierte NEPs ändern sie.
Konformitätstests belegen beobachtbare Übereinstimmung. Dieses Buch erklärt diese
Regeln; es setzt sie nicht außer Kraft.
