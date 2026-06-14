# FE-0029 Frontend Capabilities Governance Registry Entries Closeout

Status: closed.

## Leaf Node

`frontend.capabilities.governance_registry.registry_entries`

## Code Changes

- Added `frontend/src/capabilities/capabilityGovernanceRegistry.js`.
- Added `frontend/src/capabilities/capabilityGovernanceRegistry.test.js`.
- Updated `frontend/src/capabilities/capabilityGovernance.js` to consume and re-export the generated registry from the new whitebox node.

## Preserved Behavior

- The governance registry still derives entries from the support matrix truth source.
- Runtime modes, UI actions, workspace surfaces, market boundaries, indicators, frontend modules, compile boundaries, and user-facing claims keep their existing generated entry shape.
- `CAPABILITY_GOVERNANCE_REGISTRY` remains import-compatible from `capabilityGovernance.js`.

## Public Inputs

- Support matrix constants.
- Governance core class and owner-role vocabulary.

## Public Outputs

- `CAPABILITY_GOVERNANCE_REGISTRY`.

## Verification

- From `frontend/`, `npm.cmd test -- --run src/capabilities/capabilityGovernanceRegistry.test.js src/capabilities/capabilityGovernance.test.js src/capabilities/capabilityGovernanceCore.test.js`: passed, 3 test files and 14 tests.

## Further-Split Decision

No deeper split for registry entries now. The generated entry families are tightly coupled by one output contract and are already covered by family-alignment tests.

## Residuals

- Continue with `frontend.capabilities.governance_registry.public_facade`.
