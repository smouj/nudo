# Security policy

## Supported versions

**None.** NUDO is pre-alpha: `0.0.x` releases are development snapshots of an
incomplete toolchain, and no version of it is supported for production use or
receives security updates.

| Version | Supported |
| ------- | --------- |
| `0.0.x` (pre-alpha) | No — development snapshots only |
| anything ≥ `0.1.0` | Does not exist yet |

That is deliberate honesty rather than a formality: a language toolchain that
executes agent actions, enforces capabilities and handles credentials must not
imply a support commitment it cannot keep. When there is a version worth
supporting, this table changes first.

## Reporting a vulnerability

**Do not open a public issue.**

Use GitHub's private vulnerability reporting for this repository:
**Security → Report a vulnerability**
(<https://github.com/smouj/nudo/security/advisories/new>).

Please include:

* what you found, and the version or commit you tested;
* the smallest input or program that shows it;
* what an attacker gains (a crash, arbitrary file access, a capability that
  should have been denied, a credential leak, a sandbox escape);
* any constraints on exploitability you already know about.

If you need to send something that does not fit in an advisory — a PoC binary,
for example — say so in the report and a private channel will be arranged.

## What to expect

NUDO is maintained by one person. Realistic expectations:

| Stage | Target |
| ----- | ------ |
| Acknowledgement that the report was received | within 7 days |
| Initial assessment, including whether it is in scope | within 14 days |
| Fix or a documented mitigation for a confirmed issue | best effort, no promise |

There is **no bug bounty** and no payment. Credit is given in the advisory and
in [`CHANGELOG.md`](CHANGELOG.md) unless you ask not to be named.

## Scope

In scope, and interesting:

* the compiler and toolchain: panics, hangs, unbounded memory use or
  code execution triggered by source input, including malformed input;
* anything that breaks a documented promise, especially a promise the
  specification makes (see [`AGENTS.md`](AGENTS.md));
* a path that performs an effect without the capability that the specification
  requires — once capabilities exist;
* supply-chain weaknesses in the build: workflow permissions, release
  provenance, dependencies;
* credential or secret handling in the toolchain, including in CI.

Not in scope at this stage, because the feature does not exist:

* prompt injection through a model, a tool or a remote agent — specified, not
  implemented, see [`docs/security/threat-model.md`](docs/security/threat-model.md);
* sandbox escapes (`runtime/nudo-sandbox` is an empty crate);
* MCP or A2A adapter issues (`protocols/` is empty);
* anything that requires the interpreter (`backends/interpreter` is empty).

A report about a pre-alpha gap in an unimplemented feature is still welcome as
an issue — it is just not a vulnerability report, because there is nothing to
exploit.

## What the project does about its own security

* The workspace has **no third-party dependencies** in the pre-alpha toolchain.
  `cargo-deny` and `cargo-audit` run in CI to keep it that way and to catch a
  dependency problem the moment one is introduced.
* Workflows use minimal permissions and no secret is available to pull request
  workflows.
* The default posture in the specification is deny. See
  [`spec/trust/capabilities.md`](spec/trust/capabilities.md).
* The threat model is maintained as a document, not as an assumption, and is
  reviewed when a milestone that touches it is completed.
