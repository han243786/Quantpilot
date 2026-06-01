# Frontend Standard Matrix

Status: frontend-only rules, initialized independently.

## Hard Rules

1. Parent-child communication is mandatory during development refactor. Sibling leaves communicate through their parent boundary.
2. Public component props, exported hooks, store actions, API methods, capability adapters, and key public utilities are whitebox nodes.
3. View components must not own backend API mutation logic unless the leaf is explicitly an API-facing page adapter.
4. Store/actions may coordinate state, but must not own layout or visual policy.
5. Styles follow their owning component, page, or design-system leaf. Global style edits require a `frontend.global_style` node.
6. Tests may be lightweight, but every standard/heavy leaf needs an equivalence anchor or a documented reason for temporary omission.
7. AI must not propose release-transition coupling. It may only evaluate such coupling after the developer explicitly states release-transition intent.

## Whitebox Node Fields

Each durable frontend node should record:

- Node id.
- Parent node.
- Owned files.
- Public inputs: props, events, route params, query params, store selectors, API arguments.
- Public outputs: rendered state, callbacks, dispatched actions, store updates, API responses, emitted events.
- Internal handlers: key functions, hooks, reducer/actions, effects, validation, formatters.
- External dependencies: parent boundary, backend API, store, capability adapters, style tokens, test utilities.
- Equivalence anchor.
- Split decision.

## Further-Split Gate

Further split is required when at least one condition is true:

- A leaf owns two or more independent user workflows.
- A leaf mixes rendering, data transport, persistence, and orchestration in ways that block isolated testing.
- A leaf has multiple public contracts that change for unrelated reasons.
- A leaf contains reusable capability logic currently hidden inside a page/component.
- A leaf is a frequent merge-conflict hotspot across frontend workstreams.

Further split is rejected when the split only creates presentational fragments, prop drilling grows faster than clarity, or the boundary is based only on line count.
