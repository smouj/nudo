# Fixtures

Small, hand-written inputs for the unit-level tests of the compiler crates.
They are the *unit* corpus; the language-level corpus that other
implementations are expected to reproduce lives in [`../tests/conformance`](../tests/conformance).

Use fixtures when you want to pin one rule of one stage. Use conformance cases
when you want to pin language-visible behaviour.

## Layout

| Directory | Rule |
| --------- | ---- |
| `valid/`   | Must lex **and parse** with zero diagnostics. |
| `invalid/` | Must produce exactly the diagnostics declared in the file. |

Since M2 there are two stages behind these rules, so a fixture may fail either
one. Each stage's test speaks only about the diagnostics it is responsible for:
`compiler/nudo-lexer/tests/fixtures.rs` checks the lexical subset,
`compiler/nudo-parser/tests/fixtures.rs` checks the whole set and the tree.
Between them, a code that is declared but never produced fails, and so does one
that is produced but never declared.

## Declaring expected diagnostics

An invalid fixture declares what it expects in comments, which keeps the input
readable and the expectation next to the offending line:

```text
// EXPECT: NDO1002
// EXPECT: NDO1002 @ 3:9
```

* `// EXPECT: <code>` — one diagnostic with that code must be reported.
* `// EXPECT: <code> @ <line>:<column>` — as above, and a diagnostic with that
  code must start at that 1-based position.

The `// EXPECT:` lines are ordinary comments, so they do not change how the file
lexes. The multiset of declared codes must equal the multiset of reported
codes: a fixture may not declare fewer diagnostics than it triggers, and it may
not declare diagnostics it does not trigger. A `valid/` fixture may not declare
any.

`NDO1001` needs care: the parser does not report a token it cannot place when
the lexer already rejected a byte in front of it, so a fixture that declares
`NDO1002` and a syntax error from the same bytes declares only the `NDO1002`.

## Naming

Files are `<rule>.nudo`, in `kebab-case`, named after the rule they pin rather
than the symptom: `unterminated-text.nudo`, not `bug-42.nudo`.

## Non-source inputs

`valid/not-a-nudo-file.txt` is a deliberate exception: it exists so that the CLI
test can check the `NDO8001` warning about a file that does not use the `.nudo`
extension. It holds the same text as `valid/hello.nudo`. Tests that run the
compiler skip files whose extension is not `.nudo`.
