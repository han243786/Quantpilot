# FE-0216 - Frontend Test Support Vitest Runtime Setup Closeout

Status: closed.

## Leaf Node

`frontend.test_support.vitest_runtime_setup`

## Code Changes

- No source code changes were required.
- Confirmed `frontend/vitest.config.js` owns Vitest jsdom, setup file, CSS handling, E2E exclusion, and mock reset defaults.
- Confirmed `frontend/src/test/setup.js` owns global `@testing-library/jest-dom/vitest` setup.

## Preserved Behavior

- Unit tests continue to use jsdom and the shared setup file.
- E2E specs remain excluded from Vitest and are not reorganized in this leaf.

## Public Inputs

- Vitest CLI invocation through `npm.cmd test`.
- Test files that rely on jest-dom matchers and automatic mock cleanup.

## Public Outputs

- `frontend/vitest.config.js`
- `frontend/src/test/setup.js`

## Further-Split Decision

No deeper split is useful inside `vitest_runtime_setup` now. The leaf is already two compact files with a single shared test-runtime contract.

## Verification

- From repo root, `git diff --check`: passed.
- From repo root, frontend recursive state JSON parse: passed.
- From repo root, `tools/check-full-feature-tree.ps1`: passed.
- From repo root, `tools/check-matrix-governance.ps1`: passed.
