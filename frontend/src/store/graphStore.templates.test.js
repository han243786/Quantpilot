import { afterEach, beforeEach, describe, expect, it } from "vitest";
import { useGraphStore } from "./graphStore";

describe("graphStore strategy templates", () => {
  const initialState = useGraphStore.getState();

  beforeEach(() => {
    useGraphStore.setState(initialState, true);
    window.localStorage.clear();
  });

  afterEach(() => {
    useGraphStore.setState(initialState, true);
    window.localStorage.clear();
  });

  it("loads a starter template into the working draft without clearing persisted history lists", () => {
    useGraphStore.setState((state) => ({
      runtime: {
        ...state.runtime,
        history: [{ run_id: "run_alpha_01", graph_id: "alpha_strategy" }],
        historyStatus: "ready",
        backtestHistory: [{ backtest_id: "bt_alpha_01", graph_id: "alpha_strategy" }],
        backtestHistoryStatus: "ready",
        experiments: [{ experiment_id: "exp_alpha_01", graph_id: "alpha_strategy" }],
        experimentsStatus: "ready",
        selectedHistoryRunId: "run_alpha_01",
        selectedBacktestId: "bt_alpha_01",
        selectedExperimentId: "exp_alpha_01",
        backtestCompareSelection: ["bt_alpha_01"],
        events: [{ type: "RuntimeStarted" }]
      }
    }));

    const graph = useGraphStore.getState().loadStrategyTemplate("multi_symbol_rebalance");
    const state = useGraphStore.getState();

    expect(graph.metadata.template_id).toBe("multi_symbol_rebalance");
    expect(graph.metadata.template_label).toBe("多标的再平衡");
    expect(graph.metadata.name).toBe("多标的再平衡起始模板");
    expect(graph.validation_state.is_valid).toBe(true);
    expect(graph.validation_state.is_runnable).toBe(true);

    const agentNode = graph.nodes.find((node) => node.module_key === "builtin.agent.weighted");
    expect(agentNode.name).toBe("多标的再平衡代理");
    expect(agentNode.config.rebalance_symbols).toBe("BTCUSDT, ETHUSDT, SOLUSDT");
    expect(agentNode.config.rebalance_target_weights).toBe("0.5, 0.3, 0.2");

    expect(state.runtime.history).toHaveLength(1);
    expect(state.runtime.backtestHistory).toHaveLength(1);
    expect(state.runtime.experiments).toHaveLength(1);
    expect(state.runtime.selectedHistoryRunId).toBeNull();
    expect(state.runtime.selectedBacktestId).toBeNull();
    expect(state.runtime.selectedExperimentId).toBeNull();
    expect(state.runtime.backtestCompareSelection).toEqual([]);
    expect(state.runtime.events).toEqual([]);
    expect(state.compileResult).toBeNull();
    expect(state.graphVersionPreview).toBeNull();
    expect(state.graphVersionCompare).toBeNull();
  });

  it("uses easier order-triggering defaults for the three built-in templates", () => {
    const loadTemplate = (templateId) => useGraphStore.getState().loadStrategyTemplate(templateId);
    const findNode = (graph, moduleKey, name) =>
      graph.nodes.find(
        (node) => node.module_key === moduleKey && (!name || node.name === name)
      );

    const trend = loadTemplate("dual_ma_trend");
    const trendEntry = findNode(trend, "builtin.intent.double_ma");
    const trendExit = findNode(trend, "builtin.intent.ma_deviation");
    const trendAgent = findNode(trend, "builtin.agent.weighted");
    const trendRisk = findNode(trend, "builtin.risk.global");

    expect(trendEntry.config.fast_period).toBe(12);
    expect(trendEntry.config.slow_period).toBe(36);
    expect(trendEntry.config.entry_ratio).toBe(0.05);
    expect(trendExit.config.threshold_ratio).toBe(0.35);
    expect(trendAgent.config.decision_threshold).toBe(0.015);
    expect(trendAgent.config.max_quantity_ratio).toBe(0.65);
    expect(trendRisk.config.max_position).toBe(0.6);

    const rsi = loadTemplate("rsi_reversion");
    const rsiIntent = findNode(rsi, "builtin.intent.rsi");
    const rsiAgent = findNode(rsi, "builtin.agent.weighted");
    const rsiRisk = findNode(rsi, "builtin.risk.global");

    expect(rsiIntent.config.period).toBe(10);
    expect(rsiIntent.config.oversold_threshold).toBe(45);
    expect(rsiIntent.config.overbought_threshold).toBe(55);
    expect(rsiAgent.config.decision_threshold).toBe(0.015);
    expect(rsiAgent.config.max_quantity_ratio).toBe(0.5);
    expect(rsiRisk.config.max_position).toBe(0.45);

    const rebalance = loadTemplate("multi_symbol_rebalance");
    const rebalanceAgent = findNode(rebalance, "builtin.agent.weighted");
    const rebalanceRisk = findNode(rebalance, "builtin.risk.global");

    expect(rebalanceAgent.config.decision_threshold).toBe(0.01);
    expect(rebalanceAgent.config.max_quantity_ratio).toBe(0.8);
    expect(rebalanceAgent.config.rebalance_schedule).toBe("every_1d");
    expect(rebalanceRisk.config.max_position).toBe(0.75);
    expect(rebalanceRisk.config.max_concentration).toBe(0.85);
    expect(rebalanceRisk.config.max_symbol_net_exposure).toBe(0.85);
    expect(rebalanceRisk.config.max_portfolio_net_exposure).toBe(0.95);
  });
});
