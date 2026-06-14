# FE-0026 Frontend Capabilities Action Block Reason Closeout

Status: closed.

## Leaf Node

`frontend.capabilities.support_matrix_truth.action_block_reason`

## Code Changes

- Added `frontend/src/capabilities/capabilityActionBlocks.js`.
- Added `frontend/src/capabilities/capabilityActionBlocks.test.js`.
- Updated `frontend/src/capabilities/supportMatrix.js` to re-export the action block helper from the new whitebox node.

## Preserved Behavior

- Capability-gated actions still block while trusted backend capabilities are loading.
- Safe fallback still blocks risk actions and surfaces the capability message.
- Backend `ui_actions.actions` declarations remain the source of truth for supported, declared-only, or unsupported actions.
- Permission-boundary issues still block declared actions after action declaration checks.
- Existing imports from `frontend/src/capabilities/supportMatrix.js` continue to work.

## Public Inputs

- `actionKey`.
- `capabilityStatus`.
- `capabilitySource`.
- `capabilityMessage`.
- Backend capability snapshot.

## Public Outputs

- `getCapabilityActionBlockReason({ actionKey, capabilityStatus, capabilitySource, capabilityMessage, capabilities })`.

## Verification

- From `frontend/`, `npm.cmd test -- --run src/capabilities/capabilityActionBlocks.test.js src/capabilities/supportMatrix.test.js src/capabilities/capabilityProjection.test.js src/components/TopToolbar.capabilities.test.jsx src/pages/StrategyWorkspaceExperimentCard.test.jsx`: passed, 5 test files and 28 tests.

## Further-Split Decision

No further split inside `frontend.capabilities.support_matrix_truth.action_block_reason` now. The node has one private action status normalizer and one public block-reason function.

## Residuals

- `frontend.capabilities.support_matrix_truth` has no remaining leaf logic inside `frontend/src/capabilities/supportMatrix.js`; the parent can be closed after governance confirms the gateway-only entrypoint.
