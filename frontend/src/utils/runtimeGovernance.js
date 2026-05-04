export const DEFAULT_PERMISSION_BOUNDARY = {
  model_version: "quantpilot/permission-boundary/v1",
  execution_owner_module: "builtin.execution.paper",
  live_execution_allowed: false,
  ai_write_policy: "disabled",
  plugin_network_default: "deny",
  non_execution_order_access: "deny"
};

export const DEFAULT_RUNTIME_GOVERNANCE = {
  schema_version: "quantpilot/runtime-governance/v1",
  governance_source: "legacy_default",
  capability_hash: "unknown",
  strategy_version: "unknown",
  parameter_version: "unknown",
  deployment_revision: "unknown",
  permission_boundary: DEFAULT_PERMISSION_BOUNDARY
};

function nonEmptyString(value, fallback) {
  return typeof value === "string" && value.trim().length > 0 ? value.trim() : fallback;
}

function normalizePermissionBoundary(permissionBoundary) {
  const source =
    permissionBoundary && typeof permissionBoundary === "object" ? permissionBoundary : {};
  return {
    model_version: nonEmptyString(
      source.model_version,
      DEFAULT_PERMISSION_BOUNDARY.model_version
    ),
    execution_owner_module: nonEmptyString(
      source.execution_owner_module,
      DEFAULT_PERMISSION_BOUNDARY.execution_owner_module
    ),
    live_execution_allowed:
      source.live_execution_allowed === true
        ? true
        : DEFAULT_PERMISSION_BOUNDARY.live_execution_allowed,
    ai_write_policy:
      source.ai_write_policy === "proposal_only" || source.ai_write_policy === "disabled"
        ? source.ai_write_policy
        : DEFAULT_PERMISSION_BOUNDARY.ai_write_policy,
    plugin_network_default:
      source.plugin_network_default === "allow" || source.plugin_network_default === "deny"
        ? source.plugin_network_default
        : DEFAULT_PERMISSION_BOUNDARY.plugin_network_default,
    non_execution_order_access:
      source.non_execution_order_access === "allow" ||
      source.non_execution_order_access === "deny"
        ? source.non_execution_order_access
        : DEFAULT_PERMISSION_BOUNDARY.non_execution_order_access
  };
}

export function normalizeRuntimeGovernanceSnapshot(governance) {
  const source = governance && typeof governance === "object" ? governance : {};
  return {
    schema_version: nonEmptyString(
      source.schema_version,
      DEFAULT_RUNTIME_GOVERNANCE.schema_version
    ),
    governance_source: nonEmptyString(
      source.governance_source,
      DEFAULT_RUNTIME_GOVERNANCE.governance_source
    ),
    capability_hash: nonEmptyString(
      source.capability_hash,
      DEFAULT_RUNTIME_GOVERNANCE.capability_hash
    ),
    strategy_version: nonEmptyString(
      source.strategy_version,
      DEFAULT_RUNTIME_GOVERNANCE.strategy_version
    ),
    parameter_version: nonEmptyString(
      source.parameter_version,
      DEFAULT_RUNTIME_GOVERNANCE.parameter_version
    ),
    deployment_revision: nonEmptyString(
      source.deployment_revision,
      DEFAULT_RUNTIME_GOVERNANCE.deployment_revision
    ),
    permission_boundary: normalizePermissionBoundary(source.permission_boundary)
  };
}

export function governanceFromRuntime(runtime = {}) {
  if (runtime?.governance) return normalizeRuntimeGovernanceSnapshot(runtime.governance);
  if (runtime?.backtestArtifacts?.manifest?.governance) {
    return normalizeRuntimeGovernanceSnapshot(runtime.backtestArtifacts.manifest.governance);
  }

  const eventWithEnvelope = Array.isArray(runtime?.events)
    ? runtime.events.find((event) => event?.envelope)
    : null;
  if (eventWithEnvelope?.envelope) {
    const envelope = eventWithEnvelope.envelope;
    return normalizeRuntimeGovernanceSnapshot({
      governance_source: "event_envelope",
      capability_hash: envelope.capability_hash,
      strategy_version: envelope.strategy_version,
      parameter_version: envelope.parameter_version,
      deployment_revision: envelope.deployment_revision
    });
  }

  return normalizeRuntimeGovernanceSnapshot(null);
}

function shortHash(value) {
  if (typeof value !== "string" || value.length <= 18) return value || "unknown";
  return `${value.slice(0, 13)}...${value.slice(-6)}`;
}

export function buildGovernanceIdentityRows(governance) {
  const normalized = normalizeRuntimeGovernanceSnapshot(governance);
  return [
    {
      key: "capability_hash",
      label: "能力边界",
      value: shortHash(normalized.capability_hash),
      fullValue: normalized.capability_hash
    },
    {
      key: "deployment_revision",
      label: "部署修订",
      value: shortHash(normalized.deployment_revision),
      fullValue: normalized.deployment_revision
    },
    {
      key: "strategy_version",
      label: "策略版本",
      value: normalized.strategy_version
    },
    {
      key: "parameter_version",
      label: "参数版本",
      value: shortHash(normalized.parameter_version),
      fullValue: normalized.parameter_version
    },
    {
      key: "permission_model",
      label: "权限模型",
      value: normalized.permission_boundary.model_version
    },
    {
      key: "ai_write_policy",
      label: "AI 写入",
      value: normalized.permission_boundary.ai_write_policy
    },
    {
      key: "governance_source",
      label: "治理来源",
      value: normalized.governance_source
    }
  ];
}
