# NUDO — governance

NUDO is a small project with a single maintainer and a large design surface. This
document says how decisions are made, so that a contributor can predict the
answer to "who decides, and on what basis".

## Current structure

**Maintainer (BDFL): [@smouj](https://github.com/smouj).**

NUDO has no foundation, no committee and no organisation behind it. That may
change; it will be recorded here when it does, not announced as an intention.

The maintainer:

* decides whether a NEP is accepted, and records the reasoning publicly;
* owns the specification, and the authority order in
  [`spec/README.md`](spec/README.md);
* owns security reports and the disclosure process;
* is the final reviewer for changes to `spec/`, `grammar/`, `neps/`,
  `.github/`, `SECURITY.md`, `GOVERNANCE.md` and `AGENTS.md`, as listed in
  [`CODEOWNERS`](.github/CODEOWNERS);
* may delegate any of the above, and will record it in this file when they do.

Being the decision maker is not the same as being right. A decision that is
recorded with its reasoning can be revisited; that is the point of writing it
down.

## How decisions are made

| Kind of decision | Mechanism |
| ---------------- | --------- |
| Language semantics, surface syntax, trust rules, capabilities | NEP, accepted by the maintainer |
| Error codes | NEP only when the code changes meaning; otherwise a normal pull request plus `spec/errors.md` |
| Compiler internals that do not change behaviour | Normal pull request, maintainer review |
| Repository layout, tooling, CI | Normal pull request, maintainer review |
| Emergency security fix | Maintainer, disclosed in `CHANGELOG.md` and, if relevant, a NEP afterwards |
| Governance itself | This file, by pull request |

Design decisions are recorded, not remembered. If a design question is settled
in a discussion and not in a NEP or a specification chapter, it is not settled.

## The NEP process

A NEP is a [NUDO Enhancement Proposal](neps/README.md). States:

| State | Meaning |
| ----- | ------- |
| `Draft` | Written, not yet open for review |
| `Discussion` | Reviewers are engaging with it |
| `Accepted` | The design is agreed; implementation may start |
| `Implemented` | The specification and the implementation both exist |
| `Stabilized` | Covered by conformance tests and frozen for the current edition |
| `Rejected` | Not happening; the reasoning stays in the pull request |
| `Withdrawn` | The author stopped pursuing it |
| `Superseded` | Replaced by a later NEP, which is linked |

A NEP is accepted when at least one of these is true:

* nobody has raised an unresolved objection, and the maintainer accepts it;
* objections were raised and the maintainer answers them on the record.

Acceptance means "we will build this and live with the consequences", not
"this is perfect". Proposals that turn out to be wrong are superseded, which is
a normal outcome and not a failure.

## Review of design

Design review is the highest-value contribution to a pre-alpha language, and it
is welcome from anyone. Useful review says:

* which program this design makes impossible, and whether that is intended;
* which program it makes *easy* that should be hard;
* what happens at the boundary — with a runtime capability, a remote agent, a
  hostile tool, a partially available model;
* whether the rule can be checked statically, and what happens if it cannot.

"It would be more convenient" is a legitimate argument. It is not a sufficient
one.

## Stabilisation criteria

Nothing is stable before 1.0. A feature moves to `Stabilized` only when all of
these hold:

1. A specification chapter describes it completely, including its error
   behaviour.
2. An accepted NEP exists, and its open questions are closed.
3. Conformance cases cover its valid and invalid forms.
4. At least one implementation passes those cases.
5. The capability and trust consequences have been reviewed against
   [`docs/security/threat-model.md`](docs/security/threat-model.md).
6. The maintainer records the decision and the date in the NEP.

Until then, everything may change. Documented changes are announced in
[`CHANGELOG.md`](CHANGELOG.md).

## Compatibility

Before 1.0, breaking changes are allowed and expected. They are not free:

* a breaking change is described in `CHANGELOG.md` with a migration note;
* a breaking change to something in `spec/` cites the NEP that justifies it;
* the version increases, per Semantic Versioning.

After 1.0, the intent is that breaking changes require a new edition, with the
previous edition still supported by the toolchain. That design does not exist
yet; it will be written as a NEP, and it will be written against a language that
runs, not one that does not.

## Towards community teams

The direction is that decision-making widens as the project earns contributors:
reviewers with merge rights per area, then a team per area, then a steering
group for cross-cutting decisions. No timeline is promised, because a governance
structure with no contributors behind it is theatre. It happens when there are
people who will actually hold the responsibility, and it will be recorded here
first.
