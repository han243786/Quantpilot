# Frontend Test Fixtures

This directory contains shared frontend test fixtures for `vitest` and Playwright E2E.

## Layout

### `capabilities/`

Capability discovery and fallback fixtures.

- [backend-capabilities-v1.json](/D:/rust-js-pr/QuantPilot/quantpilot/frontend/src/test/fixtures/capabilities/backend-capabilities-v1.json)
  Real `/api/capabilities` snapshot exported from the Rust backend.
- [capabilityFallbacks.js](/D:/rust-js-pr/QuantPilot/quantpilot/frontend/src/test/fixtures/capabilities/capabilityFallbacks.js)
  Shared capability bootstrap helpers for tests:
  - healthy backend capability snapshot
  - capability cache key
  - service-unavailable fallback response

Use this group when testing:
- capability sync success
- cached capability fallback
- safe fallback mode
- module visibility driven by backend capability truth

### `runtime/`

Runtime bootstrap and structured error fixtures.

- [editorBootstrap.js](/D:/rust-js-pr/QuantPilot/quantpilot/frontend/src/test/fixtures/runtime/editorBootstrap.js)
  Shared editor startup responses for:
  - `/api/graphs/latest`
  - `/api/runtime/runs`
  - `/api/runtime/backtests`
- [capabilityRejections.js](/D:/rust-js-pr/QuantPilot/quantpilot/frontend/src/test/fixtures/runtime/capabilityRejections.js)
  Shared structured backend responses for:
  - compile capability rejection
  - simulation start capability rejection
  - backtest capability rejection
  - successful compile precondition fixture
- [runSuccess.js](/D:/rust-js-pr/QuantPilot/quantpilot/frontend/src/test/fixtures/runtime/runSuccess.js)
  Shared successful simulation responses for:
  - `/api/runtime/v4/run`
  - `/api/runtime/runs`
  - `/api/runtime/runs/:run_id`
  - `/api/runtime/runs/:run_id/events`
- [backtestSuccess.js](/D:/rust-js-pr/QuantPilot/quantpilot/frontend/src/test/fixtures/runtime/backtestSuccess.js)
  Shared successful backtest responses for:
  - `/api/runtime/backtest`
  - `/api/runtime/backtests`
  - `/api/runtime/backtests/:backtest_id`
  - default artifact-only payloads via `backtest_artifacts`

Use this group when testing:
- editor boot stability
- compile / run / backtest error handling
- simulation happy-path smoke flows
- backtest happy-path smoke flows
- runtime error notices
- capability rejection chains after capability sync succeeds

## Maintenance Rules

- Prefer reusing fixture modules instead of inlining JSON or response bodies in tests.
- Keep one fixture per response family. Do not duplicate the same payload across E2E and unit tests.
- When `/api/capabilities` changes, regenerate the backend snapshot via [export-capability-fixture.ps1](/D:/rust-js-pr/QuantPilot/quantpilot/tools/export-capability-fixture.ps1).
- Treat backend-exported snapshots as source-of-truth fixtures. Helper modules should wrap them, not fork them.
- If a new backend error shape is introduced, add it to `runtime/` and migrate existing tests to consume the shared fixture.

## Current Consumers

- [graphStore.capabilities.test.js](/D:/rust-js-pr/QuantPilot/quantpilot/frontend/src/store/graphStore.capabilities.test.js)
- [TopToolbar.capabilities.test.jsx](/D:/rust-js-pr/QuantPilot/quantpilot/frontend/src/components/TopToolbar.capabilities.test.jsx)
- [BacktestDetailPage.test.jsx](/D:/rust-js-pr/QuantPilot/quantpilot/frontend/src/pages/BacktestDetailPage.test.jsx)
- [EventStreamPanel.backtestArtifacts.test.jsx](/D:/rust-js-pr/QuantPilot/quantpilot/frontend/src/components/EventStreamPanel.backtestArtifacts.test.jsx)
- [editor-capabilities-smoke.spec.js](/D:/rust-js-pr/QuantPilot/quantpilot/frontend/tests/e2e/editor-capabilities-smoke.spec.js)
- [run-simulation.spec.js](/D:/rust-js-pr/QuantPilot/quantpilot/frontend/tests/e2e/run-simulation.spec.js)
- [run-backtest.spec.js](/D:/rust-js-pr/QuantPilot/quantpilot/frontend/tests/e2e/run-backtest.spec.js)
