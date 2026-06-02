import { fireEvent, render, screen, within } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import GovernedTimelinePanel from "./GovernedTimelinePanel";

const timeline = [
  {
    timeline_item_version: 1,
    event_id: "evt_capability",
    event_type: "CapabilitySnapshotTaken",
    sequence_no: 1,
    occurred_at_ms: 10,
    ingested_at_ms: 11,
    stage: "system",
    retention_class: "key",
    severity: "Info",
    module_key: "runtime_governance",
    node_id: "runtime",
    summary: "Capability snapshot taken",
    reason_code: "CAPABILITY_SNAPSHOT",
    governance: {
      capability_hash: "sha256:capability-1234567890abcdef",
      deployment_revision: "sha256:deployment",
      strategy_version: "strategy:v1",
      parameter_version: "config:abc"
    },
    payload_version: 1,
    compactability: "retain"
  },
  {
    timeline_item_version: 1,
    event_id: "evt_risk",
    event_type: "RiskDecisionProduced",
    sequence_no: 2,
    occurred_at_ms: 12,
    ingested_at_ms: 13,
    stage: "risk",
    retention_class: "key",
    severity: "Warn",
    module_key: "builtin.risk.global",
    node_id: "risk_node",
    summary: "Risk limit clamped the target.",
    reason_code: "MAX_WEIGHT_CLAMPED",
    governance: {
      capability_hash: "sha256:capability-1234567890abcdef",
      deployment_revision: "sha256:deployment",
      strategy_version: "strategy:v1",
      parameter_version: "config:abc"
    },
    payload_version: 1,
    compactability: "retain"
  },
  {
    timeline_item_version: 1,
    event_id: "evt_agent_debug",
    event_type: "AgentDecisionProduced",
    sequence_no: 3,
    occurred_at_ms: 14,
    ingested_at_ms: 15,
    stage: "agent",
    retention_class: "debug",
    severity: "Info",
    module_key: "builtin.agent.signal",
    node_id: "agent_node",
    summary: "Agent debug signal.",
    reason_code: null,
    governance: {
      capability_hash: "sha256:capability-1234567890abcdef",
      deployment_revision: "sha256:deployment",
      strategy_version: "strategy:v1",
      parameter_version: "config:abc"
    },
    payload_version: 1,
    compactability: "drop_candidate"
  }
];

describe("GovernedTimelinePanel", () => {
  it("groups by stage, filters retained key evidence, and opens selected details", () => {
    render(<GovernedTimelinePanel source={{ timeline }} testId="timeline-test" />);

    expect(screen.getByTestId("timeline-test-stage-system")).toHaveTextContent("系统");
    expect(screen.getByTestId("timeline-test-stage-risk")).toHaveTextContent("风控");
    expect(screen.getByTestId("timeline-test-retained-count")).toHaveTextContent("2/3");

    fireEvent.change(screen.getByTestId("timeline-test-retention-filter"), {
      target: { value: "key" }
    });
    expect(screen.queryByTestId("timeline-test-item-evt_agent_debug")).toBeNull();
    expect(screen.getByTestId("timeline-test-item-evt_risk")).toBeInTheDocument();

    fireEvent.click(screen.getByTestId("timeline-test-item-evt_risk"));
    const selected = screen.getByTestId("timeline-test-selected-detail");
    expect(within(selected).getByText("evt_risk")).toBeInTheDocument();
    expect(selected).toHaveTextContent("MAX_WEIGHT_CLAMPED");
    expect(selected).toHaveTextContent("sha256:capab...abcdef");
  });

  it("filters timeline evidence by severity and module without leaking stale selection", () => {
    render(<GovernedTimelinePanel source={{ timeline }} testId="timeline-filter" />);

    fireEvent.change(screen.getByTestId("timeline-filter-severity-filter"), {
      target: { value: "Warn" }
    });
    expect(screen.getByTestId("timeline-filter-item-evt_risk")).toBeInTheDocument();
    expect(screen.queryByTestId("timeline-filter-item-evt_capability")).toBeNull();
    expect(screen.queryByTestId("timeline-filter-item-evt_agent_debug")).toBeNull();
    expect(screen.getByTestId("timeline-filter-selected-detail")).toHaveTextContent("evt_risk");

    fireEvent.change(screen.getByTestId("timeline-filter-module-filter"), {
      target: { value: "runtime_governance" }
    });
    expect(screen.queryByTestId("timeline-filter-stage-risk")).toBeNull();
    expect(screen.queryByTestId("timeline-filter-selected-detail")).toBeNull();

    fireEvent.change(screen.getByTestId("timeline-filter-severity-filter"), {
      target: { value: "all" }
    });
    expect(screen.getByTestId("timeline-filter-item-evt_capability")).toBeInTheDocument();
    expect(screen.getByTestId("timeline-filter-selected-detail")).toHaveTextContent(
      "evt_capability"
    );
  });

  it("keeps empty timeline evidence inspectable without selecting a phantom event", () => {
    render(<GovernedTimelinePanel source={{ timeline: [] }} testId="timeline-empty" />);

    expect(screen.getByTestId("timeline-empty-retained-count")).toHaveTextContent("0/0");
    expect(screen.queryByTestId("timeline-empty-selected-detail")).toBeNull();
    expect(screen.queryByTestId("timeline-empty-stage-system")).toBeNull();
  });
});
