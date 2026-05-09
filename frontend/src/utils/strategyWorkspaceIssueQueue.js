export const DEFAULT_WORKSPACE_ISSUE_FILTERS = {
  severityFilter: "all",
  actionableOnly: false,
  showSourceFilters: false,
  sourceFilter: "all",
  nodeTypeFilter: "all"
};

function severityRank(severity) {
  if (severity === "warning") return 1;
  if (severity === "info") return 2;
  return 0;
}

export function diagnosticQueueSource(item) {
  if (item.source === "validation") return "校验";
  if (item.source === "strategy_ir") return "策略 IR";
  if (item.source === "runtime") return "运行时";
  if (item.source === "formal_quantscript") return "正式 QS";
  return "策略图";
}

export function diagnosticQueueNodeType(item) {
  if (!item.nodeType) return "策略图";
  return item.nodeType;
}

export function workspaceIssueQueueSeverityLabel(severity, count) {
  if (severity === "error") return `错误 ${count}`;
  if (severity === "warning") return `警告 ${count}`;
  if (severity === "info") return `提示 ${count}`;
  return `全部 ${count}`;
}

export function workspaceIssueSeverityText(severity) {
  if (severity === "warning") return "警告";
  if (severity === "info") return "提示";
  return "错误";
}

export function workspaceIssueQueueCounts(items = []) {
  return items.reduce(
    (summary, item) => {
      if (item.severity === "warning") {
        summary.warning += 1;
      } else if (item.severity === "info") {
        summary.info += 1;
      } else {
        summary.error += 1;
      }
      if (item.actionable) {
        summary.actionable += 1;
      }
      return summary;
    },
    { error: 0, warning: 0, info: 0, actionable: 0 }
  );
}

export function filterWorkspaceIssueQueue(
  items = [],
  severityFilter = "all",
  actionableOnly = false
) {
  return items.filter((item) => {
    if (severityFilter !== "all" && item.severity !== severityFilter) {
      return false;
    }
    if (actionableOnly && !item.actionable) {
      return false;
    }
    return true;
  });
}

export function workspaceIssueQueueSourceCounts(items = []) {
  return items.reduce((summary, item) => {
    const sourceKey = item.source || "graph";
    summary[sourceKey] = (summary[sourceKey] || 0) + 1;
    return summary;
  }, {});
}

export function filterWorkspaceIssueQueueBySource(items = [], sourceFilter = "all") {
  if (sourceFilter === "all") return items;
  return items.filter((item) => (item.source || "graph") === sourceFilter);
}

export function workspaceIssueQueueSourceOrder(items = []) {
  const sourceCounts = workspaceIssueQueueSourceCounts(items);
  return Object.keys(sourceCounts).sort((left, right) => {
    const countDelta = sourceCounts[right] - sourceCounts[left];
    if (countDelta !== 0) return countDelta;
    return diagnosticQueueSource({ source: left }).localeCompare(
      diagnosticQueueSource({ source: right })
    );
  });
}

export function workspaceIssueQueueNodeTypeCounts(items = []) {
  return items.reduce((summary, item) => {
    const nodeTypeKey = item.nodeType || "graph";
    summary[nodeTypeKey] = (summary[nodeTypeKey] || 0) + 1;
    return summary;
  }, {});
}

export function filterWorkspaceIssueQueueByNodeType(items = [], nodeTypeFilter = "all") {
  if (nodeTypeFilter === "all") return items;
  return items.filter((item) => (item.nodeType || "graph") === nodeTypeFilter);
}

export function workspaceIssueQueueNodeTypeOrder(items = []) {
  const nodeTypeCounts = workspaceIssueQueueNodeTypeCounts(items);
  return Object.keys(nodeTypeCounts).sort((left, right) => {
    const countDelta = nodeTypeCounts[right] - nodeTypeCounts[left];
    if (countDelta !== 0) return countDelta;
    return diagnosticQueueNodeType({ nodeType: left }).localeCompare(
      diagnosticQueueNodeType({ nodeType: right })
    );
  });
}

export function workspaceIssueFiltersDirty(filters) {
  const current = {
    ...DEFAULT_WORKSPACE_ISSUE_FILTERS,
    ...(filters || {})
  };
  return (
    current.severityFilter !== DEFAULT_WORKSPACE_ISSUE_FILTERS.severityFilter ||
    current.actionableOnly !== DEFAULT_WORKSPACE_ISSUE_FILTERS.actionableOnly ||
    current.showSourceFilters !== DEFAULT_WORKSPACE_ISSUE_FILTERS.showSourceFilters ||
    current.sourceFilter !== DEFAULT_WORKSPACE_ISSUE_FILTERS.sourceFilter ||
    current.nodeTypeFilter !== DEFAULT_WORKSPACE_ISSUE_FILTERS.nodeTypeFilter
  );
}

export function buildWorkspaceIssueQueue(graph, compileDiagnostics = []) {
  const nodeMap = new Map((graph.nodes || []).map((node) => [node.id, node]));
  const validationNodeIssues = graph.validation_state?.node_issues || {};
  const graphIssues = graph.validation_state?.graph_issues || [];
  const queueItems = [];
  const seen = new Set();

  compileDiagnostics.forEach((diagnostic, index) => {
    const target = diagnostic?.target || null;
    const targetNode = target?.node_id ? nodeMap.get(target.node_id) : null;
    const queueId = `${diagnostic.code || "compile"}_${target?.node_id || target?.edge_id || index}`;
    if (seen.has(queueId)) return;
    seen.add(queueId);

    queueItems.push({
      id: queueId,
      severity: diagnostic.severity || "error",
      source: diagnostic.source || "graph",
      nodeType: targetNode?.type || null,
      title: target?.label || targetNode?.name || diagnostic.code || "编译诊断",
      message: diagnostic.message,
      note: diagnostic.hint || "",
      routeDiagnostic: diagnostic.target ? diagnostic : null,
      actionable: Boolean(diagnostic.target)
    });
  });

  Object.entries(validationNodeIssues).forEach(([nodeId, issues]) => {
    const node = nodeMap.get(nodeId);
    (issues || []).forEach((issue) => {
      const queueId = issue.id || `validation_${nodeId}_${issue.code || issue.message}`;
      if (seen.has(queueId)) return;
      seen.add(queueId);

      queueItems.push({
        id: queueId,
        severity: issue.level === "warning" ? "warning" : issue.level === "info" ? "info" : "error",
        source: "validation",
        nodeType: node?.type || null,
        title: node?.name || nodeId,
        message: issue.message,
        note: issue.hint || "",
        routeDiagnostic: {
          code: issue.code || "VALIDATION_ISSUE",
          source: "graph",
          severity: issue.level === "warning" ? "warning" : issue.level === "info" ? "info" : "error",
          message: issue.message,
          hint: issue.hint || "",
          target: {
            scope: "node",
            node_id: nodeId,
            label: node?.name || nodeId
          }
        },
        actionable: true
      });
    });
  });

  graphIssues.forEach((issue) => {
    const queueId = issue.id || `graph_${issue.code || issue.message}`;
    if (seen.has(queueId)) return;
    seen.add(queueId);

    queueItems.push({
      id: queueId,
      severity: issue.level === "warning" ? "warning" : issue.level === "info" ? "info" : "error",
      source: "validation",
      nodeType: null,
      title: "策略图",
      message: issue.message,
      note: issue.hint || "",
      routeDiagnostic: null,
      actionable: false
    });
  });

  return queueItems
    .sort((left, right) => {
      const severityDelta = severityRank(left.severity) - severityRank(right.severity);
      if (severityDelta !== 0) return severityDelta;
      if (left.actionable !== right.actionable) return left.actionable ? -1 : 1;
      return left.title.localeCompare(right.title);
    })
    .slice(0, 5);
}
