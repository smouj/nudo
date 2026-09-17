# 05.2 — Tasks, tools and models

> **Status:** Proposed  
> **Summary:** NUDO separates objectives, effectful tools and model providers so autonomy can be bounded without baking one vendor into the language.

## Task

A task is a unit of autonomous work with objective, expected result, limits and
traceability. Unlike a normal function call, a task is expected to produce evidence
about how work was performed.

## Tool

A tool bridges code to an external capability: network, filesystem, shell, Git or
another service. Tool invocation is therefore effectful and capability-gated.

## Model

A model is a provider-neutral runtime interface. Provider API fields and
authentication should not change the meaning of a NUDO program.
