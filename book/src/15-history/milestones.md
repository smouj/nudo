# 15.1 — Milestones and evidence

> **Status:** Historical / Living  
> **Summary:** NUDO uses evidence-based milestones rather than calendar promises; a milestone is complete when its exit criteria and conformance evidence are complete.

The roadmap begins with repository foundation and the lexical pipeline, then moves
through parser/AST, type system, interpreter, effects, trust types, agent runtime,
capabilities/policies, interoperability, tooling and WebAssembly.

This ordering reflects dependency rather than marketing priority. An agent runtime
built before the parser/type/effect foundations would force semantic decisions into
runtime code and make later compiler checks harder to define cleanly.
