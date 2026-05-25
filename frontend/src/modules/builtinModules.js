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
} from "../capabilities/supportMatrix";
import { sanitizeDisplayText } from "../utils/errorText";

const exchangeOptions = [
  { label: "Binance", value: "binance" },
  { label: "OKX", value: "okx" }
];

function createSymbolOptions(symbols = SUPPORTED_SYMBOLS) {
  return symbols.map((symbol) => ({ label: symbol, value: symbol }));
}

const symbolOptions = createSymbolOptions();

const commonNode = (category, name, quickFields, summaryFields) => ({
  default_name: name,
  lane: category,
  color_token: category,
  quick_fields: quickFields,
  summary_fields: summaryFields
});

export const DEFAULT_CAPABILITIES = {
  api_version: "quantpilot-capabilities/v1",
  schema_version: "quantpilot/capabilities-schema/v1",
  schema_hash: "sha256:86c21d2a4193728bc3332b29910f1d9934ab71b710342698bb82e96fad478a45",
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

export const allBuiltinModules = [
  {
    module_key: "builtin.data.kline",
    version: "1.0.0",
    category: "data",
    display_name: "K 线数据",
    description: "提供标准化的历史 K 线市场数据。",
    node: commonNode(
      "data",
      "K 线数据",
      ["instrument", "timeframe"],
      ["exchange", "instrument", "timeframe", "window_size"]
    ),
    ports: {
      inputs: [],
      outputs: [
        { key: "market_data_out", label: "K 线输出", provides: "kline_series", multiple: true }
      ]
    },
    config_schema: {
      fields: [
        {
          key: "exchange",
          label: "交易所",
          type: "select",
          default: "binance",
          options: exchangeOptions,
          required: true
        },
        {
          key: "instrument",
          label: "交易对",
          type: "select",
          default: "BTCUSDT",
          options: symbolOptions,
          required: true
        },
        {
          key: "timeframe",
          label: "周期",
          type: "select",
          default: "1d",
          options: [
            { label: "1m", value: "1m" },
            { label: "5m", value: "5m" },
            { label: "1h", value: "1h" },
            { label: "1d", value: "1d" }
          ],
          required: true
        },
        {
          key: "window_size",
          label: "窗口大小",
          type: "number",
          default: 200,
          min: 10,
          max: 2000,
          required: true
        },
        {
          key: "ping_enabled",
          label: "Ping",
          type: "boolean",
          default: false,
          required: false
        },
        {
          key: "request_interval_ms",
          label: "Request interval (ms)",
          type: "number",
          default: 0,
          min: 0,
          max: 600000,
          required: false
        }
      ]
    }
  },
  {
    module_key: "builtin.data.quote",
    version: "1.0.0",
    category: "data",
    display_name: "报价快照",
    description: "提供实时最优买卖价快照。",
    node: commonNode("data", "报价快照", ["instrument"], ["exchange", "instrument"]),
    ports: {
      inputs: [],
      outputs: [
        { key: "market_data_out", label: "报价输出", provides: "quote_tick", multiple: true }
      ]
    },
    config_schema: {
      fields: [
        {
          key: "exchange",
          label: "交易所",
          type: "select",
          default: "binance",
          options: exchangeOptions,
          required: true
        },
        {
          key: "instrument",
          label: "交易对",
          type: "select",
          default: "BTCUSDT",
          options: symbolOptions,
          required: true
        },
        {
          key: "ping_enabled",
          label: "Ping",
          type: "boolean",
          default: false,
          required: false
        },
        {
          key: "request_interval_ms",
          label: "Request interval (ms)",
          type: "number",
          default: 0,
          min: 0,
          max: 600000,
          required: false
        }
      ]
    }
  },
  {
    module_key: "builtin.intent.double_ma",
    version: "1.0.0",
    category: "intent",
    display_name: "双均线意图",
    description: "根据均线交叉生成趋势意图。",
    node: commonNode(
      "intent",
      "双均线意图",
      ["fast_period", "slow_period"],
      ["fast_period", "slow_period", "entry_ratio"]
    ),
    ports: {
      inputs: [
        {
          key: "data_input",
          label: "数据输入",
          accepts: ["kline_series"],
          required: true,
          multiple: true
        }
      ],
      outputs: [
        { key: "intent_out", label: "意图输出", provides: "intent_signal", multiple: true }
      ]
    },
    config_schema: {
      fields: [
        {
          key: "fast_period",
          label: "快线周期",
          type: "number",
          default: 50,
          min: 1,
          max: 200,
          required: true
        },
        {
          key: "slow_period",
          label: "慢线周期",
          type: "number",
          default: 150,
          min: 2,
          max: 400,
          required: true
        },
        {
          key: "entry_ratio",
          label: "触发比例",
          type: "number",
          default: 0.8,
          min: 0.1,
          max: 5,
          required: true
        }
      ]
    },
    constraints: {
      node_rules: [
        {
          rule: "compare",
          left: "fast_period",
          operator: "<",
          right: "slow_period",
          message: "快线周期必须小于慢线周期。"
        }
      ]
    }
  },
  {
    module_key: "builtin.intent.ma_deviation",
    version: "1.0.0",
    category: "intent",
    display_name: "均线偏离意图",
    description: "观察价格相对均线的偏离程度。",
    node: commonNode(
      "intent",
      "均线偏离意图",
      ["lookback", "threshold_ratio"],
      ["lookback", "baseline_period", "threshold_ratio"]
    ),
    ports: {
      inputs: [
        {
          key: "data_input",
          label: "数据输入",
          accepts: ["kline_series"],
          required: true,
          multiple: true
        }
      ],
      outputs: [
        { key: "intent_out", label: "意图输出", provides: "intent_signal", multiple: true }
      ]
    },
    config_schema: {
      fields: [
        {
          key: "lookback",
          label: "回看周期",
          type: "number",
          default: 15,
          min: 2,
          max: 500,
          required: true
        },
        {
          key: "baseline_period",
          label: "基线周期",
          type: "number",
          default: 150,
          min: 2,
          max: 500,
          required: true
        },
        {
          key: "threshold_ratio",
          label: "阈值比例",
          type: "number",
          default: 1.4,
          min: 0.1,
          max: 10,
          required: true
        }
      ]
    }
  },
  {
    module_key: "builtin.intent.rsi",
    version: "1.0.0",
    category: "intent",
    display_name: "RSI 意图",
    description: "根据 RSI 超买超卖区间生成方向意图。",
    node: commonNode(
      "intent",
      "RSI 意图",
      ["period", "oversold_threshold"],
      ["period", "oversold_threshold", "overbought_threshold"]
    ),
    ports: {
      inputs: [
        {
          key: "data_input",
          label: "数据输入",
          accepts: ["kline_series"],
          required: true,
          multiple: true
        }
      ],
      outputs: [
        { key: "intent_out", label: "意图输出", provides: "intent_signal", multiple: true }
      ]
    },
    config_schema: {
      fields: [
        {
          key: "period",
          label: "RSI 周期",
          type: "number",
          default: 14,
          min: 2,
          max: 200,
          required: true
        },
        {
          key: "oversold_threshold",
          label: "超卖阈值",
          type: "number",
          default: 30,
          min: 1,
          max: 50,
          required: true
        },
        {
          key: "overbought_threshold",
          label: "超买阈值",
          type: "number",
          default: 70,
          min: 50,
          max: 99,
          required: true
        }
      ]
    },
    constraints: {
      node_rules: [
        {
          rule: "compare",
          left: "oversold_threshold",
          operator: "<",
          right: "overbought_threshold",
          message: "超卖阈值必须小于超买阈值。"
        }
      ]
    }
  },
  {
    module_key: "builtin.intent.macd",
    version: "1.0.0",
    category: "intent",
    display_name: "MACD 意图",
    description: "根据 MACD 柱线方向生成趋势意图。",
    node: commonNode(
      "intent",
      "MACD 意图",
      ["fast_period", "slow_period"],
      ["fast_period", "slow_period", "signal_period"]
    ),
    ports: {
      inputs: [
        {
          key: "data_input",
          label: "数据输入",
          accepts: ["kline_series"],
          required: true,
          multiple: true
        }
      ],
      outputs: [
        { key: "intent_out", label: "意图输出", provides: "intent_signal", multiple: true }
      ]
    },
    config_schema: {
      fields: [
        {
          key: "fast_period",
          label: "快线周期",
          type: "number",
          default: 12,
          min: 1,
          max: 100,
          required: true
        },
        {
          key: "slow_period",
          label: "慢线周期",
          type: "number",
          default: 26,
          min: 2,
          max: 200,
          required: true
        },
        {
          key: "signal_period",
          label: "信号线周期",
          type: "number",
          default: 9,
          min: 1,
          max: 100,
          required: true
        },
        {
          key: "histogram_threshold",
          label: "柱线阈值",
          type: "number",
          default: 0,
          min: 0,
          max: 10000,
          required: true
        }
      ]
    },
    constraints: {
      node_rules: [
        {
          rule: "compare",
          left: "fast_period",
          operator: "<",
          right: "slow_period",
          message: "快线周期必须小于慢线周期。"
        }
      ]
    }
  },
  {
    module_key: "builtin.intent.momentum",
    version: "1.0.0",
    category: "intent",
    display_name: "动量意图",
    description: "根据价格动量生成方向意图。",
    node: commonNode(
      "intent",
      "动量意图",
      ["lookback", "threshold_ratio"],
      ["lookback", "threshold_ratio"]
    ),
    ports: {
      inputs: [
        {
          key: "data_input",
          label: "数据输入",
          accepts: ["kline_series"],
          required: true,
          multiple: true
        }
      ],
      outputs: [
        { key: "intent_out", label: "意图输出", provides: "intent_signal", multiple: true }
      ]
    },
    config_schema: {
      fields: [
        {
          key: "lookback",
          label: "回看周期",
          type: "number",
          default: 10,
          min: 1,
          max: 200,
          required: true
        },
        {
          key: "threshold_ratio",
          label: "触发比例",
          type: "number",
          default: 0.02,
          min: 0,
          max: 5,
          required: true
        }
      ]
    }
  },
  {
    module_key: "builtin.intent.zscore",
    version: "1.0.0",
    category: "intent",
    display_name: "Z-Score 意图",
    description: "根据收盘价相对窗口均值的标准分生成均值回归意图。",
    node: commonNode("intent", "Z-Score 意图", ["window", "entry_z"], ["window", "entry_z"]),
    ports: {
      inputs: [
        {
          key: "data_input",
          label: "数据输入",
          accepts: ["kline_series"],
          required: true,
          multiple: true
        }
      ],
      outputs: [
        { key: "intent_out", label: "意图输出", provides: "intent_signal", multiple: true }
      ]
    },
    config_schema: {
      fields: [
        {
          key: "window",
          label: "窗口大小",
          type: "number",
          default: 20,
          min: 2,
          max: 500,
          required: true
        },
        {
          key: "entry_z",
          label: "入场阈值",
          type: "number",
          default: 2,
          min: 0.1,
          max: 10,
          required: true
        }
      ]
    }
  },
  {
    module_key: "builtin.intent.spread_observer",
    version: "1.0.0",
    category: "intent",
    display_name: "价差观察器",
    description: "观察跨交易所报价价差。",
    node: commonNode(
      "intent",
      "价差观察器",
      ["field_code", "window_size"],
      ["field_code", "resample_period_ms", "window_size", "spread_output_code"]
    ),
    ports: {
      inputs: [
        {
          key: "data_input",
          label: "数据输入",
          accepts: ["quote_tick", "kline_series"],
          required: true,
          multiple: true
        }
      ],
      outputs: [
        { key: "intent_out", label: "意图输出", provides: "intent_signal", multiple: true }
      ]
    },
    config_schema: {
      fields: [
        {
          key: "max_time_diff_ms",
          label: "最大时间偏差(ms)",
          type: "number",
          default: 5000,
          min: 0,
          max: 60000,
          required: true
        },
        {
          key: "field_code",
          label: "字段编码",
          type: "number",
          default: 0,
          min: 0,
          max: 7,
          required: true
        },
        {
          key: "align_direction_code",
          label: "对齐方向编码",
          type: "number",
          default: 0,
          min: 0,
          max: 2,
          required: true
        },
        {
          key: "resample_period_ms",
          label: "重采样周期(ms)",
          type: "number",
          default: 0,
          min: 0,
          max: 86400000,
          required: true
        },
        {
          key: "resample_agg_code",
          label: "重采样聚合编码",
          type: "number",
          default: 0,
          min: 0,
          max: 5,
          required: true
        },
        {
          key: "window_size",
          label: "窗口大小",
          type: "number",
          default: 1,
          min: 1,
          max: 512,
          required: true
        },
        {
          key: "window_agg_code",
          label: "窗口聚合编码",
          type: "number",
          default: 1,
          min: 0,
          max: 5,
          required: true
        },
        {
          key: "spread_output_code",
          label: "输出编码",
          type: "number",
          default: 0,
          min: 0,
          max: 2,
          required: true
        }
      ]
    }
  },
  {
    module_key: "builtin.agent.weighted",
    version: "1.0.0",
    category: "agent",
    display_name: "加权代理",
    description: "将多个意图合成为一个决策。",
    node: commonNode(
      "agent",
      "加权代理",
      ["decision_threshold", "rebalance_schedule"],
      [
        "decision_threshold",
        "max_quantity_ratio",
        "rebalance_schedule",
        "rebalance_allocation_kind"
      ]
    ),
    ports: {
      inputs: [
        {
          key: "intent_input",
          label: "意图输入",
          accepts: ["intent_signal"],
          required: true,
          multiple: true
        }
      ],
      outputs: [
        { key: "agent_out", label: "代理输出", provides: "agent_decision", multiple: true }
      ]
    },
    config_schema: {
      fields: [
        {
          key: "decision_threshold",
          label: "决策阈值",
          type: "number",
          default: 0.05,
          min: 0,
          max: 1,
          required: true
        },
        {
          key: "max_quantity_ratio",
          label: "最大仓位比例",
          type: "number",
          default: 0.8,
          min: 0.01,
          max: 1,
          required: true
        },
        {
          key: "rebalance_symbols",
          label: "Rebalance symbols",
          type: "text",
          default: "",
          required: false
        },
        {
          key: "rebalance_schedule",
          label: "Rebalance cadence",
          type: "select",
          default: "",
          options: [
            { label: "Disabled", value: "" },
            { label: "Every slow cycle", value: "every_slow" },
            { label: "Every 1d", value: "every_1d" },
            { label: "Weekly", value: "weekly" }
          ],
          required: false
        },
        {
          key: "rebalance_allocation_kind",
          label: "Allocation rule",
          type: "select",
          default: "",
          options: [
            { label: "Disabled", value: "" },
            { label: "Equal weight", value: "equal_weight" },
            { label: "Score weight", value: "score_weight" },
            { label: "Rank weight", value: "rank_weight" },
            { label: "Fixed weights", value: "fixed_weights" }
          ],
          required: false
        },
        {
          key: "rebalance_rank_method",
          label: "Rank method",
          type: "select",
          default: "",
          options: [
            { label: "Auto", value: "" },
            { label: "Linear", value: "linear" },
            { label: "Inverse rank", value: "inverse_rank" }
          ],
          required: false
        },
        {
          key: "rebalance_score_normalize",
          label: "Score normalize",
          type: "select",
          default: "",
          options: [
            { label: "Auto", value: "" },
            { label: "Sum", value: "sum" }
          ],
          required: false
        },
        {
          key: "rebalance_target_weights",
          label: "Target weights",
          type: "text",
          default: "",
          required: false
        }
      ]
    }
  },
  {
    module_key: "builtin.agent.arbitrage",
    version: "1.0.0",
    category: "agent",
    display_name: "套利代理",
    description: "根据价差意图生成套利决策。",
    node: commonNode(
      "agent",
      "套利代理",
      ["spread_trigger_bps"],
      ["spread_trigger_bps", "max_quantity_ratio"]
    ),
    ports: {
      inputs: [
        {
          key: "intent_input",
          label: "意图输入",
          accepts: ["intent_signal"],
          required: true,
          multiple: true
        }
      ],
      outputs: [
        { key: "agent_out", label: "代理输出", provides: "agent_decision", multiple: true }
      ]
    },
    config_schema: {
      fields: [
        {
          key: "spread_trigger_bps",
          label: "触发价差(bps)",
          type: "number",
          default: 50,
          min: 1,
          max: 5000,
          required: true
        },
        {
          key: "max_quantity_ratio",
          label: "最大仓位比例",
          type: "number",
          default: 0.5,
          min: 0.01,
          max: 1,
          required: true
        }
      ]
    }
  },
  {
    module_key: "builtin.risk.global",
    version: "1.0.0",
    category: "risk",
    display_name: "全局风控",
    description: "应用组合级别的风险限制。",
    node: commonNode(
      "risk",
      "全局风控",
      ["max_position", "max_portfolio_net_exposure"],
      [
        "max_position",
        "max_concentration",
        "max_symbol_net_exposure",
        "max_portfolio_net_exposure",
        "max_total_leverage"
      ]
    ),
    ports: {
      inputs: [
        {
          key: "agent_input",
          label: "代理输入",
          accepts: ["agent_decision"],
          required: true,
          multiple: true
        }
      ],
      outputs: [
        { key: "risk_out", label: "风控输出", provides: "risk_decision", multiple: true }
      ]
    },
    config_schema: {
      fields: [
        {
          key: "max_position",
          label: "最大仓位",
          type: "number",
          default: 0.2,
          min: 0.01,
          max: 1,
          required: true
        },
        {
          key: "max_concentration",
          label: "最大集中度",
          type: "number",
          min: 0.01,
          max: 1,
          required: false
        },
        {
          key: "max_symbol_net_exposure",
          label: "单标的净敞口上限",
          type: "number",
          min: 0.01,
          max: 1,
          required: false
        },
        {
          key: "max_portfolio_net_exposure",
          label: "组合净敞口上限",
          type: "number",
          min: 0.01,
          max: 1,
          required: false
        },
        {
          key: "max_turnover",
          label: "最大换手比",
          type: "number",
          min: 0,
          max: 5,
          required: false
        },
        {
          key: "min_trade_weight",
          label: "最小调仓权重",
          type: "number",
          min: 0,
          max: 1,
          required: false
        },
        {
          key: "max_new_positions_per_rebalance",
          label: "单次调仓最大新开仓数",
          type: "number",
          min: 0,
          max: 100,
          required: false
        },
        {
          key: "max_total_leverage",
          label: "最大总杠杆",
          type: "number",
          default: 3,
          min: 0.1,
          max: 20,
          required: true
        },
        {
          key: "max_exchange_leverage",
          label: "单交易所最大杠杆",
          type: "number",
          default: 3,
          min: 0.1,
          max: 20,
          required: true
        },
        {
          key: "min_action_interval_ms",
          label: "最小动作间隔(毫秒)",
          type: "number",
          default: 100,
          min: 0,
          max: 3600000,
          required: true
        }
      ]
    }
  },
  {
    module_key: "builtin.execution.paper",
    version: "1.0.0",
    category: "execution",
    display_name: "模拟执行",
    description: "模拟撮合执行模块。",
    node: commonNode("execution", "模拟执行", ["mode"], ["mode", "slippage_bps"]),
    ports: {
      inputs: [
        {
          key: "risk_input",
          label: "风控输入",
          accepts: ["risk_decision"],
          required: true,
          multiple: false
        }
      ],
      outputs: []
    },
    config_schema: {
      fields: [
        {
          key: "mode",
          label: "执行模式",
          type: "select",
          default: "paper",
          options: [{ label: "模拟", value: "paper" }],
          required: true
        },
        {
          key: "slippage_bps",
          label: "滑点(bps)",
          type: "number",
          default: 5,
          min: 0,
          max: 1000,
          required: true
        }
      ]
    }
  },
  {
    module_key: "builtin.runtime.control",
    version: "1.0.0",
    category: "runtime",
    display_name: "运行控制",
    description: "负责启动、停止和配置运行时。",
    node: commonNode("runtime", "运行控制", ["mode"], ["mode"]),
    ports: {
      inputs: [],
      outputs: []
    },
    config_schema: {
      fields: [
        {
          key: "mode",
          label: "运行模式",
          type: "select",
          default: "paper",
          options: [{ label: "模拟", value: "paper" }],
          required: true
        }
      ]
    }
  }
];

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

function normalizeFrontendCapabilities(frontendCapabilities = {}) {
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
            ...allBuiltinModules.map((moduleDef) => moduleDef.module_key),
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

export function normalizeCapabilities(capabilities) {
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
    frontend: normalizeFrontendCapabilities(capabilities.frontend),
    workspace: normalizeWorkspaceCapabilities(capabilities.workspace),
    ui_actions: normalizeUiActionCapabilities(capabilities.ui_actions),
    permission_boundary: normalizePermissionBoundary(capabilities.permission_boundary)
  };
}

export function applyCapabilitiesToModules(capabilities = DEFAULT_CAPABILITIES) {
  const normalized = normalizeCapabilities(capabilities);
  const supportedModuleKeys = new Set(normalized.frontend.supported_module_keys || []);
  const unsupportedReasons = normalized.frontend.unsupported_module_reasons || {};
  const moduleSupportMap = new Map(
    (normalized.frontend.module_support || []).map((entry) => [entry.module_key, entry])
  );
  const resolvedSymbolOptions = createSymbolOptions(
    normalized.market_data?.supported_symbols?.length
      ? normalized.market_data.supported_symbols
      : SUPPORTED_SYMBOLS
  );

  return allBuiltinModules.map((moduleDef) => {
    const supportEntry = moduleSupportMap.get(moduleDef.module_key);
    const supported = supportEntry
      ? supportEntry.status === "supported"
      : supportedModuleKeys.has(moduleDef.module_key);
    const fallbackReason =
      DEFAULT_CAPABILITIES.frontend.unsupported_module_reasons[moduleDef.module_key] ||
      "当前版本未开放该模块。";
    const reason = sanitizeDisplayText(
      supportEntry?.reason || unsupportedReasons[moduleDef.module_key],
      fallbackReason
    );

    return {
      ...moduleDef,
      config_schema: moduleDef.config_schema
        ? {
            ...moduleDef.config_schema,
            fields: (moduleDef.config_schema.fields || []).map((field) =>
              field.key === "instrument"
                ? {
                    ...field,
                    options: resolvedSymbolOptions,
                    default:
                      resolvedSymbolOptions.find((option) => option.value === field.default)?.value ||
                      resolvedSymbolOptions[0]?.value ||
                      field.default
                  }
                : field
            )
          }
        : moduleDef.config_schema,
      availability: supported
        ? { status: "supported", reason: "" }
        : { status: "unsupported", reason }
    };
  });
}

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

export const builtinModules = applyCapabilitiesToModules(DEFAULT_CAPABILITIES);
