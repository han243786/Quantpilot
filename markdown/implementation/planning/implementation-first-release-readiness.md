# First Release Readiness

This file is the single active entry for baseline commit readiness and
first-release owner decisions.

It does not reopen feature scope.
Completed cleanup records are archived under
[Retired Planning Docs](../../archive/planning-retired/README.md).

## Current state

Last accepted baseline review date: `2026-04-24`

Latest re-check date: `2026-04-28`

The latest `2026-04-28` dependency and visual/layout cleanup keeps the same
paper-runtime beta boundary. The previous `2026-04-26` full closeout wrapper
passed after P0 and P1 closeout.
The established product surface remains the same paper-runtime beta.
The current worktree still needs baseline handoff review before a tidy
snapshot:

- the worktree contains a wide intentional source/documentation change set that
  should be reviewed as one baseline batch or split before commit
- generated review screenshots may be present as manual evidence and should not
  be confused with runtime product truth
- remaining dependency audit risk is accepted only for private-baseline use and
  still blocks public-release claims

The repository is technically ready for a trustworthy private baseline snapshot
only after the accepted gate passes:

- the closeout gate contract is explicit
- active docs point to the current beta boundary
- active markdown and frontend files are expected to stay UTF-8 without BOM
- mojibake is guarded by text checks
- placeholder [LICENSE](/D:/rust-js-pr/QuantPilot/quantpilot/LICENSE) makes the
  current legal state explicit

The current release decisions are explicit owner decisions, not hidden feature
work:

- `LICENSE` is still a placeholder
- the owner allows a private baseline commit in principle
- the owner accepts the current closeout gate set as the intended private
  baseline gate set
- the owner accepts the current frontend dependency audit risk for private-only
  baseline use
- public repository visibility remains blocked until a separate public-release
  approval replaces the placeholder private/license posture
- public-release readiness must not be claimed while dependency audit,
  dependency migration strategy, and outbound license decisions remain open

## Thread closeout note

Thread closed on `2026-04-24` after private baseline commit
`ad0b903 Close private baseline readiness`.

The next thread should start from the clean private-baseline worktree and keep
the same release boundary:

- no new feature scope is implied by this closeout
- the repository remains private
- the accepted private-baseline gate remains `cmd /c tools\run-closeout-gates.bat`
- the current frontend audit risk is accepted only for private-baseline use
- public release remains blocked until dependency migration, final license text,
  and public visibility approval are handled in a later owner-approved thread

## Repository visibility decision

Owner decision:

The owner reports that all three release score dimensions are now at least
`9/10`:

- functional development progress is `>= 9/10`
- repository stability is `>= 9/10`
- release readiness is `>= 9/10`

The owner still chooses to keep the repository private before any public
release.
Passing the `9/10` threshold does not automatically authorize public repository
visibility, public release tags, or outbound license replacement.
The score is a private-baseline readiness signal only; it is not a
public-release-ready claim.

The current all-rights-reserved placeholder license remains the intended private
state until public-release eligibility is explicitly reconsidered.

## Current owner decisions

| Decision | Owner answer | Effect |
|---|---|---|
| Functional development progress is `>= 9/10` | yes | Score threshold is satisfied. |
| Repository stability is `>= 9/10` | yes | Score threshold is satisfied. |
| Release readiness is `>= 9/10` | yes | Score threshold is satisfied for private baseline only. |
| Private baseline commit may be created | yes | Allowed in principle. |
| Current gate set is accepted as baseline gate set | yes | `tools\run-closeout-gates.bat` is the private-baseline gate. |
| Current frontend audit risk is accepted for private-only baseline use | yes | The Vite/esbuild audit finding does not block private baseline, but still blocks public release claims. |
| Repository remains private before public release | yes | Public release and outbound license replacement remain blocked. |

## Public release blockers

The current state is private-baseline oriented.
Do not describe the repository as public-release ready until all blockers in
this section are closed by implementation or owner decision.

| Blocker | Current state | Required closeout |
|---|---|---|
| Frontend dependency audit | `npm audit --audit-level=moderate` reports only the Vite/esbuild chain after the `postcss <8.5.10` moderate finding was removed through `postcss@8.5.12` in `frontend/package-lock.json`. `npm audit fix --dry-run --audit-level=moderate` has no non-breaking fix for the remaining chain and points to a breaking Vite/Vitest migration path. The owner accepts this remaining risk for private-only baseline use. | Still blocks public release claims until the major dependency migration is completed and the full gate is green. |
| Dependency upgrade strategy | `npm outdated` shows the audit fix path is not a patch-level update. The owner chooses not to force that migration into the private baseline. | Treat Vite/Vitest major migration as a future dedicated P2 batch before any public release. |
| Outbound license | `LICENSE` remains all-rights-reserved placeholder text. | Replace it only after the owner approves public-release eligibility and final license text. |
| Repository visibility | Current posture is private baseline only. | Keep private until a separate public-release approval exists. |

Local audit evidence for the current run is stored under
`storage/audit/npm-audit-2026-04-24.json`.
That file is intentionally ignored and is not product truth.
The versioned private-only risk acceptance record is
[Private Baseline Risk Register](./implementation-private-baseline-risk-register.md).

## Baseline go or no-go

Baseline commit can proceed only when all of the following are true:

- `cmd /c tools\run-closeout-gates.bat` passes
- active docs still match the current beta boundary
- generated and runtime artifacts remain outside versioned product truth
- `LICENSE` reflects the intended legal state for the baseline
- repository visibility remains private unless a separate public-release
  approval is made
- the owner accepts a specific gate set as the baseline gate set

If any item above is false, do not create the baseline commit yet.

## Owner actions

For the current closeout phase, the owner-only actions are:

1. Keep repository visibility private until functional development progress,
   repository stability, and release readiness are all at least `9/10`.
2. Keep `tools\run-closeout-gates.bat` as the accepted private-baseline gate
   unless a later owner decision replaces it.
3. Create private baseline commits only when the accepted baseline checks are
   green.
4. Keep the current audit risk acceptance private-only; do not use it to claim
   public-release readiness.
5. Revisit outbound license text only when public release eligibility is being
   reconsidered.

Do not widen this into a roadmap.

## Pre-commit verification

Run the canonical wrapper from the repository root:

```powershell
cmd /c tools\run-closeout-gates.bat
```

The wrapper is the baseline confidence check.
It covers UTF-8, user-facing text, capability governance, Rust workspace tests,
frontend unit tests, frontend production build, and frontend E2E under the
isolated API-mock contract.

Use [Test Layer Expectations](/D:/rust-js-pr/QuantPilot/quantpilot/markdown/implementation/runtime/implementation-test-layer-expectations.md)
when reporting what the gate proves.
Do not describe a green gate as proof of unsupported product capability.

## Closeout execution flow

This section is the active working process for the remaining cleanup phase.
It does not add new feature scope.

Use every batch to remove risk, close drift, and verify the retained beta
surface. Do not widen the product surface while executing these steps.

### Scope guard

Before any implementation batch, confirm that the proposed change satisfies all
of the following:

- it fixes an existing defect, drift, gate failure, release-hygiene issue, or
  wording mismatch
- it stays inside the current paper-runtime beta boundary
- it does not add new public capability claims
- it does not expand the support matrix
- it does not expand the retained formal QuantScript trunk
- it does not introduce a second source of truth beside backend capability and
  runtime facts

If any item is false, defer the work outside the current closeout phase.

### Current optimization checklist

This checklist is the active `2026-04-26` closeout queue.
It is for defect removal, drift repair, release hygiene, and final polish only.
It must not be used to add product breadth, widen the support matrix, expand
the retained QuantScript trunk, or introduce a second source of truth.

#### P0: blocking gate recovery

Status: completed on `2026-04-26`.

| Item | Goal | Scope | Acceptance | Status |
|---|---|---|---|
| Repair run history E2E regression | A saved simulation appears in the run history card after refresh. | Runtime artifact save wiring, run-list API mapping, E2E mock contract, run history filters. | `cd frontend; cmd /c npm run test:e2e` passes the run simulation smoke path. | Done |
| Repair backtest history E2E regression | A saved backtest appears in the backtest history card after refresh. | Runtime artifact save wiring, backtest-list API mapping, E2E mock contract, backtest history filters. | `cd frontend; cmd /c npm run test:e2e` passes the run backtest smoke path. | Done |
| Remove browser alert usage | Row-level action failures render through the existing inline failure surface. | `StrategyHubRosterRowActions.jsx`, action-failure copy, related tests. | `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-user-facing-text.ps1` passes. | Done |
| Verify Chinese text rendering | Frontend and docs text remains readable UTF-8, not mojibake. | Active markdown, user-facing frontend strings, rendered UI surfaces touched by the batch. | UTF-8 and user-facing text checks pass, and changed UI copy is reviewed in rendered form. | Done |
| Re-run the accepted closeout gate | Restore baseline confidence after the blocking fixes. | Full repository gate wrapper. | `cmd /c tools\run-closeout-gates.bat` passes without hidden manual setup. | Done |

#### P1: contract and behavior closeout

Status: completed on `2026-04-26`.

| Item | Goal | Scope | Acceptance | Status |
|---|---|---|---|---|
| Align history filtering | Current graph, compile, dataset, parameter, status, and time filters do not hide freshly saved records incorrectly. | Run history, backtest history, strategy center counters, list projections. | Targeted tests cover the refreshed record path and the E2E smoke remains stable. | Done |
| Re-check save and refresh ordering | History refresh occurs after the record is actually eligible for listing. | Save/promote actions, discard paths, transient-vs-saved state, frontend refresh triggers. | Slow local runs do not produce false empty history cards. | Done |
| Keep detail reloads on persisted facts | List, event summary, and detail pages agree on IDs, event counts, and artifacts. | Runtime persistence, backtest artifacts, detail DTOs, frontend detail mapping. | Reloaded run/backtest detail pages match the corresponding list records. | Done |
| Audit capability-driven UI exposure | Visible actions remain enabled, disabled, hidden, or explained from backend capability truth. | Module sidebar, toolbar actions, workspace cards, fallback states, support matrix. | Capability-governance check and related UI tests pass. | Done |
| Tighten compile-chain wording | `strategy_ir` remains preflight only and runtime compile remains runnable truth. | Compile summary, diagnostics labels, action failures, README and active docs. | No UI or doc copy claims a second runnable source of truth. | Done |
| Unify runtime and backtest explanation | Live event history, persisted detail, diagnostics, and compare views render the same runtime facts. | Diagnostics projection, event cards, run detail, backtest detail, compare pages. | The same risk, execution, fill, and data-quality facts display consistently across surfaces. | Done |
| Harden retained QuantScript surface | Supported examples compile; unsupported constructs fail early with stable diagnostics. | Formal syntax samples, fixtures, lowering diagnostics, retained-surface docs. | Tests and docs stay inside the retained V1 trunk. | Done |
| Polish existing frontend layout | Fix overlap, overflow, unreadable controls, and dense explanation copy without adding new screens. | Strategy hub, workspace, backtest pages, shared CSS, compact cards and tables. | Key desktop and narrow viewports remain usable, with no blocking text overlap. | Done for gate-covered surfaces; manual visual review remains P2/review-only. |

P1 acceptance evidence from `2026-04-26`:

- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-user-facing-text.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-capability-governance.ps1`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cd frontend; cmd /c npm run test`
- `cd frontend; cmd /c npx playwright test tests/e2e/run-simulation.spec.js tests/e2e/run-backtest.spec.js tests/e2e/editor-capabilities-smoke.spec.js`
- `cmd /c tools\run-closeout-gates.bat`

Latest P1 implementation notes from `2026-04-26`:

- History filtering now keeps the currently selected persisted run or backtest
  visible while stale graph, compile, dataset, parameter, status, or time
  filters are still present after a save.
- Save flows are covered as `save -> refresh history -> load persisted detail`
  for both simulation runs and backtests.
- Reloaded run and backtest detail state is rebuilt from persisted detail
  responses, not from transient in-memory event state.
- Capability-driven UI exposure was re-checked through the capability
  governance snapshot, support-matrix tests, module-sidebar tests, toolbar
  fallback tests, and forbidden-claim text search.
- Compile-chain wording was re-checked against the active contract:
  `strategy_ir` remains semantic preflight only, and runtime compile remains
  runnable truth.
- Runtime and backtest explanation surfaces were re-checked through diagnostics,
  event history, backtest artifacts, detail, and compare tests.
- QuantScript retained-surface behavior was re-checked through supported formal
  examples and unsupported-construct diagnostic tests.

P1 residual test notes resolved during P2 cleanup:

- `StrategyWorkspacePage.codeMode.test.jsx` no longer emits the previous React
  `act(...)` warning from `StrategyWorkspaceCollaborationCard`; the test-owned
  store fixture now owns the collaboration audit refresh.
- The opt-in visual review is no longer blocked by route drift. The review
  script targets the current strategy hub, strategy workspace, backtest detail,
  and backtest compare routes, and its fixture covers the current supporting
  API reads.
- The `2026-04-28` visual review pass freezes motion before screenshot
  capture, so stale animation overlays no longer look like product state.
- Backtest detail uses explicit event stream detail mode, keeping the narrow
  viewport chart and feed in natural flow instead of overlapping.
- Event stream JSX kicker labels no longer expose Unicode escape literals in
  rendered review screenshots.

#### P1 item 8 visual/layout issues only

No implementation work was taken for this item in the P1 batch.
Known issues to carry into P2/review-only work:

- The opt-in responsive screenshot test can now capture the current screenshot
  set, but the screenshots remain manual review evidence rather than a
  pixel-diff quality gate.
- The strategy hub first-screen snapshot shows dense status and roster controls
  in the empty-state path; this still needs manual narrow-viewport review.
- The strategy hub duplicated `可运行策略` label was removed during P2
  no-decision cleanup; the metric keeps that label and the operational card is
  now `运行就绪`.
- Full manual overlap and overflow review remains outside the canonical smoke
  gate and should stay review-only until explicitly enabled.

#### P2: repository and release hygiene

Status: completed for artifact hygiene and documentation alignment on
`2026-04-26`; commit slicing remains a baseline handoff concern because the
working tree still contains a broad intentional product change set.

| Item | Goal | Scope | Acceptance | Status |
|---|---|---|---|---|
| Compress the current worktree | Make each remaining product change reviewable and explainable. | The current broad modified-file set. | Status no longer mixes unrelated product changes, generated files, and local artifacts. | Done for artifact separation; product diff remains broad and must be reviewed or committed as a deliberate baseline batch. |
| Clean local artifacts | Keep build, runtime, test, and audit residue out of product truth. | `target/`, `target-test-*`, `frontend/dist/`, `frontend/test-results/`, `storage/*`, local logs. | `git status --short` shows only intended product or documentation changes. | Done for visible local artifacts: `storage/*`, `frontend/dist/`, `frontend/test-results/`, and `codex-vite-dev.log` were removed. |
| Re-check ignore boundaries | Ensure generated/runtime artifacts stay ignored. | `.gitignore`, README artifact notes, cleanup script docs. | Ignored output matches the artifact boundary in this file. | Done; ignore rules and cleanup script docs now cover experiments, graph versions, audit JSON, and local Vite logs. |
| Record dependency audit state | Keep private-baseline risk acceptance separate from public-release readiness. | `npm audit`, private risk register, README release wording. | Private baseline wording is honest and public-release-ready wording is absent. | Done; dependency audit remains private-only accepted risk and public-release blocker. |
| Keep markdown indexes thin | Keep one current closeout entry and archive completed ledgers. | Docs root, overview index, planning README, archive planning README. | Readers can find the active queue from this file without following completed ledgers. | Done; active planning routes through this file and completed ledgers stay archived. |
| Preserve legal honesty | Keep the placeholder license posture explicit. | `LICENSE`, README, release docs. | Public release remains blocked until an owner-approved license and visibility decision exists. | Done; placeholder license posture remains explicit. |

P2 execution evidence from `2026-04-26`, updated on `2026-04-28`:

- `.gitignore` now excludes `storage/experiments/`,
  `storage/graphs/versions/`, and `codex-vite-dev.log`.
- `tools\cleanup-artifacts.ps1` supports explicit
  `-IncludeRuntimeArtifacts` cleanup for runtime artifacts, local graph
  snapshots, graph versions, and audit JSON.
- P2 no-decision residual cleanup removed the workspace test `act(...)`
  warning, repaired the opt-in visual review route/API fixture drift, and
  de-duplicated the strategy hub `可运行策略` status wording.
- `npm audit fix` removed the `postcss <8.5.10` moderate finding by resolving
  `postcss@8.5.12` in the frontend lockfile. The remaining audit result is the
  Vite/esbuild chain only.
- The opt-in responsive visual review was stabilized with reduced-motion
  capture, and backtest detail now uses detail-mode event stream layout to
  avoid mobile overlap.
- README repository hygiene notes now match the active artifact boundary.
- Local generated output under `storage/*`, `frontend/dist/`,
  `frontend/test-results/`, and `codex-vite-dev.log` was removed from the
  working directory.
- Full closeout verification was rerun after this cleanup:
  `cmd /c tools\run-closeout-gates.bat` passed on `2026-04-26`.

P2 residual handoff:

- The working tree remains intentionally broad across frontend, backend, tests,
  and markdown. Because these are product/source changes rather than generated
  artifacts, they were not reverted or deleted during P2 cleanup.
- Before a baseline commit, review the broad diff as one deliberate baseline
  batch or split it into smaller commits. Do not mix regenerated local
  artifacts back into that review.

#### Execution order

1. Fix the `alert` and any touched text rendering issues.
2. Repair run and backtest history E2E regressions.
3. Re-run frontend E2E, then the full closeout wrapper.
4. Compress and clean the worktree.
5. Update only active docs and indexes needed to reflect the closeout state.
6. Leave new feature ideas, plugin expansion, public release, and dependency
   major migrations outside this closeout queue unless the owner explicitly
   opens a separate batch.

### P0: blocking gate and truth failures

P0 work must be cleared before baseline confidence can be claimed.

1. Run the current baseline gate.
   Command: `cmd /c tools\run-closeout-gates.bat`.
   Acceptance: the wrapper passes without hidden manual setup.

2. Fix Rust formatting drift.
   Command: `cargo fmt --all -- --check`.
   Acceptance: the command passes. Use `cargo fmt --all` only to apply
   formatting, then re-check.

3. Fix strict Rust lint failures.
   Command: `cargo clippy --workspace --all-targets -- -D warnings`.
   Acceptance: the command passes without suppressing meaningful warnings.
   Allow a lint only when the exception is local, documented by the code shape,
   and safer than a semantic refactor.

4. Repair any full-gate failure immediately.
   Scope: UTF-8 gate, user-facing text gate, capability-governance snapshot,
   Rust tests, frontend tests, frontend build, or E2E smoke.
   Acceptance: the failed layer passes in isolation, then the wrapper passes.

5. Remove false product truth.
   Scope: README, active docs, frontend copy, fixtures, support matrix, and
   capability-driven UI exposure.
   Acceptance: no text or visible action claims live trading, research-grade
   backtest, true arbitrage platform support, third-party plugin marketplace
   support, or arbitrary host-code QuantScript support.

6. Prevent E2E contract drift.
   Command: `cd frontend; cmd /c npm run test:e2e`.
   Acceptance: smoke paths pass under the isolated API-mock contract, and
   unmocked API requests remain failures.

### P1: core contract closeout

P1 work hardens the existing product surface without adding breadth.

1. Align capability-driven frontend exposure.
   Scope: toolbar actions, module sidebar, workspace cards, fallback states,
   and cached capability behavior.
   Acceptance: visible actions are enabled, disabled, hidden, or explained
   only from the current capability contract and retained beta boundary.

2. Tighten compile-chain interpretation.
   Scope: compile routes, compile summary, diagnostics labels, stored
   artifacts, frontend action failures, and docs.
   Acceptance: `strategy_ir` remains semantic preflight only,
   `quantscript.formal_source` owns runtime lowering when present, and runtime
   compile remains the runnable source of truth.

3. Unify runtime and backtest explanations.
   Scope: runtime diagnostics, event projection, run detail, backtest detail,
   compare views, and event stream cards.
   Acceptance: the same runtime fact renders consistently across live event
   history and persisted detail views.

4. Sweep persistence and replay consistency.
   Scope: graph versions, run history, backtest history, replay checkpoints,
   experiment records, and storage-backed detail loading.
   Acceptance: reload paths do not depend on transient in-memory state or
   special reconstruction.

5. Harden the retained QuantScript surface.
   Scope: formal syntax, lowering diagnostics, authoring samples, fixtures,
   and docs.
   Acceptance: supported samples compile consistently, unsupported constructs
   fail early with stable diagnostics, and docs do not imply a broader research
   language.

6. Add or refresh targeted regression coverage only for existing promises.
   Acceptance: tests protect the fixed contract and do not become a new feature
   lane.

### P2: release hygiene and baseline cleanup

P2 work keeps the repository clean and the release story honest.

1. Clean ignored local artifacts before a baseline snapshot.
   Scope: `target/`, `target-test-*`, `frontend/dist/`,
   `frontend/test-results/`, local Playwright output, and `storage/test-*`.
   Acceptance: `git status --short` remains clean and ignored residue is not
   mistaken for product truth.

2. Keep active docs compressed.
   Scope: README, this readiness file, support matrix, test expectations, and
   planning README.
   Acceptance: a reader can find the current beta boundary, gate set, and
   owner-only release blockers from the active docs without following archived
   roadmaps.

3. Audit dependency risk without forced scope expansion.
   Scope: `npm audit`, frontend dependency updates, and lockfile changes.
   Acceptance: moderate or higher findings are recorded and fixed only through
   low-risk upgrades unless the owner explicitly accepts a breaking dependency
   migration.
   Current result: the `postcss <8.5.10` moderate finding was fixed through a
   lockfile patch to `postcss@8.5.12`. The remaining Vite/esbuild audit
   finding has no non-breaking automatic fix. The owner accepts this remaining
   risk only for private-baseline use, so it remains a public-release blocker
   until a dedicated dependency migration closes it.

4. Preserve release legal honesty.
   Scope: `LICENSE`, README release wording, and public visibility notes.
   Acceptance: private baseline status remains explicit, and public release
   stays blocked until a separate owner decision replaces the placeholder
   license posture.

5. Keep manual review tests separate from smoke gates.
   Scope: performance and visual review Playwright specs.
   Acceptance: review-only tests remain opt-in through their environment
   switches and are not confused with the canonical E2E smoke contract.

6. Re-run the full closeout wrapper after shared cleanup.
   Command: `cmd /c tools\run-closeout-gates.bat`.
   Acceptance: the baseline gate passes after cleanup.

### Batch reporting rule

Every closeout batch should report only the following facts:

- what existing risk or drift was removed
- which targeted checks passed
- whether the full closeout wrapper passed
- which owner-only decision, if any, still blocks public release

Do not report a batch as new capability delivery.

## Explicit gate commands

The wrapper should stay aligned with these commands:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-user-facing-text.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-capability-governance.ps1
cargo test --workspace
cd frontend; cmd /c npm run test
cd frontend; cmd /c npm run build
cd frontend; cmd /c npm run test:e2e
```

Gate wording must not drift between local docs, helper scripts, and CI.

Current constraints:

- frontend `npm` gates must use `cmd /c npm run ...`
- E2E must remain runnable without manually pre-starting the backend
- unmocked E2E API requests are failures, not acceptable proxy fallbacks

## Artifact boundary

The baseline commit should capture product truth, not local runtime residue.

Keep these out of the baseline snapshot:

- Rust build output under `target/`
- frontend build output and local Playwright output
- local runtime artifacts under `storage/runs/` and `storage/backtests/`
- local audit artifacts under `storage/audit/*.json`
- test runtime artifacts under `storage/test-*`
- local graph snapshots under `storage/graphs/*.json` and
  `storage/graphs/*.qs`
- local environment overrides and temporary helper files

## Active companion docs

- [Private Baseline Risk Register](/D:/rust-js-pr/QuantPilot/quantpilot/markdown/implementation/planning/implementation-private-baseline-risk-register.md)
- [Current Status And Release State](/D:/rust-js-pr/QuantPilot/quantpilot/markdown/overview/overview-current-status-and-roadmap.md)
- [Support Matrix](/D:/rust-js-pr/QuantPilot/quantpilot/markdown/implementation/governance/implementation-support-matrix.md)
- [Test Layer Expectations](/D:/rust-js-pr/QuantPilot/quantpilot/markdown/implementation/runtime/implementation-test-layer-expectations.md)
- [Active QRPC RFC Index](/D:/rust-js-pr/QuantPilot/quantpilot/markdown/protocol/README.md)
- [Archived Functional Closeout Ledger](/D:/rust-js-pr/QuantPilot/quantpilot/markdown/archive/planning-retired/implementation-functional-closeout-task-table.md)
- [Archived P2 Closeout Ledger](/D:/rust-js-pr/QuantPilot/quantpilot/markdown/archive/planning-retired/implementation-non-blocking-closeout-list.md)
