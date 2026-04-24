import { backtestDetailPath, navigateTo } from "../router";

export function projectStrategyHubRecentBacktestActionGroup(item) {
  return {
    label: "研究",
    tone: "info",
    items: [
      {
        key: "open-detail",
        label: "详情",
        ariaLabel: `打开 ${item.backtestId} 详情`
      },
      {
        key: "toggle-compare",
        label: item.checked ? "已选择" : "加入对比",
        ariaLabel: item.checked
          ? `将 ${item.backtestId} 从对比中移除`
          : `将 ${item.backtestId} 加入对比`,
        selected: item.checked
      }
    ]
  };
}

export function runStrategyHubRecentBacktestAction(graphId, item, actionKey, onToggleCompare) {
  switch (actionKey) {
    case "open-detail":
      navigateTo(backtestDetailPath(item.backtestId, graphId));
      return undefined;
    case "toggle-compare":
      return onToggleCompare(item.backtestId);
    default:
      return undefined;
  }
}
