# 07.2 — Budgets and approvals

> **Status:** Proposed  
> **Summary:** Budgets bound resource consumption; approvals create explicit human or policy-controlled gates in autonomous workflows.

A budget can represent more than money. Useful dimensions include calls, elapsed
time, token count, retries and provider cost. The runtime must define which limits
are authoritative and what happens when several limits are reached together.

Approvals should be first-class events rather than informal chat messages. An
approval record should say who/what approved, what exact value or action was
approved, and under which policy.
