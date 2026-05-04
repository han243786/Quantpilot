import { afterEach, describe, expect, it, vi } from "vitest";
import { act, fireEvent, render, screen } from "@testing-library/react";
import BaseNodeCard from "./BaseNodeCard";
import { useGraphStore } from "../store/graphStore";

vi.mock("@xyflow/react", () => ({
  Handle: () => <span data-testid="node-handle" />,
  Position: {
    Left: "left",
    Right: "right"
  }
}));

function createNodeCardData(overrides = {}) {
  return {
    nodeId: "node_1",
    nodeType: "intent",
    runtimeStatus: "idle",
    title: "Signal",
    subtitle: "Double MA",
    inputPorts: [],
    outputPorts: [],
    highlighted: false,
    simplified: false,
    handlesConnectable: true,
    summaryValues: ["MA"],
    quickFieldDefinitions: [
      {
        key: "timeframe",
        label: "周期",
        type: "select",
        value: "1m",
        options: [
          { value: "1m", label: "1 分钟" },
          { value: "5m", label: "5 分钟" }
        ]
      },
      {
        key: "fast",
        label: "快线",
        type: "number",
        value: 12
      }
    ],
    issueMessage: null,
    metricLabel: "待运行",
    collapsed: false,
    dimmed: false,
    focusMode: "selected",
    recommendationRole: null,
    ...overrides
  };
}

describe("BaseNodeCard", () => {
  const initialState = useGraphStore.getState();

  afterEach(() => {
    act(() => {
      useGraphStore.setState(initialState, true);
    });
  });

  it("keeps quick field controls from bubbling into canvas node selection", () => {
    const setSelectedNode = vi.fn();
    const updateNodeConfig = vi.fn();

    act(() => {
      useGraphStore.setState({
        ...useGraphStore.getState(),
        setSelectedNode,
        updateNodeConfig
      });
    });

    render(<BaseNodeCard data={createNodeCardData()} selected />);

    const select = screen.getByDisplayValue("1 分钟");
    fireEvent.pointerDown(select);
    fireEvent.click(select);
    fireEvent.change(select, { target: { value: "5m" } });

    expect(setSelectedNode).not.toHaveBeenCalled();
    expect(updateNodeConfig).toHaveBeenCalledWith("node_1", "timeframe", "5m");
    expect(select).toHaveClass("nodrag", "nopan");

    const input = screen.getByDisplayValue("12");
    fireEvent.pointerDown(input);
    fireEvent.click(input);

    expect(setSelectedNode).not.toHaveBeenCalled();
    expect(input).toHaveClass("nodrag", "nopan");
  });
});
