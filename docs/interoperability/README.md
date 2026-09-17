# Interoperability

NUDO is designed to work with other agent and tool systems rather than to
replace them. Both directions carry the same posture: the other side is
**untrusted by construction**.

| Direction | Mechanism | Specification | Milestone |
| --------- | --------- | ------------- | --------- |
| Consume external tools | MCP client | [`../../spec/interoperability/mcp.md`](../../spec/interoperability/mcp.md) | M9 |
| Expose NUDO tools | MCP server | same | M9 |
| Delegate to other agents | A2A | [`../../spec/interoperability/a2a.md`](../../spec/interoperability/a2a.md) | M9 |
| Compile to a portable target | WebAssembly/WASI | [`../../spec/interoperability/wasm.md`](../../spec/interoperability/wasm.md) | M11 |

**None of it is implemented.** `protocols/` contains a README and nothing else,
because an adapter written before the language can express a tool would describe
an intention rather than implement one.

## The three rules

1. **Nothing is trusted because of where it came from.** An MCP server, a remote
   agent and a web page are the same kind of thing: a source of data.
2. **Importing is not granting.** Connecting to a system grants no capability.
   Grants are per-tool, explicit and checkable.
3. **Trust does not cross a boundary.** A value that arrives from outside is
   untrusted until a local verification step makes it trusted, and that step is
   visible in the source.

## Why WASM is listed here

Because WASI has the same shape as NUDO's security model: a module starts with no
authority and receives exactly the capabilities its host grants. That alignment
is why the WebAssembly target is worth the work, and why it is a portability
target rather than a performance project.

## Reading more

* [`../security/threat-model.md`](../security/threat-model.md) — threats T2, T3
  and T7 cover MCP, remote agents and sandboxing.
* [`../../spec/trust/capabilities.md`](../../spec/trust/capabilities.md) — what a
  capability is and how it is granted.
* [`../../spec/agents/delegation.md`](../../spec/agents/delegation.md) — why
  delegation can only narrow authority.
