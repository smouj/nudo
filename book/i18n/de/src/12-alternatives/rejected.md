# 12.1 — Bewusst verworfene Alternativen

> **Status:** Historisch / Entwurfsbegründung  
> **Summary:** Mehrere einfachere Ansätze sind in Anwendungen nützlich, erfüllen aber nicht NUDOs Ziel sprachsichtbaren Vertrauens und sprachsichtbarer Autorität.

| Alternative | Warum sie als Sprachmodell nicht ausreicht |
| --- | --- |
| Prompts als gewöhnliche Strings | Der Compiler kann aus Prosa kein Vertrauen und keine Autorität ableiten. |
| Berechtigungen nur in der Deployment-Konfiguration | Aufrufe im Quelltext können nicht gegen den Gewährungspfad geprüft werden. |
| `verified: Bool`-Flags | Sicherheit wird zu einer optionalen Laufzeitkonvention. |
| Anbieterspezifische Schlüsselwörter | Koppelt die Sprachbedeutung an Anbieter-APIs. |
| Unbegrenzte Agenten-Schleife | Macht Autorität, Budget und Terminierung schwer prüfbar. |
| Provenienz nur im Log | Rekonstruiert Belege nachträglich und kann Werteherkunft verlieren. |

Diese Techniken können weiterhin in Laufzeit-Adaptern oder Bibliotheken vorkommen.
Die Ablehnung hier bedeutet „nicht das semantische Fundament der Sprache", nicht
„niemals nützlich."
