# `assets/diagrams/` — diagrams used in documentation

**Empty and pending.** The diagrams below are described, not drawn. Each is a
real explanation the documentation needs; none has been produced yet, and
checked-in placeholders would be worse than nothing.

## What is needed

| Diagram | Explains | Source of truth |
| ------- | -------- | --------------- |
| Compiler pipeline | Source → lexer → parser → HIR → types → effects → MIR → backend | [`../../docs/compiler/pipeline.md`](../../docs/compiler/pipeline.md) |
| Trust flow | `Generated<T>` → verify → `Verified<T>`, and where provenance attaches | [`../../spec/trust/verified.md`](../../spec/trust/verified.md) |
| Authority flow | Capabilities narrowing along a delegation chain | [`../../spec/agents/delegation.md`](../../spec/agents/delegation.md) |
| Runtime boundary | The single door between a program and the world, and what is checked at it | [`../../spec/agents/tool.md`](../../spec/agents/tool.md) |

Text is the source of truth for all four. A diagram that disagrees with the
specification is a bug in the diagram.

## Format

* SVG, checked in as source, so it can be reviewed as text and diffed.
* No embedded fonts; paths or a generic family only, so it renders identically
  everywhere.
* Readable in both light and dark mode, or provided in two files.

## Colours

Diagrams follow [`../brand/BRAND.md`](../brand/BRAND.md): near-black, the single
accent, and neutral greys for structure. One accent per diagram.

## Rule

A diagram is documentation. It may summarise, and it may omit — but it may not
show a component, a capability or a step that the specification does not
describe. The pipeline diagram in particular must keep marking which stages
exist and which are planned, because that distinction is the honest content of
the picture.
