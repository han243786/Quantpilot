import { act, render, screen } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { HistoryExplanationCard } from "./EventStreamPanel";
import { useGraphStore } from "../store/graphStore";

// v0.5.0: HistoryExplanationCard 是导出的独立子组件, 可直接测试

describe("EventStreamPanel history explanations", () => {
  const initialState = useGraphStore.getState();

  beforeEach(() => {
    act(() => { useGraphStore.setState(initialState, true); });
  });
  afterEach(() => {
    act(() => { useGraphStore.setState(initialState, true); });
  });

  it("renders risk explanation entries", () => {
    const entries = [
      { nodeId: "risk_1", nodeName: "全局风控", kind: "risk", explanationSummary: "最大持仓限制已触发", rows: [{ key: "limit", label: "限制", value: "max_position=0.5" }] }
    ];
    render(<HistoryExplanationCard title="风险明细" summary="运行时风险检查记录" entries={entries} testId="risk-explanations-test" />);
    expect(screen.getByTestId("risk-explanations-test")).toBeInTheDocument();
    expect(screen.getByText("全局风控")).toBeInTheDocument();
    expect(screen.getByText("最大持仓限制已触发")).toBeInTheDocument();
  });

  it("renders nothing when entries are empty", () => {
    const { container } = render(<HistoryExplanationCard title="空" summary="无" entries={[]} testId="empty-test" />);
    expect(container.querySelector('[data-testid="empty-test"]')).toBeNull();
  });
});
