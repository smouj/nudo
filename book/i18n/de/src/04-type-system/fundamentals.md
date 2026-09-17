# 04.1 — Grundlagen des Typsystems

> **Status:** Vorgeschlagen  
> **Summary:** Das Typsystem bevorzugt explizite öffentliche Signaturen, nominale Beziehungen und compiler-sichtbare Vertrauens- und Effektinformationen.

Aktuelle Entwurfsvorgaben umfassen primitive Skalartypen, Structs, Enums, Sequenzen,
Funktionen, Generics und optionale bzw. Result-artige Formen.

Öffentliche Schnittstellen sollten annotiert sein. Lokale Inferenz darf Rauschen
reduzieren, aber öffentliche API-Typen sollten nicht heimlich von
Implementierungsdetails abhängen.

Offene Fragen wie Integer-Breite, Überlauf-Semantik, Varianz und die genaue
Darstellung von zur Laufzeit bereitgestellten Capabilities müssen bewusst
entschieden werden.
