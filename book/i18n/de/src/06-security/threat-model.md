# 06.2 — Bedrohungsmodell

> **Status:** Spezifiziert / Geplant  
> **Summary:** NUDO behandelt Modellausgabe und externe Tool-Eingabe als nicht vertrauenswürdig, bis explizite Regeln Daten über Vertrauensgrenzen bewegen.

Das Bedrohungsmodell umfasst Prompt-Injection, bösartige Tool-Ausgabe,
Capability-Eskalation, Exfiltration von Secrets, unbeschränkte Ausführung,
Provenienzverlust und Fehler in Laufzeit-Adaptern.

Sprachprüfungen allein können kein Betriebssystem absichern. Statische Effektregeln
müssen mit Laufzeitdurchsetzung und Sandboxing kombiniert werden, wo Seiteneffekte
Prozess- oder Hostgrenzen überschreiten.

Eine ausgereifte Implementierung sollte positive und negative Garantien testen:
nicht nur, dass erlaubte Programme funktionieren, sondern dass verwehrte Effekte
fehlschlagen, *bevor* der Effekt eintritt.
