import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { act, fireEvent, render, screen, within } from "@testing-library/react";
import ModuleSidebar from "./ModuleSidebar";
import { useGraphStore } from "../store/graphStore";

function createRegistry(items) {
  return {
    getAll() {
      return items;
    }
  };
}

describe("ModuleSidebar capability visibility", () => {
  const initialState = useGraphStore.getState();

  beforeEach(() => {
    act(() => {
      useGraphStore.setState(initialState, true);
    });
  });

  afterEach(() => {
    act(() => {
      useGraphStore.setState(initialState, true);
    });
  });

  it("shows unsupported modules as disabled cards with an explicit reason", () => {
    const createNode = vi.fn();
    act(() => {
      useGraphStore.setState({
        registry: createRegistry([
          {
            module_key: "supported.module",
            category: "data",
            display_name: "Supported module",
            description: "Supported path",
            availability: { status: "supported", reason: "" }
          },
          {
            module_key: "unsupported.module",
            category: "agent",
            display_name: "Restricted module",
            description: "Unsupported path",
            availability: {
              status: "unsupported",
              reason: "This module is outside the current backend capability boundary."
            }
          }
        ]),
        createNode,
        capabilityStatus: "ready",
        capabilitySource: "remote",
        capabilityMessage: ""
      });
    });

    render(<ModuleSidebar />);

    const supportedButton = screen.getByTestId("module-card-supported.module");
    const unsupportedButton = screen.getByTestId("module-card-unsupported.module");

    expect(supportedButton).toBeEnabled();
    expect(unsupportedButton).toBeDisabled();
    expect(screen.getByTestId("module-card-note-unsupported.module")).toHaveTextContent(
      "This module is outside the current backend capability boundary."
    );

    fireEvent.click(supportedButton);
    expect(createNode).toHaveBeenCalledWith("supported.module");
  });

  it("locks module creation during capability sync with a shared reason", () => {
    act(() => {
      useGraphStore.setState({
        registry: createRegistry([
          {
            module_key: "sync.module",
            category: "data",
            display_name: "Sync module",
            description: "Sync path",
            availability: { status: "supported", reason: "" }
          }
        ]),
        createNode: vi.fn(),
        capabilityStatus: "loading",
        capabilitySource: "remote",
        capabilityMessage: ""
      });
    });

    render(<ModuleSidebar />);

    const syncButton = screen.getByTestId("module-card-sync.module");
    expect(syncButton).toBeDisabled();
    expect(screen.getByTestId("module-card-note-sync.module")).toBeInTheDocument();
  });

  it("shows a visible empty state when the current search filters out every module", () => {
    act(() => {
      useGraphStore.setState({
        registry: createRegistry([
          {
            module_key: "data.module",
            category: "data",
            display_name: "Data module",
            description: "Market feed",
            availability: { status: "supported", reason: "" }
          }
        ]),
        createNode: vi.fn(),
        capabilityStatus: "ready",
        capabilitySource: "remote",
        capabilityMessage: ""
      });
    });

    const { container } = render(<ModuleSidebar />);
    const searchInput = screen.getByTestId("module-sidebar-search");
    fireEvent.change(searchInput, { target: { value: "missing" } });

    expect(container.querySelector(".module-sidebar-empty")).not.toBeNull();
  });

  it("supports collapsing groups and auto-expands matches while searching", () => {
    act(() => {
      useGraphStore.setState({
        registry: createRegistry([
          {
            module_key: "data.alpha",
            category: "data",
            display_name: "Alpha feed",
            description: "Fast feed",
            availability: { status: "supported", reason: "" }
          },
          {
            module_key: "agent.beta",
            category: "agent",
            display_name: "Beta agent",
            description: "Allocator",
            availability: { status: "supported", reason: "" }
          }
        ]),
        createNode: vi.fn(),
        capabilityStatus: "ready",
        capabilitySource: "remote",
        capabilityMessage: ""
      });
    });

    const { container } = render(<ModuleSidebar />);

    const dataGroupToggle = container.querySelector(".module-group-toggle");
    fireEvent.click(dataGroupToggle);
    expect(screen.queryByRole("button", { name: /Alpha feed/i })).not.toBeInTheDocument();

    const searchInput = container.querySelector(".sidebar-search");
    fireEvent.change(searchInput, { target: { value: "alpha" } });

    expect(screen.getByRole("button", { name: /Alpha feed/i })).toBeInTheDocument();
    const toolbar = container.querySelector(".module-sidebar-toolbar");
    const toolbarButtons = within(toolbar).getAllByRole("button");
    expect(toolbarButtons).toHaveLength(2);
    expect(toolbarButtons[0]).toBeDisabled();
    expect(toolbarButtons[1]).toBeDisabled();
  });

  it("surfaces recently used modules and lane coverage for the current graph", () => {
    act(() => {
      useGraphStore.setState({
        graph: {
          ...initialState.graph,
          metadata: {
            ...initialState.graph.metadata,
            editor: {
              ...(initialState.graph.metadata?.editor || {}),
              recent_node_ids: ["node_intent_1", "node_data_1"]
            }
          },
          nodes: [
            {
              id: "node_data_1",
              type: "data",
              module_key: "data.feed",
              name: "Feed",
              position: { x: 0, y: 0 }
            },
            {
              id: "node_intent_1",
              type: "intent",
              module_key: "intent.signal",
              name: "Signal",
              position: { x: 120, y: 0 }
            }
          ]
        },
        registry: createRegistry([
          {
            module_key: "data.feed",
            category: "data",
            display_name: "Data feed",
            description: "Market feed",
            availability: { status: "supported", reason: "" }
          },
          {
            module_key: "intent.signal",
            category: "intent",
            display_name: "Intent signal",
            description: "Signal generator",
            availability: { status: "supported", reason: "" }
          }
        ]),
        selectedNodeId: "node_intent_1",
        createNode: vi.fn(),
        capabilityStatus: "ready",
        capabilitySource: "remote",
        capabilityMessage: ""
      });
    });

    render(<ModuleSidebar />);

    const recentSection = screen.getByTestId("module-sidebar-recent-section");
    const structureSection = screen.getByTestId("module-sidebar-structure-section");
    const selectedTypeCard = screen.getByTestId("module-sidebar-selected-type-card");

    expect(recentSection).toBeInTheDocument();
    expect(recentSection).toHaveTextContent("Intent signal");
    expect(recentSection).toHaveTextContent("Data feed");
    expect(structureSection).toBeInTheDocument();
    expect(selectedTypeCard).toHaveTextContent("Signal");
  });

  it("renders shared workspace context when lane and focus metadata are provided", () => {
    act(() => {
      useGraphStore.setState({
        graph: {
          ...initialState.graph,
          nodes: [
            {
              id: "node_intent_1",
              type: "intent",
              module_key: "intent.signal",
              name: "Signal",
              position: { x: 120, y: 0 }
            }
          ]
        },
        registry: createRegistry([
          {
            module_key: "data.feed",
            category: "data",
            display_name: "Data feed",
            description: "Market feed",
            availability: { status: "supported", reason: "" }
          },
          {
            module_key: "intent.signal",
            category: "intent",
            display_name: "Intent signal",
            description: "Signal generator",
            availability: { status: "supported", reason: "" }
          },
          {
            module_key: "execution.paper",
            category: "execution",
            display_name: "Paper execution",
            description: "Execution repair path",
            availability: { status: "supported", reason: "" }
          },
          {
            module_key: "risk.guard",
            category: "risk",
            display_name: "Risk guard",
            description: "Risk control",
            availability: { status: "supported", reason: "" }
          },
          {
            module_key: "runtime.clock",
            category: "runtime",
            display_name: "Runtime clock",
            description: "Runtime controls",
            availability: { status: "supported", reason: "" }
          }
        ]),
        createNode: vi.fn(),
        selectedNodeId: "node_intent_1",
        capabilityStatus: "ready",
        capabilitySource: "remote",
        capabilityMessage: ""
      });
    });

    render(
      <ModuleSidebar
        workspaceContext={{
          laneId: "diagnostics",
          laneLabel: "Validate lane",
          laneStatus: "Auto follow",
          focusLabel: "issues focus",
          reasonTitle: "Lane changed automatically",
          reasonMessage:
            "A targeted compile issue needs validation workflow, so the Validate lane was brought forward.",
          reasonFocusMessage: "Canvas focus changed to issues focus."
        }}
      />
    );

    expect(screen.getByTestId("module-sidebar-lane-card")).toHaveTextContent("Validate lane");
    expect(screen.getByTestId("module-sidebar-focus-card")).toHaveTextContent("issues focus");
    expect(screen.getByTestId("module-sidebar-reason-card")).toHaveTextContent(
      "Lane changed automatically"
    );

    const recommendedSection = screen.getByTestId("module-sidebar-recommended-section");
    expect(recommendedSection).toHaveTextContent("Paper execution");
    expect(recommendedSection).toHaveTextContent("Intent signal");
    expect(recommendedSection).toHaveTextContent("Risk guard");
  });
});
