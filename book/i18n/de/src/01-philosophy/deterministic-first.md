# 01.1 — Deterministische Software zuerst

> **Status:** Spezifiziert  
> **Summary:** Gewöhnlicher deterministischer Code muss der einfache, vorhersagbare Fall bleiben, obwohl NUDO für Systeme des KI-Zeitalters entworfen ist.

Eine Sprache für Agenten, die einfache Software umständlich macht, löst das falsche
Problem. NUDO behandelt daher Arithmetik, Kontrollfluss, Funktionen, Daten und
Module als Fundament. Agentische Konstrukte sitzen auf diesem Fundament auf.

## Determinismus um Wahrscheinlichkeit herum

Die beabsichtigte Form ist:

```text
validated deterministic input
        ↓
probabilistic operation
        ↓
Generated<T>
        ↓
explicit validation / policy / approval
        ↓
deterministic continuation
```

Das Modell darf unsicher sein. Das Programm darf nicht so tun, als sei diese
Unsicherheit verschwunden, nur weil eine Antwort erfolgreich geparst wurde.
