import { sanitizeDisplayText } from "../utils/errorText";

function sanitizeText(value, fallback) {
  return sanitizeDisplayText(value, fallback);
}

function updateRuntimeNode(nodes, status, message = "") {
  return nodes.map((node) =>
    node.type === "runtime"
      ? {
          ...node,
          runtime_state: {
            ...node.runtime_state,
            status,
            last_message: message,
            last_event_time: Date.now()
          }
        }
      : node
  );
}

function eventStatus(event) {
  return event.payload?.exec_status || event.payload?.status || null;
}

function applyDataQualityMetrics(nextState, payload = {}) {
  nextState.metrics.latest_price = payload.latest_price ?? nextState.metrics.latest_price;
  nextState.metrics.latest_bar_time =
    payload.latest_bar_time ?? nextState.metrics.latest_bar_time;
  nextState.metrics.bid_price = payload.bid_price ?? nextState.metrics.bid_price;
  nextState.metrics.ask_price = payload.ask_price ?? nextState.metrics.ask_price;
  nextState.metrics.source_status = payload.source_status ?? nextState.metrics.source_status;
  nextState.metrics.source_health = payload.source_health ?? nextState.metrics.source_health;
  nextState.metrics.source_latency_ms =
    payload.source_latency_ms ?? nextState.metrics.source_latency_ms;
  nextState.metrics.freshness_ms = payload.freshness_ms ?? nextState.metrics.freshness_ms;
  nextState.metrics.stale_after_ms = payload.stale_after_ms ?? nextState.metrics.stale_after_ms;
  nextState.metrics.gap_count = payload.gap_count ?? nextState.metrics.gap_count;
  nextState.metrics.quality_flags = payload.quality_flags ?? nextState.metrics.quality_flags;
  nextState.metrics.endpoint = payload.endpoint ?? nextState.metrics.endpoint;
}

function applyRiskMetrics(nextState, payload = {}) {
  const postRisk = payload.post_risk && typeof payload.post_risk === "object" ? payload.post_risk : null;
  const preRisk = payload.pre_risk && typeof payload.pre_risk === "object" ? payload.pre_risk : null;
  const metricsSource = postRisk || preRisk || {};

  nextState.metrics.limit_triggered =
    payload.limit_triggered ?? nextState.metrics.limit_triggered;
  nextState.metrics.concentration_ratio =
    metricsSource.concentration_ratio ?? nextState.metrics.concentration_ratio;
  nextState.metrics.max_symbol_net_exposure_ratio =
    metricsSource.max_symbol_net_exposure_ratio ??
    nextState.metrics.max_symbol_net_exposure_ratio;
  nextState.metrics.portfolio_net_exposure_ratio =
    metricsSource.portfolio_net_exposure_ratio ??
    nextState.metrics.portfolio_net_exposure_ratio;
}

function applyRuntimeEventToNode(node, event) {
  const nextState = {
    ...node.runtime_state,
    status: event.severity === "Error" ? "error" : "running",
    last_event_type: event.event_type,
    last_event_time: event.event_time_ms,
    last_message: sanitizeText(event.summary, node.runtime_state?.last_message || ""),
    metrics: { ...node.runtime_state.metrics }
  };

  if (event.event_type === "DataUpdated") {
    applyDataQualityMetrics(nextState, event.payload);
  }
  if (
    (event.event_type === "RuntimeWarning" || event.event_type === "RuntimeError") &&
    (event.payload.source_health !== undefined ||
      event.payload.source_status !== undefined ||
      event.payload.freshness_ms !== undefined)
  ) {
    applyDataQualityMetrics(nextState, event.payload);
  }
  if (event.event_type === "IntentTriggered") {
    nextState.metrics.signal_direction = event.payload.side || event.payload.signal_direction;
    nextState.metrics.signal_strength = event.payload.strength || event.payload.signal_strength;
    nextState.metrics.confidence = event.payload.confidence;
  }
  if (event.event_type === "IntentEvaluated") {
    nextState.metrics.signal_strength = event.payload.strength ?? nextState.metrics.signal_strength;
    nextState.metrics.confidence = event.payload.confidence ?? nextState.metrics.confidence;
    nextState.metrics.intent_kind = event.payload.kind || nextState.metrics.intent_kind;
  }
  if (event.event_type === "AgentDecisionProduced") {
    nextState.metrics.decision_bias = event.payload.net_side || event.payload.decision_bias;
    nextState.metrics.score = event.payload.net_strength || event.payload.score;
  }
  if (event.event_type === "RiskDecisionProduced") {
    nextState.metrics.risk_action = event.payload.status || event.payload.risk_action;
    nextState.metrics.risk_score = event.payload.risk_score;
    applyRiskMetrics(nextState, event.payload);
  }
  if (event.event_type === "ExecutionFilled") {
    nextState.metrics.fill_side = event.payload.side || event.payload.fill_side;
    nextState.metrics.fill_qty = event.payload.qty || event.payload.fill_qty;
    nextState.metrics.fill_price = event.payload.price || event.payload.fill_price;
    nextState.metrics.exec_status = eventStatus(event);
    nextState.metrics.order_id = event.payload.order_id || null;
  }
  if (event.event_type === "ExecutionPlanned") {
    nextState.metrics.exec_status = eventStatus(event);
    nextState.metrics.remaining_qty = event.payload.remaining_qty ?? null;
    nextState.metrics.limit_price = event.payload.limit_price ?? null;
    nextState.metrics.order_id = event.payload.order_id || null;
    nextState.metrics.orders = event.payload.orders ?? nextState.metrics.orders;
    if (eventStatus(event) === "Open") {
      nextState.status = "waiting";
    }
  }
  if (event.severity === "Error") {
    nextState.error = sanitizeText(event.summary, "运行时错误。");
    nextState.status = "error";
  }

  return {
    ...node,
    runtime_state: nextState
  };
}

function resetNodeRuntimeState(node) {
  return {
    ...node,
    runtime_state: {
      status: "idle",
      last_event_type: null,
      last_event_time: null,
      last_message: "",
      metrics: {},
      error: null
    }
  };
}

function applyEventsToGraphNodes(nodes, events) {
  let nextNodes = nodes.map(resetNodeRuntimeState);
  for (const event of events) {
    nextNodes = nextNodes.map((node) => {
      if (node.id === event.node_id) return applyRuntimeEventToNode(node, event);
      if (node.type === "runtime") {
        return {
          ...node,
          runtime_state: {
            ...node.runtime_state,
            status: "completed",
            last_event_type: event.event_type,
            last_event_time: event.event_time_ms,
            last_message: sanitizeText(event.summary, node.runtime_state?.last_message || "")
          }
        };
      }
      return node;
    });
  }
  return nextNodes;
}

function withRuntimeBinding(graph, patch) {
  return {
    ...graph,
    metadata: {
      ...graph.metadata,
      runtime_binding: {
        ...graph.metadata.runtime_binding,
        ...patch
      }
    }
  };
}

function accountFromPortfolioPayload(payload, fallback = null) {
  if (!payload || typeof payload !== "object") return fallback;

  const cashBalance = typeof payload.cash_balance === "number" ? payload.cash_balance : null;
  if (cashBalance === null) return fallback;

  const totalNetNotional =
    typeof payload.total_net_notional === "number" ? payload.total_net_notional : 0;
  const openOrders = Array.isArray(fallback?.open_orders) ? fallback.open_orders : [];

  return {
    equity_estimate:
      typeof payload.equity_estimate === "number"
        ? payload.equity_estimate
        : cashBalance + totalNetNotional,
    cash_balance: cashBalance,
    available_cash_balance:
      typeof payload.available_cash_balance === "number"
        ? payload.available_cash_balance
        : fallback?.available_cash_balance ?? cashBalance,
    frozen_cash_balance:
      typeof payload.frozen_cash_balance === "number"
        ? payload.frozen_cash_balance
        : fallback?.frozen_cash_balance ?? 0,
    total_leverage:
      typeof payload.total_leverage === "number"
        ? payload.total_leverage
        : fallback?.total_leverage ?? 0,
    total_gross_notional:
      typeof payload.total_gross_notional === "number"
        ? payload.total_gross_notional
        : fallback?.total_gross_notional ?? 0,
    total_net_notional: totalNetNotional,
    positions:
      typeof payload.positions === "number" ? payload.positions : fallback?.positions ?? 0,
    open_order_count:
      typeof payload.open_order_count === "number"
        ? payload.open_order_count
        : fallback?.open_order_count ?? openOrders.length,
    open_orders: openOrders
  };
}

function closeController(controller) {
  controller?.stop?.();
  controller?.close?.();
}

function scheduleBackgroundTask(task) {
  if (typeof window === "undefined") {
    void task();
    return;
  }

  const runTask = () => {
    void task();
  };

  if (typeof window.requestIdleCallback === "function") {
    window.requestIdleCallback(runTask, { timeout: 800 });
    return;
  }

  window.setTimeout(runTask, 0);
}

function collectHighlightedNodeIds(events) {
  return [...new Set((events || []).map((event) => event.node_id).filter(Boolean))];
}

function resolveBacktestEvents(payload) {
  const artifactEvents = payload?.backtest_artifacts?.event_log?.events;
  return Array.isArray(artifactEvents) ? artifactEvents : [];
}

function normalizeExperimentVariant(variant) {
  return {
    variant_id: sanitizeText(variant?.variant_id, ""),
    backtest_id: sanitizeText(variant?.backtest_id, ""),
    created_at_ms: typeof variant?.created_at_ms === "number" ? variant.created_at_ms : 0,
    fee_bps: typeof variant?.fee_bps === "number" ? variant.fee_bps : 0,
    slippage_bps: typeof variant?.slippage_bps === "number" ? variant.slippage_bps : 0,
    latency_ms: typeof variant?.latency_ms === "number" ? variant.latency_ms : 0,
    summary:
      variant?.summary && typeof variant.summary === "object"
        ? variant.summary
        : {
            step_count: 0,
            trade_count: 0,
            total_return_ratio: 0,
            max_drawdown_ratio: 0,
            final_equity: 0,
            net_profit: 0,
            turnover_ratio: 0,
            average_trade_notional: 0,
            fee_drag_ratio: 0
          },
    execution_assumptions_tag:
      variant?.execution_assumptions_tag && typeof variant.execution_assumptions_tag === "object"
        ? variant.execution_assumptions_tag
        : null
  };
}

function normalizeExperimentDetail(detail) {
  if (!detail || typeof detail !== "object") return null;
  return {
    experiment_id: sanitizeText(detail?.experiment_id, ""),
    graph_id: sanitizeText(detail?.graph_id, ""),
    compile_id: sanitizeText(detail?.compile_id, ""),
    created_at_ms: typeof detail?.created_at_ms === "number" ? detail.created_at_ms : 0,
    definition:
      detail?.definition && typeof detail.definition === "object"
        ? {
            experiment_name: sanitizeText(detail.definition.experiment_name, ""),
            replay_source: sanitizeText(detail.definition.replay_source, "historical_replay"),
            base_execution_assumptions:
              detail.definition.base_execution_assumptions &&
              typeof detail.definition.base_execution_assumptions === "object"
                ? detail.definition.base_execution_assumptions
                : {},
            parameter_grid:
              detail.definition.parameter_grid && typeof detail.definition.parameter_grid === "object"
                ? detail.definition.parameter_grid
                : { fee_bps: [], slippage_bps: [], latency_ms: [] }
          }
        : {
            experiment_name: "",
            replay_source: "historical_replay",
            base_execution_assumptions: {},
            parameter_grid: { fee_bps: [], slippage_bps: [], latency_ms: [] }
          },
    variants: Array.isArray(detail?.variants)
      ? detail.variants.map(normalizeExperimentVariant).filter((variant) => variant.backtest_id)
      : []
  };
}

function normalizeExperimentList(entries) {
  if (!Array.isArray(entries)) return [];
  return entries
    .map((entry) => ({
      experiment_id: sanitizeText(entry?.experiment_id, ""),
      graph_id: sanitizeText(entry?.graph_id, ""),
      compile_id: sanitizeText(entry?.compile_id, ""),
      created_at_ms: typeof entry?.created_at_ms === "number" ? entry.created_at_ms : 0,
      experiment_name: sanitizeText(entry?.experiment_name, ""),
      replay_source: sanitizeText(entry?.replay_source, "historical_replay"),
      variant_count: typeof entry?.variant_count === "number" ? entry.variant_count : 0,
      sweep_axes: Array.isArray(entry?.sweep_axes)
        ? entry.sweep_axes.map((value) => sanitizeText(value, "")).filter(Boolean)
        : [],
      best_backtest_id: sanitizeText(entry?.best_backtest_id, ""),
      best_total_return_ratio:
        typeof entry?.best_total_return_ratio === "number" ? entry.best_total_return_ratio : null
    }))
    .filter((entry) => entry.experiment_id);
}

/**
 * v1.0.5 统一锁包装器
 * 用法: await withLock(set, get, "saving", () => saveGraphInternal())
 * 若锁已被占用则静默返回 undefined; finally 保证释放
 */
export async function withLock(set, get, lockName, fn) {
  if (get().actionLock) return;
  set({ actionLock: lockName });
  try {
    return await fn();
  } finally {
    set({ actionLock: null });
  }
}

export {
  accountFromPortfolioPayload,
  applyEventsToGraphNodes,
  applyRuntimeEventToNode,
  closeController,
  collectHighlightedNodeIds,
  eventStatus,
  normalizeExperimentDetail,
  normalizeExperimentList,
  resetNodeRuntimeState,
  resolveBacktestEvents,
  scheduleBackgroundTask,
  updateRuntimeNode,
  withRuntimeBinding
};
