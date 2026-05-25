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
        cached_output: { event_type: "compat.observation_ready" },
        children: [
          {
            machine_id: "compat.observation.child",
            template: "observation_machine",
            state_id: "child_ready",
            status: "active",
            cached_output: null
          }
        ]
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
    simulated_execution: {
      enabled: true,
      quote_asset: "USDT",
      cash_balance: 899.3995,
      realized_fees: 0.1005,
      position_market_value: 100.5,
      portfolio_value: 999.8995,
      order_count: 1,
      open_order_count: 0,
      rejected_order_count: 0,
      fill_count: 1,
      positions: [
        {
          venue_id: "paper-local",
          symbol: "ETHUSDT",
          net_quantity: 1,
          average_price: 100.5,
          market_price: 100.5,
          market_value: 100.5
        }
      ],
      asset_curve: [{ ts_ms: 1, portfolio_value: 999.8995 }],
      last_order: {
        order_id: "v4-sim-order-1",
        venue_id: "paper-local",
        symbol: "ETHUSDT",
        action: "buy",
        side: "buy",
        order_type: "market",
        requested_quantity: 1,
        filled_quantity: 1,
        remaining_quantity: 0,
        reference_price: 100,
        fill_price: 100.5,
        status: "filled"
      },
      last_fill: {
        fill_id: "fill-v4-sim-order-1-1",
        order_id: "v4-sim-order-1",
        venue_id: "paper-local",
        symbol: "ETHUSDT",
        side: "buy",
        action: "buy",
        quantity: 1,
        price: 100.5,
        notional: 100.5,
        fee: 0.1005,
        fee_asset: "USDT"
      }
    },
    venue_adapter_boundary: {
      provider_order_submission_attached: false,
      provider_order_submission_allowed: false,
      settlement_authority: "local_simulated",
      live_actual_submission_allowed: false,
      rejection_before_provider_submit: true,
      reason: "provider-native order submission must be rejected before provider submit"
    },
    complexity_metrics: {
      state_count: 6,
      transition_count: 4,
      memory_field_count: 3,
      nested_machine_depth: 2,
      event_processing_path_count: 8
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
    expect(screen.getByTestId("v4-runtime-evidence-panel-machines")).toHaveTextContent(
      "compat.observation.child"
    );
    expect(screen.getByTestId("v4-runtime-evidence-panel-machines")).toHaveTextContent("ready");
    expect(screen.getByTestId("v4-runtime-evidence-panel-complexity-budget")).toHaveTextContent(
      "复杂度预算"
    );
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
    expect(screen.getByTestId("v4-runtime-evidence-panel-simulated-execution")).toHaveTextContent(
      "999.8995"
    );
    expect(screen.getByTestId("v4-runtime-evidence-panel-simulated-execution")).toHaveTextContent(
      "v4-sim-order-1"
    );
    expect(screen.getByTestId("v4-runtime-evidence-panel-simulated-execution")).toHaveTextContent(
      "ETHUSDT"
    );
    expect(screen.getByTestId("v4-runtime-evidence-panel-venue-boundary")).toHaveTextContent(
      "local_simulated"
    );
    expect(screen.getByTestId("v4-runtime-evidence-panel-venue-boundary")).toHaveTextContent(
      "provider-native order submission"
    );
  });

  it("does not render when the source has no v4 snapshot", () => {
    const { container } = render(<V4RuntimeEvidencePanel source={{ events: [] }} />);
    expect(container).toBeEmptyDOMElement();
  });
});
