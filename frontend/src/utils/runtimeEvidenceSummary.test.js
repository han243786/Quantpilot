import { describe, expect, it } from "vitest";
import {
  buildEvidenceSummaryCards,
  buildRetentionAwareEvidencePreview,
  compareRuntimeEvidenceSources
} from "./runtimeEvidenceSummary";

function item(sequence, eventType, stage, summary, governance = {}) {
  return {
    timeline_item_version: 1,
    event_id: `evt_${sequence}`,
    event_type: eventType,
    sequence_no: sequence,
    occurred_at_ms: 1_710_000_000_000 + sequence,
    ingested_at_ms: 1_710_000_000_000 + sequence,
    stage,
    retention_class: "key",
    severity: "Info",
    module_key: `module_${stage}`,
    node_id: `node_${stage}`,
    summary,
    reason_code: null,
    governance: {
      capability_hash: "sha256:capability-a",
      deployment_revision: "rev-a",
      strategy_version: "strategy-a",
      parameter_version: "params-a",
      ...governance
    },
    payload_version: 1,
    compactability: "retain"
  };
}

function source(entries, overrides = {}) {
  return {
    compact_evidence: {
      projection_version: 1,
      policy_version: "quantpilot/evidence-compaction/v1",
      source_event_count: 20,
      retained_event_count: entries.length,
      dropped_event_count: 20 - entries.length,
      dropped_by_retention: { debug: 4 },
      dropped_by_stage: { agent: 4 },
      key_event_count: entries.length,
      system_event_count: entries.filter((entry) => entry.event_type === "CapabilitySnapshotTaken")
        .length,
      governance: entries[0]?.governance,
      entries,
      ...overrides
    }
  };
}

describe("runtimeEvidenceSummary", () => {
  it("builds evidence summary cards from compact evidence sequence numbers", () => {
    const cards = buildEvidenceSummaryCards(
      source([
        item(1, "CapabilitySnapshotTaken", "system", "Capability snapshot"),
        item(2, "DataUpdated", "data", "Data delayed"),
        item(3, "RiskDecisionProduced", "risk", "Risk clamp"),
        item(4, "ExecutionFilled", "fill", "Filled"),
        item(5, "PortfolioUpdated", "system", "Portfolio updated"),
        item(6, "SecurityViolationDetected", "system", "Blocked"),
        item(7, "ParameterMutationActivated", "system", "Mutation activated")
      ])
    );

    expect(cards.find((card) => card.id === "capability_snapshot")).toMatchObject({
      count: 1,
      sequence_numbers: [1]
    });
    expect(cards.find((card) => card.id === "risk_decisions")).toMatchObject({
      count: 1,
      latest_sequence_no: 3
    });
    expect(cards.find((card) => card.id === "security_violations")).toMatchObject({
      count: 1,
      latest_event_type: "SecurityViolationDetected"
    });
    expect(cards.find((card) => card.id === "mutation_lifecycle")).toMatchObject({
      count: 1,
      latest_sequence_no: 7
    });
  });

  it("prefers compact evidence for retention-aware preview", () => {
    const preview = buildRetentionAwareEvidencePreview(
      source([item(1, "CapabilitySnapshotTaken", "system", "Capability snapshot")])
    );

    expect(preview.strategy).toBe("compact_evidence");
    expect(preview.source_event_count).toBe(20);
    expect(preview.retained_event_count).toBe(1);
    expect(preview.detail_window_required).toBe(false);
  });

  it("compares governance and key evidence deltas", () => {
    const left = source([
      item(1, "CapabilitySnapshotTaken", "system", "Capability snapshot"),
      item(2, "RiskDecisionProduced", "risk", "Risk clamp")
    ]);
    const right = source(
      [
        item(1, "CapabilitySnapshotTaken", "system", "Capability snapshot", {
          deployment_revision: "rev-b"
        }),
        item(2, "RiskDecisionProduced", "risk", "Risk clamp", {
          deployment_revision: "rev-b"
        }),
        item(3, "ExecutionFilled", "fill", "Filled", {
          deployment_revision: "rev-b"
        })
      ],
      { source_event_count: 30 }
    );

    const compare = compareRuntimeEvidenceSources(left, right);

    expect(compare.governance_status).toBe("different");
    expect(compare.key_event_delta).toBe(1);
    expect(compare.execution_outcome_delta).toBe(1);
    expect(compare.risk_decision_delta).toBe(0);
    expect(compare.source_event_delta).toBe(10);
  });
});
