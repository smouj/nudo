# Approvals

**State: proposed.** No approval flow exists (milestone M8).

## What an approval is

An approval is a **checkable step** in a program where a person decides whether
work continues. It is not a chat message, not a notification, and not a
convention: it is a point in the program that either received a decision or did
not.

## Why it is in the language

Every system with autonomous behaviour eventually needs a human in the loop for
the actions that are irreversible, expensive or sensitive. When that requirement
lives in a chat window, three things go wrong:

* the program does not know an approval is pending, so it cannot represent it;
* nothing records who approved what, so the audit is a screenshot;
* the requirement is invisible in the source, so a reviewer cannot see where the
  boundaries are.

Making approval a language construct fixes all three.

## Shape

Provisional:

```nudo
let approved = approve PublishArticle {
    article: article
} with reason: "Ready for publication"
```

* the request carries what is being approved, in a form a person can read;
* the result is a decision the program must handle — including refusal and
  timeout;
* the decision is recorded with the identity of the approver.

## Rules

1. **An approval is explicit.** There is no implicit "assume approved" path, and
   no default value.
2. **The request shows what is being approved.** An approval prompt that does not
   show the thing being approved is a rubber stamp. What a person sees is
   specified, not left to the runtime.
3. **Silence is not consent.** A timeout is a refusal, or a distinct outcome the
   caller must handle. It is never an implicit yes.
4. **Approvals are recorded.** Who, when, what, and on what basis. That record is
   provenance ([`provenance.md`](provenance.md)).
5. **An approval covers what it names.** Not the run, not the session: the
   specific action. Blanket approval is a policy question
   ([`policies.md`](policies.md)), not an approval.
6. **An approval cannot widen authority.** Approving an action does not grant a
   capability it did not have.
7. **Approval fatigue is a design failure.** If a program asks too often, people
   stop reading. The specification expects approvals to be rare, and a design
   that needs many of them to be wrong.

## Approver identity

Required, and harder than it looks: a person on the other end of a request has to
be identified well enough that the record means something. The mechanism is
runtime infrastructure, and the *requirement* — an approval records who gave it —
is a language rule.

## Open questions

* How approvals are delivered and answered across channels, without the language
  depending on any particular one.
* Whether an approval can be delegated, and to whom.
* How an approval interacts with a task that is suspended while waiting.
* Whether approvals can be required by a policy as well as by source code, and
  how the two compose.
