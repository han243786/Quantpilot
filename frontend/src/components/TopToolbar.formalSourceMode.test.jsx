import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { act, render, screen } from "@testing-library/react";
import TopToolbar from "./TopToolbar";
import { useGraphStore } from "../store/graphStore";

function buildGraph(overrides = {}) {
  return {
    metadata: {
      name: "Formal Source Mode Graph",
      graph_id: "formal_source_mode_graph",
      artifacts: {
        quantscript: {
          formal_source: "fn strategy() {}"
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

// v0.5.0: General_Policy §2.1 — 用户可见字符串全中文

describe("TopToolbar formal compile source mode", () => {
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
        exportRuntimeConfig: vi.fn(),
        exportQuantScript: vi.fn(),
        compileCurrentGraph: vi.fn(),
        startRuntime: vi.fn(),
        startBacktest: vi.fn(),
        stopRuntime: vi.fn(),
        resetRuntime: vi.fn(),
        resetGraph: vi.fn(),
        setSelectedNode: vi.fn(),
        setSelectedEdge: vi.fn(),
        formalQuantScriptOverride: null
      });
    });
  });

  afterEach(() => {
    act(() => {
      useGraphStore.setState(initialState, true);
    });
  });

  it("shows graph-generated formal source when no override is applied", () => {
    render(<TopToolbar variant="workspace" />);

    expect(screen.getByTestId("toolbar-formal-source-pill")).toHaveTextContent(
      "正式源码: 图谱生成"
    );
    expect(screen.getByTestId("toolbar-compile-action")).toHaveAttribute(
      "title",
      expect.stringContaining("图谱生成")
    );
  });

  it("shows applied formal override when the override lane is active", () => {
    act(() => {
      useGraphStore.setState({
        formalQuantScriptOverride: "fn strategy() {\n  emit Intent(\"buy\")\n}"
      });
    });

    render(<TopToolbar variant="workspace" />);

    expect(screen.getByTestId("toolbar-formal-source-pill")).toHaveTextContent(
      "正式源码: 覆盖"
    );
    expect(screen.getByTestId("toolbar-compile-action")).toHaveAttribute(
      "title",
      expect.stringContaining("覆盖")
    );
  });
});
