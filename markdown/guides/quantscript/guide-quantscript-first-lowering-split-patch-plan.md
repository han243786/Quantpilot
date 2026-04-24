# QuantScript Phase-Close Cleanup Notes

This document no longer keeps the historical split plan.
Completed lowering/module-split work and structured `QPQSLOW` history now live
in the formal docs:

- Current executable facts: [Formal QuantScript Syntax Guide](./guide-formal-quantscript-syntax.md)
- Resolve vs lowering boundary: [QuantScript Resolve vs Lowering Boundary](../quantscript-resolve-lowering-boundary.md)
- Product release state: [Current Status And Release State](../../overview/overview-current-status-and-roadmap.md)

This file now keeps only the remaining cleanup notes for phase close.

## Encoding Status

As of 2026-04-19, this file is normal UTF-8 text.
When mojibake appears again, repair the file directly instead of preserving a
partially broken copy.

## V1 Freeze

- The retained `V1` surface and explicit de-scope boundary are now captured in [V1 Freeze / De-scope Checklist](./guide-v1-freeze-descope-checklist.md).
- Use that freeze checklist to decide what stays active, what is deferred, and what should be deleted or compressed before phase close.
- Do not silently re-open deferred items such as wider spread contracts, `MACD` shared-core expansion, generic risk/execution DSL growth, per-trade compare, or fill-timeline compare without a new contract review.

## Active Closeout Notes

### 1. Delete duplicate truth sources

- Keep one authoritative truth path for assumptions, metrics, trade ledger,
  equity curve, and compare/report narrative.
- Remove helper logic, fields, and tests that preserve older duplicate shapes once the unified module is authoritative.
- Prefer projection over re-derivation for detail, list, artifact, and compare/report surfaces.

### 2. Compress active planning docs

- Move completed work out of the active closeout notes.
- Keep the active closeout notes limited to unfinished work only.
- Compress phase-complete slices into short status bullets and point to their formal contract docs instead of repeating large completion inventories.

### 3. Keep wording aligned with the freeze boundary

- Keep README, roadmap, syntax guide, prompts, and UI wording aligned to the same retained `V1` surface.
- Keep separating current implementation fact from future direction.
- Do not advertise deferred capabilities as if they were current product scope.

### 4. Keep UTF-8 and rendered-output checks in the close checklist

- Keep UTF-8 and rendered-output checks in the merge checklist.
- Repair mojibake directly instead of preserving partially broken docs.

### 5. Phase-close verification

- Keep only the guardrails that protect the retained `V1` contracts.
- Do not add new feature tests for deferred capabilities during the freeze.
- Before phase close, verify:
  - docs no longer present deferred work as current implementation
  - active closeout notes contain only unfinished work
  - compare/report, artifact, detail, and list surfaces point at the same truth modules
  - UTF-8 checks pass on the active docs

## Deferred After V1 Freeze

### 1. Transitional fallback promotion to `resolve`

- Only promote matcher results that can be reduced to a small, stable semantic parameter set.
- Keep runtime source identity, sign, and orientation interpretation in lowering unless the contract is clearly stable.
- The already-landed resolve-first promotions are recorded in the formal docs;
  do not keep their completion history here.
- Still intentionally retained in lowering:
  - `match_ema_spread`
  - `match_macd_line_signal_pair`
  - the outer RSI formula shell

### 2. Move risk / execution / state outward

- The minimal `risk.profile("global")` and `execution.profile("paper")`
  contracts are already landed and documented in their dedicated contract docs.
- During `V1` close, do not widen either contract into a generic risk or
  execution DSL.
- Reopen this lane only if a later phase starts a new written contract review
  for profile expansion or broker-specific layering.

### 3. Spread and custom extensibility

- The first narrow spread slice is phase-complete for `V1`; details live in:
  - [Spread Minimal Contract](./guide-spread-minimal-contract.md)
  - [Spread Graph Runtime Minimal Design](./guide-spread-graph-runtime-minimal-design.md)
  - [Spread Strategy IR Minimal Contract](./guide-spread-strategy-ir-minimal-contract.md)
  - [Spread Formal QuantScript Admission Contract](./guide-spread-formal-admission-contract.md)
  - [Spread Formal QuantScript Minimal Implementation Plan](./guide-spread-formal-minimal-implementation-plan.md)
- Do not widen spread semantics during `V1` close.
- Any future spread expansion must start as a new contract review instead of
  re-entering these notes as leftover work.

### 4. Internal helper ownership cleanup

- Only reopen this lane when ownership can be simplified without duplicating
  logic or inventing empty abstractions.
- The already-landed cleanup work is recorded in the formal docs; do not repeat
  its completion history in this queue.

### 5. Research-grade backtest improvements

- The retained `V1` `P4` surface is already documented in:
  - [Backtest Execution-Assumptions Minimal Contract](./guide-backtest-execution-assumptions-minimal-contract.md)
  - [Compare Report V1 Post-Migration Checklist](./guide-compare-report-v1-post-migration-checklist.md)
  - [V1 Freeze / De-scope Checklist](./guide-v1-freeze-descope-checklist.md)
- During phase close, do not widen `P4` beyond that retained surface.
- Future work such as wider compare dimensions, trade-outcome analytics,
  timeline compare, or report-surface expansion should re-enter only as a new
  post-`V1` phase, not as unfinished work in this queue.

## Not In Scope

These are not current trunk-direction items and should not re-enter the near-term queue:

- `async/await`
- `while`
- generalized `for`
- full `match`
- recursion as a supported language feature
- macros
- OOP / objects / methods / maps / arrays as generic language features
- arbitrary user-defined indicator DSL
- arbitrary user-defined weight DSL or comparator DSL
- free persistent state across bars
- pushing advanced portfolio policy directly into QuantScript main syntax
- presenting dynamic runtime reselection as if it were today's supported universe model

## Usage Rule

- Once a task has been written into the formal docs as completed, do not keep duplicating it here.
- These notes should only keep unfinished work that still needs execution.
- Once an item is completed, move the truth into the formal docs first, then
  remove or compress it here.
