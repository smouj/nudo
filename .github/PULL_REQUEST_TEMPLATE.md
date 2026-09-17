## Summary

<!-- What changes, in one or two sentences. -->

## Motivation

<!-- Why this change is needed. Link the issue, discussion or NEP. -->

## Changes

<!-- The list of things a reviewer should look at, in the order they matter. -->

## Language impact

<!--
Does this change what a program means, or how it is written?

If yes, link the accepted NEP and the specification chapter it changes. A
language change without an accepted NEP is not reviewable and will be closed.
If no, say so explicitly: "no language impact" is a useful sentence.
-->

## Security impact

<!--
Does this touch capabilities, policies, approvals, trust types, delegation,
sandboxing, secrets, or workflow permissions?

If yes, describe what an attacker gains or loses. If it weakens a mitigation in
docs/security/threat-model.md, say which one and why that is acceptable.
-->

## Breaking changes

<!-- What stops working? Include a migration note for CHANGELOG.md. -->

## Tests

<!--
What did you run, and what was the result? State it as output, not as
confidence. For example:

    $ scripts/check.sh
    all checks passed

A change in compiler behaviour must come with fixtures or conformance cases. If
a test expectation was regenerated, say so and explain why the diff is
intended.
-->

## Documentation

<!-- Which documents change, and which should have changed but did not. -->

## Related NEP / Issue

<!-- NEP-0002, #123, or "none". -->

## Checklist

- [ ] I read [`AGENTS.md`](../AGENTS.md) and this change respects it
- [ ] `scripts/check.sh` passes, and I am reporting its real output
- [ ] No new dependency, or the justification is in the description
- [ ] No secret, credential or personal data is in the diff
- [ ] Commit messages follow the format in `CONTRIBUTING.md`
- [ ] A language change has an accepted NEP, and the specification is updated
      in a separate, earlier pull request
- [ ] Nothing in this pull request claims to work when it does not
