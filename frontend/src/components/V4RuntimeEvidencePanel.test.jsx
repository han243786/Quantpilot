import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import V4RuntimeEvidencePanel from "./V4RuntimeEvidencePanel";

const source = {
  runtime_mode: "paper_simulated",
  memory_snapshot: {
    runtime_mode: "paper_simulated",
    machines: [
      {
        machine_id: "compat.observation",
        template: "observation_machine",
        state_id: "ready",
        status: "active",
        cached_output: { event_type: "compat.observation_ready" }
      },
      {
        machine_id: "compat.execution",
        template: "execution_machine",
        state_id: "idle",
        status: "active",
        cached_output: null
      }
    ],
    risk_plane: {
      required: true,
      machine_ids: ["compat.decision"],
      min_priority: 9000,
      approved_event_count: 1,
      rejected_event_count: 0,
      real_order_path_unlocked: true,
      last_decision: {
        accepted: true,
        source_machine_id: "compat.decision",
        target_machine_id: "compat.execution",
        reason: "Risk Plane approved execution transition"
      }
    },
    execution: {
      venue_id: "paper-local",
      required_capabilities: ["market"],
      accepted_count: 0,
      rejected_count: 1,
      last_decision: {
        accepted: false,
        target_machine_id: "compat.execution",
        venue_id: "paper-local",
        runtime_mode: "paper_simulated",
        reason: "local_simulated mode requires runtime_simulated support",
        provider_order_submission_attached: false,
        entries: [
          {
            capability: "market",
            source: "provider_native",
            status: "mode_rejected",
            reason: "requires runtime_simulated"
          }
        ]
      }
    },
    event_sequence: 9,
    provider_order_submission_attached: false
  }
};

describe("V4RuntimeEvidencePanel", () => {
  it("renders v4 machine state, risk plane, and capability source evidence", () => {
    render(<V4RuntimeEvidencePanel source={source} />);

    expect(screen.getByTestId("v4-runtime-evidence-panel-summary")).toHaveTextContent(
      "provider_order_submission_detached"
    );
    expect(screen.getByTestId("v4-runtime-evidence-panel-machines")).toHaveTextContent(
      "compat.observation"
    );
    expect(screen.getByTestId("v4-runtime-evidence-panel-machines")).toHaveTextContent("ready");
    expect(screen.getByTestId("v4-runtime-evidence-panel-risk-plane")).toHaveTextContent(
      "Risk Plane approved execution transition"
    );
    expect(screen.getByTestId("v4-runtime-evidence-panel-execution")).toHaveTextContent(
      "provider_native"
    );
    expect(screen.getByTestId("v4-runtime-evidence-panel-execution")).toHaveTextContent(
      "mode_rejected"
    );
    expect(screen.getByTestId("v4-runtime-evidence-panel-execution")).toHaveTextContent(
      "requires runtime_simulated"
    );
  });

  it("does not render when the source has no v4 snapshot", () => {
    const { container } = render(<V4RuntimeEvidencePanel source={{ events: [] }} />);
    expect(container).toBeEmptyDOMElement();
  });
});
