# 03.4 — Diagnostik als Produktdesign

> **Status:** Spezifiziert / Geplant  
> **Summary:** Compiler-Fehler sollten die Regel, die Quellposition und die Kette erklären, die die Anforderung eingeführt hat.

Diagnostik ist Teil der Spracherfahrung, kein Debug-Dump.

Ein hochwertiger Capability-Fehler sollte sowohl die Aufrufstelle als auch den Grund
benennen:

```text
error[NDO3xxx]: missing capability `Network`

  ┌─ src/main.nudo:14:5
  │
14│     fetch(url)
  │     ^^^^^^^^^^ requires Network
  │
  └─ current task was not granted Network
```

Bei Vertrauenstypen sollten Fehler erklären, dass `Generated<T>` und `Verified<T>`
absichtlich verschieden sind, und auf die explizite Verifikationsgrenze verweisen,
statt einen unsicheren Cast zu empfehlen.
