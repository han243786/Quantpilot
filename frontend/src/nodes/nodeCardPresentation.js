function formatSourceHealthLabel(value) {
  if (!value) return "";
  const normalized = String(value).trim().toLowerCase();
  if (!normalized) return "";
  if (["healthy", "ok", "normal"].includes(normalized)) return "健康";
  if (["warning", "warn"].includes(normalized)) return "告警";
  if (normalized.includes("delay")) return "延迟";
  if (normalized.includes("stale")) return "过期";
  if (normalized.includes("missing")) return "缺失";
  if (normalized.includes("error")) return "错误";
  if (normalized.includes("degrad")) return "降级";
  return String(value);
}

function formatPercent(value) {
  if (!Number.isFinite(value)) return "";
  return `${(value * 100).toFixed(1)}%`;
}

function buildDataQualityBits(metrics = {}) {
  const bits = [];
  const health = formatSourceHealthLabel(metrics.source_health || metrics.source_status);

  if (health) bits.push(health);

  if (Number.isFinite(metrics.freshness_ms) && metrics.freshness_ms >= 0) {
    if (Number.isFinite(metrics.stale_after_ms) && metrics.stale_after_ms > 0) {
      bits.push(`新鲜 ${metrics.freshness_ms}/${metrics.stale_after_ms}ms`);
    } else {
      bits.push(`新鲜 ${metrics.freshness_ms}ms`);
    }
  }

  if (Number.isFinite(metrics.source_latency_ms) && metrics.source_latency_ms > 0) {
    bits.push(`延迟 ${metrics.source_latency_ms}ms`);
  }

  if (Number.isFinite(metrics.gap_count) && metrics.gap_count > 0) {
    bits.push(`缺口 ${metrics.gap_count}`);
  }

  return bits;
}

function buildRiskGuardBits(metrics = {}) {
  const bits = [];

  if (metrics.limit_triggered) {
    bits.push(`限制 ${metrics.limit_triggered}`);
  }
  if (Number.isFinite(metrics.concentration_ratio)) {
    bits.push(`集中度 ${formatPercent(metrics.concentration_ratio)}`);
  }
  if (Number.isFinite(metrics.max_symbol_net_exposure_ratio)) {
    bits.push(`单标的净敞口 ${formatPercent(metrics.max_symbol_net_exposure_ratio)}`);
  }
  if (Number.isFinite(metrics.portfolio_net_exposure_ratio)) {
    bits.push(`组合净敞口 ${formatPercent(metrics.portfolio_net_exposure_ratio)}`);
  }

  return bits;
}

export function formatNodeMetricLabel(node) {
  const metrics = node.runtime_state?.metrics || {};

  if (node.type === "data") {
    const qualityBits = buildDataQualityBits(metrics);

    if (metrics.latest_price) {
      const suffix = qualityBits.length > 0 ? ` · ${qualityBits.join(" · ")}` : "";
      return `价格 ${metrics.latest_price}${suffix}`;
    }

    if (qualityBits.length > 0) {
      return `数据 ${qualityBits.join(" · ")}`;
    }

    return "等待数据";
  }

  if (node.type === "intent") {
    return metrics.signal_direction
      ? `${metrics.signal_direction} ${metrics.signal_strength || ""}`.trim()
      : "等待信号";
  }

  if (node.type === "agent") {
    return metrics.decision_bias
      ? `${metrics.decision_bias} ${metrics.score || ""}`.trim()
      : "等待决策";
  }

  if (node.type === "risk") {
    const riskGuardBits = buildRiskGuardBits(metrics);
    if (metrics.risk_action) {
      const prefix = `${metrics.risk_action} ${metrics.risk_score || ""}`.trim();
      return riskGuardBits.length > 0 ? `${prefix} · ${riskGuardBits.join(" · ")}` : prefix;
    }
    if (riskGuardBits.length > 0) {
      return `风控 · ${riskGuardBits.join(" · ")}`;
    }
    return "等待裁决";
  }

  if (node.type === "execution") {
    return metrics.fill_side
      ? `${metrics.fill_side} ${metrics.fill_qty || ""}`.trim()
      : "等待执行";
  }

  return node.runtime_state?.last_message || "运行控制";
}

function buildQuickFieldDefinitions(moduleDef, node) {
  const fieldMap = new Map(
    (moduleDef?.config_schema?.fields || []).map((field) => [field.key, field])
  );

  return (moduleDef?.node?.quick_fields || [])
    .map((fieldKey) => {
      const field = fieldMap.get(fieldKey);
      if (!field) return null;

      return {
        key: field.key,
        label: field.label,
        type: field.type,
        options: field.options || [],
        value: node.config?.[field.key] ?? ""
      };
    })
    .filter(Boolean);
}

function buildSummaryValues(moduleDef, node, simplified) {
  const summaryFields = moduleDef?.node?.summary_fields || [];
  const limit = simplified ? 2 : summaryFields.length;

  return summaryFields
    .slice(0, limit)
    .map((fieldKey) => node.config?.[fieldKey])
    .filter((value) => value !== undefined && value !== null && value !== "")
    .map((value) => String(value));
}

export function buildNodeCardData({
  node,
  registry,
  nodeIssues,
  highlightedNodeIds,
  simplified,
  showHandles,
  focusMode = null,
  focusedNodeIds = new Set(),
  recommendedNodeIds = new Set(),
  repairPathNodeIds = [],
  selectedNodeId = null
}) {
  const moduleDef = registry.getByKey(node.module_key);
  const issues = nodeIssues?.[node.id] || [];
  const hasFocusedTargets = focusedNodeIds.size > 0;
  const isFocusTarget = focusedNodeIds.has(node.id);
  const repairPathIndex = repairPathNodeIds.indexOf(node.id);
  let recommendationRole = null;

  if (repairPathIndex >= 0) {
    if (repairPathIndex === repairPathNodeIds.length - 1 && node.id !== selectedNodeId) {
      recommendationRole = "path-end";
    } else if (repairPathIndex === 0 && node.id === selectedNodeId) {
      recommendationRole = "path-start";
    } else {
      recommendationRole = "path";
    }
  } else if (recommendedNodeIds.has(node.id)) {
    recommendationRole = "recommended";
  }

  return {
    node,
    nodeId: node.id,
    nodeType: node.type,
    runtimeStatus: node.runtime_state?.status || "idle",
    title: node.name || node.id,
    subtitle: moduleDef?.display_name || node.module_key || "未注册模块",
    inputPorts: node.input_ports || [],
    outputPorts: node.output_ports || [],
    highlighted: highlightedNodeIds.includes(node.id),
    simplified,
    handlesConnectable: showHandles,
    summaryValues: buildSummaryValues(moduleDef, node, simplified),
    quickFieldDefinitions: buildQuickFieldDefinitions(moduleDef, node),
    issueMessage: issues[0]?.message || null,
    metricLabel: formatNodeMetricLabel(node),
    collapsed: Boolean(node.ui_state?.collapsed),
    focusMode: hasFocusedTargets && isFocusTarget ? focusMode : null,
    dimmed: hasFocusedTargets && !isFocusTarget,
    recommendationRole
  };
}
