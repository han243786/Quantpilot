# QuantPilot Docs Index

This file is the concise active catalog inside the layered Markdown structure.
Use it to find the current active docs quickly.
For subtree navigation, start with [../README.md](../README.md).

## Start here

1. [Current Status And Release State](./overview-current-status-and-roadmap.md)
2. [Implementation Planning Index](../implementation/planning/README.md)
3. [Implementation Governance Index](../implementation/governance/README.md)
4. [Implementation Runtime Index](../implementation/runtime/README.md)
5. [Guides Index](../guides/README.md)

## Release and closeout docs

- [First Release Readiness](../implementation/planning/implementation-first-release-readiness.md)
- [Completed Functional Closeout Ledger](../implementation/planning/implementation-functional-closeout-task-table.md)
- [Completed P2 Closeout Ledger](../implementation/planning/implementation-non-blocking-closeout-list.md)

## Active contract docs

- [Support Matrix](../implementation/governance/implementation-support-matrix.md)
- [Compile-Chain Contract](../implementation/governance/implementation-compile-chain-contract.md)
- [QuantScript Retained-Surface Contract](../implementation/governance/implementation-quantscript-retained-surface-contract.md)
- [Runtime / Backtest Explanation Contract](../implementation/runtime/implementation-runtime-backtest-explanation-contract.md)
- [Persistence / Replay Contract](../implementation/runtime/implementation-persistence-replay-contract.md)
- [Test Layer Expectations](../implementation/runtime/implementation-test-layer-expectations.md)
- [Trading Sandbox Implementation](../implementation/runtime/implementation-trading-sandbox.md)
- [Deterministic Test Mode](../implementation/runtime/implementation-test-mode.md)

## QuantScript reference docs

- [QuantScript Supported Surface](../../quantscript/QUANTSCRIPT_SUPPORTED_SURFACE.md)
- [QuantScript Technical Guide](../../quantscript/QUANTSCRIPT_TECHNICAL_GUIDE.md)
- [QuantScript AI Guide](../../quantscript/QUANTSCRIPT_AI_GUIDE.md)
- [QuantScript Real Strategy Authoring Trial](../../quantscript/QUANTSCRIPT_REAL_STRATEGY_AUTHORING_TRIAL.md)
- [QuantScript Trunk Baseline](../guides/quantscript/guide-quantscript-trunk-baseline.md)
- [Formal QuantScript Syntax Guide](../guides/quantscript/guide-formal-quantscript-syntax.md)
- [V1 Freeze / De-scope Checklist](../guides/quantscript/guide-v1-freeze-descope-checklist.md)

## Supporting indexes

- [Principles Index](../principles/README.md)
- [Research Reference Index](../research/README.md)
- [Review Reference Index](../reviews/README.md)
- [Active Protocol RFC Index (`RFC-001` to `RFC-020`)](../protocol/README.md)
- [Archive Index](../archive/archive-index.md)

## Structure rules

- `README.md` at each directory level is the navigation entry for that subtree.
- overview docs should summarize and route, not duplicate implementation
  contracts
- implementation docs define active product, governance, runtime, and planning
  truth
- research and review docs are reference-only and must not override active
  implementation, release, or support-boundary docs
- `archive/` keeps historical material and is not the current implementation
  source of truth
- document filenames stay ASCII, and file content should stay UTF-8
