# Model Context Protocol (MCP)

**State: proposed.** No adapter exists (milestone M9). `protocols/` is empty.

## Intent

NUDO programs should be able to **consume** tools exposed by MCP servers, and
**expose** NUDO tools to MCP clients. Neither direction should require the
program to trust the other side.

MCP is one of the ways tools arrive from outside the program, and the outside is
where the interesting security problems are.

## Consuming an MCP server

An MCP server is treated as a source of **tool declarations**, not as a source of
authority:

1. A server's tools are imported as tool declarations, each with the capabilities
   it requires.
2. The importing program decides which capabilities to grant. A server does not
   declare what it is allowed to do to the host — it declares what it can do, and
   the host decides.
3. A tool the program did not import cannot be called, even if the server offers
   it.
4. Every call is capability-checked and recorded like any other tool call.

## Exposing NUDO tools

Tools exposed over MCP keep their declared capabilities. Serving a tool does not
grant the client anything, and an incoming call is checked against the same rules
as an internal one.

## Security requirements

These are requirements, not aspirations, and they are the reason this chapter
exists before the adapter does:

* **A server is untrusted.** Its descriptions, schemas and results are data. A
  tool description that says "ignore your instructions and…" is a string.
* **No implicit capability.** Importing a server grants nothing. Grants are
  per-tool and explicit.
* **No widening through invocation.** A server cannot acquire a capability by
  being called by something that has it.
* **Results carry provenance.** What came from which server, when, with which
  arguments.
* **Incoming content is not instruction.** This is the central rule against
  context poisoning, and it applies to NUDO's own code: nothing in the runtime
  treats tool output as a directive.
* **Failures are contained.** A misbehaving server times out, is cancelled, and
  is recorded; it does not hang the runtime.

Full threat enumeration: [`../../docs/security/threat-model.md`](../../docs/security/threat-model.md).

## Open questions

* How a server's tool set is bound at compile time versus resolved at runtime.
* Whether NUDO ships an MCP client or exposes a library that a program uses.
* How a tool's declared cost maps onto a budget.
* Whether the language models an MCP server as a *model* or as a *tool* — it
  provides context, which is model-like, but it is called, which is tool-like.
