# 07.1 — Traces and provenance

> **Status:** Proposed  
> **Summary:** Important autonomous execution should leave enough structured evidence to reconstruct what happened without reverse-engineering logs.

A trace is an execution narrative. Provenance is data lineage attached to values.
They overlap but are not identical.

A useful provenance record for a verified value may need to identify the source
model/tool, relevant inputs, verification step, policy decisions and approvals.
The exact schema remains an implementation/specification task; the invariant is
that trust should not be detached from the evidence that created it.

Traces should be deterministic in structure where possible, even when the model
content inside an event is probabilistic.
