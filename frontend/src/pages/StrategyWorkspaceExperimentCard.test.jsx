import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { act, fireEvent, render, screen } from "@testing-library/react";
import StrategyWorkspaceExperimentCard from "./StrategyWorkspaceExperimentCard";
import { useGraphStore } from "../store/graphStore";
import { buildValidatedSampleGraph } from "../test/fixtures/runtime/buildValidatedSampleGraph";

describe("StrategyWorkspaceExperimentCard", () => {
  const initialState = useGraphStore.getState();

  beforeEach(() => {
    act(() => {
      useGraphStore.setState(initialState, true);
    });
  });

  afterEach(() => {
    act(() => {
      useGraphStore.setState(initialState, true);
    });
  });

  it("parses grid input and starts a backtest experiment from the workspace card", async () => {
    const startBacktestExperiment = vi.fn().mockResolvedValue(undefined);
    const loadExperimentDetail = vi.fn();
    const graph = buildValidatedSampleGraph(initialState.registry, (draft) => {
      draft.metadata.graph_id = "alpha_strategy";
      draft.metadata.name = "Alpha Strategy";
    });

    act(() => {
      useGraphStore.setState({
        runtime: {
          ...initialState.runtime,
          experiments: [],
          experimentsStatus: "ready",
          selectedExperiment: null,
          selectedExperimentId: null,
          selectedExperimentStatus: "idle"
        },
        startBacktestExperiment,
        loadExperimentDetail
      });
    });

    await act(async () => {
      render(<StrategyWorkspaceExperimentCard strategyId="alpha_strategy" currentGraph={graph} />);
      await Promise.resolve();
    });

    await act(async () => {
      fireEvent.change(screen.getByTestId("workspace-experiment-name-input"), {
        target: { value: "Assumption sweep" }
      });
      fireEvent.change(screen.getByTestId("workspace-experiment-fee-grid-input"), {
        target: { value: "5, 15" }
      });
      fireEvent.change(screen.getByTestId("workspace-experiment-slippage-grid-input"), {
        target: { value: "5" }
      });
      fireEvent.change(screen.getByTestId("workspace-experiment-latency-grid-input"), {
        target: { value: "0, 250" }
      });
      fireEvent.click(screen.getByTestId("workspace-experiment-run-action"));
      await Promise.resolve();
    });

    expect(startBacktestExperiment).toHaveBeenCalledWith({
      experimentName: "Assumption sweep",
      feeBps: [5, 15],
      slippageBps: [5],
      latencyMs: [0, 250]
    });
    expect(loadExperimentDetail).not.toHaveBeenCalled();
  });

  it("renders selected experiment variants with stable rows", async () => {
    const graph = buildValidatedSampleGraph(initialState.registry, (draft) => {
      draft.metadata.graph_id = "alpha_strategy";
      draft.metadata.name = "Alpha Strategy";
    });

    act(() => {
      useGraphStore.setState({
        runtime: {
          ...initialState.runtime,
          experiments: [
            {
              experiment_id: "experiment_1",
              graph_id: "alpha_strategy",
              compile_id: "compile_1",
              created_at_ms: 1_700_000_000_000,
              experiment_name: "Assumption sweep",
              replay_source: "deterministic_mock",
              variant_count: 2,
              sweep_axes: ["fee_bps", "latency_ms"],
              best_backtest_id: "backtest_best",
              best_total_return_ratio: 0.12
            }
          ],
          experimentsStatus: "ready",
          selectedExperimentId: "experiment_1",
          selectedExperimentStatus: "ready",
          selectedExperiment: {
            experiment_id: "experiment_1",
            graph_id: "alpha_strategy",
            compile_id: "compile_1",
            created_at_ms: 1_700_000_000_000,
            definition: {
              experiment_name: "Assumption sweep",
              replay_source: "deterministic_mock",
              base_execution_assumptions: {},
              parameter_grid: {
                fee_bps: [5, 15],
                slippage_bps: [5],
                latency_ms: [0, 250]
              }
            },
            variants: [
              {
                variant_id: "variant_1",
                backtest_id: "backtest_a",
                created_at_ms: 1_700_000_000_010,
                fee_bps: 5,
                slippage_bps: 5,
                latency_ms: 0,
                summary: {
                  step_count: 10,
                  trade_count: 3,
                  total_return_ratio: 0.08,
                  max_drawdown_ratio: 0.03,
                  final_equity: 1080,
                  net_profit: 80,
                  turnover_ratio: 0.2,
                  average_trade_notional: 100,
                  fee_drag_ratio: 0.001
                }
              },
              {
                variant_id: "variant_2",
                backtest_id: "backtest_b",
                created_at_ms: 1_700_000_000_020,
                fee_bps: 15,
                slippage_bps: 5,
                latency_ms: 250,
                summary: {
                  step_count: 10,
                  trade_count: 4,
                  total_return_ratio: 0.12,
                  max_drawdown_ratio: 0.04,
                  final_equity: 1120,
                  net_profit: 120,
                  turnover_ratio: 0.25,
                  average_trade_notional: 110,
                  fee_drag_ratio: 0.002
                }
              }
            ]
          }
        },
        loadExperimentDetail: vi.fn(),
        startBacktestExperiment: vi.fn()
      });
    });

    await act(async () => {
      render(<StrategyWorkspaceExperimentCard strategyId="alpha_strategy" currentGraph={graph} />);
      await Promise.resolve();
    });

    expect(screen.getByTestId("workspace-experiment-results")).toBeInTheDocument();
    expect(screen.getByTestId("workspace-experiment-variant-variant_1").textContent).toContain(
      "backtest_a"
    );
    expect(screen.getByTestId("workspace-experiment-variant-variant_2").textContent).toContain(
      "+12.00%"
    );
  });

  it("locks the sweep trigger while capability sync is blocked", async () => {
    const startBacktestExperiment = vi.fn().mockResolvedValue(undefined);
    const graph = buildValidatedSampleGraph(initialState.registry, (draft) => {
      draft.metadata.graph_id = "alpha_strategy";
      draft.metadata.name = "Alpha Strategy";
    });

    act(() => {
      useGraphStore.setState({
        runtime: {
          ...initialState.runtime,
          experiments: [],
          experimentsStatus: "ready",
          selectedExperiment: null,
          selectedExperimentId: null,
          selectedExperimentStatus: "idle"
        },
        capabilityStatus: "error",
        capabilitySource: "safe_fallback",
        capabilityMessage: "能力校验失败，已进入安全回退模式。",
        startBacktestExperiment,
        loadExperimentDetail: vi.fn()
      });
    });

    await act(async () => {
      render(<StrategyWorkspaceExperimentCard strategyId="alpha_strategy" currentGraph={graph} />);
      await Promise.resolve();
    });

    expect(screen.getByTestId("workspace-experiment-run-action")).toBeDisabled();
    expect(screen.getByTestId("workspace-experiment-capability-note").textContent).toContain(
      "安全回退模式"
    );

    fireEvent.click(screen.getByTestId("workspace-experiment-run-action"));
    expect(startBacktestExperiment).not.toHaveBeenCalled();
  });
});
