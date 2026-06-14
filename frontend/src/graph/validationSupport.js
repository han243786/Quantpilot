import { DEFAULT_CAPABILITIES, normalizeCapabilities } from "../modules/builtinModules";

export const allowedChain = {
  data: ["intent"],
  intent: ["agent"],
  agent: ["risk"],
  risk: ["execution"],
  execution: [],
  runtime: []
};

export const typeLabels = {
  data: "数据",
  intent: "意图",
  agent: "代理",
  risk: "风控",
  execution: "执行",
  runtime: "运行"
};

export function capabilitySet(values, fallback) {
  return new Set(Array.isArray(values) && values.length > 0 ? values : fallback);
}

export function supportMap(entries, keyField = "key") {
  return new Map(
    (Array.isArray(entries) ? entries : [])
      .filter((entry) => entry && typeof entry === "object" && entry[keyField])
      .map((entry) => [entry[keyField], entry])
  );
}

export function capabilityEntryStatus(entry, fallbackSet, key) {
  if (entry) return entry.status === "supported";
  return fallbackSet.has(key);
}

export function capabilityReason(entry, fallback = "") {
  return entry?.reason || fallback;
}

export function compareValues(leftValue, operator, rightValue) {
  if (operator === "<") return leftValue < rightValue;
  if (operator === "<=") return leftValue <= rightValue;
  if (operator === ">") return leftValue > rightValue;
  if (operator === ">=") return leftValue >= rightValue;
  if (operator === "===") return leftValue === rightValue;
  return true;
}

export function buildIssue(level, scope, targetId, code, message, hint = "") {
  return {
    id: `${scope}_${targetId}_${code}`,
    level,
    scope,
    target_id: targetId,
    code,
    message,
    hint
  };
}

export function buildCapabilityIndex(registry) {
  const capabilities = normalizeCapabilities(registry?.capabilities || DEFAULT_CAPABILITIES);
  const supportedRuntimeModes = capabilitySet(
    capabilities.runtime?.supported_modes,
    DEFAULT_CAPABILITIES.runtime.supported_modes
  );
  const supportedExecutionModules = capabilitySet(
    capabilities.runtime?.supported_execution_modules,
    DEFAULT_CAPABILITIES.runtime.supported_execution_modules
  );
  const supportedSymbols = capabilitySet(
    capabilities.market_data?.supported_symbols,
    DEFAULT_CAPABILITIES.market_data.supported_symbols
  );
  const supportedExchanges = capabilitySet(
    capabilities.market_data?.supported_exchanges,
    DEFAULT_CAPABILITIES.market_data.supported_exchanges
  );

  return {
    capabilities,
    supportedRuntimeModes,
    supportedExecutionModules,
    supportedSymbols,
    supportedExchanges,
    runtimeModeSupport: supportMap(capabilities.runtime?.mode_support),
    executionModuleSupport: supportMap(capabilities.runtime?.execution_module_support),
    exchangeSupport: supportMap(capabilities.market_data?.exchange_support),
    symbolSupport: supportMap(capabilities.market_data?.symbol_support),
    frontendModuleSupport: supportMap(capabilities.frontend?.module_support, "module_key")
  };
}
