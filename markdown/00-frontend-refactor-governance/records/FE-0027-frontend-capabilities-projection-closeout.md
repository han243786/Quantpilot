# FE-0027 Frontend Capabilities Projection Closeout

Status: closed.

## Leaf Node

`frontend.capabilities.capability_projection`

## Code Changes

- No code movement in this step.
- Confirmed `frontend/src/capabilities/capabilityProjection.js` already owns the projection boundary for workspace surfaces, UI actions, and the combined capability view.

## Preserved Behavior

- Workspace surfaces are still projected from backend `workspace.surfaces` declarations.
- UI actions are still projected from backend `ui_actions.actions` declarations.
- Capability action block reasons still flow through the action-block whitebox node via the existing compatibility entrypoint.
- Cache/degraded mode continues to keep supported actions enabled while preserving the warning block reason.

## Public Inputs

- Backend capability snapshot.
- `capabilityStatus`.
- `capabilitySource`.
- `capabilityMessage`.

## Public Outputs

- `projectWorkspaceSurfaces(capabilities)`.
- `projectUiActions({ capabilities, capabilityStatus, capabilitySource, capabilityMessage })`.
- `projectCapabilityView({ capabilities, capabilityStatus, capabilitySource, capabilityMessage })`.
- `isProjectedEntryEnabled(entry)`.

## Verification

- From `frontend/`, `npm.cmd test -- --run src/capabilities/capabilityProjection.test.js src/capabilities/capabilityActionBlocks.test.js src/components/TopToolbar.capabilities.test.jsx src/pages/StrategyWorkspaceExperimentCard.test.jsx`: passed.

## Further-Split Decision

No further split now. The file is small, cohesive, and the helper functions are private implementation details under one projection contract.

## Residuals

- Continue with `frontend.capabilities.governance_registry`.
