# 12.1 — Alternatives deliberately rejected

> **Status:** Historical / Design rationale  
> **Summary:** Several simpler approaches are useful in applications but do not satisfy NUDO’s goal of language-visible trust and authority.

| Alternative | Why it is insufficient as the language model |
| --- | --- |
| Prompts as ordinary strings | Compiler cannot infer trust or authority from prose. |
| Permissions only in deployment config | Source-level calls cannot be checked against the grant path. |
| `verified: Bool` flags | Safety becomes an optional runtime convention. |
| Provider-specific keywords | Couples language meaning to vendor APIs. |
| Unbounded agent loop | Makes authority, budget and termination difficult to review. |
| Log-only provenance | Reconstructs evidence after the fact and can lose value lineage. |

These techniques may still appear inside runtime adapters or libraries. Rejection
here means “not the semantic foundation of the language,” not “never useful.”
