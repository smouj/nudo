# 05.2 — Tasks, Tools und Modelle

> **Status:** Vorgeschlagen  
> **Summary:** NUDO trennt Zielsetzungen, effektbehaftete Tools und Modellanbieter, damit Autonomie begrenzt werden kann, ohne einen Anbieter in die Sprache einzubacken.


{{#include ../diagrams/interfaces.svg}}

*Die vier Deklarationen und das, was jede von ihnen einschränkt.*

## Task

Ein Task ist eine Einheit autonomer Arbeit mit Zielsetzung, erwartetem Ergebnis,
Grenzen und Nachvollziehbarkeit. Anders als ein normaler Funktionsaufruf soll ein
Task Belege darüber liefern, wie die Arbeit ausgeführt wurde.

## Tool

Ein Tool verbindet Code mit einer externen Capability: Netzwerk, Dateisystem, Shell,
Git oder ein anderer Dienst. Tool-Aufrufe sind daher effektbehaftet und an
Capabilities gebunden.

## Model

Ein Model ist eine anbieterneutrale Laufzeitschnittstelle. API-Felder und
Authentifizierung eines Anbieters sollten die Bedeutung eines NUDO-Programms nicht
verändern.
