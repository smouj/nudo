# 04.2 — Generated- und Verified-Werte

> **Status:** Implementiert (IMPLEMENTED) / Spezifiziert (SPECIFIED) / Geplant (PLANNED)
>
> **Summary:** Der Typechecker setzt die Unterscheidung zwischen generierten und verifizierten Typen durch; die Ausführung von Verifikation und die Erfassung von Provenienz bleiben künftige Runtime-Arbeit.

{{#include ../diagrams/trust-flow.svg}}

*Statische Typregeln sind teilweise implementiert; die verify-Operation und
der Provenienzpfad sind ein Designentwurf und heute nicht ausführbar.*

## Die Vertrauensunterscheidung

<code>Generated&lt;T&gt;</code> und <code>Verified&lt;T&gt;</code> sind verschiedene Typen. Der Typechecker konvertiert nicht implizit zwischen ihnen, und ihre Typargumente bleiben invariant. Ein generiertes Ergebnis erfüllt daher keinen Parameter, der ausdrücklich den verifizierten Typ verlangt.

Die angenommenen Regeln erlauben außerdem, das eingebaute <code>Verified&lt;T&gt;</code> an unterstützten Stellen zu verwenden, an denen <code>T</code> erwartet wird. Grundlage ist die aufgelöste Typidentität des eingebauten Typs; ein benutzerdefinierter Typ mit derselben Schreibweise erhält diese Regel nicht. Das Verhalten wird durch Typechecker-Konformitätsfälle abgedeckt, doch <code>nudo check</code> führt die Typprüfung noch nicht aus.

## Die geplante Verifikationsoperation

Der Parser akzeptiert die Syntax <code>verify value with Verifier</code> und bildet sie in HIR ab. Der Vertrag des Verifiers, die Ausführung, Fehlerbehandlung und Provenienz sind nicht implementiert. Das Verifier-Design ist weiterhin ein Vorschlag; die folgende Form zeigt daher den vorgesehenen Ablauf und ist kein ausführbares Programm:

~~~nudo
let checked: Result<Verified<Article>, VerificationError> =
    verify draft with ArticleVerifier
~~~

Das Design macht zwei Tatsachen getrennt sichtbar: Verifikation kann fehlschlagen, und ein erfolgreicher Wert trägt einen anderen Vertrauens-Typ. Es macht probabilistische Generierung nicht deterministisch, sondern die Grenze überprüfbar.

## Warum Vertrauen nicht als Boolean darstellen?

~~~text
ModelOutput<T> { value: T, verified: Bool }
~~~

Ein Aufrufer kann vergessen, ein Flag zu prüfen. Verschiedene Typen erlauben es dem Typechecker, einen Wert an einer unterstützten Typprüfgrenze zurückzuweisen, statt sich auf eine Runtime-Konvention zu verlassen.

## Was noch fehlt

Die Runtime-Operation, statisch prüfbare Verifier-Deklarationen, ein stabiler Fehlertyp, Provenienzaufzeichnungen und Traces sind weiterhin geplant oder vorgeschlagen. <code>Result&lt;T, E&gt;</code> kann mit den erforderlichen Typargumenten benannt werden; <code>match</code>-Typprüfung und vollständige Fallbehandlung sind jedoch nicht implementiert.
