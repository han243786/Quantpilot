import { navigateTo, strategyWorkspacePath } from "../router";

export default function StrategyHubRosterToolbar({ model, toolbar }) {
  return (
    <div className="strategy-roster-toolbar">
      <div className="strategy-roster-toolbar__copy">
        <strong>批量操作</strong>
        <span>{toolbar.selectedCountLabel}</span>
      </div>
      <div className="strategy-roster-toolbar__actions">
        <button
          className="ad-btn ad-btn--ghost compact-btn"
          onClick={() =>
            model.setSelectedStrategyIds(model.filteredStrategies.map((entry) => entry.graphId))
          }
          disabled={!toolbar.hasFilteredStrategies}
        >
          选择全部策略
        </button>
        <button
          className="ad-btn ad-btn--ghost compact-btn"
          onClick={() => model.setSelectedStrategyIds([])}
          disabled={!toolbar.hasSelectedStrategies}
        >
          清空选择
        </button>
        <button
          className="ad-btn ad-btn--primary compact-btn"
          disabled={!toolbar.canOpenWorkspace}
          onClick={() => navigateTo(strategyWorkspacePath(model.selectedForWorkspace))}
        >
          {toolbar.workspaceLabel}
        </button>
      </div>
    </div>
  );
}
