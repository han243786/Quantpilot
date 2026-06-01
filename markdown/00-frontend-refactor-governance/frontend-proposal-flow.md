# Frontend Proposal Flow

Status: frontend-specific proposal flow.

## Flow

1. Proposal: name the frontend node, intended extraction, and expected behavior preservation.
2. Adaptability check: verify parent boundary, dependencies, tests, and whether backend/global docs would be touched.
3. Scheme refinement: reduce coupling, clarify public contracts, and choose light/standard/heavy scale.
4. Return to adaptability check when the scheme changes.
5. Land design: write the leaf or parent baseline before code extraction.

## Required Frontend Proposal Fields

- Target frontend node.
- Scale: light, standard, or heavy.
- Owned files expected to move or be wrapped.
- Public contracts affected: props, events, routes, store, API, styles, tests.
- Equivalence anchor.
- Forbidden global files that remain untouched.
- Split decision criteria.

## Rejection Conditions

- The proposal requires global tree edits during isolated recursion.
- The proposal creates sibling-to-sibling development coupling.
- The proposal depends on release-transition optimization without explicit developer direction.
- The proposal cannot name a preserved frontend behavior.
