# Private Baseline Risk Register

This file records owner-accepted risks for the private baseline only.
It does not authorize public release, public repository visibility, outbound
license replacement, or public-release-ready wording.

## Current owner decision

Review date: `2026-04-28`

The owner accepts the remaining frontend dependency audit risk only for
private-baseline use. The previous PostCSS moderate finding was removed by a
lockfile-only patch update to `postcss@8.5.12`; the remaining audit finding is
the Vite/esbuild chain.
This is the same release posture used by README and the first-release
readiness record: private baseline may proceed after the accepted gate passes,
but public release must not be declared ready.

This acceptance means:

- the private baseline may proceed when `cmd /c tools\run-closeout-gates.bat`
  passes
- the repository must remain private
- public release remains blocked
- public-release-ready wording remains forbidden
- the Vite/Vitest migration remains required before any future public release

## Accepted private-only risk

| Risk | Evidence | Private baseline effect | Public release effect |
|---|---|---|---|
| Frontend Vite/esbuild audit chain reports moderate findings | `cd frontend; cmd /c npm audit --audit-level=moderate` reports 5 moderate findings. `npm audit fix --dry-run --audit-level=moderate` has no non-breaking automatic fix and points to a breaking Vite/Vitest migration path. The previous `postcss <8.5.10` moderate finding is no longer present after `postcss@8.5.12` was resolved into `frontend/package-lock.json`. | Accepted by owner for private-only baseline use. | Still blocks public release until the major dependency migration is completed and the full gate is green. |

## P1 follow-up risks

Recorded on `2026-04-26` after the P1 contract and behavior closeout.
These items are not new feature scope and should be repaired later as P2
cleanup, test hygiene, or review-only polish. They do not change the current
paper-runtime beta boundary.

| Risk | Evidence | Current effect | Required follow-up |
|---|---|---|---|
| Strategy hub empty-state first screen is dense | The failed visual-review page snapshot shows the strategy hub empty state with status strip, filters, roster table header, batch action controls, and inspector side rail all visible at once. | No functional gate failure, but narrow viewport readability still needs manual review before public-facing polish claims. | Run the repaired visual review across narrow and desktop viewports, then simplify only existing layout density if needed. |
| Review-only visual and performance specs remain outside the smoke gate | The canonical E2E run reports the visual and performance review specs as skipped unless their environment switches are set. | A green closeout wrapper proves smoke behavior, not full manual visual or performance review. | Keep these specs review-only unless the owner explicitly promotes them into the canonical gate. |
| Baseline diff remains broad after artifact cleanup | `git status --short` no longer shows local runtime/build artifacts after P2 cleanup, but still shows a wide intentional product/source change set across frontend, backend, tests, and markdown. | Private baseline can still be validated by the closeout gate, but review and commit discipline remain important. | Review as one deliberate baseline batch or split into smaller commits before publishing or sharing the repository state. |

## Resolved P2 cleanup items

Recorded on `2026-04-26` and updated on `2026-04-28`:

- The workspace page `act(...)` warning was removed by keeping the
  collaboration audit refresh under the test-owned store fixture.
  `cd frontend; cmd /c npm run test -- StrategyWorkspacePage.codeMode.test.jsx`
  now passes without the previous warning.
- The opt-in responsive visual review route drift was repaired. The review
  spec now targets the current strategy hub, strategy workspace, backtest
  detail, and backtest compare routes, and its API fixture covers the current
  graph list, graph version, graph audit, and experiment-history reads.
  `VISUAL_REVIEW=1 cmd /c npx playwright test tests/e2e/visual-responsive-review.spec.js`
  now passes.
- The duplicate strategy hub `可运行策略` status wording was de-duplicated by
  keeping the metric card label and renaming the operational status card to
  `运行就绪`.
- The `postcss <8.5.10` moderate audit finding was removed through
  `npm audit fix`, which updated the frontend lockfile to `postcss@8.5.12`
  without changing application source or adding new dependency scope.
- The opt-in responsive visual review was stabilized for manual screenshot
  evidence by freezing motion and emulating reduced motion before capture.
  Backtest compare no longer captures a stale darkened transition state.
- Backtest detail `EventStreamPanel` now marks detail mode explicitly and uses
  natural flow in detail pages, avoiding the mobile chart/feed overlap caused
  by editor panel grid rows.
- Event stream JSX kicker labels now use readable source text instead of
  visible Unicode escape literals.

Local machine-readable audit output may exist under
`storage/audit/npm-audit-2026-04-24.json`.
That file is intentionally ignored because local audit output is evidence, not
versioned product truth.

## Non-decisions

The owner has not approved:

- public repository visibility
- public release tags
- public-release-ready claims
- replacing the all-rights-reserved placeholder `LICENSE`
- skipping the Vite/Vitest migration for a future public release
