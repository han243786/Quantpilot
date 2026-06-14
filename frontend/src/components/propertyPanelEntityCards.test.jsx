import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import {
  ActionableValidationCard,
  EdgeOverviewCard,
  NodeConfigCard,
  NodeMetricsCard,
  NodeOverviewCard,
  NodeRuntimeCard
} from "./propertyPanelEntityCards";

const selectedNode = {
  id: "agent_1",
  type: "agent",
  name: "Allocator",
  module_key: "builtin.agent.allocator",
  config: {
    target_weight: 0.4
  },
  runtime_state: {
    status: "running",
    last_event_type: "rebalance",
    last_message: "Order submitted",
    last_event_time: "2026-05-01T12:00:00Z"
  }
};

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

describe("propertyPanelEntityCards", () => {
  it("keeps node identity and config edits wired to parent handlers", () => {
    const updateNodeName = vi.fn();
    const updateNodeConfig = vi.fn();

    render(
      <div>
        <NodeOverviewCard
          selectedNode={selectedNode}
          moduleDef={moduleDef}
          updateNodeName={updateNodeName}
        />
        <NodeConfigCard
          selectedNode={selectedNode}
          moduleDef={moduleDef}
          updateNodeConfig={updateNodeConfig}
        />
      </div>
    );

    fireEvent.change(screen.getByTestId("prop-input-node-name"), {
      target: { value: "Allocator Prime" }
    });
    fireEvent.change(screen.getByTestId("prop-input-target_weight"), {
      target: { value: "0.6" }
    });

    expect(updateNodeName).toHaveBeenCalledWith("agent_1", "Allocator Prime");
    expect(updateNodeConfig).toHaveBeenCalledWith("agent_1", "target_weight", 0.6);
  });

  it("routes actionable validation issues to their target cards", () => {
    const onSelectIssue = vi.fn();
    const issue = {
      id: "missing-weight",
      code: "FIELD_REQUIRED",
      level: "error",
      message: "Target weight is required.",
      hint: "Fill target_weight before compile."
    };

    render(<ActionableValidationCard issues={[issue]} onSelectIssue={onSelectIssue} />);

    fireEvent.click(screen.getByRole("button", { name: /Target weight is required/ }));

    expect(onSelectIssue).toHaveBeenCalledWith(issue, "config");
  });

  it("renders runtime, metrics, and edge overview cards", () => {
    const removeSelected = vi.fn();

    render(
      <div>
        <NodeRuntimeCard selectedNode={selectedNode} />
        <NodeMetricsCard metrics={[["orders", 2]]} />
        <EdgeOverviewCard
          selectedEdge={{
            source_port: "signals",
            target_port: "orders"
          }}
          sourceNode={{ name: "Signal" }}
          targetNode={{ name: "Execution" }}
          removeSelected={removeSelected}
        />
      </div>
    );

    expect(screen.getByText("rebalance")).toBeInTheDocument();
    expect(screen.getByText("Order submitted")).toBeInTheDocument();
    expect(screen.getByText("orders")).toBeInTheDocument();
    expect(screen.getByText("2")).toBeInTheDocument();

    fireEvent.click(screen.getByTestId("prop-action-delete-edge"));

    expect(removeSelected).toHaveBeenCalledTimes(1);
  });
});
