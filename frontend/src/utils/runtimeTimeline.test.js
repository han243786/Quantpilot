import { describe, expect, it } from "vitest";
import {
  buildCompactEvidenceProjection,
  buildRetainedKeyEventIndex,
  buildRuntimeTimelineItemsFromEvents,
  buildRuntimeTimelineItemsFromDetail,
  buildRuntimeTimelineItemsFromReplay,
  isRetainedKeyTimelineItem,
  normalizeRuntimeTimelineItem
} from "./runtimeTimeline";

const governedEvent = {
  event_id: "evt_timeline_001",
  event_type: "RiskDecisionProduced",
  source_id: "builtin.risk.global",
  node_id: "risk_node",
  event_time_ms: 1_710_000_000_000,
  severity: "Warn",
  summary: "Risk limit clamped the target.",
  payload: {
    reason_code: "MAX_WEIGHT_CLAMPED"
  },
  envelope: {
    event_id: "evt_timeline_001",
    event_type: "RiskDecisionProduced",
    stage: "risk",
    sequence_no: 42,
    occurred_at_ms: 1_710_000_000_010,
    ingested_at_ms: 1_710_000_000_020,
    module_key: "builtin.risk.global",
    strategy_version: "strategy:v1",
    parameter_version: "config:abc",
    deployment_revision: "sha256:deployment",
    capability_hash: "sha256:capability",
    severity: "Warn",
    retention_class: "key",
    reason_code: "MAX_WEIGHT_CLAMPED",
    payload_version: 1
  }
};

describe("runtime timeline contract reader", () => {
  it("normalizes a governed runtime event into the shared timeline item shape", () => {
    const item = normalizeRuntimeTimelineItem(governedEvent);

    expect(item).toEqual({
      timeline_item_version: 1,
      event_id: "evt_timeline_001",
      event_type: "RiskDecisionProduced",
      sequence_no: 42,
      occurred_at_ms: 1_710_000_000_010,
      ingested_at_ms: 1_710_000_000_020,
      stage: "risk",
      retention_class: "key",
      severity: "Warn",
      module_key: "builtin.risk.global",
      node_id: "risk_node",
      summary: "Risk limit clamped the target.",
      reason_code: "MAX_WEIGHT_CLAMPED",
      governance: {
        capability_hash: "sha256:capability",
        deployment_revision: "sha256:deployment",
        strategy_version: "strategy:v1",
        parameter_version: "config:abc"
      },
      payload_version: 1,
      compactability: "retain"
    });
  });

  it("normalizes replay event wrappers without losing sequence metadata", () => {
    const [item] = buildRuntimeTimelineItemsFromReplay({
      events: [
        {
          sequence_no: 7,
          event: {
            ...governedEvent,
            envelope: {
              ...governedEvent.envelope,
              sequence_no: 7,
              retention_class: "summary"
            }
          }
        }
      ]
    });

    expect(item.sequence_no).toBe(7);
    expect(item.retention_class).toBe("summary");
    expect(item.compactability).toBe("summarize");
  });

  it("prefers backend timeline projections on detail and replay responses", () => {
    const projectedTimeline = {
      timeline_item_version: 1,
      event_id: "evt_projected",
      event_type: "CapabilitySnapshotTaken",
      sequence_no: 1,
      occurred_at_ms: 10,
      ingested_at_ms: 11,
      stage: "system",
      retention_class: "key",
      severity: "Info",
      module_key: "builtin.capabilities",
      node_id: "runtime",
      summary: "Capability snapshot",
      reason_code: "CAPABILITY_SNAPSHOT",
      governance: {
        capability_hash: "sha256:projected",
        deployment_revision: "sha256:revision",
        strategy_version: "strategy:v1",
        parameter_version: "config:abc"
      },
      payload_version: 1,
      compactability: "retain"
    };
    const legacyEvent = {
      ...governedEvent,
      event_id: "evt_legacy_fallback",
      envelope: {
        ...governedEvent.envelope,
        event_id: "evt_legacy_fallback",
        sequence_no: 99
      }
    };

    expect(buildRuntimeTimelineItemsFromDetail({
      timeline: [projectedTimeline],
      events: [legacyEvent]
    })[0].event_id).toBe("evt_projected");
    expect(buildRuntimeTimelineItemsFromReplay({
      timeline: [projectedTimeline],
      events: [{ sequence_no: 99, event: legacyEvent }]
    })[0].sequence_no).toBe(1);
  });

  it("keeps an already shaped timeline item stable", () => {
    const item = normalizeRuntimeTimelineItem({
      timeline_item_version: 1,
      event_id: "evt_ready",
      event_type: "ExecutionFilled",
      sequence_no: 9,
      occurred_at_ms: 10,
      ingested_at_ms: 11,
      stage: "fill",
      retention_class: "key",
      severity: "Info",
      module_key: "builtin.execution.paper",
      node_id: "execution",
      summary: "Filled",
      reason_code: "FILLED",
      governance: {
        capability_hash: "sha256:item",
        deployment_revision: "sha256:revision",
        strategy_version: "strategy:v2",
        parameter_version: "config:def"
      },
      payload_version: 2,
      compactability: "retain"
    });

    expect(item.event_id).toBe("evt_ready");
    expect(item.payload_version).toBe(2);
    expect(item.governance.capability_hash).toBe("sha256:item");
  });

  it("builds timeline items from event arrays with restrictive unknown defaults", () => {
    const [item] = buildRuntimeTimelineItemsFromEvents([
      {
        event_id: "evt_legacy",
        event_type: "RuntimeWarning",
        source_id: "source",
        node_id: "runtime",
        event_time_ms: 12,
        severity: "Warn",
        summary: "Legacy event",
        payload: {}
      }
    ]);

    expect(item.stage).toBe("system");
    expect(item.retention_class).toBe("summary");
    expect(item.governance.capability_hash).toBe("unknown");
    expect(item.compactability).toBe("summarize");
  });

  it("builds a retained key-event index from timeline evidence", () => {
    const index = buildRetainedKeyEventIndex({
      timeline: [
        governedEvent,
        {
          ...governedEvent,
          event_id: "evt_debug",
          event_type: "AgentDecisionProduced",
          envelope: {
            ...governedEvent.envelope,
            event_id: "evt_debug",
            event_type: "AgentDecisionProduced",
            sequence_no: 43,
            stage: "agent",
            retention_class: "debug"
          }
        },
        {
          ...governedEvent,
          event_id: "evt_security",
          event_type: "SecurityViolationDetected",
          envelope: {
            ...governedEvent.envelope,
            event_id: "evt_security",
            event_type: "SecurityViolationDetected",
            sequence_no: 44,
            stage: "system",
            retention_class: "summary"
          }
        }
      ]
    });

    expect(index.source_event_count).toBe(3);
    expect(index.retained_event_count).toBe(2);
    expect(index.key_event_count).toBe(1);
    expect(index.system_event_count).toBe(1);
    expect(index.entries.map((item) => item.event_type)).toEqual([
      "RiskDecisionProduced",
      "SecurityViolationDetected"
    ]);
    expect(isRetainedKeyTimelineItem(index.entries[0])).toBe(true);
  });

  it("builds compact evidence projection with dropped counts", () => {
    const compact = buildCompactEvidenceProjection({
      timeline: [
        governedEvent,
        {
          ...governedEvent,
          event_id: "evt_summary",
          event_type: "RuntimeWarning",
          envelope: {
            ...governedEvent.envelope,
            event_id: "evt_summary",
            event_type: "RuntimeWarning",
            sequence_no: 43,
            stage: "system",
            retention_class: "summary"
          }
        },
        {
          ...governedEvent,
          event_id: "evt_debug",
          event_type: "AgentDecisionProduced",
          envelope: {
            ...governedEvent.envelope,
            event_id: "evt_debug",
            event_type: "AgentDecisionProduced",
            sequence_no: 44,
            stage: "agent",
            retention_class: "debug"
          }
        }
      ]
    });

    expect(compact.policy_version).toBe("quantpilot/evidence-compaction/v1");
    expect(compact.source_event_count).toBe(3);
    expect(compact.retained_event_count).toBe(1);
    expect(compact.dropped_event_count).toBe(2);
    expect(compact.dropped_by_retention).toEqual({ debug: 1, summary: 1 });
    expect(compact.dropped_by_stage).toEqual({ agent: 1, system: 1 });
    expect(compact.entries.map((item) => item.event_type)).toEqual(["RiskDecisionProduced"]);
  });
});
