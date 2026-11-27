# 00.1 — What is NUDO?

> **Status:** Specified / Implemented
>
> **Summary:** NUDO is a general-purpose language project whose design makes trust and authority explicit; its current implementation is a pre-alpha compiler front end.

## The one-sentence model

NUDO aims to connect deterministic code, people, agents, tools and models under
one system of types, permissions and verification.

## The problem it addresses

In many agent applications, a prompt is a string, a tool schema is JSON,
permissions live in configuration, budgets live in a provider console, and
verification happens in application code. Each part may work, but the compiler
cannot see how they relate.

NUDO explores making those boundaries visible to the language and runtime. A
type can distinguish generated data from verified data. An effect signature
could say what authority a function needs. A runtime could record which inputs
and checks produced a result. These are design goals; they are not all shipped
features.

## What exists today

The current CLI can load source files, tokenize them, parse them into a lossless
syntax tree, resolve names and show those stages. The HIR and name-resolution
crates are implemented. The type checker implements a partial set of accepted
rules and has its own tests and conformance cases, but <code>nudo check</code>
does not run it yet.

The current toolchain does not execute a NUDO program. There is no interpreter,
effect checker, model provider, agent runtime, tool sandbox, provenance store or
WASM backend yet. The <a href="status.md">project status</a> and
<a href="https://github.com/smouj/nudo/blob/main/ROADMAP.md">roadmap</a>
distinguish the working front end from planned subsystems.

## What makes the project unusual

NUDO treats deterministic software as the ordinary path and probabilistic work
as a bounded operation with a visible trust boundary. A model response should
not become trusted application data merely because it has the right shape. A
tool should not acquire authority merely because the host process has it.

The compiler and runtime are intended to make those statements reviewable and,
where the implementation exists, enforceable.

## What it is not

NUDO is not a model SDK, prompt framework, vendor-specific DSL or new skin over
Python. Provider-specific request formats belong in runtime adapters; prompt
composition mostly belongs in libraries. NUDO is also not production-ready:
the language, compiler and runtime are incomplete and remain subject to change
through the specification process.

## Source of truth

The specification defines the language. Accepted NEPs amend it. Tests pin
observable implementation behaviour. The book explains motivation and status,
but it does not silently settle an open language question.
