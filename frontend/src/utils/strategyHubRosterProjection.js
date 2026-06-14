import { formatCount, formatPercent, formatTime } from "./strategyFormatters";

export function projectStrategyHubActivityItems(activityTimeline = []) {
  const projectItem = (item) => ({
    ...item,
    createdAtLabel: formatTime(item.createdAt)
  });

  return {
    backtestItems: activityTimeline.filter((item) => item.kind === "backtest").map(projectItem),
    runItems: activityTimeline.filter((item) => item.kind === "run").map(projectItem)
  };
}

export function projectStrategyHubRosterToolbar(model) {
  return {
    filteredCountLabel: formatCount(model.filteredStrategies.length),
    selectedCountLabel:
      model.selectedStrategyCount === 0
        ? "未选择策略"
        : `已选择 ${model.selectedStrategyCount} 条`,
    hasFilteredStrategies: model.filteredStrategies.length > 0,
    hasSelectedStrategies: model.selectedStrategyCount > 0,
    canOpenWorkspace: Boolean(model.selectedForWorkspace),
    workspaceLabel:
      model.selectedStrategyCount === 1 ? "打开已选工作区" : "打开当前查看工作区"
  };
}

export function projectStrategyHubRosterRows(model) {
  return model.filteredStrategies.map((entry) => ({
    graphId: entry.graphId,
    name: entry.name,
    healthTone: entry.health.tone,
    healthLabel: entry.health.label,
    activityLabel: entry.activityLabel,
    lastActivityLabel: formatTime(entry.lastActivityAt),
    runCountLabel: formatCount(entry.runCount),
    backtestCountLabel: formatCount(entry.backtestCount),
    latestReturnLabel: formatPercent(entry.latestReturnRatio),
    selected: model.selectedStrategyIds.includes(entry.graphId),
    active: model.selectedStrategy?.graphId === entry.graphId,
    hasFilePath: Boolean(entry.filePath)
  }));
}
