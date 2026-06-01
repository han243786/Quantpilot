# Frontend Process Matrix

Status: initialized from blank frontend governance state.

## Execution Scale

| Scale | Trigger | Required Artifacts |
| --- | --- | --- |
| Light | Single component, hook, style, or adapter with no cross-parent behavior change. | Local note in the touched leaf and smoke-equivalent check. |
| Standard | Leaf extraction, route/page boundary change, store/API contract movement, or shared UI behavior. | Leaf baseline, extraction note, closeout, split decision. |
| Heavy | Parent module split, app shell/routing/store architecture change, or global style capability movement. | Parent baseline, staged leaf queue, compatibility plan, parent closeout. |

## Recursive Frontend Flow

1. Pick one frontend parent module.
2. Establish parent boundary and local whitebox contract.
3. Split first-level leaves only far enough to make ownership clear.
4. For each leaf, create an equivalence baseline before extraction.
5. Extract the leaf behind the parent boundary.
6. Run leaf closeout and decide whether it deserves another recursive split.
7. If it deserves a split, treat that leaf as a temporary parent and repeat.
8. If it does not deserve a split, freeze the leaf contract and continue siblings.
9. Close the parent only after all leaves are stable or explicitly deferred.

## Frontend-Specific Closeout

Each frontend leaf closeout must answer:

- What user-visible behavior is preserved?
- What owner now controls rendering, data access, effects, styles, and tests?
- Which props, events, store actions, API calls, or capability methods cross the boundary?
- Which files are intentionally left behind for later frontend parents?
- Is further split useful under the hard split rules?

## Forbidden During Isolated Frontend Recursion

- Updating global module tree or global full feature tree.
- Updating backend recursion state or backend milestone logs.
- Creating cross-leaf shortcuts unless the developer explicitly declares release-transition work.
- Treating E2E cleanup as part of frontend extraction. E2E整理 remains delayed.
