import {
  buildCompactEvidenceProjection,
  buildRuntimeTimelineItemsFromDetail
} from "./runtimeTimeline";

const SUMMARY_DEFINITIONS = [
  {
    id: "capability_snapshot",
    label: "能力快照",
    matches: (item) => item.event_type === "CapabilitySnapshotTaken"
  },
  {
    id: "data_quality",
    label: "数据质量",
    matches: (item) =>
      item.stage === "data" ||
      item.event_type === "DataUpdated" ||
      item.reason_code?.toLowerCase?.().includes("data")
  },
  {
    id: "risk_decisions",
    label: "风控决策",
    matches: (item) => item.stage === "risk" || item.event_type === "RiskDecisionProduced"
  },
  {
    id: "execution_outcomes",
    label: "执行结果",
    matches: (item) =>
      item.stage === "execution" ||
      item.stage === "fill" ||
      item.event_type?.startsWith?.("Execution")
  },
  {
    id: "portfolio_updates",
    label: "组合更新",
    matches: (item) => item.event_type === "PortfolioUpdated"
  },
  {
    id: "security_violations",
    label: "安全事件",
    matches: (item) => item.event_type === "SecurityViolationDetected"
  },
  {
    id: "mutation_lifecycle",
    label: "参数变更",
    matches: (item) => item.event_type?.startsWith?.("ParameterMutation")
  }
];

function evidenceItemsFromSource(source = {}) {
  const compact = buildCompactEvidenceProjection(source);
  if (Array.isArray(compact.entries) && compact.entries.length > 0) {
    return {
      source_event_count: compact.source_event_count,
      retained_event_count: compact.retained_event_count,
      items: compact.entries,
      strategy: "compact_evidence"
    };
  }
  const timeline = buildRuntimeTimelineItemsFromDetail(source);
  return {
    source_event_count: timeline.length,
    retained_event_count: timeline.length,
    items: timeline,
    strategy: "detail_window"
  };
}

export function buildEvidenceSummaryCards(source = {}) {
  const evidence = evidenceItemsFromSource(source);
  return SUMMARY_DEFINITIONS.map((definition) => {
    const matches = evidence.items.filter(definition.matches);
    const sequenceNumbers = matches
      .map((item) => Number(item.sequence_no))
      .filter((value) => Number.isFinite(value) && value > 0);
    const latest = matches[matches.length - 1] || null;
    return {
      id: definition.id,
      label: definition.label,
      count: matches.length,
      sequence_numbers: sequenceNumbers,
      latest_sequence_no: sequenceNumbers[sequenceNumbers.length - 1] || null,
      latest_summary: latest?.summary || "无匹配证据",
      latest_event_type: latest?.event_type || null
    };
  });
}

export function buildRetentionAwareEvidencePreview(source = {}) {
  const evidence = evidenceItemsFromSource(source);
  return {
    strategy: evidence.strategy,
    source_event_count: evidence.source_event_count,
    retained_event_count: evidence.retained_event_count,
    detail_window_required:
      evidence.strategy !== "compact_evidence" ||
      evidence.retained_event_count === evidence.source_event_count,
    cards: buildEvidenceSummaryCards(source)
  };
}

function cardCount(cards, id) {
  return cards.find((card) => card.id === id)?.count || 0;
}

function governanceStatus(left = {}, right = {}) {
  const leftGovernance = buildCompactEvidenceProjection(left).governance || {};
  const rightGovernance = buildCompactEvidenceProjection(right).governance || {};
  return ["capability_hash", "deployment_revision", "strategy_version", "parameter_version"].every(
    (key) => leftGovernance[key] === rightGovernance[key]
  )
    ? "same"
    : "different";
}

export function compareRuntimeEvidenceSources(left = {}, right = {}) {
  const leftPreview = buildRetentionAwareEvidencePreview(left);
  const rightPreview = buildRetentionAwareEvidencePreview(right);
  const deltaFor = (id) => cardCount(rightPreview.cards, id) - cardCount(leftPreview.cards, id);
  return {
    governance_status: governanceStatus(left, right),
    key_event_delta: rightPreview.retained_event_count - leftPreview.retained_event_count,
    risk_decision_delta: deltaFor("risk_decisions"),
    execution_outcome_delta: deltaFor("execution_outcomes"),
    source_event_delta: rightPreview.source_event_count - leftPreview.source_event_count,
    left: leftPreview,
    right: rightPreview
  };
}
