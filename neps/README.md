# NEPs — NUDO Enhancement Proposals

A NEP is how a change to the language is proposed, argued and recorded. It
exists so that a decision can be reviewed later, by someone who was not in the
room, and understood on its merits rather than on its author's authority.

The name of the process is *proposal* on purpose: a NEP is not a vote, and not a
specification. It is a design argument, and its reasoning is the part that
matters after the code has been rewritten twice.

## When a NEP is required

* any change to what a program **means**;
* any change to surface syntax, including reserving a keyword;
* a new diagnostic code, or a change to an existing one's meaning;
* a new capability, effect or trust rule;
* a change to the compiler pipeline's boundaries;
* a stabilisation decision.

Not required: bug fixes, tests, documentation, internal refactoring that does
not change behaviour, and tooling that does not affect the language.

See [`../CONTRIBUTING.md`](../CONTRIBUTING.md) for how to open one, and
[`../GOVERNANCE.md`](../GOVERNANCE.md) for who decides and on what basis.

## Index

| NEP | Title | State |
| --- | ----- | ----- |
| [0001](0001-agent-types.md) | Agent types | Draft |
| [0002](0002-generated-verified.md) | `Generated<T>` and `Verified<T>` | Discussion |
| [0003](0003-capability-system.md) | Capability system | Draft |
| [0004](0004-effects.md) | Effect system | Draft |

## States

| State | Meaning |
| ----- | ------- |
| `Draft` | Written, not yet open for review. The author may still rewrite it |
| `Discussion` | Reviewers are engaging with it. The number is stable |
| `Accepted` | The design is agreed; implementation may start |
| `Implemented` | The specification and the implementation both exist |
| `Stabilized` | Covered by conformance tests, and frozen for the current edition |
| `Rejected` | Not happening. The reasoning stays in the pull request |
| `Withdrawn` | The author stopped pursuing it |
| `Superseded` | Replaced by a later NEP, which is linked from the header |

A NEP never moves backwards. A design that needs to change after `Accepted` gets
a new NEP that supersedes it, because edits to an accepted proposal destroy the
record of what was agreed.

## Numbering

Numbers are assigned in order and never reused. `0000` is the template. A NEP
number is a permanent identifier: it appears in commit messages, in
`CHANGELOG.md`, and in the specification chapter it produced.

## Writing a NEP

Start from [`0000-template.md`](0000-template.md). A good NEP:

* states the problem before the solution, and shows a program that is awkward
  today;
* says what it makes **impossible**, not only what it makes possible — every
  design closes doors, and the ones you closed are what reviewers need to see;
* gives at least one rejected alternative and why it lost;
* lists its own open questions, so that "we will decide later" is visible
  instead of latent;
* is short enough to be read in one sitting. A NEP that needs a summary has
  already failed at being a proposal.

The authority order in [`../spec/README.md`](../spec/README.md) applies: an
accepted NEP changes the specification, and the specification then governs the
implementation. Never the other way around.
