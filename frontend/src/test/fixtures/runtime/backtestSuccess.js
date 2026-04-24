const DEFAULT_BACKTEST_ID = "backtest_smoke_001";
const DEFAULT_PROTOCOL_NAME = "quantpilot/runtime-config/v1";
const DEFAULT_CONFIG_HASH = "smoke_backtest_config_hash";
const DEFAULT_CREATED_AT_MS = 1_700_000_060_000;
const DEFAULT_STARTED_AT_MS = 1_700_000_000_000;
const DEFAULT_ENDED_AT_MS = 1_700_000_060_000;

function digest(value) {
  return {
    algorithm: "sha256_canonical_json",
    value
  };
}

function buildEvents() {
  return [
    {
      event_id: "evt_data_1",
      event_type: "DataUpdated",
      source_id: "data_node_data_2",
      node_id: "node_data_2",
      event_time_ms: DEFAULT_STARTED_AT_MS,
      severity: "Info",
      summary: "Market data updated",
      payload: {
        latest_price: 50_000,
        latest_bar_time: DEFAULT_STARTED_AT_MS,
        source_status: "Healthy",
        source_latency_ms: 0
      }
    },
    {
      event_id: "evt_intent_1",
      event_type: "IntentTriggered",
      source_id: "intent_node_intent_3",
      node_id: "node_intent_3",
      event_time_ms: DEFAULT_STARTED_AT_MS + 1_000,
      severity: "Info",
      summary: "Intent triggered",
      payload: {
        signal_direction: "Long",
        signal_strength: 0.82,
        confidence: 0.91
      }
    },
    {
      event_id: "evt_execution_1",
      event_type: "ExecutionFilled",
      source_id: "execution_node_execution_7",
      node_id: "node_execution_7",
      event_time_ms: DEFAULT_ENDED_AT_MS,
      severity: "Info",
      summary: "Execution filled",
      payload: {
        side: "Buy",
        qty: 0.2,
        price: 50_250,
        exec_status: "Filled",
        order_id: "order_smoke_001"
      }
    }
  ];
}

function buildAccount() {
  return {
    equity_estimate: 12_050,
    cash_balance: 11_500,
    available_cash_balance: 11_200,
    frozen_cash_balance: 300,
    total_leverage: 0.15,
    total_gross_notional: 550,
    total_net_notional: 550,
    positions: 1,
    open_order_count: 0,
    open_orders: []
  };
}

function buildBacktest() {
  return {
    mode: "historical_replay",
    started_at_ms: DEFAULT_STARTED_AT_MS,
    ended_at_ms: DEFAULT_ENDED_AT_MS,
    sessions: [],
    equity_curve: [
      {
        ts_ms: DEFAULT_STARTED_AT_MS,
        equity: 10_000,
        cash_balance: 10_000,
        net_notional: 0
      },
      {
        ts_ms: DEFAULT_ENDED_AT_MS,
        equity: 12_050,
        cash_balance: 11_500,
        net_notional: 550
      }
    ],
    summary: {
      step_count: 2,
      trade_count: 1,
      total_return_ratio: 0.125,
      max_drawdown_ratio: 0.02,
      final_equity: 12_050
    },
    final_portfolio: {
      cash_balance: 11_500,
      available_cash_balance: 11_200,
      frozen_cash_balance: 300,
      open_orders: [],
      positions: [],
      exchange_exposures: [],
      total_gross_notional: 550,
      total_net_notional: 550,
      total_leverage: 0.15,
      updated_at_ms: DEFAULT_ENDED_AT_MS
    }
  };
}

function buildBacktestSpec(graphId, compileId, backtestId) {
  return {
    schema_version: "quantpilot/backtest-spec/v1",
    backtest_id: backtestId,
    replay_source: "deterministic_mock",
    requested_at_ms: DEFAULT_CREATED_AT_MS,
    run_spec: {
      schema_version: "quantpilot/run-spec/v1",
      run_mode: "backtest",
      graph_id: graphId,
      compile_id: compileId,
      runtime_mode: "historical_replay",
      protocol_name: DEFAULT_PROTOCOL_NAME,
      config_hash: DEFAULT_CONFIG_HASH,
      datasets: [
        {
          dataset_id: "btc_usdt_1m",
          data_id: "btc_usdt_1m",
          exchange: "Binance",
          symbol: "BTCUSDT",
          market_type: "Spot",
          kind: "KlineSeries",
          interval: "1m",
          lookback_days: 30,
          enabled: true
        }
      ],
      execution_assumptions: {
        initial_cash_balance: 10_000,
        taker_fee_bps: 10,
        default_slippage_bps: 5,
        total_cost_buffer_bps: 20,
        time_in_force: "Gtc",
        allow_partial_fills: true,
        latency_assumption_ms: 0
      },
      core_ir_digest: digest("core_ir_digest_smoke_001")
    },
    market_data_snapshot: {
      schema_version: "quantpilot/market-data-snapshot-spec/v1",
      snapshot_id: `market_snapshot_${backtestId}`,
      replay_source: "deterministic_mock",
      captured_at_ms: DEFAULT_STARTED_AT_MS,
      datasets: [
        {
          dataset_id: "btc_usdt_1m",
          data_id: "btc_usdt_1m",
          exchange: "Binance",
          symbol: "BTCUSDT",
          market_type: "Spot",
          kind: "KlineSeries",
          interval: "1m",
          lookback_days: 30,
          enabled: true
        }
      ]
    }
  };
}

function buildCompileArtifacts(graphId, compileId) {
  return {
    strategy: {
      schema_version: "quantpilot/strategy-artifact/v1",
      artifact_id: "strategy_artifact_smoke_001",
      graph_id: graphId,
      compile_id: compileId,
      strategy_id: graphId,
      name: "Smoke Backtest Strategy",
      source_kind: "graph_json",
      source_ref: `graphs/${graphId}.json`,
      metadata: {},
      digest: digest("strategy_digest_smoke_001")
    },
    compile: {
      schema_version: "quantpilot/compile-artifact/v1",
      artifact_id: "compile_artifact_smoke_001",
      graph_id: graphId,
      compile_id: compileId,
      protocol_name: DEFAULT_PROTOCOL_NAME,
      config_hash: DEFAULT_CONFIG_HASH,
      strategy_artifact_id: "strategy_artifact_smoke_001",
      core_ir_artifact_id: "core_ir_artifact_smoke_001",
      digest: digest("compile_digest_smoke_001"),
      runtime_config: {
        metadata: {
          graph_id: graphId,
          compile_id: compileId
        }
      }
    },
    core_ir: {
      schema_version: "quantpilot/core-ir-artifact/v1",
      artifact_id: "core_ir_artifact_smoke_001",
      graph_id: graphId,
      compile_id: compileId,
      ir_version: "v1",
      digest: digest("core_ir_digest_smoke_001"),
      core_ir: {
        strategy_id: graphId,
        signals: []
      }
    }
  };
}

function buildExecutionAssumptionsModule(backtestSpec) {
  const assumptions = backtestSpec.run_spec.execution_assumptions;
  return {
    summary: {
      fee_bps: assumptions.taker_fee_bps,
      slippage_bps: assumptions.default_slippage_bps,
      latency_ms: assumptions.latency_assumption_ms ?? 0,
      sources: {
        fee_bps: "backend_fallback",
        slippage_bps: "backend_fallback",
        latency_ms: "backend_fallback"
      }
    },
    list_tag: {
      label: `fee=${assumptions.taker_fee_bps} slip=${assumptions.default_slippage_bps} lat=${assumptions.latency_assumption_ms ?? 0}`,
      sources_label: "fee:backend slip:backend lat:backend"
    }
  };
}

function buildBacktestArtifacts({
  backtestId,
  graphId,
  compileId,
  account,
  events,
  backtest,
  backtestSpec,
  compileArtifacts
}) {
  return {
    event_log: {
      schema_version: "quantpilot/event-log-artifact/v1",
      artifact_id: "event_log_artifact_smoke_001",
      backtest_id: backtestId,
      event_count: events.length,
      digest: digest("event_log_digest_smoke_001"),
      events
    },
    trade_ledger: {
      schema_version: "quantpilot/trade-ledger-artifact/v1",
      artifact_id: "trade_ledger_artifact_smoke_001",
      backtest_id: backtestId,
      trade_count: 1,
      digest: digest("trade_ledger_digest_smoke_001"),
      trades: [
        {
          fill_id: "fill_smoke_001",
          plan_id: "plan_smoke_001",
          exchange: "Binance",
          symbol: "BtcUsdt",
          side: "buy",
          filled_qty: 0.2,
          filled_price: 50_250,
          fee_paid: 1.5,
          filled_at_ms: DEFAULT_ENDED_AT_MS,
          status: "filled",
          trace_id: "trace_smoke_001",
          session_index: 0,
          cycle_name: "slow"
        }
      ]
    },
    equity_curve: {
      schema_version: "quantpilot/equity-curve-artifact/v1",
      artifact_id: "equity_curve_artifact_smoke_001",
      backtest_id: backtestId,
      point_count: backtest.equity_curve.length,
      digest: digest("equity_curve_digest_smoke_001"),
      points: backtest.equity_curve
    },
    metrics: {
      schema_version: "quantpilot/metrics-artifact/v1",
      artifact_id: "metrics_artifact_smoke_001",
      backtest_id: backtestId,
      digest: digest("metrics_digest_smoke_001"),
      summary: backtest.summary,
      event_count: events.length,
      session_count: 0,
      started_at_ms: DEFAULT_STARTED_AT_MS,
      ended_at_ms: DEFAULT_ENDED_AT_MS,
      final_account: account
    },
    manifest: {
      schema_version: "quantpilot/reproducibility-manifest/v1",
      manifest_id: `manifest_${backtestId}`,
      backtest_id: backtestId,
      graph_id: graphId,
      compile_id: compileId,
      created_at_ms: DEFAULT_CREATED_AT_MS,
      protocol_name: DEFAULT_PROTOCOL_NAME,
      config_hash: DEFAULT_CONFIG_HASH,
      account,
      summary: backtest.summary,
      backtest_spec: backtestSpec,
      compile_artifacts: compileArtifacts,
      output_artifacts: [
        {
          kind: "event_log",
          artifact_id: "event_log_artifact_smoke_001",
          digest: digest("event_log_digest_smoke_001"),
          file_name: "event_log.json"
        },
        {
          kind: "trade_ledger",
          artifact_id: "trade_ledger_artifact_smoke_001",
          digest: digest("trade_ledger_digest_smoke_001"),
          file_name: "trade_ledger.json"
        },
        {
          kind: "equity_curve",
          artifact_id: "equity_curve_artifact_smoke_001",
          digest: digest("equity_curve_digest_smoke_001"),
          file_name: "equity_curve.json"
        },
        {
          kind: "metrics",
          artifact_id: "metrics_artifact_smoke_001",
          digest: digest("metrics_digest_smoke_001"),
          file_name: "metrics.json"
        }
      ],
      backtest_output_digest: digest("backtest_output_digest_smoke_001")
    }
  };
}

function buildRuntimeDiagnostics(events) {
  const sortedEvents = [...events].sort((left, right) => right.event_time_ms - left.event_time_ms);
  const nodeIds = [...new Set(sortedEvents.map((event) => event.node_id).filter(Boolean))];
  return {
    source: "backtest_event_log",
    default_selected_node_id: nodeIds[0] || null,
    active_nodes: nodeIds.map((nodeId) => {
      const nodeEvents = sortedEvents.filter((event) => event.node_id === nodeId);
      const latestEvent = nodeEvents[0] || null;
      return {
        node_id: nodeId,
        latest_event_type: latestEvent?.event_type || null,
        latest_event_label: latestEvent?.event_type || "RuntimeNotice",
        latest_event_time_ms: latestEvent?.event_time_ms || null,
        event_count: nodeEvents.length
      };
    }),
    node_details: Object.fromEntries(
      nodeIds.map((nodeId) => {
        const nodeEvents = sortedEvents.filter((event) => event.node_id === nodeId);
        const latestEvent = nodeEvents[0] || null;
        const latestNotice =
          nodeEvents.find((event) => event.severity !== "Info") || latestEvent || null;
        return [
          nodeId,
          {
            node_id: nodeId,
            latest_event: latestEvent
              ? {
                  event_id: latestEvent.event_id,
                  event_type: latestEvent.event_type,
                  label: latestEvent.event_type,
                  summary: latestEvent.summary,
                  tone: latestEvent.severity === "Info" ? "info" : "warning",
                  severity: latestEvent.severity,
                  event_time_ms: latestEvent.event_time_ms
                }
              : null,
            explanation_summary:
              latestEvent?.payload?.explanation_summary || latestEvent?.payload?.reason_text || null,
            latest_input_rows: Object.entries(latestEvent?.payload || {}).map(([key, value]) => ({
              key,
              label: key,
              value: String(value)
            })),
            latest_output_rows: Object.entries(latestEvent?.payload || {}).map(([key, value]) => ({
              key,
              label: key,
              value: String(value)
            })),
            explanation_rows: [],
            risk_detail_rows: [],
            order_detail_rows: [],
            latest_notice: latestNotice
              ? {
                  event_id: latestNotice.event_id,
                  event_type: latestNotice.event_type,
                  label: latestNotice.event_type,
                  summary: latestNotice.summary,
                  tone: latestNotice.severity === "Info" ? "info" : "warning",
                  severity: latestNotice.severity,
                  event_time_ms: latestNotice.event_time_ms
                }
              : null,
            recent_events: nodeEvents.slice(0, 5).map((event) => ({
              event_id: event.event_id,
              event_type: event.event_type,
              label: event.event_type,
              summary: event.summary,
              tone: event.severity === "Info" ? "info" : "warning",
              severity: event.severity,
              event_time_ms: event.event_time_ms
            })),
            event_count: nodeEvents.length
          }
        ];
      })
    )
  };
}

export function buildBacktestSuccessFixture({
  graphId,
  compileId,
  backtestId = DEFAULT_BACKTEST_ID
}) {
  const account = buildAccount();
  const events = buildEvents();
  const backtest = buildBacktest();
  const backtestSpec = buildBacktestSpec(graphId, compileId, backtestId);
  const compileArtifacts = buildCompileArtifacts(graphId, compileId);
  const executionAssumptions = buildExecutionAssumptionsModule(backtestSpec);
  const backtestArtifacts = buildBacktestArtifacts({
    backtestId,
    graphId,
    compileId,
    account,
    events,
    backtest,
    backtestSpec,
    compileArtifacts
  });
  backtestArtifacts.metrics.execution_assumptions = executionAssumptions;

  const startResponse = {
    backtest_id: backtestId,
    graph_id: graphId,
    compile_id: compileId,
    protocol_name: DEFAULT_PROTOCOL_NAME,
    config_hash: DEFAULT_CONFIG_HASH,
    event_count: events.length,
    account,
    execution_assumptions: executionAssumptions,
    backtest_artifacts: backtestArtifacts
  };

  const historyItem = {
    backtest_id: backtestId,
    graph_id: graphId,
    compile_id: compileId,
    created_at_ms: DEFAULT_CREATED_AT_MS,
    protocol_name: DEFAULT_PROTOCOL_NAME,
    config_hash: DEFAULT_CONFIG_HASH,
    event_count: events.length,
    account,
    summary: backtestArtifacts.metrics.summary,
    filters: {
      replay_source: backtestSpec.replay_source,
      dataset_labels: backtestSpec.run_spec.datasets.map((dataset) => {
        const interval = dataset.interval || "na";
        return `${dataset.exchange}:${dataset.symbol}:${interval}`;
      }),
      execution_assumptions_tag: executionAssumptions.list_tag,
      started_at_ms: DEFAULT_STARTED_AT_MS,
      ended_at_ms: DEFAULT_ENDED_AT_MS
    }
  };

  const detailResponse = {
    ...historyItem,
    runtime_diagnostics: buildRuntimeDiagnostics(backtestArtifacts.event_log.events),
    execution_assumptions: executionAssumptions,
    backtest_artifacts: backtestArtifacts
  };

  return {
    startResponse,
    historyResponse: [historyItem],
    detailResponse
  };
}
