# v4.16.0 governance next optimization

> Batch: GOV-GOVERNANCE-NEXT-OPTIMIZATION-01
> Scope: recursive governance, terminal leaf control, QPCursor trial wrapper, full feature tree gate
> Movement: Tooling and governance docs only.

---

## Decision

Adopt the two-trial findings from `risk_execution_gate` and `simulated_execution_engine`.

## Changes

1. Leaf granularity output now separates:
   - `split_decision`: whether to continue splitting.
   - `governance_packaging`: how to package governance evidence.
   - `final_decision`: legacy-compatible STOP/WAVE/SPLIT/PRECISION.
2. `WAVE` is no longer a split permission by itself. It means same-parent governance packaging.
3. Oversized high-risk leaves now force `PRECISION` before movement.
4. `governance-next` is included in full feature tree coverage.
5. Untracked active files are checked for full feature tree coverage.
6. `tools/new-qpcursor-trial.ps1` can generate QPCursor drafts from the legacy recursive cursor.
7. Index reduction is recorded as a required pre-promote direction, while old indexes remain authoritative.

## Non-Changes

- The current Rust recursive cursor is not moved by this governance optimization.
- `governance-next` is not promoted.
- Legacy governance remains authoritative.
- Release transition remains inactive and AI must not propose it.

## Verification

Required gates:

- `tools/evaluate-leaf-granularity.ps1` smoke on a small terminal leaf.
- `tools/evaluate-leaf-granularity.ps1` smoke on an oversized high-risk leaf.
- `tools/new-qpcursor-trial.ps1` smoke to a temporary output path.
- `git diff --check`.
- UTF-8 check.
- full feature tree check.
- matrix governance check.

Observed smoke results:

- `risk_execution_gate`: `split_decision=STOP`, `governance_packaging=same_parent_wave`, `final_decision=WAVE`.
- `simulated_execution_engine`: `split_decision=CONTINUE`, `governance_packaging=precision_single_leaf`, `final_decision=PRECISION`.
- QPCursor generator created a temporary draft from `recursive-state.json` and the file was removed after smoke.
