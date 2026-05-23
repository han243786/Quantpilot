export const DECLARED_INDICATOR_KINDS = [
  "ma_cross",
  "rsi",
  "macd",
  "momentum",
  "spread",
  "z_score",
  "custom",
  "quote_observe",
  "atr",
  "bollinger_bands",
  "obv",
  "cmf",
  "adx",
  "stochastic",
  "cci",
  "parabolic_sar",
  "keltner_channel",
  "donchian_channel"
];

export const SUPPORTED_INDICATOR_KINDS = [...DECLARED_INDICATOR_KINDS];
export const SUPPORTED_RUNTIME_MODES = ["paper"];
export const SUPPORTED_RUNTIME_EXECUTION_MODULES = ["builtin.execution.paper", "live.okx"];
export const SUPPORTED_EXCHANGES = ["binance", "okx"];
export const SUPPORTED_SYMBOLS = ["BTCUSDT", "ETHUSDT", "SOLUSDT"];
export const SUPPORTED_FRONTEND_MODULE_KEYS = [
  "builtin.data.kline",
  "builtin.data.quote",
  "builtin.intent.double_ma",
  "builtin.intent.ma_deviation",
  "builtin.intent.rsi",
  "builtin.intent.macd",
  "builtin.intent.momentum",
  "builtin.intent.zscore",
  "builtin.intent.spread_observer",
  "builtin.agent.weighted",
  "builtin.agent.arbitrage",
  "builtin.risk.global",
  "builtin.execution.paper",
  "builtin.runtime.control"
];

export const WORKSPACE_SURFACE_MAP = {
  template_library: {
    label: "策略模板库",
    apiPaths: [],
    capabilityDriven: false,
    sourceOfTruth: "frontend_local_template_registry",
    notes: [
      "模板库是前端本地 starter graph 入口，不依赖 /api/capabilities 显隐。",
      "加载模板只替换当前内存工作草稿，不创建第二套后端模板传输。"
    ]
  },
  version_history: {
    label: "版本历史",
    apiPaths: [
      "/api/graphs/:graph_id/versions",
      "/api/graphs/:graph_id/versions/:version_id",
      "/api/graphs/:graph_id/versions/:version_id/restore",
      "/api/graphs/:graph_id/versions/compare"
    ],
    capabilityDriven: false,
    sourceOfTruth: "graph_persistence_routes",
    notes: [
      "版本历史属于图持久化工作流，不由 /api/capabilities 决定显隐。",
      "可见不代表扩展了新的 runtime capability，只代表当前图版本工件可管理。"
    ]
  },
  collaboration_audit: {
    label: "协作与审计",
    apiPaths: ["/api/graphs/:graph_id/audit"],
    capabilityDriven: false,
    sourceOfTruth: "graph_collaboration_metadata",
    notes: [
      "协作与审计属于当前图元数据和审计记录投影，不由 /api/capabilities 决定显隐。",
      "当前边界仍是本地 actor 协作切片，不应外推成远程账号系统能力。"
    ]
  },
  parameter_sweep: {
    label: "参数扫掠",
    apiPaths: [
      "/api/runtime/experiments/backtest-sweep",
      "/api/runtime/experiments",
      "/api/runtime/experiments/:experiment_id"
    ],
    capabilityDriven: true,
    sourceOfTruth: "runtime_backtest_surface",
    notes: [
      "参数扫掠是现有 backtest surface 上的窄执行假设扫描，不是第二套实验运行时。",
      "发起扫掠必须遵守与回测相同的 capability 同步和 safe-fallback 锁定规则。"
    ]
  }
};

export const CAPABILITY_ACTION_MAP = {
  export_runtime_config: {
    label: "导出运行配置",
    apiPaths: ["/api/runtime/compile"],
    blockedDuringCapabilitySync: true,
    notes: [
      "只在当前策略图编译通过后导出图生成的 runtime_config。",
      "当前端正在同步后端能力快照或进入安全回退模式时，该操作会被锁定。"
    ]
  },
  export_quantscript: {
    label: "导出策略图源码",
    apiPaths: [],
    blockedDuringCapabilitySync: false,
    notes: [
      "只导出当前 strategy_graph 草稿，不依赖后端能力门禁，也不会替代 formal QuantScript 编译链路。"
    ]
  },
  compile: {
    label: "编译",
    apiPaths: [
      "/api/strategy-ir/compile",
      "/api/quantscript/formal/compile",
      "/api/runtime/compile"
    ],
    blockedDuringCapabilitySync: true,
    notes: [
      "策略中间表示只承担语义预检。",
      "运行时编译仍然是可运行输出的最终真源。"
    ]
  },
  start_simulation: {
    label: "启动模拟",
    apiPaths: [
      "/api/runtime/test-run",
      "/api/runtime/runs/:run_id",
      "/api/runtime/runs/:run_id/save",
      "/api/runtime/runs/:run_id/events",
      "/api/runtime/runs/:run_id/status",
      "/api/runtime/mutations",
      "/api/runtime/mutations/:proposal_id",
      "/api/runtime/mutations/:proposal_id/activate",
      "/api/runtime/mutations/:proposal_id/rollback"
    ],
    blockedDuringCapabilitySync: true,
    notes: [
      "当前 Beta 边界内仅支持纸面模拟运行时。",
      "缓存回退模式下仍可见，但依旧受后端校验约束。"
    ]
  },
  run_backtest: {
    label: "运行回测",
    apiPaths: [
      "/api/runtime/backtest",
      "/api/runtime/backtests",
      "/api/runtime/backtests/:backtest_id",
      "/api/runtime/backtests/:backtest_id/save"
    ],
    blockedDuringCapabilitySync: true,
    notes: [
      "当前仅提供基础回放/回测支持，不宣称研究级回测能力。",
      "缓存回退模式下仍可见，但依旧受后端校验约束。"
    ]
  },
  run_parameter_sweep: {
    label: "运行参数扫掠",
    apiPaths: [
      "/api/runtime/experiments/backtest-sweep",
      "/api/runtime/experiments",
      "/api/runtime/experiments/:experiment_id",
      "/api/runtime/experiments/:experiment_id/save"
    ],
    blockedDuringCapabilitySync: true,
    notes: [
      "参数扫掠建立在现有回测能力边界之上，能力未同步或 safe fallback 时不得继续暴露为可执行入口。",
      "该入口只表示窄执行假设扫描，不表示通用优化器或第二套实验运行时。"
    ]
  }
};

export const SUPPORT_MATRIX = {
  runtime: {
    supportedModes: SUPPORTED_RUNTIME_MODES,
    supportedExecutionModules: SUPPORTED_RUNTIME_EXECUTION_MODULES,
    marketBoundary: {
      exchanges: SUPPORTED_EXCHANGES,
      symbols: SUPPORTED_SYMBOLS
    }
  },
  strategyIr: {
    declaredIndicatorKinds: DECLARED_INDICATOR_KINDS,
    supportedIndicatorKinds: SUPPORTED_INDICATOR_KINDS,
    boundaryNotes: [
      "Custom 仅限受约束的策略中间表示表达式路径，并会代码转换到核心中间表示。",
      "Custom 不允许任意宿主代码、直接修改风控或绕过执行链路。"
    ]
  },
  frontend: {
    supportedModuleKeys: SUPPORTED_FRONTEND_MODULE_KEYS,
    boundaryNotes: [
      "Beta 编译链路里可能出现价差和套利相关模块键，但不能对外宣称为真实套利平台能力。",
      "前端暴露的模块必须与 /api/capabilities 保持一致。"
    ]
  },
  workspace: {
    surfaces: WORKSPACE_SURFACE_MAP
  },
  compile: {
    preflightArtifact: "strategy_ir",
    runtimeSourceOfTruth: "/api/runtime/compile",
    boundaryNotes: [
      "策略中间表示仅承担语义预检。",
      "若存在 quantscript.formal_source，则由其执行运行时代码转换。",
      "当多个工件结论不一致时，运行时行为遵循 /api/runtime/compile 的输出。"
    ]
  },
  userFacingGuardrails: {
    allowedClaims: ["纸面运行时 Beta", "基础回测支持", "受限的 Custom 策略中间表示表达式路径"],
    disallowedClaims: [
      "宣称具备研究级回测能力",
      "宣称支持实盘交易",
      "宣称支持真实套利代理",
      "宣称支持第三方插件市场"
    ]
  },
  uiActionMap: CAPABILITY_ACTION_MAP
};

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

export function isCapabilitySyncBlocked(capabilityStatus, capabilitySource) {
  // v3.5.0: 缓存/降级模式仍允许模块操作, 仅完整阻断 loading 和 safe_fallback
  return (
    capabilityStatus === "loading" ||
    capabilitySource === "safe_fallback"
  );
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
  if (!action?.blockedDuringCapabilitySync) return "";

  if (capabilityStatus === "loading") {
    return `${action.label}暂时锁定，前端正在同步后端能力快照。`;
  }

  if (capabilitySource === "safe_fallback") {
    const detail =
      typeof capabilityMessage === "string" && capabilityMessage.trim().length > 0
        ? capabilityMessage.trim()
        : "能力校验失败，风险操作会在安全回退模式下继续保持锁定。";
    return `${action.label}在安全回退模式下不可用。${detail}`;
  }

  if (capabilityStatus === "degraded" || capabilitySource === "cache") {
    return `${action.label}暂时锁定，能力服务不可用，仅检测到缓存能力快照。`;
  }

  if (capabilityStatus === "error") {
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
