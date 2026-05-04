# Planning Docs

## Active docs

- [First Release Readiness](./implementation-first-release-readiness.md)
- [v0.2.0 Upgrade Worklist](./implementation-v0-2-upgrade-worklist.md)

Use [First Release Readiness](./implementation-first-release-readiness.md) as
the single active entry for:

- baseline go/no-go
- owner-only release blockers
- first baseline commit checklist
- current closeout optimization checklist

Use [v0.2.0 Upgrade Worklist](./implementation-v0-2-upgrade-worklist.md)
as the active task queue for the second-stage governance, evidence, controlled
mutation, AI approval, and contract-first hardening work.

Use [Runtime Governance Contract](../runtime/implementation-runtime-governance-contract.md)
as the active contract source of truth for capability contract fields, event
envelopes, runtime governance snapshots, deployment revisions, and permission
boundaries. The v0.2.0 worklist tracks progress only after those fields are
formalized.

Use [Runtime Mutation Contract](../runtime/implementation-runtime-mutation-contract.md)
as the active contract source of truth for controlled runtime parameter
mutation, safe-window evaluation, rollback, governed mutation events, health
metrics, and Block 4 AI approval handoff constraints.

Use [Runtime AI Approval Contract](../runtime/implementation-runtime-ai-approval-contract.md)
as the active contract source of truth for AI proposal candidates, static-check
state, AI proposal ledger records, governed AI proposal events, and the future
approval-chain handoff.

Completed closeout execution records have been moved to the archive.
They are not active task queues and must not be reopened as roadmap items.

## Historical background docs

These files are retained in the archive for background only.
They are not the active source of truth for the current cleanup phase.

- [Retired Planning Docs](../../archive/planning-retired/README.md)
- [Archived Functional Closeout Ledger](../../archive/planning-retired/implementation-functional-closeout-task-table.md)
- [Archived P2 Closeout Ledger](../../archive/planning-retired/implementation-non-blocking-closeout-list.md)
- [Retired Plugin Roadmap Summary](../../archive/planning-retired/implementation-plugin-roadmap.md)

Planning notes that mention protocol work should be interpreted against the active
continuous baseline `RFC-001` through `RFC-020` at
[../../protocol/README.md](../../protocol/README.md).
