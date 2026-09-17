# WebAssembly and WASI

**State: proposed.** No backend exists (milestone M11). `backends/wasm` is empty.

## Intent

WebAssembly is NUDO's second execution strategy, after the interpreter. It is
not the first because the interesting problems in this language are semantic, and
a tree-walking interpreter is the fastest way to find out whether the semantics
are even right.

## Why WASM, and not as an afterthought

WASI's design has the same shape as NUDO's security model: a module has no
ambient authority, and gets exactly the capabilities the host grants it. That
alignment is the reason this target is worth the work:

* a NUDO program compiled to WASI runs with the capabilities its declarations
  asked for, and nothing else;
* the host can refuse a capability, and the program sees a denial rather than a
  crash;
* the sandbox is the platform's, not a bespoke one that has to be maintained and
  audited.

## Requirements

1. **Capability fidelity.** A capability the program did not declare is not
   reachable in the compiled module, including through imports.
2. **Same semantics.** A program that runs under the interpreter and under WASM
   produces the same result, or the difference is a specification bug.
3. **Deterministic diagnostics.** Errors report the same codes and positions as
   the interpreter.
4. **No hidden host surface.** The set of host functions a module can import is
   derived from the program's declarations, not from a fixed list of everything
   the runtime could offer.
5. **Traces survive the boundary.** Provenance and traces from a WASI module are
   available to the host.

## Open questions

* Whether the compiler targets WASM directly from MIR, or through an existing
  intermediate representation. The second is faster to build and adds a
  dependency with a large surface.
* How traces are collected from a sandboxed module without granting it the
  capability to write them somewhere.
* How a WASM module performs a task that needs a model call, given that models
  are host resources.
* Whether the WASI target is a first-class citizen of the conformance corpus, or
  a portability check alongside the interpreter.
