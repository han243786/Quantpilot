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
- public repository visibility remains blocked until a separate public-release
  approval replaces the placeholder private/license posture

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
| Repository remains private before public release | yes | Public release and outbound license replacement remain blocked. |

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
4. Revisit outbound license text only when public release eligibility is being
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
- [Current Status And Release State](/D:/rust-js-pr/QuantPilot/quantpilot/markdown/overview/overview-current-status-and-roadmap.md)
- [Support Matrix](/D:/rust-js-pr/QuantPilot/quantpilot/markdown/implementation/governance/implementation-support-matrix.md)
- [Test Layer Expectations](/D:/rust-js-pr/QuantPilot/quantpilot/markdown/implementation/runtime/implementation-test-layer-expectations.md)
- [Active QRPC RFC Index](/D:/rust-js-pr/QuantPilot/quantpilot/markdown/protocol/README.md)
