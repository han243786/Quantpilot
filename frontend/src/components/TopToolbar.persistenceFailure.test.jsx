import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { act, fireEvent, render, screen } from "@testing-library/react";
import TopToolbar from "./TopToolbar";
import { I18nProvider } from "../i18n";
import { useGraphStore } from "../store/graphStore";

function buildGraph(overrides = {}) {
  return {
    metadata: {
      name: "Persistence Failure Graph",
      graph_id: "persistence_failure_graph",
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

describe("TopToolbar save/load failure notices", () => {
  const initialState = useGraphStore.getState();

  beforeEach(() => {
    act(() => {
      useGraphStore.setState(initialState, true);
      useGraphStore.setState({
        graph: buildGraph(),
        runtime: {
          ...useGraphStore.getState().runtime,
          status: "idle"
        },
        capabilityStatus: "ready",
        capabilitySource: "remote",
        capabilityMessage: "",
        exportRuntimeConfig: vi.fn(async () => ({
          compile_summary: { compilable: true },
          runtime_config: { mode: "paper" }
        })),
        exportQuantScript: vi.fn(() => "graph persistence_failure_graph"),
        compileCurrentGraph: vi.fn(),
        startV4Simulation: vi.fn(),
        startBacktest: vi.fn(),
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

  it("shows save graph failures as reason plus next action", async () => {
    act(() => {
      useGraphStore.setState({
        saveGraph: vi.fn(async () => {
          throw new Error("backend unavailable");
        }),
        loadLatestGraph: vi.fn()
      });
    });

    render(<I18nProvider><TopToolbar /></I18nProvider>);
    await act(async () => {
      fireEvent.click(screen.getByRole("button", { name: "保存策略图" }));
    });

    expect(screen.getByRole("status")).toHaveTextContent(
      "原因：backend unavailable 后续：检查当前策略图校验结果和后端可用性后，再重新保存策略图关闭"
    );
  });

  it("shows load latest failures as reason plus next action", async () => {
    act(() => {
      useGraphStore.setState({
        saveGraph: vi.fn(),
        loadLatestGraph: vi.fn(async () => {
          throw new Error("Latest saved strategy graph is unavailable.");
        })
      });
    });

    render(<I18nProvider><TopToolbar /></I18nProvider>);
    await act(async () => {
      fireEvent.click(screen.getByRole("button", { name: "加载最新" }));
    });

    expect(screen.getByRole("status")).toHaveTextContent(
      "原因：Latest saved strategy graph is unavailable 后续：检查后端可用性以及是否存在已保存的可运行策略图后，再重新加载最新图关闭"
    );
  });
});
