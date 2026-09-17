# 00.1 — What is NUDO?

> **Status:** Specified  
> **Summary:** NUDO is designed as a general-purpose language in which deterministic code and bounded probabilistic work can coexist without hiding trust or authority boundaries.

## The one-sentence model

NUDO connects people, code, agents, tools and models under one system of types,
permissions and verification.

## What makes the project unusual

Most application languages can represent a model call only as ordinary library
I/O. That means the compiler sees a function call and perhaps a JSON value, but
it does not know that the value was generated probabilistically, what authority
the caller granted, what budget was consumed or whether somebody verified the
result.

NUDO explores moving those boundaries into language-visible structures while
keeping ordinary deterministic programming as the default path.

## What it is not

NUDO is not a model SDK, a prompt framework, a vendor-specific DSL or a new skin
over Python. Provider-specific request shapes belong in runtime adapters. Prompt
composition belongs mostly in libraries. The language is responsible for meaning,
trust and authority rules that independent implementations must agree on.
