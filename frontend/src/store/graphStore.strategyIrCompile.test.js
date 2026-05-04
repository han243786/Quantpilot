import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { act } from "@testing-library/react";
import { useGraphStore } from "./graphStore";
import { buildValidatedSampleGraph } from "../test/fixtures/runtime/buildValidatedSampleGraph";
import { backendCapabilitiesFixture } from "../test/fixtures/capabilities/capabilityFallbacks";

function cloneJson(value) {
  return JSON.parse(JSON.stringify(value));
}

function buildStrategyIrArtifact() {
  return {
    ir_version: "strategy_ir/v0",
    metadata: {
      strategy_id: "restricted_custom_v1",
      name: "Restricted Custom",
      summary: "Custom signal lowered into Core IR.",
      source: {
        source_type: "manual_paper_analysis",
        paper_title: "Restricted Custom",
        paper_reference: null
      },
      authors: ["QuantPilot"],
      tags: ["custom"]
    },
    signals: [
      {
        signal_id: "custom_signal",
        name: "Custom Signal",
        indicator: {
          kind: "custom",
          inputs: ["price_daily"],
          params: {
            custom_expr: {
              expr_version: "custom_expr/v1",
              signal_kind: "long_entry",
              predicate: {
                type: "comparison",
                op: "gt",
                left: {
                  type: "window_agg",
                  data_id: "price_daily",
                  field: "close",
                  aggregation: "sma",
                  window_size: 5
                },
                right: {
                  type: "window_agg",
                  data_id: "price_daily",
                  field: "close",
                  aggregation: "sma",
                  window_size: 20
                }
              }
            }
          }
        }
      }
    ],
    logic: {
      entry_rules: [
        {
          rule_id: "entry_custom",
          condition: "custom_signal == true",
          action: "open_long"
        }
      ],
      exit_rules: [],
      position_sizing: {
        method: "fixed_ratio",
        value: 0.1,
        unit: "portfolio_ratio"
      },
      rebalance_rule: null
    },
    risk_rules: {
      max_position_ratio: 0.2,
      stop_loss_ratio: 0.03,
      take_profit_ratio: null,
      max_drawdown_ratio: null,
      max_trades_per_day: null,
      notes: []
    },
    data_requirements: [
      {
        data_id: "price_daily",
        venue: "binance",
        symbol: "BTCUSDT",
        data_type: "kline",
        granularity: "1d",
        lookback: 200,
        fields: ["close"]
      }
    ],
    execution: {
      venue_type: "paper",
      order_type: "market",
      time_in_force: null,
      slippage_model: "fixed_bps",
      latency_assumption_ms: null,
      capital_base: null
    },
    gap_annotations: [],
    unknowns: []
  };
}

function buildGraphWithStrategyIr(baseGraph) {
  const graph = cloneJson(baseGraph);
  graph.metadata.graph_id = "strategy_ir_frontend_graph";
  graph.metadata.source_mode = "strategy_ir";
  graph.metadata.artifacts = {
    ...(graph.metadata.artifacts || {}),
    strategy_ir: {
      document: buildStrategyIrArtifact()
    }
  };
  return graph;
}

describe("graphStore Strategy IR compile integration", () => {
  const initialState = useGraphStore.getState();

  beforeEach(() => {
    act(() => {
      useGraphStore.setState(initialState, true);
      useGraphStore.setState({
        graph: buildGraphWithStrategyIr(buildValidatedSampleGraph(initialState.registry))
      });
    });
    window.localStorage.clear();
    vi.unstubAllGlobals();
  });

  afterEach(() => {
    act(() => {
      useGraphStore.setState(initialState, true);
    });
    window.localStorage.clear();
    vi.unstubAllGlobals();
  });

  it("blocks compile when loaded capabilities are missing permission boundary", async () => {
    const { permission_boundary: _permissionBoundary, ...malformedCapabilities } =
      backendCapabilitiesFixture;
    const fetchMock = vi.fn();
    vi.stubGlobal("fetch", fetchMock);

    act(() => {
      useGraphStore.setState({
        capabilities: malformedCapabilities,
        capabilityStatus: "ready",
        capabilitySource: "remote",
        capabilityMessage: ""
      });
    });

    let result = "not-run";
    await act(async () => {
      result = await useGraphStore.getState().compileCurrentGraph();
    });

    const state = useGraphStore.getState();
    expect(result).toBeNull();
    expect(fetchMock).not.toHaveBeenCalled();
    expect(state.runtime.backendError).toContain("缺少 permission_boundary");
    expect(state.graph.compile_summary.compilable).toBe(false);
    expect(state.graph.compile_summary.diagnostics[0].code).toBe("CAPABILITY_BOUNDARY");
  });

  it("preflights Strategy IR before runtime compile when a strategy_ir artifact exists", async () => {
    const strategyIrResponse = {
      graph_id: "strategy_ir_frontend_graph",
      compile_id: "compile_strategy_ir",
      compilable: true,
      diagnostics: [],
      core_ir: {
        ir_version: "core_strategy_ir/v1"
      }
    };
    const runtimeCompileResponse = {
      compile_id: "compile_backend",
      protocol_name: "runtime_protocol/v1",
      config_hash: "cfg_hash_test",
      counts: {
        data_sources: 1,
        intent_generators: 2,
        agents: 1,
        risk_controls: 1,
        executions: 1
      },
      diagnostics: [],
      core_ir: {
        ir_version: "core_strategy_ir/v1"
      },
      runtime_config: {
        metadata: {
          graph_id: "strategy_ir_frontend_graph",
          compile_id: "compile_backend"
        }
      },
      runtime_targets: {
        source_to_node: {},
        runtime_node_id: "runtime_1",
        execution_node_id: "execution_1"
      }
    };
    let strategyIrRequestBody = "";
    let formalCompileRequestBody = "";
    const fetchMock = vi.fn(async (url, options = {}) => {
      if (url.endsWith("/api/strategy-ir/compile")) {
        strategyIrRequestBody = String(options.body || "");
        return {
          ok: true,
          json: async () => strategyIrResponse
        };
      }
      if (url.endsWith("/api/quantscript/formal/compile")) {
        formalCompileRequestBody = String(options.body || "");
        return {
          ok: true,
          json: async () => runtimeCompileResponse
        };
      }
      throw new Error(`Unexpected fetch: ${url}`);
    });
    vi.stubGlobal("fetch", fetchMock);

    let result = null;
    await act(async () => {
      result = await useGraphStore.getState().compileCurrentGraph();
    });

    const state = useGraphStore.getState();
    expect(result).not.toBeNull();
    expect(result.strategy_ir_compile).toEqual(strategyIrResponse);
    expect(result.backend_compile).toEqual(runtimeCompileResponse);
    expect(state.graph.compile_summary.compilable).toBe(true);
    expect(state.graph.compile_summary.backend_verified).toBe(true);
    expect(fetchMock).toHaveBeenCalledTimes(2);
    expect(fetchMock.mock.calls[0][0]).toMatch(/\/api\/strategy-ir\/compile$/);
    expect(fetchMock.mock.calls[1][0]).toMatch(/\/api\/quantscript\/formal\/compile$/);
    expect(strategyIrRequestBody).toContain("custom_signal");
    expect(formalCompileRequestBody).toContain("runtime_template");
  });

  it("surfaces structured Strategy IR diagnostics in compile_summary when backend lowering fails", async () => {
    const fetchMock = vi.fn(async (url) => {
      if (url.endsWith("/api/strategy-ir/compile")) {
        return {
          ok: false,
          status: 400,
          text: async () =>
            JSON.stringify({
              error: "strategy_ir_compile_failed",
              message: "Strategy IR lowering failed",
              details: [
                {
                  code: "CUSTOM006",
                  target: "custom_signal.params.custom_expr",
                  message:
                    "CUSTOM006 signal `custom_signal` uses undeclared input `other_data` in custom_expr",
                  reason: "Custom only allows the restricted expression whitelist and must lower into Core IR."
                }
              ]
            })
        };
      }
      throw new Error(`Unexpected fetch: ${url}`);
    });
    vi.stubGlobal("fetch", fetchMock);

    let result = null;
    await act(async () => {
      result = await useGraphStore.getState().compileCurrentGraph();
    });

    const state = useGraphStore.getState();
    expect(result).toBeNull();
    expect(fetchMock).toHaveBeenCalledTimes(1);
    expect(state.graph.compile_summary.compilable).toBe(false);
    expect(state.graph.compile_summary.backend_verified).toBe(false);
    expect(state.graph.compile_summary.diagnostics).toEqual(
      expect.arrayContaining([
        expect.objectContaining({
          code: "CUSTOM006",
          severity: "error",
          message:
            "CUSTOM006 signal `custom_signal` uses undeclared input `other_data` in custom_expr",
          hint: "Custom only allows the restricted expression whitelist and must lower into Core IR.",
          target: expect.objectContaining({
            scope: "strategy_ir",
            field: "custom_signal.params.custom_expr",
            label: "custom_signal.params.custom_expr"
          })
        })
      ])
    );
  });

  it("keeps runtime compile as the source of truth when formal QuantScript lowering falls back", async () => {
    const strategyIrResponse = {
      graph_id: "strategy_ir_frontend_graph",
      compile_id: "compile_strategy_ir",
      compilable: true,
      diagnostics: [],
      core_ir: {
        ir_version: "core_strategy_ir/v1"
      }
    };
    const runtimeCompileResponse = {
      compile_id: "compile_backend_runtime",
      protocol_name: "runtime_protocol/v1",
      config_hash: "cfg_hash_runtime_fallback",
      counts: {
        data_sources: 1,
        intent_generators: 2,
        agents: 1,
        risk_controls: 1,
        executions: 1
      },
      diagnostics: [],
      core_ir: {
        ir_version: "core_strategy_ir/v1"
      },
      runtime_config: {
        metadata: {
          graph_id: "strategy_ir_frontend_graph",
          compile_id: "compile_backend_runtime"
        }
      }
    };
    const fetchMock = vi.fn(async (url) => {
      if (url.endsWith("/api/strategy-ir/compile")) {
        return {
          ok: true,
          json: async () => strategyIrResponse
        };
      }
      if (url.endsWith("/api/quantscript/formal/compile")) {
        return {
          ok: false,
          status: 404,
          text: async () =>
            JSON.stringify({ error: "not_found", message: "formal compile unavailable" })
        };
      }
      if (url.endsWith("/api/runtime/compile")) {
        return {
          ok: true,
          json: async () => runtimeCompileResponse
        };
      }
      throw new Error(`Unexpected fetch: ${url}`);
    });
    vi.stubGlobal("fetch", fetchMock);

    let result = null;
    await act(async () => {
      result = await useGraphStore.getState().compileCurrentGraph();
    });

    const artifactResolution = useGraphStore.getState().graph.compile_summary.artifact_resolution;
    expect(result).not.toBeNull();
    expect(fetchMock).toHaveBeenCalledTimes(3);
    expect(artifactResolution.runtime_source).toBe("runtime_fallback");
    expect(artifactResolution.runtime_source_label).toBe("图生成的 runtime_config 回退输入");
    expect(artifactResolution.source_of_truth).toBe("runtime_compile");
    expect(artifactResolution.source_of_truth_label).toBe("以 /api/runtime/compile 输出为准");
    expect(artifactResolution.notes[0]).toContain("语义预检");
    expect(artifactResolution.notes[1]).toContain("运行时编译输出为准");
  });
});
