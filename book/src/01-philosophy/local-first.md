# 01.3 — Local-first and provider-agnostic

> **Status:** Specified  
> **Summary:** Local models, local tools and offline execution are intended to be first-class rather than degraded modes.

A provider name should not appear in the core language merely because that provider
is popular today. Providers change; language semantics should not.

NUDO therefore separates a model abstraction from provider-specific adapters. A
local model should be able to satisfy the same runtime interface as a remote model,
subject to capabilities and policy.

Local-first also improves testability: deterministic test harnesses and local model
fixtures can exercise a program without requiring paid external infrastructure.
