# 03.2 — Quellmodell und Lexer

> **Status:** Implementiert  
> **Summary:** Der Lexer ist die erste echte Sprachimplementierungsschicht und für deterministische Tokenisierung sowie behebbare lexikalische Diagnostik verantwortlich.

Ein robuster Lexer muss Byte-Positionen, die Zeilen-/Spaltenzuordnung und Quell-IDs
erhalten, damit spätere Diagnosen präzise auf den Originalquelltext verweisen können.

Wichtige Eigenschaften sind:

- UTF-8-Laden von Quelltext;
- deterministische Token-Ströme;
- Regeln für verschachtelte und Block-Kommentare gemäß der lexikalischen
  Spezifikation;
- Erholung von fehlerhafter Eingabe statt Panik beim ersten Fehler;
- stabile Diagnosecodes;
- Konformitätsfälle, die eine andere Implementierung reproduzieren kann.

Der derzeitige Erfolgspfad von `nudo check` ist rein lexikalisch. Ein sauberes
Ergebnis heißt noch nicht, dass ein Programm syntaktisch oder semantisch gültig ist.
