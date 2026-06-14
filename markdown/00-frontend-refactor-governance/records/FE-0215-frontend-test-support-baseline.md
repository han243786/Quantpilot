# FE-0215 - Frontend Test Support Baseline

Status: closed.

## Parent Node

`frontend.test_support`

## Scope

- Vitest runtime setup:
  - `frontend/vitest.config.js`
  - `frontend/src/test/setup.js`
- DEV-only test bridge:
  - `frontend/src/test/testBridge.js`
- Shared unit fixture catalog:
  - `frontend/src/test/fixtures/README.md`
  - `frontend/src/test/fixtures/capabilities/backend-capabilities-v1.json`
  - `frontend/src/test/fixtures/capabilities/capabilityFallbacks.js`
  - `frontend/src/test/fixtures/runtime/backtestSuccess.js`
  - `frontend/src/test/fixtures/runtime/buildValidatedSampleGraph.js`
  - `frontend/src/test/fixtures/runtime/capabilityRejections.js`
  - `frontend/src/test/fixtures/runtime/editorBootstrap.js`
  - `frontend/src/test/fixtures/runtime/runSuccess.js`
- E2E support harness and reusable fixtures:
  - `frontend/tests/e2e/support/analysisReviewFixtures.js`
  - `frontend/tests/e2e/support/apiHarness.js`
  - `frontend/tests/e2e/support/workspaceBootstrapMocks.js`
  - `frontend/tests/e2e/support/workspaceGraphFixture.js`

## Out Of Scope

- Individual unit test files remain owned by their feature parents or historical test ownership.
- E2E spec bodies are deferred and must not be reorganized during this parent unless a developer explicitly reopens E2E整理.
- Deleting or replacing the large test suite remains a developer decision.

## Why This Becomes A Parent

- It owns shared test infrastructure that cuts across many feature parents.
- It defines reusable equivalence and smoke-test support surfaces.
- It needs a clear boundary so future refactors can keep production code separate from test helpers.

## Initial Child Queue

- `frontend.test_support.vitest_runtime_setup`
- `frontend.test_support.dev_test_bridge`
- `frontend.test_support.unit_fixture_catalog`
- `frontend.test_support.e2e_api_harness`
- `frontend.test_support.e2e_bootstrap_review_fixtures`

## Parent Return

- After this parent closes, return to `root.frontend`.
- Remaining parent queue after closeout: none known in frontend-local governance.

## Verification

- From repo root, `git diff --check`: passed.
- From repo root, frontend recursive state JSON parse: passed.
- From repo root, `tools/check-full-feature-tree.ps1`: passed.
- From repo root, `tools/check-matrix-governance.ps1`: passed.
