import {
  DECLARED_INDICATOR_KINDS,
  SUPPORTED_EXCHANGES,
  SUPPORTED_FRONTEND_MODULE_KEYS,
  SUPPORTED_INDICATOR_KINDS,
  SUPPORTED_RUNTIME_EXECUTION_MODULES,
  SUPPORTED_RUNTIME_MODES,
  SUPPORTED_SYMBOLS,
  WORKSPACE_SURFACE_MAP,
  CAPABILITY_ACTION_MAP
} from "./supportMatrix";

export const DEFAULT_CAPABILITIES = {
  api_version: "quantpilot-capabilities/v1",
  schema_version: "quantpilot/capabilities-schema/v1",
  schema_hash: "sha256:b4422abf545a8ef161cb800880b89ae76a56a6eec7f74289432491d19a03cd11",
  chain_stages: ["data", "intent", "agent", "risk", "execution", "fill"],
  strategy_ir: {
    declared_indicator_kinds: DECLARED_INDICATOR_KINDS,
    supported_indicator_kinds: SUPPORTED_INDICATOR_KINDS
  },
  runtime: {
    supported_modes: SUPPORTED_RUNTIME_MODES,
    supported_execution_modules: SUPPORTED_RUNTIME_EXECUTION_MODULES
  },
  market_data: {
    supported_exchanges: SUPPORTED_EXCHANGES,
    supported_symbols: SUPPORTED_SYMBOLS
  },
  frontend: {
    supported_module_keys: SUPPORTED_FRONTEND_MODULE_KEYS,
    unsupported_module_reasons: {}
  },
  workspace: {
    surfaces: Object.keys(WORKSPACE_SURFACE_MAP).map((key) => ({
      key,
      status: "supported",
      reason: null,
      source: "backend:/api/capabilities.workspace.surfaces"
    }))
  },
  ui_actions: {
    actions: Object.keys(CAPABILITY_ACTION_MAP).map((key) => ({
      key,
      status: "supported",
      reason: null,
      source: "backend:/api/capabilities.ui_actions.actions"
    }))
  },
  versioning: {
    model_version: "quantpilot/versioning-model/v1",
    strategy_version_source: "frontend_runtime_config.metadata.version",
    parameter_version_policy: "immutable_generation_pointer",
    deployment_revision_policy: "strategy_version_plus_compile_id_plus_capability_hash"
  },
  permission_boundary: {
    model_version: "quantpilot/permission-boundary/v1",
    execution_owner_module: "builtin.execution.paper",
    live_execution_allowed: false,
    ai_write_policy: "proposal_only",
    plugin_network_default: "deny",
    non_execution_order_access: "deny"
  }
};

export function createSafeFallbackCapabilities(reason = "能力清单加载失败，当前进入安全回退模式。") {
  return {
    api_version: DEFAULT_CAPABILITIES.api_version,
    schema_version: DEFAULT_CAPABILITIES.schema_version,
    schema_hash: "safe-fallback",
    chain_stages: [...DEFAULT_CAPABILITIES.chain_stages],
    strategy_ir: {
      declared_indicator_kinds: [...DEFAULT_CAPABILITIES.strategy_ir.declared_indicator_kinds],
      supported_indicator_kinds: [...DEFAULT_CAPABILITIES.strategy_ir.declared_indicator_kinds],
      indicator_support: DEFAULT_CAPABILITIES.strategy_ir.declared_indicator_kinds.map((kind) => ({
        kind,
        status: "declared_only",
        reason
      }))
    },
    runtime: {
      supported_modes: ["paper"],
      supported_execution_modules: ["builtin.execution.paper"],
      mode_support: [{ key: "paper", status: "declared_only", reason }],
      execution_module_support: [{ key: "builtin.execution.paper", status: "declared_only", reason }]
    },
    market_data: {
      supported_exchanges: ["binance", "okx"],
      supported_symbols: ["BTCUSDT", "ETHUSDT", "SOLUSDT"],
      exchange_support: ["binance", "okx"].map(e => ({ exchange: e, status: "declared_only", reason })),
      symbol_support: ["BTCUSDT", "ETHUSDT", "SOLUSDT"].map(s => ({ symbol: s, status: "declared_only", reason }))
    },
    frontend: {
      declared_module_keys: [...SUPPORTED_FRONTEND_MODULE_KEYS],
      supported_module_keys: [...SUPPORTED_FRONTEND_MODULE_KEYS],
      unsupported_module_reasons: {},
      module_support: SUPPORTED_FRONTEND_MODULE_KEYS.map((moduleKey) => ({
        module_key: moduleKey,
        status: "declared_only",
        reason
      }))
    },
    workspace: {
      surfaces: Object.keys(WORKSPACE_SURFACE_MAP).map((key) => ({
        key,
        status: "declared_only",
        reason,
        source: "safe_fallback"
      }))
    },
    ui_actions: {
      actions: Object.keys(CAPABILITY_ACTION_MAP).map((key) => ({
        key,
        status: "declared_only",
        reason,
        source: "safe_fallback"
      }))
    },
    versioning: { ...DEFAULT_CAPABILITIES.versioning },
    permission_boundary: {
      ...DEFAULT_CAPABILITIES.permission_boundary,
      live_execution_allowed: false,
      ai_write_policy: "disabled",
      plugin_network_default: "deny"
    }
  };
}
