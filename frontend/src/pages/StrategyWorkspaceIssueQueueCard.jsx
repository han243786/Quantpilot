import { useMemo } from "react";
import { StrategyCardNote } from "./StrategyHubSharedComponents";
import { buildRepairPathInsight } from "../utils/repairPathInsights";
import {
  DEFAULT_WORKSPACE_ISSUE_FILTERS,
  diagnosticQueueNodeType,
  diagnosticQueueSource,
  filterWorkspaceIssueQueue,
  filterWorkspaceIssueQueueByNodeType,
  filterWorkspaceIssueQueueBySource,
  workspaceIssueFiltersDirty,
  workspaceIssueQueueCounts,
  workspaceIssueQueueNodeTypeCounts,
  workspaceIssueQueueNodeTypeOrder,
  workspaceIssueQueueSeverityLabel,
  workspaceIssueQueueSourceCounts,
  workspaceIssueQueueSourceOrder,
  workspaceIssueSeverityText
} from "../utils/strategyWorkspaceIssueQueue";

function severityTone(severity) {
  if (severity === "warning") return "warning";
  if (severity === "info") return "info";
  return "danger";
}

function WorkspaceIssueQueueCard({
  title,
  subtitle,
  items,
  emptyText,
  actionLabel = null,
  onAction = null,
  onSelectItem,
  filters,
  onFiltersChange,
  graph = null,
  repairPathState = null
}) {
  const {
    severityFilter,
    actionableOnly,
    showSourceFilters,
    sourceFilter,
    nodeTypeFilter
  } = {
    ...DEFAULT_WORKSPACE_ISSUE_FILTERS,
    ...(filters || {})
  };
  const isDirty = workspaceIssueFiltersDirty(filters);
  const counts = useMemo(() => workspaceIssueQueueCounts(items), [items]);
  const sourceCounts = useMemo(() => workspaceIssueQueueSourceCounts(items), [items]);
  const orderedSources = useMemo(() => workspaceIssueQueueSourceOrder(items), [items]);
  const baseFilteredItems = useMemo(
    () => filterWorkspaceIssueQueue(items, severityFilter, actionableOnly),
    [actionableOnly, items, severityFilter]
  );
  const sourceFilteredItems = useMemo(
    () => filterWorkspaceIssueQueueBySource(baseFilteredItems, sourceFilter),
    [baseFilteredItems, sourceFilter]
  );
  const nodeTypeCounts = useMemo(
    () => workspaceIssueQueueNodeTypeCounts(sourceFilteredItems),
    [sourceFilteredItems]
  );
  const orderedNodeTypes = useMemo(
    () => workspaceIssueQueueNodeTypeOrder(sourceFilteredItems),
    [sourceFilteredItems]
  );
  const filteredItems = useMemo(
    () => filterWorkspaceIssueQueueByNodeType(sourceFilteredItems, nodeTypeFilter),
    [nodeTypeFilter, sourceFilteredItems]
  );

  const updateFilters = (patch) => {
    const nextPatch =
      typeof patch === "function" ? patch(filters || DEFAULT_WORKSPACE_ISSUE_FILTERS) : patch;
    onFiltersChange?.(nextPatch);
  };

  return (
    <div className="workspace-issue-queue">
      <div className="workspace-issue-queue__header">
        <div>
          <div className="mini-list-title strategy-card-title-note">
            <StrategyCardNote label={title} note={subtitle} />
          </div>
        </div>
        <div className="workspace-issue-queue__actions">
          {isDirty ? (
            <button
              type="button"
              className="ad-btn ad-btn--ghost compact-btn"
              onClick={() => updateFilters(DEFAULT_WORKSPACE_ISSUE_FILTERS)}
            >
              重置筛选
            </button>
          ) : null}
          {actionLabel && onAction ? (
            <button className="ad-btn ad-btn--ghost compact-btn" onClick={onAction}>
              {actionLabel}
            </button>
          ) : null}
        </div>
      </div>

      <div className="workspace-issue-queue__filters">
        <div className="workspace-issue-queue__severity">
          {[
            { id: "all", label: workspaceIssueQueueSeverityLabel("all", items.length) },
            { id: "error", label: workspaceIssueQueueSeverityLabel("error", counts.error) },
            { id: "warning", label: workspaceIssueQueueSeverityLabel("warning", counts.warning) }
          ].map((option) => (
            <button
              key={option.id}
              type="button"
              className={`ad-btn ad-btn--ghost compact-btn workspace-issue-queue__filter${
                severityFilter === option.id ? " workspace-issue-queue__filter--active" : ""
              }`}
              onClick={() => updateFilters({ severityFilter: option.id })}
            >
              {option.label}
            </button>
          ))}
        </div>
        <button
          type="button"
          className={`ad-btn ad-btn--ghost compact-btn workspace-issue-queue__filter${
            actionableOnly ? " workspace-issue-queue__filter--active" : ""
          }`}
          onClick={() => updateFilters((current) => ({ actionableOnly: !current.actionableOnly }))}
        >
          {`可定位 ${counts.actionable}`}
        </button>
        <button
          type="button"
          className={`ad-btn ad-btn--ghost compact-btn workspace-issue-queue__filter${
            showSourceFilters ? " workspace-issue-queue__filter--active" : ""
          }`}
          onClick={() => updateFilters((current) => ({ showSourceFilters: !current.showSourceFilters }))}
        >
          {`来源 ${orderedSources.length}`}
        </button>
      </div>

      {showSourceFilters ? (
        <div className="workspace-issue-queue__sources">
          <button
            type="button"
            className={`ad-btn ad-btn--ghost compact-btn workspace-issue-queue__filter${
              sourceFilter === "all" ? " workspace-issue-queue__filter--active" : ""
            }`}
            onClick={() => updateFilters({ sourceFilter: "all", nodeTypeFilter: "all" })}
          >
            {`全部来源 ${items.length}`}
          </button>
          {orderedSources.map((source) => (
            <button
              key={source}
              type="button"
              className={`ad-btn ad-btn--ghost compact-btn workspace-issue-queue__filter${
                sourceFilter === source ? " workspace-issue-queue__filter--active" : ""
              }`}
              onClick={() => updateFilters({ sourceFilter: source, nodeTypeFilter: "all" })}
            >
              {`${diagnosticQueueSource({ source })} ${sourceCounts[source] || 0}`}
            </button>
          ))}
        </div>
      ) : null}

      {showSourceFilters && sourceFilter !== "all" && orderedNodeTypes.length > 0 ? (
        <div className="workspace-issue-queue__node-types">
          <button
            type="button"
            className={`ad-btn ad-btn--ghost compact-btn workspace-issue-queue__filter${
              nodeTypeFilter === "all" ? " workspace-issue-queue__filter--active" : ""
            }`}
            onClick={() => updateFilters({ nodeTypeFilter: "all" })}
          >
            {`全部节点类型 ${sourceFilteredItems.length}`}
          </button>
          {orderedNodeTypes.map((nodeType) => (
            <button
              key={nodeType}
              type="button"
              className={`ad-btn ad-btn--ghost compact-btn workspace-issue-queue__filter${
                nodeTypeFilter === nodeType ? " workspace-issue-queue__filter--active" : ""
              }`}
              onClick={() => updateFilters({ nodeTypeFilter: nodeType })}
            >
              {`${diagnosticQueueNodeType({ nodeType })} ${nodeTypeCounts[nodeType] || 0}`}
            </button>
          ))}
        </div>
      ) : null}

      {filteredItems.length === 0 ? (
        <div className="muted-line">{emptyText}</div>
      ) : (
        <div className="workspace-issue-queue__list">
          {filteredItems.map((item) => {
            const repairPathInsight = buildRepairPathInsight(
              item.routeDiagnostic?.target || null,
              graph,
              repairPathState
            );

            return (
              <button
                key={item.id}
                type="button"
                className={`workspace-issue-queue__item${
                  item.actionable ? " workspace-issue-queue__item--actionable" : ""
                }${repairPathInsight ? " workspace-issue-queue__item--path" : ""}`}
                onClick={() => onSelectItem(item)}
              >
                <div className="workspace-issue-queue__meta">
                  <span className={`status-pill ${severityTone(item.severity)}`}>
                    {workspaceIssueSeverityText(item.severity)}
                  </span>
                  <span className="diagnostic-chip">{diagnosticQueueSource(item)}</span>
                  {item.actionable ? (
                    <span className="diagnostic-chip">可定位目标</span>
                  ) : null}
                  {repairPathInsight ? (
                    <>
                      <span className="diagnostic-chip diagnostic-chip--path">
                        {repairPathInsight.chip}
                      </span>
                      <span className="diagnostic-chip diagnostic-chip--segment">
                        {repairPathInsight.segment}
                      </span>
                    </>
                  ) : null}
                </div>
                <div className="workspace-issue-queue__title">{item.title}</div>
                <div className="workspace-issue-queue__message">{item.message}</div>
                {repairPathInsight ? (
                  <div className="workspace-issue-queue__path-note">{repairPathInsight.note}</div>
                ) : null}
                {item.note ? <div className="workspace-issue-queue__note">{item.note}</div> : null}
              </button>
            );
          })}
        </div>
      )}
    </div>
  );
}

export { WorkspaceIssueQueueCard };
