# Runtime Artifact Retention And Save Gate

This document records the storage-safety optimization for validation, test run,
simulation, and backtest artifacts.

## Problem

During development, every availability check or validation test can create a new
folder under `storage`, especially below runtime and backtest paths. Repeated
checks produce dense `storage/test-*` or backtest artifact directories, and the
accumulated event logs, ledgers, curves, and manifests can quickly consume many
gigabytes.

This is a storage-safety risk. Validation should prove whether a strategy can
run; it should not automatically become a durable artifact.

## Target Rule

Validation and test execution are transient by default.

Artifacts enter `storage` only after an explicit save action.

This applies to:

- compile validation checks
- strategy availability tests
- simulation smoke runs
- backtest preview runs
- parameter validation runs
- frontend and backend development tests

The user-facing rule should be simple:

1. Run validation or test.
2. Inspect the result.
3. Click save if the result should become part of persistent history.
4. Click discard if the transient result should be removed from the current session.
5. Only the save path writes the artifact folder into `storage`.

## Storage Tiers

### Transient Tier

Transient artifacts are used for immediate UI feedback only.

Required behavior:

- do not write under `storage/graphs`, `storage/runs`, `storage/backtests`, or
  `storage/experiments`
- prefer memory for small records
- spill large backtest previews to `.quantpilot-tmp/runtime-artifacts/backtests`
  rather than the persistent history folders
- attach a TTL and quota to the transient root
- clean transient files on server start, after successful promotion, after
  explicit discard, and after TTL expiry
- exclude transient artifacts from strategy center counts and history lists
- allow manual discard without touching persistent history

### Persistent Tier

Persistent artifacts are the durable project record.

Required behavior:

- write only after explicit save
- use existing `storage` subtrees for saved graphs, runs, backtests,
  experiments, and audit records
- preserve replay compatibility for saved artifacts
- list only saved artifacts in strategy center history panels
- record audit metadata when a transient artifact is promoted

### Evidence Report Tier

Evidence report records are persisted metadata. They link to source run/backtest
evidence, sequence ranges, governance identity, generation policy, and exported
artifact metadata. They do not copy raw logs.

Required behavior:

- report JSON records under the report store are protected persistent metadata
- report export payloads are derived from report metadata and can be regenerated
- compact evidence remains a projection of governed source evidence, not an
  independently retained cache
- transient report-generation outputs may be deleted after the cleanup TTL
- cleanup may remove names prefixed by `report-generation-tmp-` or
  `report-generation-partial-`
- cleanup must not delete saved report JSON records, saved runs, saved
  backtests, or saved experiments

## Backend Plan

1. Add an artifact persistence scope.

   Suggested values:

   - `transient`
   - `saved`

   The scope should be carried by run, backtest, and experiment creation flows.

2. Make test and validation endpoints default to `transient`.

   Backend functions that currently call durable writes during tests should not
   call these by default:

   - `persist_run_record`
   - `persist_backtest_record`
   - `persist_backtest_artifacts`
   - `persist_experiment_record`

3. Add a promotion path.

   A transient result should be promoted by a dedicated save action rather than
   by the original test action.

   Endpoint shapes:

   - `POST /api/runtime/runs/:run_id/save`
   - `POST /api/runtime/backtests/:backtest_id/save`
   - `POST /api/runtime/experiments/:experiment_id/save`

   Promotion copies or rebuilds the existing transient record into the normal
   persistent storage layout.

4. Add an explicit discard path.

   Unsaved artifacts can be discarded by dedicated delete actions:

   - `DELETE /api/runtime/runs/:run_id`
   - `DELETE /api/runtime/backtests/:backtest_id`
   - `DELETE /api/runtime/experiments/:experiment_id`

   These endpoints must reject already persisted artifacts, because saved
   history is not part of the transient cleanup path.

5. Keep persistent writes atomic.

   Backtest promotion should write to a temporary sibling directory first, then
   rename into `storage/backtests/:backtest_id`. This prevents half-written
   artifact folders from entering history.

6. Add transient spillover for large previews.

   Backtest previews stay in memory while small. Once a preview record exceeds
   the configured threshold, it is written to the transient root as a complete
   artifact bundle plus minimal transient metadata. The normal detail, replay,
   save, and discard paths must load from the same abstraction, so the UI does
   not need a second runtime model.

7. Add cleanup.

   The server should clean transient or promotion-work artifacts:

   - on startup
   - after successful promotion
   - after explicit discard
   - after TTL expiry

   Evidence report cleanup is narrower: it removes only transient
   report-generation outputs and keeps persisted report records intact.

8. Add quotas.

   Runtime temp storage should have guardrails:

   - max transient artifact count
   - max transient bytes
   - max age
   - warning when cleanup cannot reclaim space

## Frontend Plan

1. Rename test actions around intent, not persistence.

   Buttons that validate or test should not imply saving. Suggested labels:

   - `验证`
   - `试运行`
   - `预览回测`

2. Add explicit save and discard actions after a successful transient result.

   Suggested labels:

   - `保存本次结果`
   - `保存为回测记录`
   - `保存到历史`
   - `丢弃临时结果`

3. Show the artifact state.

   The result panel should distinguish:

   - `临时结果`
   - `已保存`
   - `已丢弃`

4. Keep strategy center clean.

   Strategy center counters, history panels, and compare queues should ignore
   transient artifacts until they are saved.

5. Protect compare flows.

   A compare queue should only accept saved backtests, or it must clearly prompt
   the user to save a transient backtest before adding it.

## Test Harness Plan

Development and CI tests must not write into shared `storage/test-*` paths.

Required changes:

- replace hard-coded `storage/test-graphs`, `storage/test-runs`, and
  `storage/test-backtests` with per-test temp directories
- ensure temp directories are removed after each test
- add a regression test that fails if backend tests create new entries under the
  real `storage` root
- keep fixtures small and deterministic

## Migration Plan

1. Keep existing saved artifacts readable.
2. Stop listing `storage/test-*` folders in product views.
3. Provide a cleanup command for obsolete development artifacts.
4. Do not delete user-saved artifacts automatically.
5. Document cleanup output before removal so users can review what will be
   deleted.

## Acceptance Criteria

- Running validation repeatedly does not create new folders under `storage`.
- Running preview backtests repeatedly does not grow persistent history.
- Large unsaved preview backtests spill only into the transient root.
- Strategy center counts remain unchanged after unsaved tests.
- A saved transient result appears in persistent history exactly once.
- Unsaved transient results can be explicitly discarded from the current session.
- Discarding a persisted artifact is rejected and does not delete saved history.
- Compare pages can only consume saved backtests or require save before compare.
- Backend tests use isolated temp directories and clean them after execution.
- UTF-8 and user-facing text checks remain green after UI wording changes.

## Implemented Scope

Current implementation keeps the existing runtime model and narrows only the
persistence boundary.

Implemented:

- `POST /api/runtime/test-run` now creates an in-memory run record only.
- `POST /api/runtime/backtest` now creates a transient backtest preview.
- Small backtest previews remain in memory.
- Large backtest previews spill into
  `.quantpilot-tmp/runtime-artifacts/backtests` after crossing the configured
  spill threshold.
- The transient spill root has a 24 hour TTL cleanup path, a maximum of 32
  transient directories, a 512 MB total quota, and a 256 MB single-artifact
  quota.
- Transient spill cleanup runs on server startup, before spill writes, after
  explicit discard, and after successful save promotion.
- `POST /api/runtime/experiments/backtest-sweep` keeps sweep records in memory
  and lets large variant backtests use the same transient spill path.
- `POST /api/runtime/runs/:run_id/save` promotes the current run into
  `storage/runs`.
- `POST /api/runtime/backtests/:backtest_id/save` promotes the current backtest
  into `storage/backtests/:backtest_id`, whether it is still in memory or has
  spilled to the transient root.
- `POST /api/runtime/experiments/:experiment_id/save` promotes the sweep record
  and its variant backtests into the normal persistent stores.
- `DELETE /api/runtime/runs/:run_id` explicitly discards an unsaved in-memory
  simulation run.
- `DELETE /api/runtime/backtests/:backtest_id` explicitly discards an unsaved
  in-memory or spilled backtest preview.
- `DELETE /api/runtime/experiments/:experiment_id` explicitly discards an
  unsaved in-memory sweep record and its unsaved variant backtests.
- Discard endpoints reject already persisted artifacts with conflict responses,
  so saved history cannot be removed by the transient cleanup path.
- Backtest promotion now writes into a sibling `.saving-*` directory, validates
  the completed artifact bundle, then renames it into
  `storage/backtests/:backtest_id`.
- Re-saving an existing backtest first moves the old bundle to a sibling
  `.replacing-*` backup and restores it if the final rename fails.
- Runtime history listing skips `.saving-*` and `.replacing-*` work directories,
  so half-written bundles never enter product history.
- Backtest promotion work directories now have a 24 hour TTL cleanup path.
  Cleanup runs on server startup and before each backtest artifact promotion.
- Promotion work directories are quota-checked before save. The current guard
  rejects more than 32 promotion work directories, more than 512 MB of total
  promotion work data, or a single promotion bundle above 256 MB.
- Runtime history and strategy center counts continue to read saved artifacts
  only.
- The event panel exposes `保存本次结果` and `丢弃临时结果` only when the current
  simulation or backtest is completed but still transient.
- Backend API tests no longer use hard-coded `storage/test-*` directories.

Not implemented in this slice:

- none

## Non-Goals

- Do not introduce a second runtime model.
- Do not change the meaning of a successful validation.
- Do not remove existing replay or history compatibility for saved artifacts.
- Do not automatically delete saved user artifacts.
