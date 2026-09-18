# 03.4 — Diagnostik als Produktdesign

> **Status:** Implementiert / Geplant (IMPLEMENTED / PLANNED)  
> **Summary:** Compiler-Fehler erklären die Regel, die Quellposition und die Kette, die die Anforderung eingeführt hat.

Diagnostik ist Teil der Spracherfahrung, kein Debug-Dump. Die lexikalischen und
syntaktischen Familien (`NDO1xxx`) sind implementiert: eine Datei wird mit einem
stabilen Code, einem exakten Span und einem Erwartet/Gefunden-Paar zurückgewiesen.
Die Typ-, Effekt- und Capability-Familien sind geplant, und die Kette, die sie
zeigen müssen, ist in `spec/errors.md` spezifiziert.

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
