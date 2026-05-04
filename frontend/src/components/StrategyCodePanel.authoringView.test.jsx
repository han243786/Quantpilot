import { act, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { vi } from "vitest";
import StrategyCodePanel from "./StrategyCodePanel";
import { useGraphStore } from "../store/graphStore";

describe("StrategyCodePanel authoring view", () => {
  const initialState = useGraphStore.getState();
  const graphSource = `fn strategy() {
    # risk
    risk.profile("global", max_drawdown=0.12)

    # execution
    execution.profile("paper", fee_bps=10.0)

    # data
    close = fetch("close")
    rsi14 = rsi(close, 14)
    base = universe(exchange="binance", market="spot", quote="USDT")
    liquid = filter(base, min_volume_24h=1000000000, min_listing_age_days=180)
    leaders = top(sort_by(liquid, key="market_cap", order="desc"), 2)

    # intent
    if rsi14 < 30 {
      emit Intent("open_long")
    }

    # agent
    rebalance(rank_weight(leaders, method="linear"), every="weekly")
}`;

  beforeEach(() => {
    act(() => {
      useGraphStore.setState(initialState, true);
      useGraphStore.setState({
        selectedNodeId: null,
        selectedEdgeId: null,
        graph: {
          ...useGraphStore.getState().graph,
          metadata: {
            ...useGraphStore.getState().graph.metadata,
            artifacts: {
              ...useGraphStore.getState().graph.metadata?.artifacts,
              quantscript: {
                ...(useGraphStore.getState().graph.metadata?.artifacts?.quantscript || {}),
                graph_source: graphSource,
                formal_source: graphSource
              }
            }
          }
        },
        compileResult: {
          backend_compile: {
            artifacts: {
              strategy: {
                metadata: {
                  quantscript_authoring_view: {
                    kind: "quantscript_authoring_view",
                    source_hash: "qs_hash_001",
                    source_order: ["risk", "execution", "data", "intent", "agent"],
                    pipeline_order: ["data", "intent", "agent", "risk", "execution"],
                    sections: [
                      {
                        id: "sec_risk",
                        declared_kind: "risk",
                        effective_kind: "risk",
                        origin: "authored",
                        status: "ok",
                        start_line: 2,
                        end_line: 3,
                        snippet: '# risk\nrisk.profile("global", max_drawdown=0.12)\n',
                        symbols_defined: [],
                        symbols_used: ["max_drawdown"]
                      },
                      {
                        id: "sec_execution",
                        declared_kind: "execution",
                        effective_kind: "execution",
                        origin: "authored",
                        status: "ok",
                        start_line: 5,
                        end_line: 6,
                        snippet: '# execution\nexecution.profile("paper", fee_bps=10.0)\n',
                        symbols_defined: [],
                        symbols_used: ["fee_bps"]
                      },
                      {
                        id: "sec_data",
                        declared_kind: "data",
                        effective_kind: "data",
                        origin: "authored",
                        status: "ok",
                        start_line: 8,
                        end_line: 13,
                        snippet:
                          '# data\nclose = fetch("close")\nrsi14 = rsi(close, 14)\nbase = universe(exchange="binance", market="spot", quote="USDT")\nliquid = filter(base, min_volume_24h=1000000000, min_listing_age_days=180)\nleaders = top(sort_by(liquid, key="market_cap", order="desc"), 2)\n',
                        symbols_defined: ["close", "rsi14", "base", "liquid", "leaders"],
                        symbols_used: []
                      },
                      {
                        id: "sec_intent",
                        declared_kind: "intent",
                        effective_kind: "intent",
                        origin: "hybrid",
                        status: "ok",
                        start_line: 15,
                        end_line: 17,
                        snippet: '# intent\nif rsi14 < 30 {\n  emit Intent("open_long")\n}\n',
                        symbols_defined: [],
                        symbols_used: ["rsi14", "Intent"]
                      },
                      {
                        id: "sec_agent",
                        declared_kind: "agent",
                        effective_kind: "agent",
                        origin: "authored",
                        status: "ok",
                        start_line: 19,
                        end_line: 20,
                        snippet: '# agent\nrebalance(rank_weight(leaders, method="linear"), every="weekly")\n',
                        symbols_defined: [],
                        symbols_used: ["leaders"]
                      }
                    ],
                    edges: [
                      {
                        from: "sec_data",
                        to: "sec_intent",
                        relation: "dataflow",
                        reason: "intent_reads_data"
                      },
                      {
                        from: "sec_intent",
                        to: "sec_agent",
                        relation: "decision_flow",
                        reason: "agent_uses_intent"
                      }
                    ],
                    pool_pipeline: {
                      order: [
                        "source",
                        "eligibility",
                        "features",
                        "selection",
                        "weighting",
                        "rebalance"
                      ],
                      stages: [
                        {
                          kind: "source",
                          status: "present",
                          summary: "universe(exchange=binance, market=spot, quote=USDT)",
                          details: [
                            "exchange=binance",
                            "market=spot",
                            "quote=USDT"
                          ],
                          related_section_ids: ["sec_data"]
                        },
                        {
                          kind: "eligibility",
                          status: "present",
                          summary: "2 eligibility rule(s)",
                          details: [
                            "volume_24h >= 1000000000",
                            "listing_age_days >= 180"
                          ],
                          related_section_ids: ["sec_data"]
                        },
                        {
                          kind: "features",
                          status: "empty",
                          summary: "no derived feature defs yet",
                          details: [],
                          related_section_ids: ["sec_data"]
                        },
                        {
                          kind: "selection",
                          status: "present",
                          summary: "ordered_top_n by metadata.market_cap desc top 2",
                          details: [
                            "kind=ordered_top_n",
                            "key=metadata.market_cap",
                            "order=desc",
                            "count=2"
                          ],
                          related_section_ids: ["sec_data"]
                        },
                        {
                          kind: "weighting",
                          status: "present",
                          summary: "rank_weight (linear)",
                          details: ["kind=rank_weight", "method=linear"],
                          related_section_ids: ["sec_agent"]
                        },
                        {
                          kind: "rebalance",
                          status: "present",
                          summary: "rebalance weekly",
                          details: ["every=weekly"],
                          related_section_ids: ["sec_agent"]
                        }
                      ]
                    }
                  }
                }
              }
            }
          }
        }
      });
    });
  });

  afterEach(() => {
    act(() => {
      useGraphStore.setState(initialState, true);
    });
    vi.unstubAllGlobals();
  });

  it("renders source-order modules, flow view, and pool pipeline from backend authoring view", () => {
    render(<StrategyCodePanel />);

    expect(screen.getByTestId("qs-authoring-source-order")).toHaveTextContent(
      "源码顺序：风控 -> 执行 -> 数据 -> 意图 -> 代理"
    );
    expect(screen.getByTestId("qs-authoring-pipeline-order")).toHaveTextContent(
      "管线顺序：数据 -> 意图 -> 代理 -> 风控 -> 执行"
    );

    const poolCard = screen.getByTestId("qs-authoring-pool-pipeline");
    expect(poolCard).toHaveTextContent(
      "池顺序：来源 -> 资格 -> 特征 -> 选择 -> 权重 -> 再平衡"
    );
    expect(screen.getByTestId("authoring-pool-stage-source")).toHaveTextContent(
      "universe(exchange=binance, market=spot, quote=USDT)"
    );
    expect(screen.getByTestId("authoring-pool-stage-selection")).toHaveTextContent(
      "ordered_top_n by metadata.market_cap desc top 2"
    );
    expect(screen.getByTestId("authoring-pool-stage-weighting")).toHaveTextContent(
      "rank_weight (linear)"
    );
    expect(screen.getByTestId("authoring-pool-stage-rebalance")).toHaveTextContent(
      "rebalance weekly"
    );
  });

  it("falls back to partial authoring view when formal compile fails", () => {
    act(() => {
      useGraphStore.setState({
        compileResult: {
          backend_compile_error: {
            error: "quantscript_lowering_failed",
            message: "formal QuantScript lowering failed",
            details: [
              {
                code: "QPQSLOW011",
                message: "unsupported universe sort key",
                reason: "dynamic feature-ranked pool selection is not admitted yet"
              }
            ],
            partial_artifacts: {
              quantscript_authoring_view: {
                kind: "quantscript_authoring_view",
                source_hash: "qs_hash_partial_001",
                source_order: ["risk", "execution", "data", "intent", "agent"],
                pipeline_order: ["data", "intent", "agent", "risk", "execution"],
                sections: [
                  {
                    id: "sec_partial_data",
                    declared_kind: "data",
                    effective_kind: "mixed",
                    origin: "authored",
                    status: "partial",
                    start_line: 8,
                    end_line: 13,
                    snippet:
                      '# data\nbase = universe(exchange="binance", market="spot", quote="USDT")\nliquid = filter(base, min_volume_24h=1000000000, min_listing_age_days=180)\nranked = top(sort_by(liquid, key="factor_score", order="desc"), 3)\n',
                    symbols_defined: ["base", "liquid", "ranked"],
                    symbols_used: []
                  },
                  {
                    id: "sec_partial_agent",
                    declared_kind: "agent",
                    effective_kind: "mixed",
                    origin: "authored",
                    status: "partial",
                    start_line: 19,
                    end_line: 20,
                    snippet: '# agent\nrebalance(rank_weight(ranked, method="linear"), every="weekly")\n',
                    symbols_defined: [],
                    symbols_used: ["ranked"]
                  }
                ],
                edges: [],
                pool_pipeline: {
                  order: [
                    "source",
                    "eligibility",
                    "features",
                    "selection",
                    "weighting",
                    "rebalance"
                  ],
                  stages: [
                    {
                      kind: "source",
                      status: "present",
                      summary: "universe(exchange=binance, market=spot, quote=USDT)",
                      details: [],
                      related_section_ids: ["sec_partial_data"]
                    },
                    {
                      kind: "eligibility",
                      status: "present",
                      summary: "2 eligibility rule(s)",
                      details: [
                        "volume_24h >= 1000000000",
                        "listing_age_days >= 180"
                      ],
                      related_section_ids: ["sec_partial_data"]
                    },
                    {
                      kind: "features",
                      status: "empty",
                      summary: "no derived feature defs yet",
                      details: [],
                      related_section_ids: ["sec_partial_data"]
                    },
                    {
                      kind: "selection",
                      status: "present",
                      summary: "ordered_top_n by feature.factor_score desc top 3",
                      details: [
                        "kind=ordered_top_n",
                        "key=feature.factor_score",
                        "order=desc",
                        "count=3"
                      ],
                      related_section_ids: ["sec_partial_agent", "sec_partial_data"]
                    },
                    {
                      kind: "weighting",
                      status: "present",
                      summary: "rank_weight (linear)",
                      details: ["kind=rank_weight", "method=linear"],
                      related_section_ids: ["sec_partial_agent"]
                    },
                    {
                      kind: "rebalance",
                      status: "present",
                      summary: "rebalance weekly",
                      details: ["every=weekly"],
                      related_section_ids: ["sec_partial_agent"]
                    }
                  ]
                }
              }
            }
          }
        }
      });
    });

    render(<StrategyCodePanel />);

    expect(screen.getByTestId("qs-authoring-partial-state")).toHaveTextContent(
      "编译失败，已回退到部分工件"
    );
    expect(screen.getByTestId("qs-authoring-partial-state")).toHaveTextContent("QPQSLOW011");
    expect(screen.getByTestId("qs-authoring-source-order")).toHaveTextContent(
      "源码顺序：风控 -> 执行 -> 数据 -> 意图 -> 代理"
    );
    expect(screen.getByTestId("authoring-pool-stage-selection")).toHaveTextContent(
      "ordered_top_n by feature.factor_score desc top 3"
    );
    expect(screen.getAllByText("部分").length).toBeGreaterThan(0);
  });

  it("keeps source highlighting interactions when using partial authoring view fallback", async () => {
    act(() => {
      useGraphStore.setState({
        compileResult: {
          backend_compile_error: {
            error: "quantscript_lowering_failed",
            message: "formal QuantScript lowering failed",
            details: [
              {
                code: "QPQSLOW011",
                message: "unsupported universe sort key"
              }
            ],
            partial_artifacts: {
              quantscript_authoring_view: {
                kind: "quantscript_authoring_view",
                source_hash: "qs_hash_partial_002",
                source_order: ["risk", "execution", "data", "intent", "agent"],
                pipeline_order: ["data", "intent", "agent", "risk", "execution"],
                sections: [
                  {
                    id: "sec_partial_data",
                    declared_kind: "data",
                    effective_kind: "mixed",
                    origin: "authored",
                    status: "partial",
                    start_line: 8,
                    end_line: 13,
                    snippet:
                      '# data\nclose = fetch("close")\nrsi14 = rsi(close, 14)\nbase = universe(exchange="binance", market="spot", quote="USDT")\nleaders = top(sort_by(base, key="factor_score", order="desc"), 3)\n',
                    symbols_defined: ["close", "rsi14", "base", "leaders"],
                    symbols_used: []
                  },
                  {
                    id: "sec_partial_agent",
                    declared_kind: "agent",
                    effective_kind: "mixed",
                    origin: "authored",
                    status: "partial",
                    start_line: 19,
                    end_line: 20,
                    snippet: '# agent\nrebalance(rank_weight(leaders, method="linear"), every="weekly")\n',
                    symbols_defined: [],
                    symbols_used: ["leaders"]
                  }
                ],
                edges: [
                  {
                    from: "sec_partial_data",
                    to: "sec_partial_agent",
                    relation: "decision_flow",
                    reason: "agent_uses_intent"
                  }
                ],
                pool_pipeline: {
                  order: [
                    "source",
                    "eligibility",
                    "features",
                    "selection",
                    "weighting",
                    "rebalance"
                  ],
                  stages: [
                    {
                      kind: "source",
                      status: "present",
                      summary: "universe(exchange=binance, market=spot, quote=USDT)",
                      details: [],
                      related_section_ids: ["sec_partial_data"]
                    },
                    {
                      kind: "eligibility",
                      status: "empty",
                      summary: "no eligibility rules yet",
                      details: [],
                      related_section_ids: ["sec_partial_data"]
                    },
                    {
                      kind: "features",
                      status: "empty",
                      summary: "no derived feature defs yet",
                      details: [],
                      related_section_ids: ["sec_partial_data"]
                    },
                    {
                      kind: "selection",
                      status: "present",
                      summary: "ordered_top_n by feature.factor_score desc top 3",
                      details: [],
                      related_section_ids: ["sec_partial_data", "sec_partial_agent"]
                    },
                    {
                      kind: "weighting",
                      status: "present",
                      summary: "rank_weight (linear)",
                      details: [],
                      related_section_ids: ["sec_partial_agent"]
                    },
                    {
                      kind: "rebalance",
                      status: "present",
                      summary: "rebalance weekly",
                      details: [],
                      related_section_ids: ["sec_partial_agent"]
                    }
                  ]
                }
              }
            }
          }
        }
      });
    });

    render(<StrategyCodePanel />);

    const sourceEditor = screen.getByLabelText("Formal QuantScript");

    fireEvent.click(screen.getByTestId("authoring-section-highlight-sec_partial_data"));
    await waitFor(() => {
      const selectedText = sourceEditor.value.slice(
        sourceEditor.selectionStart,
        sourceEditor.selectionEnd
      );
      expect(selectedText).toContain("# data");
      expect(selectedText).toContain('leaders = top(sort_by(liquid, key="market_cap", order="desc"), 2)');
    });

    fireEvent.click(screen.getByTestId("authoring-edge-sec_partial_data-sec_partial_agent"));
    await waitFor(() => {
      const selectedText = sourceEditor.value.slice(
        sourceEditor.selectionStart,
        sourceEditor.selectionEnd
      );
      expect(selectedText).toContain("# data");
      expect(selectedText).toContain("# agent");
    });

    fireEvent.click(screen.getByTestId("authoring-pool-stage-highlight-selection"));
    await waitFor(() => {
      const selectedText = sourceEditor.value.slice(
        sourceEditor.selectionStart,
        sourceEditor.selectionEnd
      );
      expect(selectedText).toContain("# data");
      expect(selectedText).toContain("# agent");
    });
  });

  it("highlights graph source when clicking sections, stages, edges, and pool stages", async () => {
    render(<StrategyCodePanel />);

    const sourceEditor = screen.getByLabelText("Formal QuantScript");
    expect(sourceEditor.selectionStart).toBe(0);
    expect(sourceEditor.selectionEnd).toBe(0);

    fireEvent.click(screen.getByTestId("authoring-section-highlight-sec_data"));
    await waitFor(() => {
      const selectedText = sourceEditor.value.slice(
        sourceEditor.selectionStart,
        sourceEditor.selectionEnd
      );
      expect(selectedText).toContain("# data");
      expect(selectedText).toContain('close = fetch("close")');
      expect(selectedText).toContain("rsi14 = rsi(close, 14)");
      expect(selectedText).toContain('leaders = top(sort_by(liquid, key="market_cap", order="desc"), 2)');
    });

    fireEvent.click(screen.getByTestId("authoring-stage-intent"));
    await waitFor(() => {
      const selectedText = sourceEditor.value.slice(
        sourceEditor.selectionStart,
        sourceEditor.selectionEnd
      );
      expect(selectedText).toContain("# intent");
      expect(selectedText).toContain("if rsi14 < 30");
      expect(selectedText).toContain('emit Intent("open_long")');
    });

    fireEvent.click(screen.getByTestId("authoring-edge-sec_data-sec_intent"));
    await waitFor(() => {
      const selectedText = sourceEditor.value.slice(
        sourceEditor.selectionStart,
        sourceEditor.selectionEnd
      );
      expect(selectedText).toContain("# data");
      expect(selectedText).toContain("# intent");
    });

    fireEvent.click(screen.getByTestId("authoring-pool-stage-highlight-selection"));
    await waitFor(() => {
      const selectedText = sourceEditor.value.slice(
        sourceEditor.selectionStart,
        sourceEditor.selectionEnd
      );
      expect(selectedText).toContain("# data");
      expect(selectedText).toContain('sort_by(liquid, key="market_cap", order="desc")');
      expect(selectedText).toContain('leaders = top(sort_by(liquid, key="market_cap", order="desc"), 2)');
    });
  });

  it("uses the applied formal source lane for failed-compile fallback and highlighting", async () => {
    const failingSource = `fn strategy() {
    # risk
    risk.profile("global", max_drawdown=0.12)

    # execution
    execution.profile("paper", fee_bps=10.0)

    # data
    close = fetch("close")
    rsi14 = rsi(close, 14)
    base = universe(exchange="binance", market="spot", quote="USDT")
    liquid = filter(base, min_volume_24h=1000000000, min_listing_age_days=180)
    ranked = top(sort_by(liquid, key="factor_score", order="desc"), 3)

    # intent
    if rsi14 < 30 {
      emit Intent("open_long")
    }

    # agent
    rebalance(rank_weight(ranked, method="linear"), every="weekly")
}`;

    const partialArtifactsPayload = {
      quantscript_authoring_view: {
        kind: "quantscript_authoring_view",
        source_hash: "qs_hash_partial_formal_lane",
        source_order: ["risk", "execution", "data", "intent", "agent"],
        pipeline_order: ["data", "intent", "agent", "risk", "execution"],
        sections: [
          {
            id: "sec_partial_data",
            declared_kind: "data",
            effective_kind: "mixed",
            origin: "authored",
            status: "partial",
            start_line: 8,
            end_line: 13,
            snippet:
              '# data\nclose = fetch("close")\nrsi14 = rsi(close, 14)\nbase = universe(exchange="binance", market="spot", quote="USDT")\nliquid = filter(base, min_volume_24h=1000000000, min_listing_age_days=180)\nranked = top(sort_by(liquid, key="factor_score", order="desc"), 3)\n',
            symbols_defined: ["close", "rsi14", "base", "liquid", "ranked"],
            symbols_used: []
          },
          {
            id: "sec_partial_agent",
            declared_kind: "agent",
            effective_kind: "mixed",
            origin: "authored",
            status: "partial",
            start_line: 19,
            end_line: 20,
            snippet: '# agent\nrebalance(rank_weight(ranked, method="linear"), every="weekly")\n',
            symbols_defined: [],
            symbols_used: ["ranked"]
          }
        ],
        edges: [
          {
            from: "sec_partial_data",
            to: "sec_partial_agent",
            relation: "decision_flow",
            reason: "pool_selection_flows_into_agent_rebalance"
          }
        ],
        pool_pipeline: {
          order: ["source", "eligibility", "features", "selection", "weighting", "rebalance"],
          stages: [
            {
              kind: "source",
              status: "present",
              summary: "universe(exchange=binance, market=spot, quote=USDT)",
              details: [],
              related_section_ids: ["sec_partial_data"]
            },
            {
              kind: "eligibility",
              status: "present",
              summary: "2 eligibility rule(s)",
              details: ["volume_24h >= 1000000000", "listing_age_days >= 180"],
              related_section_ids: ["sec_partial_data"]
            },
            {
              kind: "features",
              status: "empty",
              summary: "no derived feature defs yet",
              details: [],
              related_section_ids: ["sec_partial_data"]
            },
            {
              kind: "selection",
              status: "present",
              summary: "ordered_top_n by feature.factor_score desc top 3",
              details: [
                "kind=ordered_top_n",
                "key=feature.factor_score",
                "order=desc",
                "count=3"
              ],
              related_section_ids: ["sec_partial_data", "sec_partial_agent"]
            },
            {
              kind: "weighting",
              status: "present",
              summary: "rank_weight (linear)",
              details: ["kind=rank_weight", "method=linear"],
              related_section_ids: ["sec_partial_agent"]
            },
            {
              kind: "rebalance",
              status: "present",
              summary: "rebalance weekly",
              details: ["every=weekly"],
              related_section_ids: ["sec_partial_agent"]
            }
          ]
        }
      }
    };

    const compileCurrentGraphMock = vi.fn(async () => {
      expect(useGraphStore.getState().formalQuantScriptOverride).toContain(
        'sort_by(liquid, key="factor_score", order="desc")'
      );
      act(() => {
        useGraphStore.setState((state) => ({
          compileResult: {
            ...(state.compileResult || {}),
            backend_compile_error: {
              error: "quantscript_lowering_failed",
              message: "formal QuantScript lowering failed",
              details: [
                {
                  code: "QPQSLOW011",
                  message: "unsupported universe sort key",
                  reason: "dynamic feature-ranked pool selection is not admitted yet"
                }
              ],
              partial_artifacts: partialArtifactsPayload
            }
          }
        }));
      });
      return null;
    });
    act(() => {
      useGraphStore.setState({
        compileCurrentGraph: compileCurrentGraphMock
      });
    });

    render(<StrategyCodePanel />);

    const sourceEditor = screen.getByLabelText("Formal QuantScript");
    fireEvent.change(sourceEditor, { target: { value: failingSource } });
    await waitFor(() => {
      expect(sourceEditor.value).toContain('sort_by(liquid, key="factor_score", order="desc")');
      expect(useGraphStore.getState().formalQuantScriptDraft).toContain(
        'sort_by(liquid, key="factor_score", order="desc")'
      );
    });
    fireEvent.click(screen.getByRole("button", { name: "应用 Formal QuantScript" }));

    expect(useGraphStore.getState().formalQuantScriptOverride).toContain(
      'sort_by(liquid, key="factor_score", order="desc")'
    );

    await act(async () => {
      await useGraphStore.getState().compileCurrentGraph();
    });

    expect(compileCurrentGraphMock).toHaveBeenCalledTimes(1);
    expect(useGraphStore.getState().compileResult?.backend_compile_error?.partial_artifacts).toEqual(
      partialArtifactsPayload
    );
    expect(screen.getByTestId("qs-authoring-partial-state")).toHaveTextContent(
      "编译失败，已回退到部分工件"
    );

    fireEvent.click(screen.getByTestId("authoring-section-highlight-sec_partial_data"));
    await waitFor(() => {
      const selectedText = sourceEditor.value.slice(
        sourceEditor.selectionStart,
        sourceEditor.selectionEnd
      );
      expect(selectedText).toContain("# data");
      expect(selectedText).toContain('ranked = top(sort_by(liquid, key="factor_score", order="desc"), 3)');
    });

    fireEvent.click(screen.getByTestId("authoring-edge-sec_partial_data-sec_partial_agent"));
    await waitFor(() => {
      const selectedText = sourceEditor.value.slice(
        sourceEditor.selectionStart,
        sourceEditor.selectionEnd
      );
      expect(selectedText).toContain("# data");
      expect(selectedText).toContain("# agent");
    });

    fireEvent.click(screen.getByTestId("authoring-pool-stage-highlight-selection"));
    await waitFor(() => {
      const selectedText = sourceEditor.value.slice(
        sourceEditor.selectionStart,
        sourceEditor.selectionEnd
      );
      expect(selectedText).toContain("# data");
      expect(selectedText).toContain("# agent");
      expect(selectedText).toContain('sort_by(liquid, key="factor_score", order="desc")');
    });
  });
});
