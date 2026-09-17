# 06.1 — Capabilities, Effekte und Autorität

> **Status:** Vorgeschlagen  
> **Summary:** Effekte beschreiben, was eine Ausführung tun darf; Capabilities beschreiben, welche Autorität der aktuelle Kontext tatsächlich gewähren kann.


{{#include ../diagrams/authority.svg}}

*Autorität verengt sich entlang einer Delegationskette und kann nicht zurückfließen.*

Diese Konzepte müssen zueinander in Beziehung stehen, ohne verwechselt zu werden.

- **Effekt**: eine statische Beschreibung, dass eine Berechnung eine Operation wie
  Netzwerkzugriff ausführen kann.
- **Capability-Gewährung**: einem Kontext zur Verfügung gestellte Autorität.
- **Policy**: Regeln, die einschränken, wann oder wie diese Autorität ausgeübt
  werden darf.

Als Standard ist die Verweigerung gedacht. Code sollte nicht allein deshalb
Netzwerk-, Dateisystem-, Shell- oder Secret-Zugriff erhalten, weil der Host-Prozess
ihn zufällig besitzt.

Eine kritische Invariante ist die Nicht-Verstärkung: Ein Tool oder ein delegierter
Agent darf seine eigene Autorität nicht über das hinaus erweitern, was der Aufrufer
gewährt hat.
