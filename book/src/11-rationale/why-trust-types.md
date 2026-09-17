# 11.2 — Why trust belongs in the type story

> **Status:** Proposed rationale  
> **Summary:** Trust state is important enough to be visible in interfaces because forgetting a verification step is a class of programming error, not merely a logging concern.

A reviewer should be able to inspect a function signature and distinguish a value
that came from a model from one that passed an explicit verifier.

That distinction also creates a place for tooling: diagnostics can explain the
missing boundary, IDEs can visualize trust flow and audits can search for verified
values whose provenance is incomplete.
