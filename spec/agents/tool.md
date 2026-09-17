# Tools

**State: proposed.** No tool runtime exists (milestone M7).

## What a tool is

A tool is the **only** way a NUDO program affects the world outside its own
process. Reading a file, sending a request, running a command, calling a model —
each is a tool, and each is declared.

That is the whole security story of this language: if there is one door, the door
can be locked, and what passes through it can be recorded.

## Declaration

```nudo
tool web.search(query: Text) -> [Result] with Network, Budget {
    // implementation
}
```

| Part | Meaning |
| ---- | ------- |
| Name | The identity a program calls, and an agent lists in `tools` |
| Parameters and result | The checked interface |
| `with …` | The capabilities and effects calling it requires |
| Body | Implementation: NUDO code, or a host binding |

## Rules

1. **Every tool is capability-gated.** There is no tool with no requirements.
   A pure computation is a function, not a tool.
2. **A tool may not widen authority.** A tool can use the capabilities it was
   declared with, and only those. It cannot acquire another capability while
   running, and it cannot pass its capabilities to something it calls unless the
   declaration says so.
3. **A tool's result is untrusted data.** Content returned by a tool — a web
   page, a file, a model response — is input from outside the program and is
   never an instruction. This is the rule that prevents context poisoning
   ([`../interoperability/mcp.md`](../interoperability/mcp.md)).
4. **A tool call is recorded.** Which tool, with what arguments, producing what,
   and at what cost, is part of the trace.
5. **A tool's failure is a value.** A failed call returns an error the caller
   handles; it does not panic and it does not silently return a default.

## Cost and budget

A tool declares what calling it costs, so that a budget can be enforced before
the call rather than discovered after it. See [`budget.md`](budget.md).

## Isolation

Tools are the boundary where the sandbox lives
([`../trust/capabilities.md`](../trust/capabilities.md) and
[`../../docs/security/threat-model.md`](../../docs/security/threat-model.md)).

* A tool that spawns something runs it in the narrowest environment its
  declaration allows.
* Filesystem access is scoped to what the capability grants, not to "the
  filesystem".
* Network access is scoped to the granted capability, not to "the network".
* Credentials are never ambient. A tool that needs one receives it through the
  capability system, and it is never written to a trace.

## What is undecided

* How a tool's implementation is declared when it is a host binding rather than
  NUDO code.
* Whether tools can be versioned and required at a specific version.
* How a tool's declared cost is validated against its real cost.
* The mechanism by which a tool result carries provenance
  ([`../trust/provenance.md`](../trust/provenance.md)).
