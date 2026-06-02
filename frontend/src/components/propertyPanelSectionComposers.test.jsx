import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { GraphConfigSection, LaneAwareNodeParamsSection } from "./propertyPanelSectionComposers";

const graph = {
  metadata: {
    name: "Momentum Graph",
    graph_id: "graph_momentum",
    source_mode: "formal"
  },
  nodes: [
    {
      id: "data_1",
      name: "Market Data",
      module_key: "builtin.data.market",
      config: {},
      runtime_state: { status: "idle" }
    },
    {
      id: "agent_1",
      type: "agent",
      name: "Allocator",
      module_key: "builtin.agent.allocator",
      config: {
        target_weight: 0.4
      },
      runtime_state: { status: "running" }
    }
  ],
  edges: [
    {
      id: "edge_data_agent",
      source_node_id: "data_1",
      target_node_id: "agent_1",
      source_port: "bars",
      target_port: "signals"
    }
  ],
  validation_state: {
    issue_counts: {
      error: 1,
      warning: 0
    }
  }
};

const compileSummary = {
  compilable: true,
  last_compile_id: "compile_ok",
  backend_verified: true,
  protocol_name: "quantpilot/runtime-config/v1",
  config_hash: "cfg_ok",
  outputs: {},
  errors: [],
  warnings: [],
  diagnostics: []
};

const selectedNode = graph.nodes[1];

const moduleDef = {
  display_name: "Allocator Agent",
  category: "agent",
  config_schema: {
    fields: [
      {
        key: "target_weight",
        label: "Target weight",
        type: "number"
      }
    ]
  }
};

describe("propertyPanelSectionComposers", () => {
  it("composes graph configuration from the graph overview card", () => {
    render(<GraphConfigSection model={{ graph, compileSummary }} />);

    expect(screen.getByTestId("property-section-graph-config")).toBeInTheDocument();
    expect(screen.getByText("Momentum Graph")).toBeInTheDocument();
    expect(screen.getByText("graph_momentum")).toBeInTheDocument();
  });

  it("keeps lane-aware node parameters wired through child cards", () => {
    const updateNodeName = vi.fn();
    const updateNodeConfig = vi.fn();
    const removeSelected = vi.fn();

    const { container } = render(
      <LaneAwareNodeParamsSection
        model={{
          graph,
          selectedNode,
          moduleDef,
          nodeIssues: [
            {
              id: "missing-target",
              code: "FIELD_REQUIRED",
              level: "error",
              message: "Target weight is required.",
              hint: "Fill target_weight before compiling."
            }
          ],
          updateNodeName,
          updateNodeConfig,
          removeSelected
        }}
        prioritizePathFields
      />
    );

    const configAnchor = container.querySelector('[data-configure-card="config"]');
    expect(configAnchor).toBeInTheDocument();

    fireEvent.click(screen.getByRole("button", { name: /Target weight is required/i }));
    expect(configAnchor).toHaveClass("configure-card-anchor--active");

    fireEvent.change(screen.getByTestId("prop-input-node-name"), {
      target: { value: "Allocator Prime" }
    });
    fireEvent.change(screen.getByTestId("prop-input-target_weight"), {
      target: { value: "0.6" }
    });

    expect(updateNodeName).toHaveBeenCalledWith("agent_1", "Allocator Prime");
    expect(updateNodeConfig).toHaveBeenCalledWith("agent_1", "target_weight", 0.6);
  });
});
