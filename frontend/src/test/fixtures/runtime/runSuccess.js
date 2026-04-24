const DEFAULT_RUN_ID = "run_smoke_001";
const DEFAULT_CREATED_AT_MS = 1_700_000_030_000;

function buildAccount() {
  return {
    equity_estimate: 10_250,
    cash_balance: 10_100,
    available_cash_balance: 10_000,
    frozen_cash_balance: 100,
    total_leverage: 0.05,
    total_gross_notional: 150,
    total_net_notional: 150,
    positions: 1,
    open_order_count: 0,
    open_orders: []
  };
}

function buildEvents() {
  return [
    {
      event_id: "evt_run_data_1",
      event_type: "DataUpdated",
      source_id: "data_node_data_2",
      node_id: "node_data_2",
      event_time_ms: DEFAULT_CREATED_AT_MS,
      severity: "Info",
      summary: "Market data updated",
      payload: {
        latest_price: 50_000,
        latest_bar_time: DEFAULT_CREATED_AT_MS,
        source_status: "Healthy",
        source_latency_ms: 0
      }
    },
    {
      event_id: "evt_run_intent_1",
      event_type: "IntentTriggered",
      source_id: "intent_node_intent_3",
      node_id: "node_intent_3",
      event_time_ms: DEFAULT_CREATED_AT_MS + 1_000,
      severity: "Info",
      summary: "Intent triggered",
      payload: {
        signal_direction: "Long",
        signal_strength: 0.78,
        confidence: 0.88
      }
    },
    {
      event_id: "evt_run_exec_1",
      event_type: "ExecutionFilled",
      source_id: "execution_node_execution_7",
      node_id: "node_execution_7",
      event_time_ms: DEFAULT_CREATED_AT_MS + 2_000,
      severity: "Info",
      summary: "Execution filled",
      payload: {
        side: "Buy",
        qty: 0.1,
        price: 50_200,
        exec_status: "Filled",
        order_id: "order_run_smoke_001"
      }
    },
    {
      event_id: "evt_run_portfolio_1",
      event_type: "PortfolioUpdated",
      source_id: "runtime_node_runtime_1",
      node_id: "node_runtime_1",
      event_time_ms: DEFAULT_CREATED_AT_MS + 3_000,
      severity: "Info",
      summary: "Portfolio updated",
      payload: {
        cash_balance: 10_100,
        available_cash_balance: 10_000,
        frozen_cash_balance: 100,
        total_leverage: 0.05,
        total_gross_notional: 150,
        total_net_notional: 150,
        positions: 1,
        open_order_count: 0
      }
    }
  ];
}

function buildSseBody(runId, graphId, compileId, events, account) {
  const frames = [
    ["run_started", { run_id: runId, graph_id: graphId, compile_id: compileId, status: "started" }],
    ...events.map((event) => ["runtime_event", event]),
    ["account", account],
    ["run_completed", { run_id: runId, status: "completed", event_count: events.length }]
  ];

  return `${frames
    .map(([name, payload]) => `event: ${name}\ndata: ${JSON.stringify(payload)}\n\n`)
    .join("")}`;
}

export function buildRunSuccessFixture({
  graphId,
  compileId,
  runId = DEFAULT_RUN_ID
}) {
  const account = buildAccount();
  const events = buildEvents();

  const startResponse = {
    run_id: runId,
    graph_id: graphId,
    compile_id: compileId,
    event_count: events.length,
    status: "queued"
  };

  const historyItem = {
    run_id: runId,
    graph_id: graphId,
    compile_id: compileId,
    created_at_ms: DEFAULT_CREATED_AT_MS,
    event_count: events.length,
    account
  };

  const detailResponse = {
    ...historyItem,
    events,
    session: {
      slow_cycle: { runtime_events: events },
      fast_cycle: { runtime_events: [] }
    }
  };

  return {
    startResponse,
    historyResponse: [historyItem],
    detailResponse,
    sseBody: buildSseBody(runId, graphId, compileId, events, account)
  };
}
