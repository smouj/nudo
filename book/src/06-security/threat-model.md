# 06.2 — Threat model

> **Status:** Specified / Planned  
> **Summary:** NUDO treats model output and external tool input as untrusted until explicit rules move data across trust boundaries.

The threat model includes prompt injection, malicious tool output, capability
escalation, secret exfiltration, unbounded execution, provenance loss and runtime
adapter bugs.

Language checks alone cannot secure an operating system. Static effect rules must
be paired with runtime enforcement and sandboxing where side effects cross process
or host boundaries.

A mature implementation should test both positive and negative guarantees: not
only that allowed programs work, but that denied effects fail *before* the effect
occurs.
