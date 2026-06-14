import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { CompileSummaryCard, GraphOverviewCard } from "./propertyPanelCompileSourceCards";

const graph = {
  metadata: {
    name: "Momentum Graph",
    graph_id: "graph_momentum",
    source_mode: "formal"
  },
  nodes: [{ id: "data" }, { id: "exec" }],
  edges: [{ id: "edge_data_exec" }],
  validation_state: {
    issue_counts: {
      error: 1,
      warning: 2
    }
  }
};

const compileSummary = {
  compilable: true,
  last_compile_id: "compile_ok",
  backend_verified: true,
  protocol_name: "quantpilot/runtime-config/v1",
  config_hash: "cfg_ok",
  outputs: {
    data_sources: 1,
    executions: 1
  },
  errors: [],
  warnings: ["Risk threshold is permissive."],
  diagnostics: [{ severity: "warning" }, { severity: "info" }],
  strategy_ir_check: {
    performed: true,
    compilable: true,
    compile_id: "compile_ir_ok",
    has_core_ir: true
  },
  artifact_resolution: {
    strategy_ir_role_label: "semantic preflight",
    runtime_source_label: "runtime config",
    source_of_truth_label: "runtime compile",
    notes: ["Strategy IR does not override runtime output."]
  }
};

describe("propertyPanelCompileSourceCards", () => {
  it("renders graph overview counts and compile status", () => {
    const { container } = render(
      <GraphOverviewCard graph={graph} compileSummary={compileSummary} />
    );

    expect(screen.getByText("Momentum Graph")).toBeInTheDocument();
    expect(screen.getByText("graph_momentum")).toBeInTheDocument();
    expect(container).toHaveTextContent("来源模式：formal");
    expect(container).toHaveTextContent("节点数2");
    expect(container).toHaveTextContent("边数1");
    expect(container).toHaveTextContent("错误1");
    expect(container).toHaveTextContent("警告2");
  });

  it("renders compile summary source-of-truth details", () => {
    render(<CompileSummaryCard compileSummary={compileSummary} />);

    const card = screen.getByTestId("compile-summary-card");

    expect(card).toHaveTextContent("compile_ok");
    expect(card).toHaveTextContent("cfg_ok");
    expect(card).toHaveTextContent("compile_ir_ok");
    expect(card).toHaveTextContent("semantic preflight");
    expect(card).toHaveTextContent("runtime config");
    expect(card).toHaveTextContent("runtime compile");
    expect(card).toHaveTextContent("Strategy IR does not override runtime output.");
  });
});
