# NEP-0011: Path and list spelling

| Field | Value |
| ----- | ----- |
| Status | Accepted |
| Created | 2026-09-18 |
| Accepted | 2026-09-18, as gate 6 of M3.0 |
| Supersedes | — |
| Superseded by | — |
| Specification chapters | [`spec/grammar.md`](../spec/grammar.md), [`spec/declarations.md`](../spec/declarations.md) |
| Related NEPs | NEP-0005, NEP-0010 |

## Summary

Settle the three places where the frozen grammar and the examples disagreed, by
deciding the language rather than patching the parser: **a path is written with
`::`**, **a list inside a declaration is comma-separated**, and therefore
`web.search` and one-name-per-line lists are **not** the language.

## Motivation

`nudo check` reported `examples/05-agent` as unreadable for three reasons at once,
and each of them was a real disagreement rather than a bug:

| Writing | Grammar | Examples |
| ------- | ------- | -------- |
| Tool name | `tool-name = path`, and a path uses `::` | `tool web.search` |
| Name list | `{ ",", tool-name }` | one name per line |
| Path in a clause | `path` | `tools: web.search` |

A grammar that disagrees with the examples teaches a reader the wrong language,
and a parser patched to accept both would be a silent extension. So the decision
is made here, and the examples move to it.

## Reference-level explanation

```nudo
agent Researcher {
    role: "Research reliable information."
    tools: web::search, web::open
    allow: Network
    budget: Budget(tokens: 20_000, tool_calls: 30)
}

tool web::search(query: Text) -> [Result] with Network, Budget {
    // …
}
```

* **`::` separates path segments**, and a path is what names a tool, a module, a
  verifier and a declaration reached through one. `.` remains what it is in every
  other language: field access, on a *value*.
* **Lists are comma-separated**, with an optional trailing comma in a
  multi-line list. That covers the `tools:`, `allow:` and capability lists.
* **Blocks are not lists.** Newline-separated members are for declarations whose
  members are *fields* (`struct`, `enum`, `model`), where each member is a
  `name: type` pair and no separator could be ambiguous.
* **What this costs a reader**: `tools: a::b, c::d` is slightly denser than one
  per line. What it buys: one rule for every list, no dependence on line breaks
  for meaning, and a formatter can reflow a list without changing the program.

## What this makes impossible

* A program whose meaning depends on a newline, which is where "the formatter
  changed my code's meaning" bugs live.
* Two spellings of the same name, one for declarations (`::`) and one for
  values (`.`) — the confusion ends here, and the compiler cannot be asked to
  guess which one a program meant.

## Alternatives

| Alternative | Why it lost |
| ----------- | ----------- |
| Accept dotted tool names too | Two spellings for one name, and `web.search(x)` already means "field `search` of value `web`, called" — the two cannot both be true |
| Accept newline-separated lists | Meaning depends on layout, and the lossless tree would need a rule about newlines that no other construct has |
| Make everything newline-separated, including function arguments | The same layout-dependence, in more places |
| Leave it to the examples | The corpus and the toolchain would disagree permanently, and `nudo check` would keep rejecting programs that the manual shows |

## Compatibility

Nothing is published, so no program breaks. Three examples change their spelling,
and one of them (`05-agent`) becomes readable by the toolchain as a result. The
preview markers in the others stay until their remaining syntax is accepted.

## Unresolved questions

* Whether a trailing comma is allowed in single-line lists (this NEP says it is
  allowed in multi-line lists; `a, b,` on one line is currently rejected).
* Whether `use`/import paths need a different spelling once modules exist (M10).

## Implementation status

**Not implemented.** `nudo-parser` already implements the grammar exactly as this
NEP describes — that is *why* the examples were rejected. What changes is the
examples and the manual, which move to the accepted spelling in M3.1; until then
they are previews, and each says why.
