# 04.2 — `Generated<T>` und `Verified<T>`

> **Status:** Vorgeschlagen  
> **Summary:** Die Vertrauenstypen sollen verhindern, dass probabilistische Ausgabe durch eine implizite Konvertierung zu vertrauenswürdigen Anwendungsdaten wird.


{{#include ../diagrams/trust-flow.svg}}

*Eine Modellausgabe wird nur durch einen expliziten Schritt zu einem verifizierten Wert.*

## Warum nicht `T`?

Wenn ein Modell direkt ein `Article` zurückgibt, verschwindet die Provenienz
darüber, wie dieser Wert erzeugt wurde, aus der Typgrenze.

## Warum nicht `Result<T, E>`?

`Result` kann Erfolg/Fehlschlag einer Operation darstellen. Es stellt nicht den
Vertrauensstatus eines erfolgreichen Werts dar. Eine perfekt geparste Modellantwort
kann trotzdem unverifiziert sein.

## Warum kein Boolesches Flag?

```text
ModelOutput<T> { value: T, verified: Bool }
```

Das verschiebt die Regel in eine Laufzeitkonvention. Ein Aufrufer kann vergessen,
das Flag zu prüfen. Verschiedene Typen erlauben es dem Checker, den fehlenden
Übergang unmöglich zu ignorieren.

## Notwendige Entwurfsfragen

Vor der Stabilisierung muss NUDO Verifier-Fehlertypen, Komposition, Provenienzerhalt,
Serialisierungsgrenzen und die Wechselwirkung mit Generics und Pattern Matching
spezifizieren.
