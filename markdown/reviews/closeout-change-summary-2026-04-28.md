# Closeout Change Summary 2026-04-28

This note records the current closeout batch only.
It is not a roadmap, not a new capability list, and not a public-release
readiness claim.

## Scope

This batch only tightened existing behavior and repository hygiene:

- runtime and backtest persistence surfaces were aligned to existing saved,
  transient, and discarded artifact states
- strategy hub and strategy workspace narrow-screen layout was tightened only
  for overflow, wrapping, and readability
- capability and release wording was kept inside the existing paper-runtime beta
  boundary
- completed planning ledgers were moved under
  `markdown/archive/planning-retired/`
- targeted tests were added for already-landed runtime artifact actions,
  strategy research selectors, and base node card rendering
- local build, test, Rust target, and visual-review outputs were kept out of
  versioned product truth

## Non-Scope

This batch does not introduce:

- new trading capability
- new exchange, symbol, strategy, or QuantScript language support
- plugin marketplace support
- research-grade backtest claims
- public-release readiness
- a replacement for the current placeholder `LICENSE`

## Artifact Boundary

The following outputs are local evidence or build products only and must not be
kept as product truth:

- `frontend/dist/`
- `frontend/test-results/`
- `target/`
- `markdown/visual-review/`
- generated visual-review PNG screenshots

Visual review screenshots may be regenerated for inspection, but they should be
deleted after review unless the owner explicitly asks for an archived evidence
set.

## Current Release Wording

The current release posture remains:

- private baseline may proceed only after the accepted baseline gate passes
- public release must not be described as ready
- remaining public-release blockers include the Vite/esbuild audit chain,
  final outbound license text, and repository visibility approval

## Verification Used

The batch was checked with targeted visual review and closeout hygiene checks:

- `VISUAL_REVIEW=1 npx playwright test tests/e2e/visual-responsive-review.spec.js --project=msedge`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-user-facing-text.ps1`
- `git diff --check`

