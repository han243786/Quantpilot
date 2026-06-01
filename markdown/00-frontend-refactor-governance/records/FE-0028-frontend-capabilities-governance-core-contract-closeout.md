# FE-0028 Frontend Capabilities Governance Core Contract Closeout

Status: closed.

## Leaf Node

`frontend.capabilities.governance_registry.core_contract`

## Code Changes

- Added `frontend/src/capabilities/capabilityGovernanceCore.js`.
- Added `frontend/src/capabilities/capabilityGovernanceCore.test.js`.
- Updated `frontend/src/capabilities/capabilityGovernance.js` to consume and re-export the core governance contract.

## Preserved Behavior

- `capabilityGovernance.js` still exports the schema version, class vocabulary, owner roles, text gates, registry, aggregate object, and lookup helper.
- Existing governance registry entries keep the same generated shape.
- Text-gate positive claim audit metadata remains unchanged.

## Public Inputs

- Governance entry fields: `id`, `family`, `value`, `className`, `ownerRole`, `reviewResponsibility`, `sourceOfTruth`, `notes`, and `textGate`.

## Public Outputs

- `CAPABILITY_GOVERNANCE_SCHEMA_VERSION`.
- `CAPABILITY_CLASSES`.
- `CAPABILITY_OWNER_ROLES`.
- `CAPABILITY_TEXT_GATES`.
- `buildCapabilityGovernanceEntry(entryFields)`.

## Verification

- From `frontend/`, `npm.cmd test -- --run src/capabilities/capabilityGovernanceCore.test.js src/capabilities/capabilityGovernance.test.js`: passed, 2 test files and 12 tests.

## Further-Split Decision

`frontend.capabilities.governance_registry` is worth continuing to split. Remaining responsibilities are registry entry generation and public facade/query export.

## Residuals

- Continue with `frontend.capabilities.governance_registry.registry_entries`.
- Then close `frontend.capabilities.governance_registry.public_facade`.
