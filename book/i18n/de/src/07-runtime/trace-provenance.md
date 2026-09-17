# 07.1 — Traces und Provenienz

> **Status:** Vorgeschlagen  
> **Summary:** Wichtige autonome Ausführung sollte genügend strukturierte Belege hinterlassen, um zu rekonstruieren, was geschehen ist, ohne Logs rückwärts zu entschlüsseln.

Ein Trace ist eine Ausführungserzählung. Provenienz sind an Werte gebundene
Herkunftsinformationen. Sie überschneiden sich, sind aber nicht identisch.

Ein brauchbarer Provenienzeintrag für einen verifizierten Wert muss möglicherweise
das Quellmodell bzw. Tool, relevante Eingaben, den Verifikationsschritt,
Policy-Entscheidungen und Freigaben benennen. Das genaue Schema bleibt eine
Implementierungs- und Spezifikationsaufgabe; die Invariante ist, dass Vertrauen nicht
von den Belegen getrennt werden sollte, die es erzeugt haben.

Traces sollten in ihrer Struktur deterministisch sein, wo das möglich ist, auch wenn
der Modellinhalt innerhalb eines Ereignisses probabilistisch ist.
