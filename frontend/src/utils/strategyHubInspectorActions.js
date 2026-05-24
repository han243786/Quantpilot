import {
  navigateTo,
  strategyBacktestsPath,
  strategyWorkspacePath
} from "../router";
import { buildStrategyIdentity } from "./strategyHubStrategyIdentity";

export function projectStrategyHubInspectorActionGroups(selectedStrategy) {
  if (!selectedStrategy) return [];
  const strategyIdentity = buildStrategyIdentity(selectedStrategy);

  return [
    {
      key: "build",
      label: "构建",
      tone: "info",
      items: [
        {
          key: "open-workspace",
          label: "打开工作区",
          ariaLabel: `打开策略 ${strategyIdentity}的工作区`
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
          ariaLabel: `打开策略 ${strategyIdentity}的回测页`
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
          ariaLabel: `刷新策略 ${strategyIdentity}的数据`
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
