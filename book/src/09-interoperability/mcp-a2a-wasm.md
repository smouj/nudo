# 09.1 — MCP, A2A and WebAssembly

> **Status:** Planned  
> **Summary:** Interoperability should let NUDO participate in existing tool and agent ecosystems without allowing adapters to widen authority.

MCP interoperability can let NUDO consume or expose tools. A2A-style delegation can
connect independent agent implementations. WASI can offer a portable execution
target with explicit host capabilities.

The security rule is stronger than protocol compatibility: adapters must not be
trusted to grant themselves capabilities that the NUDO caller did not possess.
