# 00.2 — Warum es NUDO gibt

> **Status:** Spezifiziert  
> **Summary:** Das Projekt beginnt bei einem Problem der Compiler-Sichtbarkeit: Kritischer Programmzustand im KI-Zeitalter ist häufig in Strings, Dashboards und Konventionen verborgen.

## Die Sichtbarkeitslücke

Betrachtet man eine typische Agentenanwendung: Ein Prompt ist ein String. Ein
Tool-Schema ist JSON. Berechtigungen liegen in der Konfiguration. Token- oder
Geldbudgets liegen in einer Anbieterkonsole. Menschliche Freigabe findet im Chat
statt. Traces werden nach der Ausführung rekonstruiert.

Jeder Teil kann funktionieren, aber die Programmiersprache hat meist keine
einheitliche Möglichkeit, über sie zu schließen.

## NUDOs These

Wenn Vertrauen, Autorität, Effekte, Budgets und Provenienz explizit dargestellt
werden, können manche Fehler von Konventionen zu Compiler- und Laufzeitprüfungen
wandern.

Das macht probabilistische Ausgabe **nicht** deterministisch. Es macht den
deterministischen Code um probabilistische Ausgabe herum expliziter und prüfbarer.

## Entwurfsdruck

Die Sprache optimiert daher auf:

- deterministische gewöhnliche Programme zuerst;
- keine ambient Authority als Standard;
- explizite Vertrauensübergänge;
- Anbieterunabhängigkeit;
- Local-first-Ausführung;
- Traces und Provenienz als normale Ausführungsprodukte.
