# Frontend Recursive Speed Protocol

Status: frontend-only acceleration rules.

## Speed Rules

1. Prefer parent-level batching for discovery, then leaf-level commits for extraction.
2. Do not build a full tree before evidence exists. Seed only candidate nodes, then promote nodes after baseline/closeout.
3. Use one-page leaf records: boundary, owned files, equivalence anchor, split decision, residuals.
4. For repeated UI leaf patterns, reuse the same closeout checklist instead of rearguing the process.
5. Delay E2E整理. Use frontend-local smoke, unit, or component anchors when feasible.
6. If a leaf has no meaningful split trigger, close it immediately and move on.
7. If a split decision needs product or architecture judgment, stop and ask the developer instead of guessing.

## Fast Path

A frontend leaf may use fast path when all are true:

- No route contract changes.
- No backend API contract changes.
- No shared store schema changes.
- No global style edits.
- Equivalence can be checked by build, focused test, or static import validation.

Fast path still requires a short closeout and split decision.
