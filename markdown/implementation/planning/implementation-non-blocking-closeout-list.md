# Completed P2 Closeout Ledger

## Purpose

This file is the completed ledger for the non-blocking `P2` cleanup items that
followed the `P0` and `P1` closeout slices.

It is no longer an active task queue.
Do not add new cleanup items here.
If a new release blocker appears, route it through
[First Release Readiness](./implementation-first-release-readiness.md).
If a new implementation contract changes, update the owning contract doc
directly instead of reopening this ledger.

`P2-02` active markdown entry compression is already landed.
Overview docs now route to active contract and planning docs instead of acting
as a second implementation log.

`P2-05` terminology drift sweep is also landed.
High-signal active docs no longer keep the old `optimization queue` or
`active queue` wording in the live closeout path.

`P2-07` test-layer expectation notes are also landed.
The active runtime docs now define what targeted tests, full closeout gates, and
isolated E2E runs prove.

`P2-06` baseline and release-readiness thin-entry cleanup is also landed.
First release readiness is now the single short owner-decision entry instead of
a broad planning checklist.

`P2-04` targeted regression command hygiene is also landed.
Common closeout regression commands now live in
[Test Layer Expectations](/D:/rust-js-pr/QuantPilot/quantpilot/markdown/implementation/runtime/implementation-test-layer-expectations.md).

`P2-03` QuantScript compatibility wording hardening is also landed.
Parser-only compatibility coverage now stays visibly separate from release-facing
authoring support in tests and active contracts.

`P2-01` copy source consolidation sweep is also landed.
Compile truth, action-failure recovery, and capability exposure wording now have
explicit shared-source owners in the active compile-chain contract.

`P2-08` retired planning and stale-doc cleanup is also landed.
The Plugin Roadmap no longer appears as an active planning entry; only a retired
archive summary remains for historical context.

It is not a second feature roadmap.
It does not authorize new capability growth.
Use it only for wording cleanup, duplicate-truth-source removal, doc/index
compression, and release-hygiene finishing work that does not expand the beta
boundary.

## Current list

No active `P2` cleanup item remains in this short list.

## Guardrails

- no new features
- no new platform claims
- no support-matrix widening
- no second roadmap
- delete or archive clearly stale material instead of keeping `maybe later`
  growth language in active docs
