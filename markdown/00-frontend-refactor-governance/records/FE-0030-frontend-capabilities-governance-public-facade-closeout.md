# FE-0030 Frontend Capabilities Governance Public Facade Closeout

Status: closed.

## Leaf Node

`frontend.capabilities.governance_registry.public_facade`

## Code Changes

- No code movement in this step.
- Confirmed `frontend/src/capabilities/capabilityGovernance.js` is now a narrow public facade over governance core and registry entries.

## Preserved Behavior

- Existing imports from `capabilityGovernance.js` continue to work.
- The aggregate `CAPABILITY_GOVERNANCE` object still exposes schema version, classes, owner roles, text gates, and registry.
- `findCapabilityGovernanceEntry(id)` still returns the matching registry entry or `null`.

## Public Inputs

- Governance entry id.

## Public Outputs

- `CAPABILITY_GOVERNANCE`.
- `findCapabilityGovernanceEntry(id)`.
- Compatibility re-exports from governance core and registry.

## Verification

- From `frontend/`, `npm.cmd test -- --run src/capabilities/capabilityGovernance.test.js src/capabilities/capabilityGovernanceRegistry.test.js src/capabilities/capabilityGovernanceCore.test.js`: passed.

## Further-Split Decision

No further split. The facade is intentionally small and exists to preserve the public import boundary for consumers.

## Residuals

- `frontend.capabilities.governance_registry` is closed.
- Continue with `frontend.capabilities.builtin_capability_snapshot`.
