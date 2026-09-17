# 11.1 — Warum Rust und warum ein eigener Compiler?

> **Status:** Spezifizierte Begründung  
> **Summary:** Rust bietet ein starkes Implementierungssubstrat, während ein eigener Compiler notwendig ist, wenn NUDO-Semantik unabhängig von einer anderen Hostsprache existieren soll.

## Warum Rust für die Implementierung

Rust bietet explizites Ownership, starke statische Prüfung, gute Performance,
ausgereifte Parser- und Compiler-Werkzeuge und eine praktikable
plattformübergreifende Distribution. Das macht Rust nicht zu einem Teil der
NUDO-Sprachsemantik; Rust ist die Implementierungssprache.

## Warum nicht nach Python kompilieren als Definition?

Wenn NUDO-Semantik einfach Python-Semantik plus Syntax wäre, würde Python zur
eigentlichen Spezifikation. Merkmale wie Vertrauenstypen, Capability-Prüfungen und
stabile Compiler-Diagnostik würden Beschränkungen der Hostsprache erben.

Ein eigener Compiler lässt die NUDO-Spezifikation die Autorität bleiben. Backends
können sich später ändern, ohne neu zu definieren, was ein Programm bedeutet.
