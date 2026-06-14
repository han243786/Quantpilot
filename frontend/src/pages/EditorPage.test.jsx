import { act, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import EditorPage from "./EditorPage";
import { backtestDetailPath, navigateTo } from "../router";
import { useGraphStore } from "../store/graphStore";

vi.mock("../i18n", () => ({
  useI18n: () => ({
    t: (text) => text
  })
}));

vi.mock("../components/TopToolbar", () => ({
  default: () => <div data-testid="top-toolbar-stub" />
}));

vi.mock("../components/ModuleSidebar", () => ({
  default: () => <div data-testid="module-sidebar-stub" />
}));

vi.mock("../components/StrategyCanvas", () => ({
  default: () => <div data-testid="strategy-canvas-stub" />
}));

vi.mock("../components/PropertyPanel", () => ({
  default: () => <div data-testid="property-panel-stub" />
}));

vi.mock("../components/EventStreamPanel", () => ({
  default: ({ onOpenBacktestDetail }) => (
    <button
      type="button"
      data-testid="event-stream-panel-stub"
      onClick={() => onOpenBacktestDetail("bt-42")}
    >
      open detail
    </button>
  )
}));

vi.mock("../router", () => ({
  backtestDetailPath: vi.fn((backtestId, graphId) => `/backtests/${backtestId}?strategy=${graphId}`),
  navigateTo: vi.fn()
}));

describe("EditorPage legacy shell", () => {
  const initialState = useGraphStore.getState();

  beforeEach(() => {
    act(() => {
      useGraphStore.setState(initialState, true);
      useGraphStore.setState({
        graph: {
          ...initialState.graph,
          metadata: {
            ...initialState.graph.metadata,
            graph_id: "legacy_graph"
          }
        }
      });
    });
  });

  afterEach(() => {
    act(() => {
      useGraphStore.setState(initialState, true);
    });
  });

  it("renders the legacy graph editor workspace surfaces", async () => {
    render(<EditorPage />);

    expect(screen.getByTestId("top-toolbar-stub")).toBeInTheDocument();
    expect(screen.getByTestId("module-sidebar-stub")).toBeInTheDocument();
    expect(screen.getByTestId("strategy-canvas-stub")).toBeInTheDocument();
    expect(screen.getByTestId("property-panel-stub")).toBeInTheDocument();
    expect(await screen.findByTestId("event-stream-panel-stub")).toBeInTheDocument();
  });

  it("bridges event stream backtest detail actions through route helpers", async () => {
    render(<EditorPage />);

    fireEvent.click(await screen.findByTestId("event-stream-panel-stub"));

    expect(backtestDetailPath).toHaveBeenCalledWith("bt-42", "legacy_graph");
    expect(navigateTo).toHaveBeenCalledWith("/backtests/bt-42?strategy=legacy_graph");
  });
});
