# Provenance

**State: proposed.** No provenance store exists (milestone M6).

## What provenance is

Provenance is the record of where a value came from and what was done to it. It
is **data attached to values**, produced during execution.

This is the difference between a system that can be audited and a system that
can only be hoped about. Logs assembled afterwards answer "what did we print?";
provenance answers "what produced this value, from what, and who approved it?".

## What a record contains

Provisional, and expected to grow:

| Field | Meaning |
| ----- | ------- |
| Origin | Model, tool, human, or computation |
| Identity | Which model version, which tool version, which person |
| Inputs | What was consumed, by reference to their own provenance |
| Parameters | Temperature, seed, prompt identity — not the prompt text by default |
| Verification | Which verifier ran, against which criteria, with what outcome |
| Approvals | Who approved, when, and for what |
| Cost | What was spent producing it |
| Time | When it happened |

## Rules

1. **Produced, not reconstructed.** Provenance is written as work happens. A
   system that infers provenance from logs is guessing.
2. **Attached, not global.** `Verified<T>` and `Generated<T>` carry their
   provenance. A side table that can drift out of sync is not provenance.
3. **Not a secret.** Provenance never contains credentials or secret material.
   It may contain references to them; it must not contain their values.
4. **Bounded.** Provenance records have limits. An unbounded record of an
   unbounded run is a memory leak with extra steps.
5. **Inspectable by the program.** Provenance is available to NUDO code, so a
   program can make decisions based on where a value came from — and so can an
   audit.
6. **Untrusted content is not elevated by being recorded.** Remembering that a
   value came from a web page does not make it instruction.

## Privacy

Provenance records inputs, and inputs are frequently personal data. Therefore:

* recording a prompt's *identity* does not require recording its *text*;
* what gets recorded is a declared policy, not an accident of implementation;
* a trace can be redacted, and redaction is visible (a redacted field says it was
  redacted rather than becoming empty).

## Open questions

* The representation format, and whether it is versioned and portable between
  implementations.
* How provenance survives serialisation and process boundaries, and what is lost.
* How much provenance is retained by default versus on request.
* How provenance interacts with memory
  ([`../agents/memory.md`](../agents/memory.md)) — a remembered value whose
  provenance was dropped is a value that has silently become trusted.
