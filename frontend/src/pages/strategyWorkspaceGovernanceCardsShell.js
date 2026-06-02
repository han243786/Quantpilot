export function formatWorkspaceGovernanceTime(value) {
  return value ? new Date(value).toLocaleString() : "-";
}

export function formatWorkspaceVersionList(items = []) {
  return items.length ? items.join(", ") : "-";
}

export function workspaceConfigDomainLabel(domainId) {
  const labels = {
    market: "市场与数据",
    observation: "观察与信号",
    state_machine: "状态机",
    risk: "Risk Plane",
    execution: "执行边界",
    evidence: "证据",
    ai_governance: "AI 治理",
    snapshot: "快照"
  };
  return labels[domainId] || domainId || "-";
}

export function workspaceConfigChangeLabels(change = {}) {
  const labels = [];
  if (change.lifecycle_changed) labels.push("生命周期");
  if (change.readiness_changed) labels.push("就绪状态");
  if (change.source_refs_changed) labels.push("来源证据");
  if (change.findings_changed) labels.push("诊断");
  return labels.length ? labels.join(" / ") : "-";
}

export function formatWorkspaceVersionCountChanges(changes = []) {
  return changes.length
    ? changes.map((change) => `${change.key}: ${change.left_count}->${change.right_count}`).join(" / ")
    : "-";
}

export function buildWorkspaceVersionDraftSummary(currentGraph) {
  return {
    graphId: currentGraph?.metadata?.graph_id || "draft_graph",
    updatedAt: currentGraph?.metadata?.updated_at || null,
    nodeCount: currentGraph?.nodes?.length || 0,
    edgeCount: currentGraph?.edges?.length || 0
  };
}

export function selectWorkspaceVersionCompareEntries(compareSelection = [], graphVersions = []) {
  return compareSelection
    .map((versionId) => graphVersions.find((entry) => entry.version_id === versionId))
    .filter(Boolean);
}

export function buildWorkspaceVersionEvidenceOptions(backtestHistory = [], graphId) {
  return (backtestHistory || [])
    .filter((entry) => !entry.graph_id || entry.graph_id === graphId)
    .map((entry) => ({
      id: entry.backtest_id,
      label: `${entry.backtest_id}${entry.created_at_ms ? ` · ${formatWorkspaceGovernanceTime(entry.created_at_ms)}` : ""}`
    }))
    .filter((entry) => entry.id);
}

export function toggleWorkspaceVersionCompareSelection(current = [], versionId) {
  if (current.includes(versionId)) {
    return current.filter((item) => item !== versionId);
  }
  if (current.length >= 2) {
    return [current[1], versionId];
  }
  return [...current, versionId];
}

export function parseWorkspaceExperimentNumberList(input, parser = Number) {
  return input
    .split(",")
    .map((value) => value.trim())
    .filter(Boolean)
    .map((value) => parser(value))
    .filter((value) => Number.isFinite(value));
}

export function formatWorkspaceExperimentPercent(value) {
  return `${value >= 0 ? "+" : ""}${(value * 100).toFixed(2)}%`;
}

export function selectWorkspaceGraphExperiments(experiments = [], graphId) {
  return (experiments || []).filter((entry) => entry.graph_id === graphId);
}

export function selectWorkspaceActiveExperiment(selectedExperiment, graphId) {
  return selectedExperiment?.graph_id === graphId ? selectedExperiment : null;
}

export function buildWorkspaceExperimentStartPayload({
  experimentName,
  feeGridDraft,
  slippageGridDraft,
  latencyGridDraft
}) {
  return {
    experimentName,
    feeBps: parseWorkspaceExperimentNumberList(feeGridDraft, Number),
    slippageBps: parseWorkspaceExperimentNumberList(slippageGridDraft, Number),
    latencyMs: parseWorkspaceExperimentNumberList(latencyGridDraft, (value) =>
      Number.parseInt(value, 10)
    )
  };
}

export function formatWorkspaceActor(actor, fallback = "未分配") {
  return actor?.display_name || actor?.actor_id || fallback;
}

export function buildWorkspaceCollaborationRows({
  collaboration,
  lastRun,
  lastBacktest
}) {
  return [
    {
      testId: "workspace-owner-row",
      label: "所有者",
      value: formatWorkspaceActor(collaboration?.owner)
    },
    {
      testId: "workspace-editors-row",
      label: "协作者",
      value:
        Array.isArray(collaboration?.editors) && collaboration.editors.length > 0
          ? collaboration.editors.map((actor) => formatWorkspaceActor(actor)).join(", ")
          : "未分配协作者"
    },
    {
      testId: "workspace-last-saved-row",
      label: "最近保存人",
      value: formatWorkspaceActor(collaboration?.last_saved_by, "-")
    },
    {
      testId: "workspace-last-run-row",
      label: "最近执行人",
      value: formatWorkspaceActor(
        lastRun?.actor || lastBacktest?.actor || collaboration?.last_run_actor,
        "-"
      )
    }
  ];
}

export function shouldRefreshWorkspaceAuditHistory(graphId) {
  return Boolean(graphId && graphId !== "draft_graph");
}

export function formatWorkspaceAuditActorLine(entry) {
  return `${formatWorkspaceActor(entry.actor)}${entry.target_id ? ` / ${entry.target_id}` : ""}`;
}
