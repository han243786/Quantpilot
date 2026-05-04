# Retired Plugin Roadmap Summary

This document is retained for historical context only.
It is not an active planning entry for the current closeout phase.

## Retired status

The former active `implementation-plugin-roadmap.md` described mid-term plugin
expansion after sandbox stabilization.
That framing is now retired from active planning navigation because the current
work is release closeout, wording cleanup, and contract stabilization.

## Current truth sources

Use these instead:

- active plugin protocol boundary:
  [RFC-020 Plugin Manifest Protocol](../../protocol/RFC-020-plugin-manifest-protocol.md)
- current capability exposure:
  [Capability Governance](../../implementation/governance/implementation-capability-governance.md)
- release closeout execution:
  [Archived Functional Closeout Ledger](./implementation-functional-closeout-task-table.md)
- non-blocking cleanup:
  [Archived P2 Closeout Ledger](./implementation-non-blocking-closeout-list.md)

## Preserved context

The first narrow local plugin-metadata slice had already landed:

- `qrpc_core` owns canonical plugin manifest and validation structures
- `qrpc_runtime` owns lifecycle-bounded runtime plugin registry primitives
- the frontend validates local external plugin metadata before admission

This never became a remote registry, third-party install flow, graph-lock
feature set, signature system, or broader plugin marketplace commitment.
