import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { act, render, screen } from "@testing-library/react";
import TopToolbar from "./TopToolbar";
import { I18nProvider } from "../i18n";
import { useGraphStore } from "../store/graphStore";

function buildGraph(overrides = {}) {
  return {
    metadata: {
      name: "Capability Test Graph",
      graph_id: "capability_test_graph",
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

describe("TopToolbar capability fallback UI", () => {
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
        saveGraph: vi.fn(),
        loadLatestGraph: vi.fn(),
        exportRuntimeConfig: vi.fn(() => ({
          compile_summary: { compilable: true },
          runtime_config: { mode: "paper" }
        })),
        exportQuantScript: vi.fn(() => "graph capability_test_graph"),
        compileCurrentGraph: vi.fn(),
        startRuntime: vi.fn(),
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

  function expectPrimaryActionsDisabled() {
    expect(screen.getByTestId("toolbar-compile-action")).toBeDisabled();
    expect(screen.getByTestId("toolbar-export-runtime-config-action")).toBeDisabled();
    expect(screen.getByTestId("toolbar-start-runtime-action")).toBeDisabled();
    expect(screen.getByTestId("toolbar-start-backtest-action")).toBeDisabled();
  }

  function expectPrimaryActionsEnabled() {
    expect(screen.getByTestId("toolbar-compile-action")).toBeEnabled();
    expect(screen.getByTestId("toolbar-export-runtime-config-action")).toBeEnabled();
    expect(screen.getByTestId("toolbar-start-runtime-action")).toBeEnabled();
    expect(screen.getByTestId("toolbar-start-backtest-action")).toBeEnabled();
  }

  it("shows safe fallback banner and disables risky actions", () => {
    act(() => {
      useGraphStore.setState({
        capabilityStatus: "error",
        capabilitySource: "safe_fallback",
        capabilityMessage:
          "Capability fetch failed. Entering safe fallback mode. To avoid exposing fake capabilities, module visibility and compile/run actions were tightened to the safest profile."
      });
    });

    render(<I18nProvider><TopToolbar /></I18nProvider>);

    expect(screen.getByTestId("toolbar-capability-alert")).toHaveTextContent(
      /to avoid exposing fake capabilities/i
    );
    expectPrimaryActionsDisabled();
  });

  it("shows cache fallback warning with compile allowed (v3.5.0: cache does not block)", () => {
    act(() => {
      useGraphStore.setState({
        capabilityStatus: "degraded",
        capabilitySource: "cache",
        capabilityMessage:
          "Capability fetch failed. Using the latest cached capability snapshot. Final availability still depends on live backend validation."
      });
    });

    render(<I18nProvider><TopToolbar /></I18nProvider>);

    expect(screen.getByTestId("toolbar-capability-alert")).toHaveTextContent(
      /latest cached capability snapshot/i
    );
    // v3.5.0: 缓存/降级模式不再阻断编译
    expect(screen.getByTestId("toolbar-compile-action")).toBeEnabled();
  });

  it("shows syncing banner and locks actions while capabilities are loading", () => {
    act(() => {
      useGraphStore.setState({
        capabilityStatus: "loading",
        capabilitySource: "remote",
        capabilityMessage: ""
      });
    });

    render(<I18nProvider><TopToolbar /></I18nProvider>);

    expect(screen.getByTestId("toolbar-capability-alert")).toHaveTextContent("前端正在同步后端能力快照");
    expectPrimaryActionsDisabled();
  });
});
