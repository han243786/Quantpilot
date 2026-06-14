# FE-0038 Frontend Capabilities Parent Closeout

Status: closed.

## Parent Node

`frontend.capabilities`

## Closed Leaves

- `frontend.capabilities.support_matrix_truth.sync_block_gate`
- `frontend.capabilities.support_matrix_truth.catalog_maps`
- `frontend.capabilities.support_matrix_truth.boundary_context`
- `frontend.capabilities.support_matrix_truth.action_block_reason`
- `frontend.capabilities.capability_projection`
- `frontend.capabilities.governance_registry.core_contract`
- `frontend.capabilities.governance_registry.registry_entries`
- `frontend.capabilities.governance_registry.public_facade`
- `frontend.capabilities.governance_registry`
- `frontend.capabilities.builtin_capability_snapshot.default_snapshot`
- `frontend.capabilities.builtin_capability_snapshot.normalization`
- `frontend.capabilities.builtin_capability_snapshot`
- `frontend.capabilities.module_registry_gate.contract_validation`
- `frontend.capabilities.module_registry_gate.registry_assembly`
- `frontend.capabilities.module_registry_gate.public_facade`
- `frontend.capabilities.module_registry_gate`
- `frontend.capabilities.store_capability_refresh.refresh_state_projection`
- `frontend.capabilities.store_capability_refresh.public_action_facade`
- `frontend.capabilities.store_capability_refresh`

## Final Parent Boundary

`frontend.capabilities` now owns frontend support-matrix truth, capability catalog/boundary/sync helpers, capability projection, capability governance registry, built-in/default/safe-fallback capability snapshots, capability normalization, module registry contracts and assembly, and graph-store capability refresh projection.

## Whitebox Contract

### Public Inputs

- Backend `/api/capabilities` snapshots.
- Cached capability snapshots from local storage.
- Safe fallback reasons.
- Frontend module definitions and optional external module metadata.
- Capability status, source, and message from graph store state.

### Public Outputs

- Support matrix constants, maps, and compatibility exports.
- Capability boundary issues and runtime capability contexts.
- UI action block reasons and capability-driven projection maps.
- Capability governance registry and public facade.
- Default and safe fallback capability snapshots.
- Capability normalization helpers.
- Plugin manifest/capability contract validation.
- External module metadata assembly and registry facade.
- Capability refresh state projection for graph store.

### Parent-Owned Files

- `frontend/src/capabilities/supportMatrix.js`
- `frontend/src/capabilities/supportMatrix.test.js`
- `frontend/src/capabilities/capabilityActionBlocks.js`
- `frontend/src/capabilities/capabilityActionBlocks.test.js`
- `frontend/src/capabilities/capabilityCatalog.js`
- `frontend/src/capabilities/capabilityCatalog.test.js`
- `frontend/src/capabilities/capabilityBoundary.js`
- `frontend/src/capabilities/capabilityBoundary.test.js`
- `frontend/src/capabilities/capabilitySync.js`
- `frontend/src/capabilities/capabilitySync.test.js`
- `frontend/src/capabilities/capabilityProjection.js`
- `frontend/src/capabilities/capabilityProjection.test.js`
- `frontend/src/capabilities/capabilityGovernanceCore.js`
- `frontend/src/capabilities/capabilityGovernanceCore.test.js`
- `frontend/src/capabilities/capabilityGovernanceRegistry.js`
- `frontend/src/capabilities/capabilityGovernanceRegistry.test.js`
- `frontend/src/capabilities/capabilityGovernance.js`
- `frontend/src/capabilities/capabilityGovernance.test.js`
- `frontend/src/capabilities/builtinCapabilitySnapshot.js`
- `frontend/src/capabilities/builtinCapabilitySnapshot.test.js`
- `frontend/src/capabilities/capabilityNormalization.js`
- `frontend/src/capabilities/capabilityNormalization.test.js`
- `frontend/src/modules/moduleRegistryContracts.js`
- `frontend/src/modules/moduleRegistryContracts.test.js`
- `frontend/src/modules/moduleRegistryAssembly.js`
- `frontend/src/modules/moduleRegistryAssembly.test.js`
- `frontend/src/modules/moduleRegistry.js`
- `frontend/src/modules/moduleRegistry.test.js`
- `frontend/src/modules/builtinModules.js`
- `frontend/src/store/graphStoreCapabilityRefresh.js`
- `frontend/src/store/graphStoreCapabilityRefresh.test.js`
- `frontend/src/store/graphStore.capabilities.test.js`

## Preserved Behavior

- Safe fallback capability snapshots still stay outside the trusted capability boundary.
- Unsupported modules are still visible only as unsupported marketplace entries and are not activated.
- Capability-gated compile/runtime UI actions still block during loading or safe fallback.
- Store capability refresh still follows remote, cache, and safe fallback paths.
- Existing compatibility imports from support matrix and module registry gateways continue to work.

## Further-Split Decision

No further split is useful inside `frontend.capabilities` now. Remaining capability consumers should be handled inside their owning feature parents rather than forcing more cross-parent churn here.

## Verification

- FE-0036 amended pre-commit: frontend build passed.
- FE-0036 amended pre-commit: Vitest passed, 126 test files and 361 tests.
- FE-0037 pre-commit: full feature tree check passed.
- Parent closeout targeted anchor tests passed for support matrix, projection, governance, module registry, store capability refresh, module sidebar, toolbar capabilities, and workspace code mode.

## Next Parent Candidate

`frontend.strategy_workspace`
