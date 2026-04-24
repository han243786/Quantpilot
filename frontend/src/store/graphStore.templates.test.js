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
    expect(graph.metadata.template_label).toBe("Multi-symbol rebalance");
    expect(graph.validation_state.is_valid).toBe(true);
    expect(graph.validation_state.is_runnable).toBe(true);

    const agentNode = graph.nodes.find((node) => node.module_key === "builtin.agent.weighted");
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
});
