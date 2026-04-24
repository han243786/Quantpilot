# First Release Readiness

This file is the single active entry for baseline commit readiness and
first-release owner decisions.

It does not reopen feature scope.
Completed cleanup records are retained in
[Completed P2 Closeout Ledger](./implementation-non-blocking-closeout-list.md).

## Current state

Review date: `2026-04-24`

The repository is technically ready for a trustworthy private baseline snapshot
after the accepted gate passes:

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

The current all-rights-reserved placeholder license remains the intended private
state until public-release eligibility is explicitly reconsidered.

## Current owner decisions

| Decision | Owner answer | Effect |
|---|---|---|
| Functional development progress is `>= 9/10` | yes | Score threshold is satisfied. |
| Repository stability is `>= 9/10` | yes | Score threshold is satisfied. |
| Release readiness is `>= 9/10` | yes | Score threshold is satisfied. |
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
| Frontend dependency audit | `npm audit --audit-level=moderate` reports the Vite/esbuild chain. `npm audit fix --dry-run --audit-level=moderate` has no non-breaking fix and points to a breaking Vite/Vitest migration path. The owner accepts this risk for private-only baseline use. | Still blocks public release claims until the major dependency migration is completed and the full gate is green. |
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
   Current result: the active Vite/esbuild audit finding has no non-breaking
   automatic fix. The owner accepts this risk only for private-baseline use,
   so it remains a public-release blocker until a dedicated dependency
   migration closes it.

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

- [Completed Functional Closeout Ledger](/D:/rust-js-pr/QuantPilot/quantpilot/markdown/implementation/planning/implementation-functional-closeout-task-table.md)
- [Completed P2 Closeout Ledger](/D:/rust-js-pr/QuantPilot/quantpilot/markdown/implementation/planning/implementation-non-blocking-closeout-list.md)
- [Private Baseline Risk Register](/D:/rust-js-pr/QuantPilot/quantpilot/markdown/implementation/planning/implementation-private-baseline-risk-register.md)
- [Current Status And Release State](/D:/rust-js-pr/QuantPilot/quantpilot/markdown/overview/overview-current-status-and-roadmap.md)
- [Support Matrix](/D:/rust-js-pr/QuantPilot/quantpilot/markdown/implementation/governance/implementation-support-matrix.md)
- [Test Layer Expectations](/D:/rust-js-pr/QuantPilot/quantpilot/markdown/implementation/runtime/implementation-test-layer-expectations.md)
- [Active QRPC RFC Index](/D:/rust-js-pr/QuantPilot/quantpilot/markdown/protocol/README.md)
