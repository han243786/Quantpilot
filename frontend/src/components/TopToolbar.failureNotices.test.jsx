import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { act, fireEvent, render, screen } from "@testing-library/react";
import TopToolbar from "./TopToolbar";
import { I18nProvider } from "../i18n";
import { useGraphStore } from "../store/graphStore";

const TEST_V4_SOURCE = "v4_strategy toolbar.failure { machine observe {} }";

function buildGraph(overrides = {}) {
  return {
    metadata: {
      name: "Failure Notice Graph",
      graph_id: "failure_notice_graph",
      runtime_kind: "v4",
      artifacts: {
        quantscript: {
          formal_source: TEST_V4_SOURCE
        }
      },
      ...(overrides.metadata || {})
    },
    nodes: [],
    edges: [],
    validation_state: {
      is_valid: true,
      is_runnable: true,
      issue_counts: { error: 0, warning: 0, info: 0 },
      graph_issues: [],
      node_issues: {},
      edge_issues: {},
      ...(overrides.validation_state || {})
    },
    compile_summary: {},
    ...overrides
  };
}

describe("TopToolbar failure notices", () => {
  const initialState = useGraphStore.getState();

  beforeEach(() => {
    act(() => {
      useGraphStore.setState(initialState, true);
      useGraphStore.setState({
        graph: buildGraph(),
        runtime: {
          ...useGraphStore.getState().runtime,
          status: "idle",
          backendError: null
        },
        capabilityStatus: "ready",
        capabilitySource: "remote",
        capabilityMessage: "",
        saveGraph: vi.fn(),
        loadLatestGraph: vi.fn(),
        exportRuntimeConfig: vi.fn(() => ({
          compile_summary: { compilable: true },
          runtime_config: { mode: "paper" }
        })),
        exportQuantScript: vi.fn(() => "graph failure_notice_graph"),
        stopRuntime: vi.fn(),
        resetRuntime: vi.fn(),
        resetGraph: vi.fn(),
        setSelectedNode: vi.fn(),
        setSelectedEdge: vi.fn()
      });
    });
  });

  afterEach(() => {
    act(() => {
      useGraphStore.setState(initialState, true);
    });
  });

  it("shows compile failures as reason plus next action", async () => {
    act(() => {
      useGraphStore.setState({
        compileCurrentGraph: vi.fn(async () => {
          useGraphStore.setState((state) => ({
            graph: {
              ...state.graph,
              compile_summary: {
                ...state.graph.compile_summary,
                errors: ["Runtime compile rejected the generated output."]
              }
            }
          }));
          return null;
        }),
        startV4Simulation: vi.fn(),
        startBacktest: vi.fn()
      });
    });

    render(<I18nProvider><TopToolbar /></I18nProvider>);
    await act(async () => {
      fireEvent.click(screen.getByRole("button", { name: "编译" }));
    });

    expect(screen.getByRole("status")).toHaveTextContent(
      "原因：Runtime compile rejected the generated output 后续：检查编译诊断信息，确认策略图节点配置完整且参数有效后重新编译关闭"
    );
  });

  it("shows simulation failures as reason plus next action", async () => {
    act(() => {
      useGraphStore.setState({
        compileCurrentGraph: vi.fn(async () => ({ runtime_config: {}, compile_id: "compile_ok" })),
        startV4Simulation: vi.fn(async () => {
          useGraphStore.setState((state) => ({
            runtime: {
              ...state.runtime,
              status: "error",
              backendError:
                "Capability rejected: runtime mode live is not enabled for this beta backend."
            }
          }));
        }),
        startBacktest: vi.fn()
      });
    });

    render(<I18nProvider><TopToolbar /></I18nProvider>);
    await act(async () => {
      fireEvent.click(screen.getByRole("button", { name: "启动模拟" }));
    });

    expect(screen.getByRole("status")).toHaveTextContent(
      "原因：Capability rejected: runtime mode live is not enabled for this beta backend 后续：检查编译结果、运行模式、执行模块和当前 capability 配置后，再重新启动模拟运行关闭"
    );
  });

  it("shows backtest failures as reason plus next action", async () => {
    act(() => {
      useGraphStore.setState({
        compileCurrentGraph: vi.fn(async () => ({ runtime_config: {}, compile_id: "compile_ok" })),
        startV4Simulation: vi.fn(),
        startBacktest: vi.fn(async () => {
          useGraphStore.setState((state) => ({
            runtime: {
              ...state.runtime,
              status: "error",
              backendError:
                "Capability rejected: symbol XRPUSDT is outside the current beta market-data profile."
            }
          }));
        })
      });
    });

    render(<I18nProvider><TopToolbar /></I18nProvider>);
    await act(async () => {
      fireEvent.click(screen.getByRole("button", { name: "运行回测" }));
    });

    expect(screen.getByRole("status")).toHaveTextContent(
      "原因：Capability rejected: symbol XRPUSDT is outside the current beta market-data profile 后续：检查编译结果、回放来源、市场数据边界和当前 capability 配置后，再重新运行回测关闭"
    );
  });
});
