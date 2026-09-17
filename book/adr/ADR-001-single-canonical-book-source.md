# ADR-001 — One canonical Markdown source for web and PDF

- **Status:** Accepted
- **Date:** 2026-09-17

## Context

Maintaining separate web and PDF manuscripts creates drift.

## Decision

`book/src/` is canonical. mdBook renders web output. The PDF pipeline converts the
same chapter order from `SUMMARY.md` into Typst before typesetting.

## Consequences

The web and PDF can have different visual systems without duplicating prose. Any
format-specific feature must degrade safely in the other renderer.
