# 05.1 — Agenten als begrenzte Ausführer

> **Status:** Vorgeschlagen  
> **Summary:** Ein Agent soll ein deklarierter Ausführer mit expliziten Tools, Autorität, Grenzen und Abnahmekriterien sein, nicht eine unbeschränkte Prompt-Schleife.

Agentendeklarationen sind nur nützlich, wenn sie etwas mitteilen, das Compiler oder
Laufzeit durchsetzen können. Ihr Zweck ist daher nicht kosmetische Organisation.

Wer prüft, sollte beantworten können:

- welche Tools kann dieser Agent aufrufen?
- welche Capabilities können diese Tools verbrauchen?
- welche Modellschnittstelle steht zur Verfügung?
- welches Budget begrenzt die Ausführung?
- welcher Ausgabetyp wird erwartet?
- welche Verifikation oder Freigabe ist vor der Fortsetzung erforderlich?

Anbieterspezifische Prompt- und Anfragedetails bleiben Laufzeitangelegenheiten.
