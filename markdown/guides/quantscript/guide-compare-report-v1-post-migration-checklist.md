# Compare Report V1 Post-Migration Checklist

This document defines the post-`V1` migration plan for collapsing the current
dual compare/report response surface into one authoritative external report
truth.

The current `V1` contract still keeps both:

- `report_narrative`
- `compare_report`

That dual shape is still intentional during `V1` phase close because:

- it preserves the already-landed response contract
- it avoids a late-phase API break while freeze work is still active
- it lets `compare_report` mature as the richer modular report view before it
  becomes the only outward-facing report truth

This checklist exists to stop an ad hoc or premature deletion.

## Current Truth Boundary

At the end of `V1`, treat the compare/report truth layers as follows:

- internal compare truth comes from the shared compare blocks plus the shared
  compare/report bundle builder
- `compare_report` is the stronger long-term external report shape
- `report_narrative` is still a stable compatibility projection during `V1`

Do not delete `report_narrative` during `V1` phase close.

## Migration Goal

After `V1` is phase-closed, move to this target state:

- `compare_report` becomes the only authoritative external report module for
  compare/report consumers
- `report_narrative` becomes deprecated first, then removable
- tests and docs stop treating both fields as equal top-level truths

## Preconditions Before Any Removal

Only start the removal phase once all of the following are true:

1. The `V1` freeze checklist is already satisfied.
2. `compare_report` fully covers the retained `V1` compare/report needs:
   - assumptions
   - metrics
   - trade ledger
   - equity curve
   - overview/headline/highlights
3. No active docs still present `report_narrative` as the preferred long-term
   report truth.
4. API consumers have either:
   - migrated to `compare_report`, or
   - accepted a deprecation window.

## Migration Steps

### Step 1. Documentation rewording

Before any response-shape deletion:

- reword docs so `compare_report` is described as the canonical external report
  view
- reword `report_narrative` as a compatibility layer retained for transition
- remove wording that presents both as equal long-term outputs

### Step 2. Test rewording

Before removing any field:

- change compare API tests so:
  - `compare_report` is the primary asserted report truth
  - `report_narrative` is checked only for compatibility presence and basic
    consistency
- stop duplicating full expected JSON for both shapes where the intent is
  simply "same report truth, different outward wrapper"

### Step 3. Deprecation window

Once docs and tests are aligned:

- keep `report_narrative` present for one explicit transition window
- mark it as deprecated in docs
- avoid adding any new capabilities only to `report_narrative`

### Step 4. Removal

After the transition window:

- remove `report_narrative` from the compare response
- delete its dedicated builder if it no longer serves any internal purpose
- keep the shared compare/report bundle only if it still feeds
  `compare_report`

### Step 5. Final cleanup

After removal:

- delete compatibility-only tests
- delete compatibility-only docs wording
- make `compare_report` the only named compare/report truth in roadmap and
  queue docs

## Deletion Priority

When this migration is reopened, delete in this order:

1. test wording that treats both fields as equal long-term truth
2. doc wording that treats both fields as equal long-term truth
3. `report_narrative` compatibility assertions
4. `report_narrative` response field
5. any now-unused compatibility builder or helper

## Do Not Remove During V1

The following actions are explicitly out of bounds during `V1` phase close:

- removing `report_narrative`
- renaming `compare_report`
- changing compare response shape in a breaking way
- forcing clients to switch during freeze cleanup

## Success Condition

Treat the post-`V1` migration as complete only when:

1. `compare_report` is the only documented external report truth.
2. The compare API exposes only one authoritative report module.
3. There are no compatibility-only tests left for `report_narrative`.
4. Queue and roadmap docs no longer describe a dual report truth.
