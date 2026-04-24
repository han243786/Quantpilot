# Test Layer Expectations

## Purpose

This document defines how to interpret QuantPilot test results during closeout.
It does not add a new test framework or widen product scope.

The goal is to keep green tests honest:

- targeted tests prove the contract they name
- the full gate proves the current repository baseline still holds together
- E2E proves browser-level entry behavior against the isolated API-mock contract
- no test layer should be described as proof of capability that the beta product
  does not actually expose

## Layer contracts

| Layer | Canonical command | Proves | Does not prove |
|---|---|---|---|
| UTF-8 gate | `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1` | active frontend and markdown files do not contain encoding regressions covered by the gate | semantic correctness of wording |
| user-facing text gate | `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-user-facing-text.ps1` | known mojibake and banned wording patterns are absent from active product paths | every sentence is product-accurate |
| capability governance gate | `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-capability-governance.ps1` | generated capability governance snapshot matches the frontend support matrix | runtime behavior for every capability |
| Rust workspace tests | `cargo test --workspace` | Rust crates, API integration tests, runtime contracts, compile paths, and protocol fixtures remain internally consistent | frontend rendering or browser interaction |
| targeted Rust tests | focused `cargo test ...` commands | the named Rust contract or regression still behaves as expected | unrelated crate-wide health |
| frontend unit tests | `cd frontend; cmd /c npm run test` | components, stores, capability projection, compile state, runtime projection, and page-level UI contracts remain stable | full browser navigation or production asset output |
| targeted frontend tests | focused `cmd /c npm run test -- src/...` commands | the named component, store, or utility contract still behaves as expected | full frontend baseline health |
| frontend build | `cd frontend; cmd /c npm run build` | production assets compile and route-level chunks can be emitted | runtime correctness or E2E behavior |
| frontend E2E | `cd frontend; cmd /c npm run test:e2e` | browser entry flows, capability fallback behavior, compile/run/backtest smoke paths, and blocked-path surfacing work under the isolated mock contract | live backend integration, exchange connectivity, research-grade behavior, or unsupported product capability |
| closeout wrapper | `cmd /c tools\run-closeout-gates.bat` | the current baseline gate set passes in the documented Windows form | that no narrower targeted test is needed for a risky local change |

## E2E interpretation rules

The current E2E suite is intentionally isolated.
It must remain runnable without manually pre-starting the backend.

Rules:

- E2E may use fixed API fixtures and route-level mocks for browser contract
  coverage.
- unmocked API requests are failures, not acceptable proxy fallbacks.
- E2E should stay small and focused on entry behavior, fallback behavior, and
  critical blocked paths.
- E2E pass status must not be described as proof of live backend availability,
  external exchange connectivity, or broad strategy support.

## Targeted test interpretation rules

Targeted tests are useful during closeout because they keep feedback fast.
They should be named and selected by the contract they protect.

Use targeted tests when:

- a change touches one component, store, utility, or backend contract
- a previous regression needs a tight guardrail
- the full gate would be slow while iterating

Do not treat targeted test success as a substitute for the full closeout gate
when a change touches shared behavior, cross-module contracts, user-facing
workflow, or release documentation.

## Common targeted regression commands

Run these from the repository root unless a command explicitly changes into
`frontend`.

### Compile-chain wording and failure guidance

Use this when compile summary, compile action failure wording, Strategy IR
preflight, or runtime compile source wording changes:

```powershell
cd frontend; cmd /c npm run test -- src/components/PropertyPanel.compileSummary.test.jsx src/utils/actionFailure.test.js src/store/graphStore.strategyIrCompile.test.js
```

### Capability exposure and support matrix

Use this when `/api/capabilities` interpretation, frontend capability gates,
module exposure, or support-matrix wording changes:

```powershell
cd frontend; cmd /c npm run test -- src/capabilities/supportMatrix.test.js src/capabilities/capabilityGovernance.test.js src/components/TopToolbar.capabilities.test.jsx src/components/ModuleSidebar.test.jsx src/pages/StrategyWorkspaceExperimentCard.test.jsx src/store/graphStore.capabilities.test.js
```

### Runtime, replay, and persisted-detail explanation

Use this when runtime explanation rows, backtest detail explanation, event
replay, or persisted-selection shaping changes:

```powershell
cd frontend; cmd /c npm run test -- src/utils/runtimeExplanation.test.js src/components/EventStreamPanel.historyExplanation.test.jsx src/components/EventStreamPanel.backtestHistory.test.jsx src/components/EventStreamPanel.backtestArtifacts.test.jsx src/components/EventReplaySection.test.jsx src/pages/BacktestDetailPage.test.jsx src/pages/BacktestComparePage.test.jsx src/pages/StrategyBacktestsPage.test.jsx src/store/graphStoreRuntimeHistoryFlow.test.js src/store/graphStoreRuntimeSelectionState.test.js src/store/graphStorePersistenceConsistency.test.js
```

Use these backend checks when API detail, replay, persisted run/backtest
records, or artifact-backed reload behavior changes:

```powershell
cargo test --test api_run -- --nocapture
cargo test --test api_backtest -- --nocapture
```

### Formal QuantScript retained surface

Use this when retained authoring samples, boundary fixtures, parser/lowering
wording, or formal QuantScript compile behavior changes:

```powershell
cargo test --test quantscript_real_strategy_authoring -- --nocapture
cargo test -p quantscript --lib
cd frontend; cmd /c npm run test -- src/graph/quantscript.test.js src/components/StrategyCodePanel.authoringView.test.jsx
```

### E2E browser smoke paths

Use this when capability fallback behavior or compile/run/backtest browser
entry paths change:

```powershell
cd frontend; cmd /c npx playwright test tests/e2e/editor-capabilities-smoke.spec.js tests/e2e/run-simulation.spec.js tests/e2e/run-backtest.spec.js --project=msedge --workers=1
```

These commands are focused tools.
Run the full closeout wrapper before claiming a shared closeout slice is done.

## Full gate interpretation rules

The full closeout wrapper is the baseline confidence check.
It should pass before claiming a closeout slice is complete.

The full gate is expected to prove:

- active text gates still pass
- capability governance is current
- Rust workspace behavior is green
- frontend unit tests are green
- frontend production build is green
- frontend E2E smoke paths are green under the isolated API-mock contract

The full gate does not replace code review.
It also does not prove deferred capability support.

## Documentation rule

When reporting verification, state the layer that was run and what it proves.
Avoid vague summaries such as `tests prove everything works`.

Preferred wording:

- targeted frontend regression passed for the touched compile-summary surface
- full closeout gate passed for the current baseline
- E2E passed under the isolated API-mock contract

Avoid wording that suggests:

- E2E covered live backend integration
- a mock-backed smoke path proves live market connectivity
- a parser acceptance test proves release-facing QuantScript authoring support
- a compatibility-only parser test proves the retained executable QuantScript
  surface
