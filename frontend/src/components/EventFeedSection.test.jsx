import { act, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { EventFeedSection } from "./EventStreamPanel";

function buildProps(overrides = {}) {
  return {
    runtime: {
      eventTypeFilter: "all",
      eventSearchTerm: ""
    },
    eventTypes: ["all", "IntentTriggered", "DataQualityWarning"],
    eventNodeOptions: [
      {
        nodeId: "node_entry",
        nodeName: "Entry signal"
      }
    ],
    selectedEventNodeId: null,
    filteredEvents: [
      {
        event_id: "evt_001",
        event_time_ms: 1_710_000_000_000,
        event_type: "DataQualityWarning",
        node_id: "node_entry",
        summary: "Gap detected",
        payload: {
          source_health: "warning",
          freshness_ms: 1500,
          gap_count: 2,
          quality_flags: ["gap", "stale"],
          post_risk: {
            concentration_ratio: 0.18,
            max_symbol_net_exposure_ratio: 0.3,
            portfolio_net_exposure_ratio: 0.4
          },
          side: "Buy"
        }
      }
    ],
    eventTypeFilter: "all",
    eventSearchTerm: "",
    setEventNodeScope: vi.fn(),
    setEventTypeFilter: vi.fn(),
    setEventSearchTerm: vi.fn(),
    setSelectedNode: vi.fn(),
    ...overrides
  };
}

describe("EventFeedSection", () => {
  afterEach(() => {
    vi.useRealTimers();
    if (window._qpEventSearchTimer) {
      clearTimeout(window._qpEventSearchTimer);
      window._qpEventSearchTimer = null;
    }
  });

  it("wires node scope, event type, search debounce, clear, and event-row focus actions", () => {
    vi.useFakeTimers();
    const props = buildProps();
    const { container } = render(<EventFeedSection {...props} />);

    expect(screen.getByTestId("event-feed-section")).toBeInTheDocument();
    expect(screen.getByTestId("event-feed-node-filter")).toBeInTheDocument();
    expect(screen.getByTestId("event-feed-source-health-evt_001")).toBeInTheDocument();
    expect(screen.getByTestId("event-feed-freshness-evt_001")).toHaveTextContent("1500 ms");
    expect(screen.getByTestId("event-feed-gap-count-evt_001")).toHaveTextContent("2");
    expect(screen.getByTestId("event-feed-quality-flags-evt_001")).toHaveTextContent("gap");

    fireEvent.click(screen.getByTestId("event-feed-node-chip-all"));
    expect(props.setEventNodeScope).toHaveBeenCalledWith("all");
    expect(props.setSelectedNode).toHaveBeenCalledWith(null);

    fireEvent.click(screen.getByTestId("event-feed-node-chip-node_entry"));
    expect(props.setEventNodeScope).toHaveBeenCalledWith("auto");
    expect(props.setSelectedNode).toHaveBeenCalledWith("node_entry");

    fireEvent.change(container.querySelector(".event-filter-bar select"), {
      target: { value: "DataQualityWarning" }
    });
    expect(props.setEventTypeFilter).toHaveBeenCalledWith("DataQualityWarning");

    fireEvent.change(container.querySelector(".event-filter-bar input"), {
      target: { value: "gap" }
    });
    act(() => {
      vi.advanceTimersByTime(200);
    });
    expect(props.setEventSearchTerm).toHaveBeenCalledWith("gap");

    fireEvent.click(container.querySelector(".event-filter-bar button"));
    expect(props.setEventTypeFilter).toHaveBeenCalledWith("all");
    expect(props.setEventSearchTerm).toHaveBeenCalledWith("");

    fireEvent.click(screen.getByTestId("event-feed-row-evt_001"));
    expect(props.setEventNodeScope).toHaveBeenCalledWith("auto");
    expect(props.setSelectedNode).toHaveBeenCalledWith("node_entry");
  });

  it("shows an empty state when filtered events are empty", () => {
    render(<EventFeedSection {...buildProps({ filteredEvents: [], eventNodeOptions: [] })} />);

    expect(screen.getByTestId("event-feed-section")).toBeInTheDocument();
    expect(screen.queryByTestId("event-feed-node-filter")).not.toBeInTheDocument();
    expect(screen.queryByTestId("event-feed-row-evt_001")).not.toBeInTheDocument();
    expect(screen.getByText(/\u5f53\u524d\u7b5b\u9009\u6761\u4ef6\u4e0b\u6ca1\u6709\u4e8b\u4ef6/)).toBeInTheDocument();
  });
});
