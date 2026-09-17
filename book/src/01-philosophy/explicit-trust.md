# 01.2 — Explicit trust

> **Status:** Proposed  
> **Summary:** NUDO separates generated and verified values so that trust transitions can be visible to reviewers and checkers.

## The core distinction

```text
Generated<Article>
        ↓ verify
Verified<Article>
```

A generated value is not automatically a verified value, and parsing is not the
same operation as verification.

This prevents one of the most common conceptual collapses in model-heavy systems:
"the output had the right shape" becoming "the output is safe to rely on".

## What verification is not

It is not intended to be a magic cast. A verifier can fail. Verification should
leave provenance explaining what input, rule, model output and verifier produced
the trusted value.
