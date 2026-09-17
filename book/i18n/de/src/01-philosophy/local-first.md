# 01.3 — Local-first und anbieterunabhängig

> **Status:** Spezifiziert  
> **Summary:** Lokale Modelle, lokale Tools und Offline-Ausführung sind als erstklassig gedacht und nicht als herabgestufte Betriebsarten.

Ein Anbietername sollte nicht allein deshalb in der Kernsprache auftauchen, weil
dieser Anbieter heute populär ist. Anbieter wechseln; Sprachsemantik sollte das
nicht.

NUDO trennt daher eine Modellabstraktion von anbieterspezifischen Adaptern. Ein
lokales Modell sollte dieselbe Laufzeitschnittstelle erfüllen können wie ein
entferntes Modell, vorbehaltlich Capabilities und Policy.

Local-first verbessert außerdem die Testbarkeit: Deterministische Testumgebungen und
lokale Modell-Fixtures können ein Programm prüfen, ohne bezahlte externe
Infrastruktur vorauszusetzen.
