import { CAPABILITY_ACTION_MAP, WORKSPACE_SURFACE_MAP } from "./supportMatrix";
import { DEFAULT_CAPABILITIES } from "./builtinCapabilitySnapshot";
import { sanitizeDisplayText } from "../utils/errorText";

function normalizeSupportStatus(status) {
  if (status === "supported") return "supported";
  if (status === "declared_only") return "declared_only";
  return "unsupported";
}

function normalizeNamedSupportEntries(entries, fallbackKeys = []) {
  if (Array.isArray(entries) && entries.length > 0) {
    return entries.map((entry) => ({
      key: entry.key,
      status: normalizeSupportStatus(entry.status),
      reason: sanitizeDisplayText(entry.reason, "")
    }));
  }

  return fallbackKeys.map((key) => ({
    key,
    status: "supported",
    reason: ""
  }));
}

function normalizeIndicatorSupportEntries(entries, declaredKinds = [], supportedKinds = []) {
  if (Array.isArray(entries) && entries.length > 0) {
    return entries.map((entry) => ({
      kind: entry.kind,
      status: normalizeSupportStatus(entry.status),
      reason: sanitizeDisplayText(entry.reason, "")
    }));
  }

  const supportedKindSet = new Set(supportedKinds);
  return declaredKinds.map((kind) => ({
    kind,
    status: supportedKindSet.has(kind) ? "supported" : "declared_only",
    reason: ""
  }));
}

function normalizeEnumValue(value, allowedValues, fallbackValue) {
  return allowedValues.includes(value) ? value : fallbackValue;
}

function normalizeBooleanValue(value, fallbackValue) {
  return typeof value === "boolean" ? value : fallbackValue;
}

function normalizePermissionBoundary(permissionBoundary) {
  const source =
    permissionBoundary && typeof permissionBoundary === "object" ? permissionBoundary : {};

  return {
    model_version: sanitizeDisplayText(
      source.model_version,
      DEFAULT_CAPABILITIES.permission_boundary.model_version
    ),
    execution_owner_module: sanitizeDisplayText(
      source.execution_owner_module,
      DEFAULT_CAPABILITIES.permission_boundary.execution_owner_module
    ),
    live_execution_allowed: normalizeBooleanValue(source.live_execution_allowed, false),
    ai_write_policy: normalizeEnumValue(
      source.ai_write_policy,
      ["proposal_only", "disabled"],
      "disabled"
    ),
    plugin_network_default: normalizeEnumValue(
      source.plugin_network_default,
      ["deny", "allow"],
      "deny"
    ),
    non_execution_order_access: normalizeEnumValue(
      source.non_execution_order_access,
      ["deny", "allow"],
      "deny"
    )
  };
}

function normalizeFrontendCapabilities(frontendCapabilities = {}, knownModuleKeys = []) {
  const legacySupportedModuleKeys = Array.isArray(frontendCapabilities.supported_module_keys)
    ? frontendCapabilities.supported_module_keys
    : DEFAULT_CAPABILITIES.frontend.supported_module_keys;
  const legacyUnsupportedReasons = {
    ...DEFAULT_CAPABILITIES.frontend.unsupported_module_reasons,
    ...(frontendCapabilities.unsupported_module_reasons || {})
  };
  const declaredModuleKeys =
    Array.isArray(frontendCapabilities.declared_module_keys) &&
    frontendCapabilities.declared_module_keys.length > 0
      ? frontendCapabilities.declared_module_keys
      : Array.from(
          new Set([
            ...knownModuleKeys,
            ...legacySupportedModuleKeys,
            ...Object.keys(legacyUnsupportedReasons)
          ])
        );

  const moduleSupportEntries =
    Array.isArray(frontendCapabilities.module_support) && frontendCapabilities.module_support.length > 0
      ? frontendCapabilities.module_support.map((entry) => ({
          module_key: entry.module_key,
          status: normalizeSupportStatus(entry.status),
          reason: sanitizeDisplayText(entry.reason, "")
        }))
      : declaredModuleKeys.map((module_key) => ({
          module_key,
          status: legacySupportedModuleKeys.includes(module_key) ? "supported" : "declared_only",
          reason: sanitizeDisplayText(legacyUnsupportedReasons[module_key], "")
        }));

  const supportedModuleKeys = moduleSupportEntries
    .filter((entry) => entry.status === "supported")
    .map((entry) => entry.module_key);
  const unsupportedModuleReasons = { ...legacyUnsupportedReasons };

  for (const entry of moduleSupportEntries) {
    if (entry.status !== "supported" && entry.reason) {
      unsupportedModuleReasons[entry.module_key] = entry.reason;
    }
  }

  return {
    ...DEFAULT_CAPABILITIES.frontend,
    ...frontendCapabilities,
    declared_module_keys: declaredModuleKeys,
    supported_module_keys: supportedModuleKeys,
    unsupported_module_reasons: unsupportedModuleReasons,
    module_support: moduleSupportEntries
  };
}

function normalizeUiCapabilityEntries(entries, fallbackMap, source) {
  const fallbackEntries = Object.keys(fallbackMap).map((key) => ({
    key,
    status: "supported",
    reason: "",
    source
  }));
  const sourceEntries = Array.isArray(entries) && entries.length > 0 ? entries : fallbackEntries;

  return sourceEntries
    .filter((entry) => entry && typeof entry === "object" && entry.key)
    .map((entry) => ({
      key: entry.key,
      status: normalizeSupportStatus(entry.status),
      reason: sanitizeDisplayText(entry.reason, ""),
      source: sanitizeDisplayText(entry.source, source)
    }));
}

function normalizeWorkspaceCapabilities(workspaceCapabilities = {}) {
  return {
    surfaces: normalizeUiCapabilityEntries(
      workspaceCapabilities.surfaces,
      WORKSPACE_SURFACE_MAP,
      "backend:/api/capabilities.workspace.surfaces"
    )
  };
}

function normalizeUiActionCapabilities(uiActionCapabilities = {}) {
  return {
    actions: normalizeUiCapabilityEntries(
      uiActionCapabilities.actions,
      CAPABILITY_ACTION_MAP,
      "backend:/api/capabilities.ui_actions.actions"
    )
  };
}

export function normalizeCapabilities(capabilities, { knownModuleKeys = [] } = {}) {
  if (!capabilities || typeof capabilities !== "object") {
    return DEFAULT_CAPABILITIES;
  }

  const strategyIr = {
    ...DEFAULT_CAPABILITIES.strategy_ir,
    ...capabilities.strategy_ir
  };
  const runtime = {
    ...DEFAULT_CAPABILITIES.runtime,
    ...capabilities.runtime
  };
  const marketData = {
    ...DEFAULT_CAPABILITIES.market_data,
    ...capabilities.market_data
  };

  return {
    ...DEFAULT_CAPABILITIES,
    ...capabilities,
    strategy_ir: {
      ...strategyIr,
      indicator_support: normalizeIndicatorSupportEntries(
        strategyIr.indicator_support,
        strategyIr.declared_indicator_kinds,
        strategyIr.supported_indicator_kinds
      )
    },
    runtime: {
      ...runtime,
      mode_support: normalizeNamedSupportEntries(runtime.mode_support, runtime.supported_modes),
      execution_module_support: normalizeNamedSupportEntries(
        runtime.execution_module_support,
        runtime.supported_execution_modules
      )
    },
    market_data: {
      ...marketData,
      exchange_support: normalizeNamedSupportEntries(
        marketData.exchange_support,
        marketData.supported_exchanges
      ),
      symbol_support: normalizeNamedSupportEntries(
        marketData.symbol_support,
        marketData.supported_symbols
      )
    },
    frontend: normalizeFrontendCapabilities(capabilities.frontend, knownModuleKeys),
    workspace: normalizeWorkspaceCapabilities(capabilities.workspace),
    ui_actions: normalizeUiActionCapabilities(capabilities.ui_actions),
    permission_boundary: normalizePermissionBoundary(capabilities.permission_boundary)
  };
}
