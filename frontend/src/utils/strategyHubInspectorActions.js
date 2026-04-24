import {
  navigateTo,
  strategyBacktestsPath,
  strategyWorkspacePath
} from "../router";

export function projectStrategyHubInspectorActionGroups(selectedStrategy) {
  if (!selectedStrategy) return [];

  return [
    {
      key: "build",
      label: "构建",
      tone: "info",
      items: [
        {
          key: "open-workspace",
          label: "打开工作区",
          ariaLabel: `打开 ${selectedStrategy.name} 工作区`
        }
      ]
    },
    {
      key: "research",
      label: "研究",
      tone: "muted",
      items: [
        {
          key: "open-backtests",
          label: "打开回测页",
          ariaLabel: `打开 ${selectedStrategy.name} 回测页`
        }
      ]
    },
    {
      key: "manage",
      label: "管理",
      tone: "muted",
      items: [
        {
          key: "refresh-strategy-data",
          label: "刷新策略数据",
          ariaLabel: `刷新 ${selectedStrategy.name} 策略数据`
        }
      ]
    }
  ];
}

export function runStrategyHubInspectorAction(model, selectedStrategy, actionKey) {
  switch (actionKey) {
    case "open-workspace":
      navigateTo(strategyWorkspacePath(selectedStrategy.graphId));
      return undefined;
    case "open-backtests":
      navigateTo(strategyBacktestsPath(selectedStrategy.graphId));
      return undefined;
    case "refresh-strategy-data":
      return Promise.all([model.refreshRunHistory(), model.refreshBacktestHistory()]);
    default:
      return undefined;
  }
}
