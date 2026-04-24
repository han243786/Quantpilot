# Private Baseline Risk Register

This file records owner-accepted risks for the private baseline only.
It does not authorize public release, public repository visibility, outbound
license replacement, or public-release-ready wording.

## Current owner decision

Review date: `2026-04-24`

The owner accepts the current frontend dependency audit risk only for
private-baseline use.

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
| Frontend Vite/esbuild audit chain reports moderate findings | `cd frontend; cmd /c npm audit --audit-level=moderate` reports 5 moderate findings. `npm audit fix --dry-run --audit-level=moderate` has no non-breaking automatic fix and points to a breaking Vite/Vitest migration path. | Accepted by owner for private-only baseline use. | Still blocks public release until the major dependency migration is completed and the full gate is green. |

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
