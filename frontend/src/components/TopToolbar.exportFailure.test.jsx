import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { act, fireEvent, render, screen } from "@testing-library/react";
import TopToolbar from "./TopToolbar";
import { I18nProvider } from "../i18n";
import { useGraphStore } from "../store/graphStore";

function buildGraph(overrides = {}) {
  return {
    metadata: {
      name: "Export Failure Graph",
      graph_id: "export_failure_graph",
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

describe("TopToolbar export config failure notice", () => {
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
        saveGraph: vi.fn(),
        loadLatestGraph: vi.fn(),
        exportRuntimeConfig: vi.fn(async () => ({
          compile_summary: {
            compilable: false,
            errors: ["Runtime compile rejected the generated output."]
          }
        })),
        exportQuantScript: vi.fn(() => "graph export_failure_graph"),
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

  it("shows export config failures as reason plus next action", async () => {
    render(<I18nProvider><TopToolbar /></I18nProvider>);

    await act(async () => {
      fireEvent.click(screen.getByRole("button", { name: "导出运行配置" }));
    });

    expect(screen.getByRole("status")).toHaveTextContent(
      "原因：Runtime compile rejected the generated output 后续：检查 compile diagnostics，并确认运行时编译成功后再重新导出 runtime_config关闭"
    );
  });
});
