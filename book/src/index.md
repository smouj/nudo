# Introduction

> **Status:** Historical / Living  
> **Summary:** A structured explanation of what NUDO is, why it exists, and how its compiler, trust model and agent-oriented runtime are intended to fit together.

<div class="status-key">
DOCUMENT: NUDO-BOOK-001 · EDITION: ENGINEERING PAPER · REVISION: R0 · STATUS: LIVING
</div>

NUDO is a general-purpose programming language project whose design treats agents,
tools, models, permissions, budgets, verification and provenance as things the
language and runtime should be able to reason about explicitly.

This book is explanatory. It is deliberately broader than the normative
specification: it records motivation, trade-offs, architecture, rejected
alternatives and practical examples. It must never silently upgrade a proposed
feature into an implemented one.

## How to read this book

If you want to understand the **idea**, begin with Philosophy and Trust. If you
want to work on the compiler, follow Language Design → Compiler → Type System.
If you want to understand agent safety, read Agents and AI → Security → Runtime.

## Current reality

{{#include diagrams/status.svg}}

*What the toolchain does today, and what is only written down. The same table is
maintained in [`ROADMAP.md`](https://github.com/smouj/nudo/blob/main/ROADMAP.md), which wins if the two disagree.*


At revision R0, NUDO is pre-alpha. The repository foundation and lexical pipeline
are the mature implementation layer. Parser, typed AST, type checker, interpreter,
effect checker, agent runtime, policy system and WASM backend are design/roadmap
work rather than production claims.

## Authority

The specification defines the language. Accepted NEPs amend it. Conformance tests
prove observable agreement. This book explains those rules; it does not override
them.
