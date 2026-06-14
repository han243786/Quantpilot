import {
  DEFAULT_WORKSPACE_ISSUE_FILTERS,
  filterWorkspaceIssueQueue,
  filterWorkspaceIssueQueueByNodeType,
  filterWorkspaceIssueQueueBySource,
  workspaceIssueFiltersDirty,
  workspaceIssueQueueCounts,
  workspaceIssueQueueNodeTypeCounts,
  workspaceIssueQueueNodeTypeOrder,
  workspaceIssueQueueSourceCounts,
  workspaceIssueQueueSourceOrder
} from "../utils/strategyWorkspaceIssueQueue";

export const WORKSPACE_ISSUE_FILTERS_STORAGE_KEY = "quantpilot_workspace_issue_filters";

export function workspaceIssueFiltersStorageScope(strategyId, graphId) {
  return strategyId || graphId || "draft_graph";
}

export function readStoredWorkspaceIssueFilters(scope) {
  if (typeof window === "undefined" || !window.localStorage) {
    return DEFAULT_WORKSPACE_ISSUE_FILTERS;
  }

  try {
    const raw = window.localStorage.getItem(WORKSPACE_ISSUE_FILTERS_STORAGE_KEY);
    if (!raw) return DEFAULT_WORKSPACE_ISSUE_FILTERS;
    const parsed = JSON.parse(raw);
    if (!parsed || typeof parsed !== "object") {
      return DEFAULT_WORKSPACE_ISSUE_FILTERS;
    }
    const storedFilters = parsed[scope];
    if (!storedFilters || typeof storedFilters !== "object") {
      return DEFAULT_WORKSPACE_ISSUE_FILTERS;
    }
    return {
      ...DEFAULT_WORKSPACE_ISSUE_FILTERS,
      ...storedFilters
    };
  } catch (e) {
    console.warn("strategyWorkspaceIssueQueueState: read filters failed", e);
    return DEFAULT_WORKSPACE_ISSUE_FILTERS;
  }
}

export function persistWorkspaceIssueFilters(scope, filters) {
  if (typeof window === "undefined" || !window.localStorage) {
    return;
  }

  try {
    const raw = window.localStorage.getItem(WORKSPACE_ISSUE_FILTERS_STORAGE_KEY);
    const parsed = raw ? JSON.parse(raw) : {};
    const nextPayload = parsed && typeof parsed === "object" ? parsed : {};
    nextPayload[scope] = {
      ...DEFAULT_WORKSPACE_ISSUE_FILTERS,
      ...(filters || {})
    };
    window.localStorage.setItem(WORKSPACE_ISSUE_FILTERS_STORAGE_KEY, JSON.stringify(nextPayload));
  } catch (e) {
    console.warn("strategyWorkspaceIssueQueueState: persist filters failed", e);
  }
}

export function normalizeWorkspaceIssueFilters(filters, items = []) {
  const nextFilters = {
    ...DEFAULT_WORKSPACE_ISSUE_FILTERS,
    ...(filters || {})
  };
  const baseFilteredItems = filterWorkspaceIssueQueue(
    items,
    nextFilters.severityFilter,
    nextFilters.actionableOnly
  );
  const orderedSources = workspaceIssueQueueSourceOrder(items);
  if (
    nextFilters.sourceFilter !== "all" &&
    !orderedSources.includes(nextFilters.sourceFilter)
  ) {
    nextFilters.sourceFilter = "all";
  }
  if (nextFilters.sourceFilter === "all") {
    nextFilters.nodeTypeFilter = "all";
    return nextFilters;
  }
  const sourceFilteredItems = filterWorkspaceIssueQueueBySource(
    baseFilteredItems,
    nextFilters.sourceFilter
  );
  const orderedNodeTypes = workspaceIssueQueueNodeTypeOrder(sourceFilteredItems);
  if (
    nextFilters.nodeTypeFilter !== "all" &&
    !orderedNodeTypes.includes(nextFilters.nodeTypeFilter)
  ) {
    nextFilters.nodeTypeFilter = "all";
  }
  return nextFilters;
}

export function workspaceIssueFiltersSummary(filters) {
  const current = {
    ...DEFAULT_WORKSPACE_ISSUE_FILTERS,
    ...(filters || {})
  };
  const parts = [];

  if (current.severityFilter !== "all") {
    parts.push(
      current.severityFilter === "error"
        ? "\u9519\u8bef"
        : current.severityFilter === "warning"
          ? "\u8b66\u544a"
          : current.severityFilter
    );
  }
  if (current.actionableOnly) {
    parts.push("\u4ec5\u53ef\u64cd\u4f5c\u9879");
  }
  if (current.sourceFilter !== "all") {
    parts.push(current.sourceFilter);
  }
  if (current.nodeTypeFilter !== "all") {
    parts.push(current.nodeTypeFilter);
  }

  return parts.length > 0 ? parts.join(" / ") : "\u65e0\u6d3b\u52a8\u7b5b\u9009";
}

export function buildWorkspaceIssueQueueFilterModel(items = [], filters) {
  const currentFilters = {
    ...DEFAULT_WORKSPACE_ISSUE_FILTERS,
    ...(filters || {})
  };
  const counts = workspaceIssueQueueCounts(items);
  const sourceCounts = workspaceIssueQueueSourceCounts(items);
  const orderedSources = workspaceIssueQueueSourceOrder(items);
  const baseFilteredItems = filterWorkspaceIssueQueue(
    items,
    currentFilters.severityFilter,
    currentFilters.actionableOnly
  );
  const sourceFilteredItems = filterWorkspaceIssueQueueBySource(
    baseFilteredItems,
    currentFilters.sourceFilter
  );
  const nodeTypeCounts = workspaceIssueQueueNodeTypeCounts(sourceFilteredItems);
  const orderedNodeTypes = workspaceIssueQueueNodeTypeOrder(sourceFilteredItems);
  const filteredItems = filterWorkspaceIssueQueueByNodeType(
    sourceFilteredItems,
    currentFilters.nodeTypeFilter
  );

  return {
    filters: currentFilters,
    isDirty: workspaceIssueFiltersDirty(currentFilters),
    counts,
    sourceCounts,
    orderedSources,
    sourceFilteredItems,
    nodeTypeCounts,
    orderedNodeTypes,
    filteredItems
  };
}
