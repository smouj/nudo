# 00.1 — Was ist NUDO?

> **Status:** Spezifiziert (SPECIFIED) / Implementiert (IMPLEMENTED)
>
> **Summary:** NUDO ist ein allgemeines Sprachprojekt, dessen Design Vertrauen und Autorität explizit macht; die aktuelle Implementierung ist ein Prä-Alpha-Compiler mit funktionsfähigem Frontend.

## Das Modell in einem Satz

NUDO soll deterministischen Code, Menschen, Agenten, Tools und Modelle unter einem gemeinsamen System aus Typen, Berechtigungen und Verifikation verbinden.

## Das Problem

In vielen Agentenanwendungen ist ein Prompt eine Zeichenfolge, das Schema eines Tools JSON, Berechtigungen liegen in der Konfiguration, Budgets in der Konsole des Anbieters und Verifikation im Anwendungscode. Jeder Teil kann funktionieren, doch der Compiler sieht ihre Zusammenhänge nicht.

NUDO untersucht, wie sich diese Grenzen in Sprache und Runtime sichtbar machen lassen. Ein Typ kann generierte von verifizierten Daten unterscheiden. Eine Effektsignatur könnte ausdrücken, welche Autorität eine Funktion benötigt. Eine Runtime könnte festhalten, welche Eingaben und Prüfungen ein Ergebnis hervorgebracht haben. Das sind Designziele; sie sind noch nicht alle ausgelieferte Funktionen.

## Was heute existiert

Die CLI kann Quelldateien laden, tokenisieren, in einen verlustfreien Syntaxbaum parsen, Namen auflösen und die bisher implementierten angenommenen Typregeln anwenden. Die HIR-, Namensauflösungs- und partiellen Typechecker-Crates sind implementiert. Der Typechecker hat außerdem eigene Tests und Konformitätsfälle.

Die aktuelle Toolchain führt keine NUDO-Programme aus. Es gibt noch keinen Interpreter, Effektprüfer, Modellanbieter, Agenten-Runtime, Tool-Sandbox, Provenienzspeicher oder WASM-Backend. Der <a href="status.md">Projektstatus</a> und die <a href="https://github.com/smouj/nudo/blob/main/ROADMAP.md">Roadmap</a> unterscheiden das funktionierende Frontend von geplanten Subsystemen.

## Was das Projekt besonders macht

NUDO behandelt deterministische Software als Normalfall und probabilistische Arbeit als begrenzte Operation mit sichtbarer Vertrauensgrenze. Eine Modellantwort sollte nicht allein deshalb zu vertrauenswürdigen Anwendungsdaten werden, weil sie die passende Form hat. Ein Tool sollte nicht allein deshalb Autorität erhalten, weil der Hostprozess sie besitzt.

Compiler und Runtime sollen diese Aussagen überprüfbar machen und, soweit implementiert, durchsetzen.

## Was es nicht ist

NUDO ist kein Modell-SDK, Prompt-Framework, anbieterspezifisches DSL oder neuer Anstrich für Python. Anbieterspezifische Anfrageformate gehören in Runtime-Adapter; Prompt-Zusammensetzung gehört meist in Bibliotheken. NUDO ist außerdem nicht produktionsreif: Sprache, Compiler und Runtime sind unvollständig und können sich durch das Spezifikationsverfahren ändern.

## Maßgebliche Quelle

Die Spezifikation definiert die Sprache. Angenommene NEPs ändern sie. Tests legen das beobachtbare Implementierungsverhalten fest. Das Buch erklärt Motivation und Status, entscheidet aber keine offene Sprachfrage stillschweigend.
