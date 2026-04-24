const NODE_TYPE_FALLBACK_KEYS = {
  data: [
    "exchange",
    "instrument",
    "timeframe",
    "window_size",
    "ping_enabled",
    "request_interval_ms"
  ],
  intent: [
    "fast_period",
    "slow_period",
    "entry_ratio",
    "lookback",
    "baseline_period",
    "threshold_ratio"
  ],
  agent: [
    "rebalance_interval",
    "capital_fraction",
    "score_threshold",
    "target_weight",
    "max_positions"
  ],
  risk: [
    "stop_loss_ratio",
    "take_profit_ratio",
    "max_drawdown_ratio",
    "max_position_notional"
  ],
  execution: ["execution_mode", "order_type", "slippage_bps", "account", "venue"],
  runtime: ["mode", "initial_cash", "base_currency"]
};

const ISSUE_CODE_FIELD_KEYS = {
  UNSUPPORTED_EXCHANGE: ["exchange"],
  UNSUPPORTED_SYMBOL: ["instrument"],
  UNSUPPORTED_RUNTIME_MODE: ["mode"]
};

const CONFIG_ISSUE_CODES = new Set([
  "FIELD_REQUIRED",
  "FIELD_TYPE",
  "FIELD_MIN",
  "FIELD_MAX",
  "FIELD_COMPARE",
  "UNSUPPORTED_EXCHANGE",
  "UNSUPPORTED_SYMBOL",
  "UNSUPPORTED_RUNTIME_MODE",
  "UNSUPPORTED_EXECUTION",
  "UNSUPPORTED_MODULE"
]);

const CONNECTION_ISSUE_CODES = new Set([
  "INTENT_NO_INPUT",
  "INTENT_NO_OUTPUT",
  "AGENT_NO_INPUT",
  "AGENT_NO_OUTPUT",
  "RISK_NO_INPUT",
  "RISK_NO_OUTPUT",
  "EXECUTION_NO_INPUT",
  "SPREAD_DUPLICATE_SOURCE",
  "SPREAD_NON_DATA_INPUT",
  "ARBITRAGE_INPUT_KIND"
]);

function uniqueKeys(keys = []) {
  return keys.filter((key, index) => key && keys.indexOf(key) === index);
}

function orderFieldsByKeys(fields = [], orderedKeys = []) {
  if (orderedKeys.length === 0) return fields;

  const rank = new Map(orderedKeys.map((key, index) => [key, index]));
  return [...fields].sort((left, right) => {
    const leftRank = rank.has(left.key) ? rank.get(left.key) : Number.POSITIVE_INFINITY;
    const rightRank = rank.has(right.key) ? rank.get(right.key) : Number.POSITIVE_INFINITY;
    if (leftRank !== rightRank) {
      return leftRank - rightRank;
    }
    return 0;
  });
}

function issueFieldKeysFromMessage(issue, moduleDef) {
  const message = issue?.message || "";
  if (!message) return [];

  return (moduleDef?.config_schema?.fields || [])
    .filter((field) => message.includes(field.label))
    .map((field) => field.key);
}

function issuePriorityFieldKeys(moduleDef, nodeIssues = []) {
  const compareRuleKeys = (moduleDef?.constraints?.node_rules || [])
    .filter((rule) => rule.rule === "compare")
    .flatMap((rule) => [rule.left, rule.right]);

  return uniqueKeys(
    nodeIssues.flatMap((issue) => {
      const code = issue?.code || "";
      if (!code) return [];

      if (
        code === "FIELD_REQUIRED" ||
        code === "FIELD_TYPE" ||
        code === "FIELD_MIN" ||
        code === "FIELD_MAX"
      ) {
        return issueFieldKeysFromMessage(issue, moduleDef);
      }

      if (code === "FIELD_COMPARE") {
        return compareRuleKeys;
      }

      return ISSUE_CODE_FIELD_KEYS[code] || [];
    })
  );
}

function prioritySummary(issuePriorityKeys, nodeType) {
  if (issuePriorityKeys.length > 0) {
    return "与当前节点问题直接相关的字段会优先置顶，其后再显示该类型节点最关键的设置项。";
  }

  if (nodeType) {
    return "该节点类型在当前修复路径上的高优先级设置会优先置顶。";
  }

  return "这些设置是调整当前修复路径的最快入口。";
}

export function derivePriorityFieldGroups({
  moduleDef,
  nodeIssues = [],
  nodeType = null,
  prioritizePathFields = false
}) {
  const allFields = moduleDef?.config_schema?.fields || [];
  if (allFields.length === 0) return [];

  if (!prioritizePathFields) {
    return [
      {
        id: "all",
        title: "全部设置",
        summary: "在一个位置统一调整节点的完整配置。",
        fields: allFields
      }
    ];
  }

  const issuePriorityKeys = issuePriorityFieldKeys(moduleDef, nodeIssues);
  const modulePriorityKeys = [
    ...(moduleDef?.node?.quick_fields || []),
    ...(moduleDef?.node?.summary_fields || [])
  ];
  const typePriorityKeys = NODE_TYPE_FALLBACK_KEYS[nodeType] || [];
  const prioritizedKeys = uniqueKeys([
    ...issuePriorityKeys,
    ...modulePriorityKeys,
    ...typePriorityKeys
  ]);
  const prioritizedSet = new Set(prioritizedKeys);
  const prioritizedFields = orderFieldsByKeys(
    allFields.filter((field) => prioritizedSet.has(field.key)),
    prioritizedKeys
  );
  const remainingFields = allFields.filter((field) => !prioritizedSet.has(field.key));
  const groups = [];

  if (prioritizedFields.length > 0) {
    groups.push({
      id: "priority",
      title: "优先字段",
      summary: prioritySummary(issuePriorityKeys, nodeType),
      fields: prioritizedFields
    });
  }

  if (remainingFields.length > 0) {
    groups.push({
      id: "remaining",
      title: "更多设置",
      summary: "其余配置仍然保留在这里，避免把顶层操作区挤得过满。",
      fields: remainingFields
    });
  }

  return groups;
}

export function deriveConfigureCardOrder({
  nodeIssues = [],
  prioritizePathFields = false
}) {
  const defaultOrder = ["config", "connections", "validation"];
  if (!prioritizePathFields || nodeIssues.length === 0) {
    return defaultOrder;
  }

  const counts = nodeIssues.reduce(
    (summary, issue) => {
      const code = issue?.code || "";
      if (CONFIG_ISSUE_CODES.has(code)) {
        summary.config += 1;
      } else if (CONNECTION_ISSUE_CODES.has(code)) {
        summary.connections += 1;
      } else {
        summary.validation += 1;
      }
      return summary;
    },
    { config: 0, connections: 0, validation: 0 }
  );

  if (counts.connections > counts.config && counts.connections >= counts.validation) {
    return ["connections", "validation", "config"];
  }

  if (counts.config > counts.connections && counts.config >= counts.validation) {
    return ["config", "validation", "connections"];
  }

  if (counts.validation > 0) {
    return ["validation", "config", "connections"];
  }

  return defaultOrder;
}

export function resolveConfigureIssueTargetCard(issue) {
  const code = issue?.code || "";
  if (CONFIG_ISSUE_CODES.has(code)) {
    return "config";
  }
  if (CONNECTION_ISSUE_CODES.has(code)) {
    return "connections";
  }
  return "validation";
}
