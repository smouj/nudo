# 03.3 — Parser, verlustfreie Syntax und AST

> **Status:** Implementiert (IMPLEMENTED)  
> **Summary:** M2 hat den Token-Strom in eine verlustfreie Syntaxstruktur überführt, sich von fehlerhafter Eingabe erholt und einen typisierten AST für spätere semantische Stufen bereitgestellt.

## Warum zuerst verlustfreie Syntax

Ein verlustfreier Baum behält Trivia und jedes Quellbyte. Diese Eigenschaft ist
wichtig für einen Formatter, Editor-Werkzeuge und automatisierte Agenten, die Code
ändern müssen, ohne Kommentare oder Layout stillschweigend zu zerstören.

Der Parser hält sie als geprüfte Eigenschaft statt als Absicht. Jede Fixture, jeder
Konformitätsfall und jeder Property-Test stellt sicher, dass das Aneinanderhängen
des Textes der Tokens des Baums die Datei Byte für Byte reproduziert. `nudo check
--dump-tree` gibt den Baum aus, und der Dump ist länger als die Datei, aus der er
stammt — genau weil Leerraum und Kommentare darin enthalten sind.

Diese ehrlichen Kosten sind der Grund, warum der Baum so geformt ist, wie er ist.
`nudo-ast` verpackt den Baum in typisierte Werte — eine `Function` mit einem Namen
und einem Rumpf, eine `Binding` mit einem Wert — und liefert nichts, wenn der Baum
den Teil nicht enthält. Ein Konsument, der sicher sein muss, etwa ein Formatter,
der entscheidet, ob er umschreiben darf, muss „das ist eine Funktion" von „das sah
bis zum achten Token danach aus" unterscheiden können.

## Fehlererholung

Ein brauchbarer Parser hört nicht beim ersten fehlenden Token auf. Erholungspunkte
umfassen Elementgrenzen, Semikolons und schließende Begrenzer, damit ein Fehler
nicht den Rest einer Datei in Rauschen verwandelt.

Drei Regeln machen das konkret, und jede ist aus einem Fehler hervorgegangen, den
die Tests gefunden haben:

* **Erholung verbraucht immer ein Token oder endet.** Eine Erholung, die schleift,
  ist ein Hänger, und ein Hänger bei Nutzereingabe ist ein Bug.
* **Erholung meldet, bevor sie überspringt.** Eine frühe Fassung übersprang still,
  was sie nicht einordnen konnte, und `nudo check` beendete sich danach mit `0` für
  eine Datei, die es nicht verstanden hatte. Das ist schlimmer, als die Datei
  zurückzuweisen, weil der Aufrufer glaubt, sie sei gelesen worden.
* **Eine Position, eine Diagnose.** Wenn der Lexer ein Byte bereits zurückgewiesen
  hat, meldet der Parser nicht die Anweisung, die dieses Byte unlesbar gemacht hat.
  Ein Zeichen erzeugt eine Meldung, und ein Agent, der die Datei repariert, korrigiert
  ein Byte.

Ein Token, das der Parser nicht einordnen kann, bleibt in einem `Error`-Knoten
erhalten statt verworfen zu werden, sodass die Datei weiterhin neu ausgegeben werden
kann — einschließlich der Teile, die falsch sind.

## Typisierter AST

Der AST gibt nachgelagerten Durchläufen semantische Formen (`Function`, `Call`,
`Type`), ohne dass diese rohe Details von Syntaxknoten kennen müssen.
Konvertierungen von Syntax zu AST bleiben Span-erhaltend, und der Span eines Knotens
beginnt genau bei seinem ersten Token.

Nichts in `nudo-ast` löst einen Namen auf oder prüft einen Typ: ein Pfad ist eine
Liste von Namen, keine Referenz. Das ist Meilenstein M3, und der Parser weiß
absichtlich nichts davon.

## Was absichtlich fehlt

* Generische Parameter an der Deklarationsstelle (`enum Outcome<T, E>`) stehen nicht
  in der eingefrorenen Grammatik und werden zurückgewiesen; NEP-0006 hält fest, warum.
* `verify (expr) with V` wird zurückgewiesen, weil `verify` ein kontextabhängiges Wort
  ist und der Parser ein Token Lookahead hat. Die Einschränkung ist durch einen Test
  festgehalten und verschwindet, wenn NEP-0002 entscheidet, was `verify` ist.
* Die Pfade der Grammatik verwenden `::` und ihre Listen sind komma-getrennt, während
  mehrere Beispiele im Handbuch und in `examples/` `web.search` je Zeile schreiben.
  Der Parser folgt der Grammatik; die Beispiele sagen, dass sie Vorschauen sind, und
  welche Schreibweise die Sprache haben sollte, ist ein NEP und kein Parser-Bug.
