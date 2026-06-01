export { buildCapabilityContext, getCapabilityBoundaryIssues } from "./capabilityBoundary";
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
import { getCapabilityBoundaryIssues } from "./capabilityBoundary";

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
