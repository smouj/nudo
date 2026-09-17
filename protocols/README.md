# `protocols/` — interoperability with other agent systems

**PLANNED. Empty on purpose.**

NUDO is designed to interoperate with agent and tool ecosystems rather than to
replace them, but interoperability is milestone M9 and later in
[`ROADMAP.md`](../ROADMAP.md). Writing protocol adapters before the language can
express a tool would produce code that describes an intention instead of
implementing one.

## Planned adapters

| Adapter | Purpose | Specification | Milestone |
| ------- | ------- | ------------- | --------- |
| `mcp/`  | Model Context Protocol: consume and expose tools | [`spec/interoperability/mcp.md`](../spec/interoperability/mcp.md) | M9 |
| `a2a/`  | Agent-to-agent delegation between implementations | [`spec/interoperability/a2a.md`](../spec/interoperability/a2a.md) | M9 |

## Rules

An adapter is the least trusted code in the tree. It parses input from another
system and it is where context poisoning, tool abuse and capability escalation
enter a program. Therefore:

* every adapter parses into a validated type before anything acts on it;
* every adapter is capability-gated like any other tool;
* no adapter may widen the capabilities granted to the program it serves;
* untrusted content from an adapter is data, never instruction.

See [`docs/security/threat-model.md`](../docs/security/threat-model.md).
