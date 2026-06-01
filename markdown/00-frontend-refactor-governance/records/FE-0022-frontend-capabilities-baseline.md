# FE-0022 Frontend Capabilities Baseline

Status: baseline established.

## Parent Node

`frontend.capabilities`

## Current Scope

The current frontend capability system is a cross-cutting contract hub. Pure capability truth lives under `frontend/src/capabilities/`, while default capability snapshots and module filtering still live inside module and store files. This parent should extract stable capability whitebox nodes without mass-migrating feature UI consumers.

## Initial Child Queue

- `frontend.capabilities.support_matrix_truth`
- `frontend.capabilities.capability_projection`
- `frontend.capabilities.governance_registry`
- `frontend.capabilities.builtin_capability_snapshot`
- `frontend.capabilities.module_registry_gate`
- `frontend.capabilities.store_capability_refresh`

## Current Owned And Split-Target Files

- `frontend/src/capabilities/supportMatrix.js`
- `frontend/src/capabilities/supportMatrix.test.js`
- `frontend/src/capabilities/capabilityProjection.js`
- `frontend/src/capabilities/capabilityProjection.test.js`
- `frontend/src/capabilities/capabilityGovernance.js`
- `frontend/src/capabilities/capabilityGovernance.test.js`
- `frontend/src/modules/moduleRegistry.js`
- `frontend/src/modules/moduleRegistry.test.js`
- `frontend/src/modules/builtinModules.js`
- `frontend/src/store/graphStore.capabilities.test.js`

## Important Consumers

- `frontend/src/store/graphStore.js`
- `frontend/src/store/graphStorePersistenceHelpers.js`
- `frontend/src/store/graphStoreCompileActions.js`
- `frontend/src/store/graphStoreRuntimeSessionActions.js`
- `frontend/src/components/ModuleSidebar.jsx`
- `frontend/src/components/TopToolbar.jsx`
- `frontend/src/pages/StrategyWorkspacePage.jsx`
- `frontend/src/pages/StrategyWorkspaceExperimentCard.jsx`
- `frontend/src/graph/compileGraph.js`

## Whitebox Contract

### Public Inputs

- Backend `/api/capabilities` snapshots.
- Cached capability snapshots from local storage.
- Safe fallback capability reasons.
- Capability status, source, and message from the graph store.
- Frontend module definitions and external module metadata.

### Public Outputs

- Support matrix constants and maps.
- Capability boundary issues and runtime capability contexts.
- UI action block reasons.
- Projected workspace surfaces and UI actions.
- Capability governance registry and text gates.
- Normalized/default/safe-fallback capability snapshots.
- Capability-filtered module registries.

## Equivalence Anchors

- `frontend/src/capabilities/supportMatrix.test.js`
- `frontend/src/capabilities/capabilityProjection.test.js`
- `frontend/src/capabilities/capabilityGovernance.test.js`
- `frontend/src/modules/moduleRegistry.test.js`
- `frontend/src/store/graphStore.capabilities.test.js`
- `frontend/src/components/ModuleSidebar.test.jsx`
- `frontend/src/components/TopToolbar.capabilities.test.jsx`
- `frontend/src/pages/StrategyWorkspacePage.codeMode.test.jsx`
- Frontend build.

## Split Rules

- Do not loosen capability safe-fallback behavior.
- Do not change supported/disallowed user-facing claims.
- Do not expose unsupported modules as enabled.
- Do not bypass permission boundary checks for runtime, compile, or backtest actions.
- Keep existing imports from `../capabilities/supportMatrix`, `../capabilities/capabilityProjection`, and module registry files stable unless the current leaf creates a compatibility gateway.

## First Leaf

`frontend.capabilities.support_matrix_truth`
