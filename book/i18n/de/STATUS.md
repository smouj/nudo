# de — Übersetzungsstatus

**Kanonische Quelle:** das englische Verzeichnis <code>book/src</code>.  
**Abdeckung:** 34 von 34 Markdown-Seiten.  
**Quellnachverfolgung:** <code>translation.json</code> hält den Hash jedes englischen Kapitels fest.

Die deutsche Ausgabe ist mit den englischen Kapiteln dieser Revision abgeglichen. Diagramme werden mit der kanonischen Ausgabe geteilt; ihre Include-Pfade bleiben erhalten. Die englische Spezifikation bleibt die normative Quelle. Diese Übersetzung legt keine Sprachregeln fest.

Namen, Bezeichner, Befehle, NDO-Codes, Pfade und Codebeispiele behalten ihre kanonische Form. Statuslabels bewahren den englischen Rang in Großbuchstaben in Klammern gemäß [Übersetzungsvertrag](../README.md) und [redaktionellem Vertrag](../../EDITORIAL.md).

Diese Aktualisierung behauptet keine unabhängige redaktionelle Prüfung durch eine muttersprachliche Person. Die Hashes belegen die Übereinstimmung mit der Quellrevision, nicht allein die sprachliche Qualität.

## Prüfung

~~~sh
python3 book/scripts/check_translations.py --strict
python3 book/scripts/build_site.py de --strict
~~~
