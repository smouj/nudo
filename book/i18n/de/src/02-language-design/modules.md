# 02.3 — Module und Programm-Grenzen

> **Status:** Vorgeschlagen  
> **Summary:** Module müssen explizite Namensräume und reproduzierbare Programmstruktur bieten, ohne Autoritäts- oder Abhängigkeitsgrenzen zu verbergen.

Moduldesign ist eng mit Paketdesign, Namensauflösung und öffentlichen Schnittstellen
verbunden. NUDO sollte implizite globale Namensräume vermeiden und die öffentliche
API-Oberfläche bewusst gestalten.

Offene Entwurfsarbeit umfasst Paketmanifeste, Sichtbarkeit,
Abhängigkeitsauflösung und die Frage, wie von Abhängigkeiten deklarierte
Capabilities den Aufrufern sichtbar gemacht werden.
