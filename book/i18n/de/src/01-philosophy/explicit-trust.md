# 01.2 — Explizites Vertrauen

> **Status:** Vorgeschlagen  
> **Summary:** NUDO trennt generierte und verifizierte Werte, damit Vertrauensübergänge für Prüfende und Checker sichtbar sind.

## Die zentrale Unterscheidung

```text
Generated<Article>
        ↓ verify
Verified<Article>
```

Ein generierter Wert ist nicht automatisch ein verifizierter Wert, und Parsen ist
nicht dieselbe Operation wie Verifikation.

Das verhindert einen der häufigsten begrifflichen Zusammenbrüche in modelllastigen
Systemen: dass „die Ausgabe hatte die richtige Form" zu „auf die Ausgabe kann man
sich verlassen" wird.

## Was Verifikation nicht ist

Sie ist nicht als magischer Cast gedacht. Ein Verifier kann fehlschlagen.
Verifikation sollte Provenienz hinterlassen, die erklärt, welche Eingabe, welche
Regel, welche Modellausgabe und welcher Verifier den vertrauenswürdigen Wert
erzeugt haben.
