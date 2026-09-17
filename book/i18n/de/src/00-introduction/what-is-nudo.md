# 00.1 — Was ist NUDO?

> **Status:** Spezifiziert  
> **Summary:** NUDO ist als universell einsetzbare Sprache entworfen, in der deterministischer Code und begrenzte probabilistische Arbeit nebeneinander bestehen können, ohne Vertrauens- oder Autoritätsgrenzen zu verbergen.

## Das Modell in einem Satz

NUDO verbindet Menschen, Code, Agenten, Tools und Modelle unter einem gemeinsamen
System aus Typen, Berechtigungen und Verifikation.

## Was das Projekt ungewöhnlich macht

Die meisten Anwendungssprachen können einen Modellaufruf nur als gewöhnliche
Bibliotheks-Ein-/Ausgabe darstellen. Das heißt: Der Compiler sieht einen
Funktionsaufruf und vielleicht einen JSON-Wert, aber er weiß nicht, dass der Wert
probabilistisch erzeugt wurde, welche Autorität der Aufrufer gewährt hat, welches
Budget verbraucht wurde oder ob jemand das Ergebnis verifiziert hat.

NUDO untersucht, diese Grenzen in sprachsichtbare Strukturen zu verschieben,
während gewöhnliche deterministische Programmierung der Standardweg bleibt.

## Was es nicht ist

NUDO ist kein Modell-SDK, kein Prompt-Framework, keine anbieterspezifische DSL und
keine neue Hülle über Python. Anbieterspezifische Anfrageformen gehören in
Laufzeit-Adapter. Prompt-Komposition gehört überwiegend in Bibliotheken. Die
Sprache ist zuständig für Bedeutung, Vertrauen und Autoritätsregeln, über die
unabhängige Implementierungen übereinstimmen müssen.
