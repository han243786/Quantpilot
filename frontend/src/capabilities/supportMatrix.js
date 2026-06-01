export { isCapabilitySyncBlocked } from "./capabilitySync";
export {
  CAPABILITY_ACTION_MAP,
  DECLARED_INDICATOR_KINDS,
  SUPPORT_MATRIX,
  SUPPORTED_EXCHANGES,
  SUPPORTED_FRONTEND_MODULE_KEYS,
  SUPPORTED_INDICATOR_KINDS,
  SUPPORTED_RUNTIME_EXECUTION_MODULES,
  SUPPORTED_RUNTIME_MODES,
  SUPPORTED_SYMBOLS,
  WORKSPACE_SURFACE_MAP
} from "./capabilityCatalog";

import { CAPABILITY_ACTION_MAP } from "./capabilityCatalog";

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

function normalizeUiActionStatus(actionKey, capabilities) {
  const action = CAPABILITY_ACTION_MAP[actionKey];
  if (!action) {
    return {
      status: "unsupported",
      reason: "未知操作。"
    };
  }

  const entries = capabilities?.ui_actions?.actions;
  if (!Array.isArray(entries)) {
    return {
      status: "unsupported",
      reason: "后端能力快照缺少 ui_actions.actions。"
    };
  }

  const entry = entries.find((item) => item?.key === actionKey);
  if (!entry) {
    return {
      status: "unsupported",
      reason: "后端能力快照未声明该操作。"
    };
  }

  if (entry.status === "supported") {
    return {
      status: "supported",
      reason: ""
    };
  }

  return {
    status: entry.status === "declared_only" ? "declared_only" : "unsupported",
    reason:
      typeof entry.reason === "string" && entry.reason.trim().length > 0
        ? entry.reason.trim()
        : "后端已声明该操作，但当前版本未开放。"
  };
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

export function getCapabilityActionBlockReason({
  actionKey,
  capabilityStatus,
  capabilitySource,
  capabilityMessage,
  capabilities
}) {
  const action = CAPABILITY_ACTION_MAP[actionKey];
  if (!action) return "";

  if (action.blockedDuringCapabilitySync && capabilityStatus === "loading") {
    return `${action.label}暂时锁定，前端正在同步后端能力快照。`;
  }

  if (action.blockedDuringCapabilitySync && capabilitySource === "safe_fallback") {
    const detail =
      typeof capabilityMessage === "string" && capabilityMessage.trim().length > 0
        ? capabilityMessage.trim()
        : "能力校验失败，风险操作会在安全回退模式下继续保持锁定。";
    return `${action.label}在安全回退模式下不可用。${detail}`;
  }

  const uiActionStatus = normalizeUiActionStatus(actionKey, capabilities);
  if (uiActionStatus.status !== "supported") {
    return `${action.label}暂时不可用，${uiActionStatus.reason}`;
  }

  if (
    action.blockedDuringCapabilitySync &&
    (capabilityStatus === "degraded" || capabilitySource === "cache")
  ) {
    return `${action.label}正在使用缓存能力快照，最终可用性仍由后端实时校验。`;
  }

  if (action.blockedDuringCapabilitySync && capabilityStatus === "error") {
    const detail =
      typeof capabilityMessage === "string" && capabilityMessage.trim().length > 0
        ? capabilityMessage.trim()
        : "能力服务不可用。";
    return `${action.label}暂时锁定，${detail}`;
  }

  const boundaryIssues = getCapabilityBoundaryIssues(capabilities);
  if (boundaryIssues.length > 0) {
    return `${action.label}暂时锁定，${boundaryIssues[0]}`;
  }

  return "";
}
