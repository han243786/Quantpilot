const EVENT_LABELS = {
  DataUpdated: "数据更新",
  IntentTriggered: "意图触发",
  IntentEvaluated: "意图评估",
  AgentDecisionProduced: "代理决策",
  RiskDecisionProduced: "风控决策",
  ExecutionPlanned: "执行计划",
  ExecutionFilled: "执行成交",
  PortfolioUpdated: "组合更新",
  RuntimeNotice: "运行提示",
  RuntimeWarning: "数据告警",
  RuntimeError: "数据错误"
};

const INPUT_FIELDS_BY_EVENT = {
  DataUpdated: [
    "latest_price",
    "latest_bar_time",
    "source_status",
    "source_health",
    "source_latency_ms",
    "freshness_ms",
    "stale_after_ms",
    "gap_count"
  ],
  IntentTriggered: ["signal_direction", "signal_strength", "confidence"],
  IntentEvaluated: ["kind", "strength", "confidence"],
  AgentDecisionProduced: ["net_side", "net_strength", "score"],
  RiskDecisionProduced: ["status", "risk_score"],
  ExecutionPlanned: ["side", "qty", "limit_price", "remaining_qty"],
  ExecutionFilled: ["side", "qty", "price", "exec_status"],
  RuntimeWarning: [
    "source_health",
    "source_status",
    "freshness_ms",
    "source_latency_ms",
    "gap_count",
    "quality_flags"
  ],
  RuntimeError: [
    "source_health",
    "source_status",
    "freshness_ms",
    "source_latency_ms",
    "gap_count",
    "quality_flags"
  ]
};

const OUTPUT_FIELDS_BY_EVENT = {
  DataUpdated: ["latest_price", "source_health", "source_status", "gap_count"],
  IntentTriggered: ["signal_direction", "signal_strength"],
  IntentEvaluated: ["kind", "strength"],
  AgentDecisionProduced: ["net_side", "score"],
  RiskDecisionProduced: ["status", "risk_score"],
  ExecutionPlanned: ["exec_status", "order_id", "remaining_qty"],
  ExecutionFilled: ["exec_status", "order_id", "price", "qty"],
  RuntimeWarning: ["source_health", "source_status", "freshness_ms", "gap_count"],
  RuntimeError: ["source_health", "source_status", "freshness_ms", "gap_count"]
};

const FIELD_LABELS = {
  ask_price: "卖一价",
  bid_price: "买一价",
  confidence: "置信度",
  endpoint: "数据端点",
  error: "错误",
  exec_status: "执行状态",
  freshness_ms: "新鲜度 (ms)",
  gap_count: "缺口数量",
  kind: "意图类型",
  latest_bar_time: "最新 K 线时间",
  latest_price: "最新价格",
  limit_price: "限价",
  limit_triggered: "触发限制",
  lifecycle_stage: "生命周期",
  net_side: "决策方向",
  net_strength: "决策强度",
  order_count: "订单数量",
  order_id: "订单 ID",
  order_type: "订单类型",
  order_type_decision_reason: "下单语义",
  ping_error: "Ping 错误",
  ping_latency_ms: "Ping 延迟 (ms)",
  price: "成交价格",
  qty: "数量",
  quality_flags: "质量标记",
  reason_text: "原因",
  remaining_qty: "剩余数量",
  risk_score: "风控评分",
  score: "评分",
  side: "方向",
  signal_direction: "信号方向",
  signal_strength: "信号强度",
  sizing_mode: "定量模式",
  sizing_source: "定量来源",
  source_health: "源健康度",
  source_latency_ms: "源延迟 (ms)",
  source_status: "源状态",
  stale_after_ms: "过期阈值 (ms)",
  status: "状态",
  strength: "强度",
  time_in_force: "有效期",
  fallback: "回退路径"
};

function unique(values = []) {
  return [...new Set(values.filter(Boolean))];
}

function formatValue(value) {
  if (value === null || value === undefined || value === "") return "-";
  if (typeof value === "number") {
    return Number.isInteger(value) ? String(value) : value.toFixed(4);
  }
  if (typeof value === "boolean") {
    return value ? "Yes" : "No";
  }
  if (Array.isArray(value)) {
    return value.map((item) => formatValue(item)).join(", ");
  }
  return String(value);
}

function formatEventTime(value) {
  if (!value) return "-";
  const time = new Date(value);
  return Number.isNaN(time.getTime()) ? "-" : time.toLocaleTimeString();
}

function sortEvents(events = []) {
  return [...events].sort((left, right) => {
    const leftTime = Number(left?.event_time_ms) || 0;
    const rightTime = Number(right?.event_time_ms) || 0;
    return rightTime - leftTime;
  });
}

function buildPayloadRows(payload = {}, keys = []) {
  const preferredRows = keys
    .filter((key) => payload[key] !== undefined && payload[key] !== null && payload[key] !== "")
    .map((key) => ({
      key,
      label: FIELD_LABELS[key] || key,
      value: formatValue(payload[key])
    }));

  if (preferredRows.length > 0) return preferredRows;

  return Object.entries(payload)
    .filter(([, value]) => value !== undefined && value !== null && value !== "")
    .slice(0, 4)
    .map(([key, value]) => ({
      key,
      label: FIELD_LABELS[key] || key,
      value: formatValue(value)
    }));
}

function buildNestedRow(payload = {}, parentKey, childKey, label) {
  const parent = payload?.[parentKey];
  if (!parent || typeof parent !== "object") return null;
  const value = parent[childKey];
  if (value === undefined || value === null || value === "") return null;
  return {
    key: `${parentKey}.${childKey}`,
    label,
    value: formatValue(value)
  };
}

function buildExplanationRowsFromPayload(eventType, payload = {}) {
  const rows = [];

  if (payload.explanation_summary) {
    rows.push({
      key: "explanation_summary",
      label: "解释摘要",
      value: formatValue(payload.explanation_summary)
    });
  }

  if (eventType === "RiskDecisionProduced") {
    for (const key of ["reason_text", "limit_triggered", "sizing_mode"]) {
      if (payload[key] !== undefined && payload[key] !== null && payload[key] !== "") {
        rows.push({
          key,
          label: FIELD_LABELS[key] || key,
          value: formatValue(payload[key])
        });
      }
    }
  }

  if (eventType === "ExecutionPlanned" || eventType === "ExecutionFilled") {
    for (const key of [
      "reason_text",
      "lifecycle_stage",
      "sizing_source",
      "order_type_decision_reason",
      "time_in_force"
    ]) {
      if (payload[key] !== undefined && payload[key] !== null && payload[key] !== "") {
        rows.push({
          key,
          label: FIELD_LABELS[key] || key,
          value: formatValue(payload[key])
        });
      }
    }
  }

  if (eventType === "DataUpdated" || eventType === "RuntimeWarning" || eventType === "RuntimeError") {
    for (const key of ["source_health", "quality_flags", "fallback", "error"]) {
      if (payload[key] !== undefined && payload[key] !== null && payload[key] !== "") {
        rows.push({
          key,
          label: FIELD_LABELS[key] || key,
          value: formatValue(payload[key])
        });
      }
    }
  }

  return rows;
}

function buildDataQualityRowsFromPayload(payload = {}) {
  const rows = [];
  for (const key of [
    "source_health",
    "source_status",
    "freshness_ms",
    "stale_after_ms",
    "source_latency_ms",
    "ping_latency_ms",
    "gap_count",
    "quality_flags",
    "fallback",
    "error",
    "ping_error"
  ]) {
    if (payload[key] !== undefined && payload[key] !== null && payload[key] !== "") {
      rows.push({
        key,
        label: FIELD_LABELS[key] || key,
        value: formatValue(payload[key])
      });
    }
  }
  return rows;
}

function buildRiskDetailRowsFromPayload(payload = {}) {
  const rows = [];
  for (const key of ["status", "limit_triggered", "sizing_mode", "reason_text"]) {
    if (payload[key] !== undefined && payload[key] !== null && payload[key] !== "") {
      rows.push({
        key,
        label: FIELD_LABELS[key] || key,
        value: formatValue(payload[key])
      });
    }
  }

  for (const candidate of [
    buildNestedRow(payload, "pre_risk", "concentration_ratio", "风控前集中度"),
    buildNestedRow(payload, "post_risk", "concentration_ratio", "风控后集中度"),
    buildNestedRow(
      payload,
      "pre_risk",
      "max_symbol_net_exposure_ratio",
      "风控前单标的净敞口"
    ),
    buildNestedRow(
      payload,
      "post_risk",
      "max_symbol_net_exposure_ratio",
      "风控后单标的净敞口"
    ),
    buildNestedRow(
      payload,
      "pre_risk",
      "portfolio_net_exposure_ratio",
      "风控前组合净敞口"
    ),
    buildNestedRow(
      payload,
      "post_risk",
      "portfolio_net_exposure_ratio",
      "风控后组合净敞口"
    ),
    buildNestedRow(payload, "pre_risk", "max_target_weight", "风控前最大目标权重"),
    buildNestedRow(payload, "post_risk", "max_target_weight", "风控后最大目标权重"),
    buildNestedRow(payload, "pre_risk", "turnover_ratio", "风控前换手比"),
    buildNestedRow(payload, "post_risk", "turnover_ratio", "风控后换手比"),
    buildNestedRow(payload, "pre_risk", "basket_members", "风控前持仓数"),
    buildNestedRow(payload, "post_risk", "basket_members", "风控后持仓数"),
    buildNestedRow(payload, "pre_risk", "action_count", "风控前动作数"),
    buildNestedRow(payload, "post_risk", "action_count", "风控后动作数")
  ]) {
    if (candidate) rows.push(candidate);
  }

  return rows;
}

function buildOrderDetailRowsFromPayload(eventType, payload = {}) {
  const rows = [];
  for (const key of [
    "order_id",
    "side",
    "qty",
    "remaining_qty",
    "limit_price",
    "exec_status",
    "lifecycle_stage",
    "sizing_source",
    "order_type_decision_reason",
    "time_in_force",
    "reason_text"
  ]) {
    if (payload[key] !== undefined && payload[key] !== null && payload[key] !== "") {
      rows.push({
        key,
        label: FIELD_LABELS[key] || key,
        value: formatValue(payload[key])
      });
    }
  }

  if (eventType === "ExecutionPlanned" && Array.isArray(payload.order_previews)) {
    rows.push({
      key: "order_count",
      label: FIELD_LABELS.order_count,
      value: String(payload.order_previews.length)
    });
    const first = payload.order_previews[0];
    if (first) {
      if (first.side !== undefined) {
        rows.push({
          key: "preview_side",
          label: "首个订单方向",
          value: formatValue(first.side)
        });
      }
      if (first.qty !== undefined) {
        rows.push({
          key: "preview_qty",
          label: "首个订单数量",
          value: formatValue(first.qty)
        });
      }
      if (first.order_type !== undefined) {
        rows.push({
          key: "preview_order_type",
          label: "首个订单类型",
          value: formatValue(first.order_type)
        });
      }
      if (first.order_type_decision_reason !== undefined) {
        rows.push({
          key: "preview_order_type_decision_reason",
          label: "首个订单下单语义",
          value: formatValue(first.order_type_decision_reason)
        });
      }
    }
  }

  return rows;
}

function eventLabel(eventType) {
  return EVENT_LABELS[eventType] || eventType || "运行事件";
}

function severityTone(severity) {
  if (severity === "Error" || severity === "error") return "danger";
  if (severity === "Warn" || severity === "Warning" || severity === "warning") return "warning";
  return "info";
}

function buildStructuredProjection(graph, runtime, selectedNodeId = null) {
  const diagnostics = runtime?.diagnostics;
  if (!diagnostics || !diagnostics.node_details) return null;

  const nodes = Array.isArray(graph?.nodes) ? graph.nodes : [];
  const nodeMap = new Map(nodes.map((node) => [node.id, node]));
  const detailIds = Object.keys(diagnostics.node_details || {});
  const activeNodes = (diagnostics.active_nodes || [])
    .map((node) => {
      const resolvedNode = nodeMap.get(node.node_id);
      return {
        nodeId: node.node_id,
        nodeName: resolvedNode?.name || node.node_id,
        nodeType: resolvedNode?.type || "node",
        status: resolvedNode?.runtime_state?.status || "idle",
        latestEventLabel: node.latest_event_label || eventLabel(node.latest_event_type),
        latestEventTimeLabel: formatEventTime(node.latest_event_time_ms),
        eventCount: node.event_count || 0
      };
    })
    .filter((node) => detailIds.includes(node.nodeId));

  const activeNodeIds = unique([
    selectedNodeId,
    diagnostics.default_selected_node_id,
    ...activeNodes.map((node) => node.nodeId),
    ...detailIds
  ]);

  const resolvedNodeId =
    (selectedNodeId && activeNodeIds.includes(selectedNodeId) && selectedNodeId) ||
    activeNodeIds[0] ||
    null;
  const selectedNode = resolvedNodeId ? nodeMap.get(resolvedNodeId) || null : null;
  const selectedDetail = resolvedNodeId ? diagnostics.node_details[resolvedNodeId] || null : null;

  if (!selectedDetail) return null;

  return {
    activeNodes,
    selectedNode,
    selectedNodeId: resolvedNodeId,
    latestEvent: selectedDetail.latest_event || null,
    explanationSummary: selectedDetail.explanation_summary || null,
    latestInputRows: selectedDetail.latest_input_rows || [],
    latestOutputRows: selectedDetail.latest_output_rows || [],
    explanationRows: selectedDetail.explanation_rows || [],
    dataQualityRows: selectedDetail.data_quality_rows || [],
    riskDetailRows: selectedDetail.risk_detail_rows || [],
    orderDetailRows: selectedDetail.order_detail_rows || [],
    latestNotice: selectedDetail.latest_notice
      ? {
          ...selectedDetail.latest_notice,
          timeLabel: formatEventTime(selectedDetail.latest_notice.event_time_ms)
        }
      : null,
    recentEvents: (selectedDetail.recent_events || []).map((event) => ({
      eventId: event.event_id,
      label: event.label || eventLabel(event.event_type),
      summary: event.summary || "运行事件",
      timeLabel: formatEventTime(event.event_time_ms),
      tone: event.tone || severityTone(event.severity)
    })),
    eventCount: selectedDetail.event_count || 0
  };
}

function buildEventProjection(graph, runtime, selectedNodeId = null) {
  const nodes = Array.isArray(graph?.nodes) ? graph.nodes : [];
  const nodeMap = new Map(nodes.map((node) => [node.id, node]));
  const events = sortEvents(runtime?.events || []);
  const activeNodeIds = unique([
    selectedNodeId,
    ...(runtime?.highlightedNodeIds || []),
    ...events.map((event) => event.node_id)
  ]).filter((nodeId) => nodeMap.has(nodeId));

  const resolvedNodeId =
    (selectedNodeId && activeNodeIds.includes(selectedNodeId) && selectedNodeId) ||
    activeNodeIds[0] ||
    null;
  const selectedNode = resolvedNodeId ? nodeMap.get(resolvedNodeId) || null : null;
  const nodeEvents = resolvedNodeId ? events.filter((event) => event.node_id === resolvedNodeId) : [];
  const latestEvent = nodeEvents[0] || null;
  const latestDataEvent =
    nodeEvents.find(
      (event) =>
        event?.payload?.source_health !== undefined || event?.payload?.source_status !== undefined
    ) || null;
  const latestRiskEvent =
    nodeEvents.find((event) => event?.event_type === "RiskDecisionProduced") || null;
  const latestOrderEvent =
    nodeEvents.find(
      (event) =>
        event?.event_type === "ExecutionPlanned" || event?.event_type === "ExecutionFilled"
    ) || null;
  const latestNotice =
    nodeEvents.find((event) => severityTone(event.severity) !== "info") ||
    (selectedNode?.runtime_state?.error
      ? {
          severity: "Error",
          summary: selectedNode.runtime_state.error,
          event_type: selectedNode.runtime_state.last_event_type || "RuntimeNotice",
          event_time_ms: selectedNode.runtime_state.last_event_time || null
        }
      : null);

  return {
    activeNodes: activeNodeIds.map((nodeId) => {
      const node = nodeMap.get(nodeId);
      const latestNodeEvent = events.find((event) => event.node_id === nodeId) || null;
      const nodeEventCount = events.filter((event) => event.node_id === nodeId).length;
      return {
        nodeId,
        nodeName: node?.name || nodeId,
        nodeType: node?.type || "node",
        status: node?.runtime_state?.status || "idle",
        latestEventLabel: eventLabel(latestNodeEvent?.event_type),
        latestEventTimeLabel: formatEventTime(latestNodeEvent?.event_time_ms),
        eventCount: nodeEventCount
      };
    }),
    selectedNode,
    selectedNodeId: resolvedNodeId,
    latestEvent,
    explanationSummary:
      latestEvent?.payload?.explanation_summary || latestEvent?.payload?.reason_text || null,
    latestInputRows: buildPayloadRows(
      latestEvent?.payload || {},
      INPUT_FIELDS_BY_EVENT[latestEvent?.event_type] || []
    ),
    latestOutputRows: buildPayloadRows(
      latestEvent?.payload || {},
      OUTPUT_FIELDS_BY_EVENT[latestEvent?.event_type] || []
    ),
    explanationRows: buildExplanationRowsFromPayload(latestEvent?.event_type, latestEvent?.payload || {}),
    dataQualityRows: buildDataQualityRowsFromPayload(latestDataEvent?.payload || {}),
    riskDetailRows:
      latestRiskEvent?.event_type === "RiskDecisionProduced"
        ? buildRiskDetailRowsFromPayload(latestRiskEvent?.payload || {})
        : [],
    orderDetailRows:
      latestOrderEvent?.event_type === "ExecutionPlanned" ||
      latestOrderEvent?.event_type === "ExecutionFilled"
        ? buildOrderDetailRowsFromPayload(
            latestOrderEvent?.event_type,
            latestOrderEvent?.payload || {}
          )
        : [],
    latestNotice: latestNotice
      ? {
          tone: severityTone(latestNotice.severity),
          label: eventLabel(latestNotice.event_type),
          summary: latestNotice.summary || "运行提示",
          timeLabel: formatEventTime(latestNotice.event_time_ms)
        }
      : null,
    recentEvents: nodeEvents.slice(0, 5).map((event) => ({
      eventId: event.event_id,
      label: eventLabel(event.event_type),
      summary: event.summary || "运行事件",
      timeLabel: formatEventTime(event.event_time_ms),
      tone: severityTone(event.severity)
    })),
    eventCount: nodeEvents.length
  };
}

export function buildRuntimeDiagnosticsProjection(graph, runtime, selectedNodeId = null) {
  if (runtime?.diagnostics?.node_details) {
    return buildStructuredProjection(graph, runtime, selectedNodeId);
  }
  return buildEventProjection(graph, runtime, selectedNodeId);
}
