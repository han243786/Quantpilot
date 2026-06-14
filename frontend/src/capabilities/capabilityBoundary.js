const EXPECTED_PERMISSION_BOUNDARY = {
  model_version: "quantpilot/permission-boundary/v1",
  execution_owner_module: "builtin.execution.paper",
  live_execution_allowed: false,
  ai_write_policy: "proposal_only",
  plugin_network_default: "deny",
  non_execution_order_access: "deny"
};

function isTrustedCapabilityHash(value) {
  return typeof value === "string" && /^sha256:[0-9a-f]{64}$/.test(value);
}

export function getCapabilityBoundaryIssues(capabilities) {
  const issues = [];
  if (!capabilities || typeof capabilities !== "object") {
    return ["缺少后端能力快照。"];
  }

  if (!isTrustedCapabilityHash(capabilities.schema_hash)) {
    issues.push("能力 hash 缺失或格式非法。");
  }

  const permission = capabilities.permission_boundary;
  if (!permission || typeof permission !== "object") {
    issues.push("缺少 permission_boundary。");
    return issues;
  }

  for (const [key, expected] of Object.entries(EXPECTED_PERMISSION_BOUNDARY)) {
    if (permission[key] !== expected) {
      issues.push(`permission_boundary.${key} 必须为 ${String(expected)}。`);
    }
  }

  return issues;
}

export function buildCapabilityContext(capabilities) {
  if (getCapabilityBoundaryIssues(capabilities).length > 0) return null;
  return {
    schema_hash: capabilities.schema_hash,
    permission_boundary: {
      model_version: capabilities.permission_boundary.model_version,
      execution_owner_module: capabilities.permission_boundary.execution_owner_module,
      live_execution_allowed: capabilities.permission_boundary.live_execution_allowed,
      ai_write_policy: capabilities.permission_boundary.ai_write_policy,
      plugin_network_default: capabilities.permission_boundary.plugin_network_default,
      non_execution_order_access: capabilities.permission_boundary.non_execution_order_access
    }
  };
}
