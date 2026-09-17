# 05.1 — Agents as bounded executors

> **Status:** Proposed  
> **Summary:** An agent is intended to be a declared executor with explicit tools, authority, limits and acceptance criteria, not an unconstrained prompt loop.

Agent declarations are useful only if they communicate something the compiler or
runtime can enforce. Their purpose is therefore not cosmetic organization.

A reviewer should be able to answer:

- which tools can this agent invoke?
- which capabilities can those tools consume?
- which model interface is available?
- what budget limits execution?
- what output type is expected?
- what verification or approval is required before continuation?

Provider-specific prompt and request details remain runtime concerns.
