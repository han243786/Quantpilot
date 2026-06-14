import {
  CAPABILITY_CLASSES,
  CAPABILITY_GOVERNANCE_SCHEMA_VERSION,
  CAPABILITY_OWNER_ROLES,
  CAPABILITY_TEXT_GATES
} from "./capabilityGovernanceCore.js";
import { CAPABILITY_GOVERNANCE_REGISTRY } from "./capabilityGovernanceRegistry.js";

export {
  CAPABILITY_CLASSES,
  CAPABILITY_GOVERNANCE_SCHEMA_VERSION,
  CAPABILITY_OWNER_ROLES,
  CAPABILITY_TEXT_GATES
} from "./capabilityGovernanceCore.js";
export { CAPABILITY_GOVERNANCE_REGISTRY } from "./capabilityGovernanceRegistry.js";

export const CAPABILITY_GOVERNANCE = {
  schemaVersion: CAPABILITY_GOVERNANCE_SCHEMA_VERSION,
  classes: CAPABILITY_CLASSES,
  ownerRoles: CAPABILITY_OWNER_ROLES,
  textGates: CAPABILITY_TEXT_GATES,
  registry: CAPABILITY_GOVERNANCE_REGISTRY
};

export function findCapabilityGovernanceEntry(id) {
  return CAPABILITY_GOVERNANCE_REGISTRY.find((entry) => entry.id === id) || null;
}
