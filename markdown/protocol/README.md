# Quant Runtime Protocol Core RFC Index

This directory is the active QRPC protocol baseline.

Only the RFC files that remain under `markdown/protocol` are normative for current
development, review, and implementation work. The active protocol surface was
renumbered into the continuous range `RFC-001` through `RFC-020`.

Retired RFCs were moved to `markdown/archive/protocol-retired`, keep their legacy
identifiers there, and must not be used as active design input.

## Active Protocol Set

QuantPilot now uses a continuous 20-RFC active surface:

1. `RFC-001` Data Request Protocol
2. `RFC-002` Normalized Market Data Protocol
3. `RFC-003` Runtime State Protocol
4. `RFC-004` Agent Protocol
5. `RFC-005` Intent Protocol
6. `RFC-006` Intent Generator Protocol
7. `RFC-007` Portfolio Protocol
8. `RFC-008` Global Risk Control Protocol
9. `RFC-009` Risk Decision Protocol
10. `RFC-010` Allocation Protocol
11. `RFC-011` Execution Plan Protocol
12. `RFC-012` Order Protocol
13. `RFC-013` Execution Feedback Protocol
14. `RFC-014` Runtime Mode Protocol
15. `RFC-015` Runtime Event Protocol
16. `RFC-016` Capability Discovery Protocol
17. `RFC-017` Backtest Artifact Protocol
18. `RFC-018` Backtest Input Protocol
19. `RFC-019` Backtest Output Artifact Protocol
20. `RFC-020` Plugin Manifest Protocol

## Core Chain

The active runtime chain remains:

`Data Request -> Normalized Market Data -> Intent -> Agent Decision -> Global Risk Decision -> Execution Plan -> Runtime Events`

## Active RFC Mapping

- [RFC-001](D:/rust-js-pr/QuantPilot/quantpilot/markdown/protocol/RFC-001-data-request-protocol.md): `DataRequest`
- [RFC-002](D:/rust-js-pr/QuantPilot/quantpilot/markdown/protocol/RFC-002-normalized-market-data-protocol.md): `NormalizedMarketData`
- [RFC-003](D:/rust-js-pr/QuantPilot/quantpilot/markdown/protocol/RFC-003-runtime-state-protocol.md): `RuntimeState`
- [RFC-004](D:/rust-js-pr/QuantPilot/quantpilot/markdown/protocol/RFC-004-agent-protocol.md): `Agent`
- [RFC-005](D:/rust-js-pr/QuantPilot/quantpilot/markdown/protocol/RFC-005-intent-protocol.md): `Intent`
- [RFC-006](D:/rust-js-pr/QuantPilot/quantpilot/markdown/protocol/RFC-006-intent-generator-protocol.md): `IntentGenerator`
- [RFC-007](D:/rust-js-pr/QuantPilot/quantpilot/markdown/protocol/RFC-007-portfolio-protocol.md): `Portfolio`
- [RFC-008](D:/rust-js-pr/QuantPilot/quantpilot/markdown/protocol/RFC-008-risk-protocol.md): `GlobalRiskController`
- [RFC-009](D:/rust-js-pr/QuantPilot/quantpilot/markdown/protocol/RFC-009-risk-decision-protocol.md): `RiskDecision`
- [RFC-010](D:/rust-js-pr/QuantPilot/quantpilot/markdown/protocol/RFC-010-allocation-protocol.md): `Allocation`
- [RFC-011](D:/rust-js-pr/QuantPilot/quantpilot/markdown/protocol/RFC-011-execution-plan-protocol.md): `ExecutionPlan`
- [RFC-012](D:/rust-js-pr/QuantPilot/quantpilot/markdown/protocol/RFC-012-order-protocol.md): `Order`
- [RFC-013](D:/rust-js-pr/QuantPilot/quantpilot/markdown/protocol/RFC-013-execution-feedback-protocol.md): `ExecutionFeedback`
- [RFC-014](D:/rust-js-pr/QuantPilot/quantpilot/markdown/protocol/RFC-014-runtime-mode-protocol.md): `RuntimeMode`
- [RFC-015](D:/rust-js-pr/QuantPilot/quantpilot/markdown/protocol/RFC-015-runtime-event-protocol.md): `RuntimeEvent`
- [RFC-016](D:/rust-js-pr/QuantPilot/quantpilot/markdown/protocol/RFC-016-capability-discovery-protocol.md): capability discovery and current beta support boundary
- [RFC-017](D:/rust-js-pr/QuantPilot/quantpilot/markdown/protocol/RFC-017-backtest-artifact-protocol.md): compile artifact bundle and backtest artifact identity
- [RFC-018](D:/rust-js-pr/QuantPilot/quantpilot/markdown/protocol/RFC-018-backtest-input-protocol.md): `RunSpec`, `BacktestSpec`, and input-side replay schema
- [RFC-019](D:/rust-js-pr/QuantPilot/quantpilot/markdown/protocol/RFC-019-backtest-output-artifact-protocol.md): `EventLogArtifact`, projection artifacts, and reproducibility manifest
- [RFC-020](D:/rust-js-pr/QuantPilot/quantpilot/markdown/protocol/RFC-020-plugin-manifest-protocol.md): minimal plugin manifest, compatibility boundary, and extension point whitelist

## Archive Boundary

Retired RFCs live only under
[protocol-retired](D:/rust-js-pr/QuantPilot/quantpilot/markdown/archive/protocol-retired/README.md).

Current development should treat the 20 files in this directory as the only active
protocol numbering space.
