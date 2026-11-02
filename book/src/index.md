# Introduction

> **Status:** Historical / Living  
> **Summary:** A guided, evidence-based account of NUDO’s design, the compiler that exists today, and the runtime the project is working toward.

<div class="status-key">
DOCUMENT: NUDO-BOOK-001 · EDITION: ENGINEERING PAPER · REVISION: R0 · STATUS: LIVING
</div>

NUDO is a general-purpose programming language project. Its design aims to let
ordinary deterministic programs include bounded probabilistic work while
keeping data trust, required authority and verification visible in the program.
The project is pre-alpha: the language has a working front end and a partial
type-checking crate, but it does not yet execute NUDO programs.

This book explains the motivation, accepted language rules, compiler design,
implemented behaviour, open proposals and planned runtime. It is broader than
the normative specification, so every example must be read with its status.
The specification defines the language; this book does not add rules.

## How to read this book

For the design, begin with Philosophy and Trust. For the implemented compiler,
read Language Design → Compiler → Type System. For the future agent runtime,
read Agents and AI → Security → Runtime. The milestone table below distinguishes
working code from accepted rules and proposals.

## What works today

- <code>nudo check</code> loads <code>.nudo</code> files, lexes and parses them,
  resolves names, and reports lexical, syntax and name-resolution diagnostics.
- <code>--dump-tokens</code>, <code>--dump-tree</code> and
  <code>--dump-resolutions</code> expose stable views of those stages.
- The Rust crates contain a partial type checker. Its implemented rules are
  exercised by unit tests and the type-checking conformance corpus, but the
  checker is not yet wired into <code>nudo check</code>.
- The benchmark suite measures synthetic workloads across lexer, parser, HIR
  lowering and type checking. It does not measure execution of NUDO programs.

There is no <code>nudo run</code>, <code>nudo build</code>, interpreter, effect
checker, agent runtime, tool dispatch, policy enforcement, provenance recorder
or WebAssembly backend yet.

## Current project status

{{#include diagrams/status.svg}}

*Milestone state and dependency path through M5. Accent identifies completed or
active implementation; muted rows are planned. The
<a href="https://github.com/smouj/nudo/blob/main/ROADMAP.md">roadmap</a> is the
source of truth if this summary and the repository ever diverge.*

M3 is in progress: HIR and name resolution are implemented, while type checking
and trust-type support cover only accepted slices. M4 (the first executable
program) and M5 (effect checking) remain planned. M5 depends on completing the
earlier milestones and accepting the effect-system semantics.

## Authority

The specification defines the language. Accepted NEPs amend it. Conformance
tests pin observable behaviour. This book explains those rules and labels future
work; it never upgrades a proposal into an implemented capability.
