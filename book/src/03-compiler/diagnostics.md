# 03.4 — Diagnostics as product design

> **Status:** Implemented / Planned  
> **Summary:** NUDO diagnostics use stable codes and source spans; the implemented families cover lexical, syntax, name-resolution and partial type-checking errors.

Diagnostics are part of the language experience, not a debug dump. The
<code>nudo-diagnostics</code> crate provides structured diagnostics with severity,
code, source location, notes and help. The renderer is shared so each compiler
stage can report facts without formatting its own output.

## Implemented diagnostic families

- <code>NDO1xxx</code> covers lexical and syntax errors. The lexer and parser
  recover where possible and report source spans.
- <code>NDO2001</code>–<code>NDO2003</code> cover name-resolution errors and
  are emitted by <code>nudo check</code>.
- <code>NDO2004</code>–<code>NDO2007</code> and <code>NDO2017</code> cover
  implemented type-checking rules. The type-checker tests pin structured
  expected/found notes and source locations for those rules.

The type checker is partial and is not yet called by the CLI. A clean
<code>nudo check</code> therefore reports that source passed lexing, parsing and
name resolution; it does not mean the source passed type checking.

## Planned diagnostic families

The effect-system family <code>NDO3xxx</code> is planned with M5. No effect
checker emits those diagnostics today. The following is an illustrative target,
not output from the current toolchain:

~~~text
error[NDO3xxx]: missing capability Network

  --> src/main.nudo:14:5
   |
14 |     fetch(url)
   |     ^^^^^^^^^^ requires Network
   |
   = note: the call path requires Network
~~~

The accepted diagnostic specification defines the structure and source
requirements. The effect syntax and checking rules still depend on a proposal
being accepted.

## Diagnostic quality

A useful diagnostic answers what rule failed, where the source broke the rule,
and which requirement led to the error. For trust-type mismatches it should
explain that <code>Generated&lt;T&gt;</code> and
<code>Verified&lt;T&gt;</code> are distinct and identify the value that violates
the promise. It should not suggest an unsafe cast.

\n