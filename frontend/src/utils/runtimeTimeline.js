const TIMELINE_ITEM_VERSION = 1;

const DEFAULT_GOVERNANCE_IDENTITY = {
  capability_hash: "unknown",
  deployment_revision: "unknown",
  strategy_version: "unknown",
  parameter_version: "unknown"
};
const KEY_EVENT_INDEX_VERSION = 1;
const KEY_EVENT_INDEX_POLICY_VERSION = "quantpilot/key-event-index/v1";
const COMPACT_EVIDENCE_PROJECTION_VERSION = 1;
const COMPACT_EVIDENCE_POLICY_VERSION = "quantpilot/evidence-compaction/v1";
const SYSTEM_GOVERNANCE_EVENT_TYPES = new Set([
  "CapabilitySnapshotTaken",
  "SecurityViolationDetected",
  "ParameterMutationProposed",
  "ParameterMutationRejected",
  "ParameterMutationActivationScheduled",
  "ParameterMutationActivated",
  "ParameterMutationActivationFailed",
  "ParameterMutationSafeWindowDenied",
  "ParameterMutationRollbackScheduled",
  "ParameterMutationRolledBack",
  "ParameterMutationRollbackFailed"
]);
const DEFAULT_COMPACT_GOVERNANCE = {
  capability_hash: "unknown",
  deployment_revision: "unknown",
  strategy_version: "unknown",
  parameter_version: "unknown"
};

function nonEmptyString(value, fallback) {
  return typeof value === "string" && value.trim().length > 0 ? value.trim() : fallback;
}

function compactabilityForRetention(retentionClass) {
  if (retentionClass === "key") return "retain";
  if (retentionClass === "summary") return "summarize";
  return "drop_candidate";
}

function governanceIdentityFromEnvelope(envelope = {}) {
  return {
    capability_hash: nonEmptyString(
      envelope.capability_hash,
      DEFAULT_GOVERNANCE_IDENTITY.capability_hash
    ),
    deployment_revision: nonEmptyString(
      envelope.deployment_revision,
      DEFAULT_GOVERNANCE_IDENTITY.deployment_revision
    ),
    strategy_version: nonEmptyString(
      envelope.strategy_version,
      DEFAULT_GOVERNANCE_IDENTITY.strategy_version
    ),
    parameter_version: nonEmptyString(
      envelope.parameter_version,
      DEFAULT_GOVERNANCE_IDENTITY.parameter_version
    )
  };
}

function normalizeGovernanceIdentity(governance = {}, envelope = {}) {
  const envelopeGovernance = governanceIdentityFromEnvelope(envelope);
  return {
    capability_hash: nonEmptyString(governance.capability_hash, envelopeGovernance.capability_hash),
    deployment_revision: nonEmptyString(
      governance.deployment_revision,
      envelopeGovernance.deployment_revision
    ),
    strategy_version: nonEmptyString(governance.strategy_version, envelopeGovernance.strategy_version),
    parameter_version: nonEmptyString(
      governance.parameter_version,
      envelopeGovernance.parameter_version
    )
  };
}

function sourceFromInput(input = {}) {
  if (input.event && typeof input.event === "object") {
    return {
      event: input.event,
      sequenceNo: input.sequence_no
    };
  }
  return {
    event: input,
    sequenceNo: input.sequence_no
  };
}

export function normalizeRuntimeTimelineItem(input = {}) {
  const { event, sequenceNo } = sourceFromInput(input);
  const envelope = event?.envelope || {};
  const retentionClass = nonEmptyString(
    input.retention_class || envelope.retention_class,
    "summary"
  );
  const occurredAtMs =
    Number(input.occurred_at_ms || envelope.occurred_at_ms || event?.event_time_ms) || 0;
  const ingestedAtMs = Number(input.ingested_at_ms || envelope.ingested_at_ms) || occurredAtMs;

  return {
    timeline_item_version:
      Number(input.timeline_item_version) || TIMELINE_ITEM_VERSION,
    event_id: nonEmptyString(input.event_id || event?.event_id || envelope.event_id, "unknown"),
    event_type: nonEmptyString(input.event_type || event?.event_type || envelope.event_type, "unknown"),
    sequence_no: Number(input.sequence_no || envelope.sequence_no || sequenceNo) || 0,
    occurred_at_ms: occurredAtMs,
    ingested_at_ms: ingestedAtMs,
    stage: nonEmptyString(input.stage || envelope.stage, "system"),
    retention_class: retentionClass,
    severity: nonEmptyString(input.severity || envelope.severity || event?.severity, "Info"),
    module_key: nonEmptyString(input.module_key || envelope.module_key || event?.source_id, "unknown"),
    node_id: nonEmptyString(input.node_id || event?.node_id, "runtime"),
    summary: nonEmptyString(input.summary || event?.summary, "-"),
    reason_code:
      input.reason_code || envelope.reason_code || event?.payload?.reason_code || null,
    governance: normalizeGovernanceIdentity(input.governance || {}, envelope),
    payload_version: Number(input.payload_version || envelope.payload_version) || 1,
    compactability: nonEmptyString(
      input.compactability,
      compactabilityForRetention(retentionClass)
    )
  };
}

export function buildRuntimeTimelineItemsFromEvents(events = []) {
  return Array.isArray(events) ? events.map((event) => normalizeRuntimeTimelineItem(event)) : [];
}

export function buildRuntimeTimelineItemsFromDetail(detail = {}) {
  if (Array.isArray(detail.timeline)) {
    return detail.timeline.map((item) => normalizeRuntimeTimelineItem(item));
  }
  return buildRuntimeTimelineItemsFromEvents(detail.events);
}

export function buildRuntimeTimelineItemsFromReplay(replay = {}) {
  if (Array.isArray(replay.timeline)) {
    return replay.timeline.map((item) => normalizeRuntimeTimelineItem(item));
  }
  return Array.isArray(replay.events)
    ? replay.events.map((item) => normalizeRuntimeTimelineItem(item))
    : [];
}

export function isRetainedKeyTimelineItem(input = {}) {
  const item = normalizeRuntimeTimelineItem(input);
  return item.retention_class === "key" || SYSTEM_GOVERNANCE_EVENT_TYPES.has(item.event_type);
}

function buildRetainedKeyIndexFromTimeline(timeline = []) {
  const entries = timeline
    .map((item) => normalizeRuntimeTimelineItem(item))
    .filter((item) => isRetainedKeyTimelineItem(item));

  return {
    index_version: KEY_EVENT_INDEX_VERSION,
    policy_version: KEY_EVENT_INDEX_POLICY_VERSION,
    source_event_count: timeline.length,
    retained_event_count: entries.length,
    key_event_count: entries.filter((item) => item.retention_class === "key").length,
    system_event_count: entries.filter((item) =>
      SYSTEM_GOVERNANCE_EVENT_TYPES.has(item.event_type)
    ).length,
    entries
  };
}

function incrementCounter(counter, key) {
  counter[key] = (counter[key] || 0) + 1;
}

function governanceFromTimeline(timeline = []) {
  return timeline[0]?.governance || DEFAULT_COMPACT_GOVERNANCE;
}

export function normalizeRetainedKeyEventIndex(index = {}) {
  if (!Array.isArray(index.entries)) {
    return buildRetainedKeyIndexFromTimeline([]);
  }
  const built = buildRetainedKeyIndexFromTimeline(index.entries);
  return {
    ...built,
    source_event_count: Number(index.source_event_count) || built.source_event_count
  };
}

export function buildRetainedKeyEventIndex(source = {}) {
  if (source.retained_key_event_index) {
    return normalizeRetainedKeyEventIndex(source.retained_key_event_index);
  }
  if (Array.isArray(source)) {
    return buildRetainedKeyIndexFromTimeline(buildRuntimeTimelineItemsFromEvents(source));
  }
  return buildRetainedKeyIndexFromTimeline(buildRuntimeTimelineItemsFromDetail(source));
}

function buildCompactEvidenceFromTimeline(timeline = []) {
  const normalizedTimeline = timeline.map((item) => normalizeRuntimeTimelineItem(item));
  const retainedIndex = buildRetainedKeyIndexFromTimeline(normalizedTimeline);
  const retainedIds = new Set(retainedIndex.entries.map((item) => item.event_id));
  const droppedByRetention = {};
  const droppedByStage = {};

  normalizedTimeline.forEach((item) => {
    if (retainedIds.has(item.event_id)) return;
    incrementCounter(droppedByRetention, item.retention_class);
    incrementCounter(droppedByStage, item.stage);
  });

  return {
    projection_version: COMPACT_EVIDENCE_PROJECTION_VERSION,
    policy_version: COMPACT_EVIDENCE_POLICY_VERSION,
    source_event_count: normalizedTimeline.length,
    retained_event_count: retainedIndex.retained_event_count,
    dropped_event_count: normalizedTimeline.length - retainedIndex.retained_event_count,
    dropped_by_retention: droppedByRetention,
    dropped_by_stage: droppedByStage,
    key_event_count: retainedIndex.key_event_count,
    system_event_count: retainedIndex.system_event_count,
    governance: governanceFromTimeline(retainedIndex.entries.length > 0 ? retainedIndex.entries : normalizedTimeline),
    entries: retainedIndex.entries
  };
}

export function normalizeCompactEvidenceProjection(compact = {}) {
  if (!Array.isArray(compact.entries)) {
    return buildCompactEvidenceFromTimeline([]);
  }
  const entries = compact.entries.map((item) => normalizeRuntimeTimelineItem(item));
  return {
    projection_version:
      Number(compact.projection_version) || COMPACT_EVIDENCE_PROJECTION_VERSION,
    policy_version:
      nonEmptyString(compact.policy_version, COMPACT_EVIDENCE_POLICY_VERSION),
    source_event_count: Number(compact.source_event_count) || entries.length,
    retained_event_count: Number(compact.retained_event_count) || entries.length,
    dropped_event_count: Number(compact.dropped_event_count) || 0,
    dropped_by_retention: compact.dropped_by_retention || {},
    dropped_by_stage: compact.dropped_by_stage || {},
    key_event_count:
      Number(compact.key_event_count) ||
      entries.filter((item) => item.retention_class === "key").length,
    system_event_count:
      Number(compact.system_event_count) ||
      entries.filter((item) => SYSTEM_GOVERNANCE_EVENT_TYPES.has(item.event_type)).length,
    governance: normalizeGovernanceIdentity(compact.governance || {}, {}),
    entries
  };
}

export function buildCompactEvidenceProjection(source = {}) {
  if (source.compact_evidence) {
    return normalizeCompactEvidenceProjection(source.compact_evidence);
  }
  if (source.retained_key_event_index) {
    const retainedIndex = normalizeRetainedKeyEventIndex(source.retained_key_event_index);
    return {
      ...buildCompactEvidenceFromTimeline(retainedIndex.entries),
      source_event_count: retainedIndex.source_event_count,
      retained_event_count: retainedIndex.retained_event_count,
      dropped_event_count: Math.max(
        0,
        retainedIndex.source_event_count - retainedIndex.retained_event_count
      ),
      key_event_count: retainedIndex.key_event_count,
      system_event_count: retainedIndex.system_event_count
    };
  }
  if (Array.isArray(source)) {
    return buildCompactEvidenceFromTimeline(buildRuntimeTimelineItemsFromEvents(source));
  }
  return buildCompactEvidenceFromTimeline(buildRuntimeTimelineItemsFromDetail(source));
}
