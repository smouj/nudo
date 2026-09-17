# 02.3 — Modules and program boundaries

> **Status:** Proposed  
> **Summary:** Modules must provide explicit namespaces and reproducible program structure without hiding authority or dependency boundaries.

Module design is closely connected to package design, name resolution and public
interfaces. NUDO should avoid implicit global namespaces and should make public API
surface deliberate.

Open design work includes package manifests, visibility, dependency resolution and
how capabilities declared by dependencies are surfaced to callers.
