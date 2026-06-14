# FE-0221 - Frontend Test Support Parent Closeout

Status: closed.

## Parent Node

`frontend.test_support`

## Closed Children

- `frontend.test_support.vitest_runtime_setup`
- `frontend.test_support.dev_test_bridge`
- `frontend.test_support.unit_fixture_catalog`
- `frontend.test_support.e2e_api_harness`
- `frontend.test_support.e2e_bootstrap_review_fixtures`

## Final Parent Boundary

`frontend.test_support` owns frontend unit-test runtime setup, the dev-only test bridge, shared unit fixtures, E2E API mocking, and E2E bootstrap/review support fixtures.

Application shell, routing, API clients, feature pages, graph editor logic, store behavior, styles, backend routes, and E2E spec bodies remain outside this parent.

## Whitebox Contract

### Public Inputs

- Vitest environment and setup lifecycle.
- Playwright `page` and API harness usage from E2E specs.
- Shared runtime, graph, capability, editor bootstrap, and backtest fixture inputs.
- Dev-only browser test bridge access through `window.__QUANTPILOT_TEST__`.

### Public Outputs

- Stable Vitest configuration and setup hooks.
- Stable shared test fixture modules under `frontend/src/test/fixtures`.
- Stable E2E support helpers under `frontend/tests/e2e/support`.
- Dev-only bridge methods for E2E and manual development checks.

## Preserved Behavior

- Unit tests and Playwright specs keep their existing import paths.
- The dev test bridge remains development-only and does not create production behavior.
- E2E API mocking remains guarded by the existing harness contract.
- No E2E spec body cleanup or global governance merge-back was performed.

## Further-Split Decision

No further split is useful inside `frontend.test_support` now. All planned child leaves are closed, and the remaining E2E spec-body cleanup is explicitly deferred outside this frontend support parent.

## Verification

- This parent closeout only changes frontend-local governance files.
- From repo root, `git diff --check`: passed.
- From repo root, frontend recursive state JSON parse: passed.
- From repo root, `powershell -ExecutionPolicy Bypass -File tools/check-full-feature-tree.ps1`: passed.
- From repo root, `powershell -ExecutionPolicy Bypass -File tools/check-matrix-governance.ps1`: passed.

## Next Parent Candidate

- none. All frontend-local parent queues are closed.
