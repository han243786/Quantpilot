const MODE_LABELS = {
  paper_actual: "PaperActual",
  paper_simulated: "PaperSimulated",
  live_actual: "LiveActual",
  live_simulated: "LiveSimulated",
  PaperActual: "PaperActual",
  PaperSimulated: "PaperSimulated",
  LiveActual: "LiveActual",
  LiveSimulated: "LiveSimulated"
};

const STATUS_LABELS = {
  accepted: "accepted",
  unsupported: "unsupported",
  not_declared: "not_declared",
  mode_rejected: "mode_rejected",
  policy_missing: "policy_missing"
};

function asObject(value) {
  return value && typeof value === "object" && !Array.isArray(value) ? value : {};
}

function compactValue(value, fallback = "-") {
  if (value === null || value === undefined || value === "") return fallback;
  return String(value);
}

function isV4RuntimeMemorySnapshot(value) {
  return Boolean(
    value &&
      typeof value === "object" &&
      Array.isArray(value.machines) &&
      value.risk_plane &&
      value.execution
  );
}

export function resolveV4RuntimeMemorySnapshot(source = {}) {
  const candidates = [
    source?.memory_snapshot,
    source?.v4_memory_snapshot,
    source?.v4RuntimeMemorySnapshot,
    source?.v4_runtime?.memory_snapshot,
    source?.run_output?.memory_snapshot,
    source?.output?.memory_snapshot,
    source
  ];
  return candidates.find(isV4RuntimeMemorySnapshot) || null;
}

function runtimeModeLabel(mode) {
  return MODE_LABELS[mode] || compactValue(mode, "unknown");
}

function normalizeMachine(machine = {}) {
  const cachedOutput = machine.cached_output || null;
  return {
    machine_id: compactValue(machine.machine_id, "unknown"),
    template: compactValue(machine.template, "unknown"),
    state_id: compactValue(machine.state_id, "unknown"),
    status: compactValue(machine.status, "unknown"),
    has_cache: Boolean(cachedOutput),
    cache_event_type: cachedOutput?.event_type || null,
    last_event_ts_ms: machine.last_event_ts_ms || null,
    last_pull_ts_ms: machine.last_pull_ts_ms || null
  };
}

function capabilityStatusTone(status) {
  if (status === "accepted" || status === "Accepted") return "success";
  if (
    status === "unsupported" ||
    status === "not_declared" ||
    status === "mode_rejected" ||
    status === "policy_missing" ||
    status === "Unsupported" ||
    status === "NotDeclared" ||
    status === "ModeRejected" ||
    status === "PolicyMissing"
  ) {
    return "danger";
  }
  return "neutral";
}

function sourceTone(source) {
  if (source === "runtime_simulated" || source === "RuntimeSimulated") return "warning";
  if (source === "provider_native" || source === "ProviderNative") return "success";
  if (source === "unsupported" || source === "Unsupported") return "danger";
  return "neutral";
}

function normalizeExecutionEntry(entry = {}) {
  const status = STATUS_LABELS[entry.status] || compactValue(entry.status, "unknown");
  return {
    capability: compactValue(entry.capability, "unknown"),
    source: compactValue(entry.source, "unknown"),
    status,
    status_tone: capabilityStatusTone(status),
    source_tone: sourceTone(entry.source),
    reason: compactValue(entry.reason, "-")
  };
}

function normalizeRiskPlane(snapshot = {}) {
  const decision = snapshot.last_decision || null;
  return {
    required: Boolean(snapshot.required),
    machine_ids: Array.isArray(snapshot.machine_ids) ? snapshot.machine_ids : [],
    min_priority: Number(snapshot.min_priority) || 0,
    approved_event_count: Number(snapshot.approved_event_count) || 0,
    rejected_event_count: Number(snapshot.rejected_event_count) || 0,
    real_order_path_unlocked: Boolean(snapshot.real_order_path_unlocked),
    last_decision: decision
      ? {
          accepted: Boolean(decision.accepted),
          source_machine_id: compactValue(decision.source_machine_id, "unknown"),
          target_machine_id: compactValue(decision.target_machine_id, "unknown"),
          reason: compactValue(decision.reason, "-"),
          tone: decision.accepted ? "success" : "danger"
        }
      : null
  };
}

function normalizeExecution(snapshot = {}) {
  const decision = snapshot.last_decision || null;
  const entries = Array.isArray(decision?.entries)
    ? decision.entries.map(normalizeExecutionEntry)
    : [];
  return {
    venue_id: snapshot.venue_id || null,
    required_capabilities: Array.isArray(snapshot.required_capabilities)
      ? snapshot.required_capabilities
      : [],
    accepted_count: Number(snapshot.accepted_count) || 0,
    rejected_count: Number(snapshot.rejected_count) || 0,
    last_decision: decision
      ? {
          accepted: Boolean(decision.accepted),
          target_machine_id: compactValue(decision.target_machine_id, "unknown"),
          venue_id: compactValue(decision.venue_id, "unknown"),
          runtime_mode: runtimeModeLabel(decision.runtime_mode),
          reason: compactValue(decision.reason, "-"),
          provider_order_submission_attached: Boolean(
            decision.provider_order_submission_attached
          ),
          tone: decision.accepted ? "success" : "danger",
          entries
        }
      : null,
    entries
  };
}

function normalizeSimulatedOrder(order = null) {
  if (!order) return null;
  return {
    order_id: compactValue(order.order_id, "unknown"),
    client_order_id: order.client_order_id || null,
    venue_id: compactValue(order.venue_id, "unknown"),
    symbol: compactValue(order.symbol, "unknown"),
    action: compactValue(order.action, "unknown"),
    side: compactValue(order.side, "unknown"),
    order_type: compactValue(order.order_type, "unknown"),
    status: compactValue(order.status, "unknown"),
    requested_quantity: Number(order.requested_quantity) || 0,
    filled_quantity: Number(order.filled_quantity) || 0,
    remaining_quantity: Number(order.remaining_quantity) || 0,
    reference_price: Number(order.reference_price) || 0,
    fill_price: order.fill_price === null || order.fill_price === undefined ? null : Number(order.fill_price),
    rejection_reason: order.rejection_reason || null
  };
}

function normalizeSimulatedFill(fill = null) {
  if (!fill) return null;
  return {
    fill_id: compactValue(fill.fill_id, "unknown"),
    order_id: compactValue(fill.order_id, "unknown"),
    venue_id: compactValue(fill.venue_id, "unknown"),
    symbol: compactValue(fill.symbol, "unknown"),
    side: compactValue(fill.side, "unknown"),
    action: compactValue(fill.action, "unknown"),
    quantity: Number(fill.quantity) || 0,
    price: Number(fill.price) || 0,
    notional: Number(fill.notional) || 0,
    fee: Number(fill.fee) || 0,
    fee_asset: compactValue(fill.fee_asset, "-")
  };
}

function normalizeSimulatedPosition(position = {}) {
  return {
    venue_id: compactValue(position.venue_id, "unknown"),
    symbol: compactValue(position.symbol, "unknown"),
    net_quantity: Number(position.net_quantity) || 0,
    average_price: Number(position.average_price) || 0,
    market_price: Number(position.market_price) || 0,
    market_value: Number(position.market_value) || 0
  };
}

function normalizeSimulatedExecution(snapshot = {}) {
  return {
    enabled: Boolean(snapshot.enabled),
    quote_asset: compactValue(snapshot.quote_asset, "-"),
    cash_balance: Number(snapshot.cash_balance) || 0,
    realized_fees: Number(snapshot.realized_fees) || 0,
    position_market_value: Number(snapshot.position_market_value) || 0,
    portfolio_value: Number(snapshot.portfolio_value) || 0,
    order_count: Number(snapshot.order_count) || 0,
    open_order_count: Number(snapshot.open_order_count) || 0,
    rejected_order_count: Number(snapshot.rejected_order_count) || 0,
    fill_count: Number(snapshot.fill_count) || 0,
    positions: Array.isArray(snapshot.positions)
      ? snapshot.positions.map(normalizeSimulatedPosition)
      : [],
    asset_curve_points: Array.isArray(snapshot.asset_curve) ? snapshot.asset_curve.length : 0,
    last_order: normalizeSimulatedOrder(snapshot.last_order),
    last_fill: normalizeSimulatedFill(snapshot.last_fill)
  };
}

function normalizeVenueBoundary(snapshot = {}) {
  return {
    provider_order_submission_attached: Boolean(snapshot.provider_order_submission_attached),
    provider_order_submission_allowed: Boolean(snapshot.provider_order_submission_allowed),
    settlement_authority: compactValue(snapshot.settlement_authority, "unknown"),
    live_actual_submission_allowed: Boolean(snapshot.live_actual_submission_allowed),
    rejection_before_provider_submit: Boolean(snapshot.rejection_before_provider_submit),
    reason: compactValue(snapshot.reason, "-")
  };
}

export function buildV4RuntimeEvidenceProjection(source = {}) {
  const snapshot = resolveV4RuntimeMemorySnapshot(source);
  if (!snapshot) {
    return {
      available: false,
      machines: [],
      risk_plane: null,
      execution: null,
      simulated_execution: null,
      venue_adapter_boundary: null
    };
  }

  const root = asObject(source);
  const runtimeMode = snapshot.runtime_mode || root.runtime_mode || root.runtimeMode;
  const machines = snapshot.machines.map(normalizeMachine);
  const softSilentCount = machines.filter((machine) => machine.status === "soft_silent").length;
  const activeCount = machines.filter((machine) => machine.status === "active").length;
  const riskPlane = normalizeRiskPlane(snapshot.risk_plane);
  const execution = normalizeExecution(snapshot.execution);
  const simulatedExecution = normalizeSimulatedExecution(snapshot.simulated_execution || {});
  const venueAdapterBoundary = normalizeVenueBoundary(snapshot.venue_adapter_boundary || {});
  const providerOrderSubmissionAttached = Boolean(
    snapshot.provider_order_submission_attached || root.provider_order_submission_attached
  );

  return {
    available: true,
    runtime_mode: runtimeModeLabel(runtimeMode),
    provider_order_submission_attached: providerOrderSubmissionAttached,
    machine_count: machines.length,
    active_machine_count: activeCount,
    soft_silent_machine_count: softSilentCount,
    machines,
    risk_plane: riskPlane,
    execution,
    simulated_execution: simulatedExecution,
    venue_adapter_boundary: venueAdapterBoundary,
    boundary_label: providerOrderSubmissionAttached
      ? "provider_order_submission_attached"
      : "provider_order_submission_detached"
  };
}
