import { describe, expect, it } from "vitest";
import {
  buildV4RuntimeEvidenceProjection,
  resolveV4RuntimeMemorySnapshot
} from "./v4RuntimeEvidence";

function sampleSource(overrides = {}) {
  return {
    runtime_mode: "paper_simulated",
    provider_order_submission_attached: false,
    memory_snapshot: {
      runtime_mode: "paper_simulated",
      machines: [
        {
          machine_id: "compat.observation",
          template: "observation_machine",
          state_id: "ready",
          status: "active",
          cached_output: {
            event_type: "compat.observation_ready"
          }
        },
        {
          machine_id: "compat.execution",
          template: "execution_machine",
          state_id: "ready",
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
        accepted_count: 1,
        rejected_count: 0,
        last_decision: {
          accepted: true,
          target_machine_id: "compat.execution",
          venue_id: "paper-local",
          runtime_mode: "paper_simulated",
          reason: "Execution capabilities accepted for runtime mode",
          provider_order_submission_attached: false,
          entries: [
            {
              capability: "market",
              source: "runtime_simulated",
              status: "accepted",
              reason: "execution capability accepted"
            }
          ]
        }
      },
      event_sequence: 8,
      provider_order_submission_attached: false,
      ...overrides
    }
  };
}

describe("v4RuntimeEvidence", () => {
  it("resolves v4 memory snapshots from runtime output wrappers", () => {
    const source = sampleSource();

    expect(resolveV4RuntimeMemorySnapshot(source)).toBe(source.memory_snapshot);
    expect(resolveV4RuntimeMemorySnapshot({ output: source })).toBe(source.memory_snapshot);
    expect(resolveV4RuntimeMemorySnapshot({ run_output: source })).toBe(source.memory_snapshot);
  });

  it("projects machine, risk plane, and execution capability evidence", () => {
    const projection = buildV4RuntimeEvidenceProjection(sampleSource());

    expect(projection.available).toBe(true);
    expect(projection.runtime_mode).toBe("PaperSimulated");
    expect(projection.provider_order_submission_attached).toBe(false);
    expect(projection.boundary_label).toBe("provider_order_submission_detached");
    expect(projection.machine_count).toBe(2);
    expect(projection.active_machine_count).toBe(2);
    expect(projection.machines[0]).toMatchObject({
      machine_id: "compat.observation",
      state_id: "ready",
      has_cache: true
    });
    expect(projection.risk_plane).toMatchObject({
      required: true,
      approved_event_count: 1,
      rejected_event_count: 0,
      real_order_path_unlocked: true
    });
    expect(projection.execution.last_decision.entries[0]).toMatchObject({
      capability: "market",
      source: "runtime_simulated",
      status: "accepted",
      status_tone: "success",
      source_tone: "warning"
    });
  });

  it("marks unsupported execution capability decisions as danger", () => {
    const projection = buildV4RuntimeEvidenceProjection(
      sampleSource({
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
            reason: "execution capability `Market` is unsupported",
            provider_order_submission_attached: false,
            entries: [
              {
                capability: "market",
                source: "unsupported",
                status: "unsupported",
                reason: "unsupported"
              }
            ]
          }
        }
      })
    );

    expect(projection.execution.rejected_count).toBe(1);
    expect(projection.execution.last_decision.tone).toBe("danger");
    expect(projection.execution.entries[0]).toMatchObject({
      source_tone: "danger",
      status_tone: "danger"
    });
  });
});
