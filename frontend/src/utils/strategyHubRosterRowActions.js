import { navigateTo, strategyBacktestsPath, strategyWorkspacePath } from "../router";

export function projectStrategyHubRosterRowActionGroups(row) {
  return [
    {
      key: "build",
      label: "构建",
      tone: "info",
      items: [
        {
          key: "open-workspace",
          label: "打开工作区",
          ariaLabel: `打开 ${row.name} 工作区`,
          disabled: false
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
          ariaLabel: `打开 ${row.name} 回测页`,
          disabled: false
        }
      ]
    },
    {
      key: "files",
      label: "文件",
      tone: "muted",
      items: [
        {
          key: "reveal-file",
          label: "打开文件位置",
          ariaLabel: `打开 ${row.name} 文件位置`,
          disabled: !row.hasFilePath
        }
      ]
    },
    {
      key: "manage",
      label: "管理",
      tone: "danger",
      items: [
        {
          key: "delete-strategy",
          label: "删除策略",
          ariaLabel: `删除 ${row.name} 策略`,
          disabled: false,
          buttonClassName: "danger-btn compact-btn"
        }
      ]
    }
  ];
}

export function runStrategyHubRosterRowAction(model, row, actionKey) {
  switch (actionKey) {
    case "open-workspace":
      navigateTo(strategyWorkspacePath(row.graphId));
      return undefined;
    case "open-backtests":
      navigateTo(strategyBacktestsPath(row.graphId));
      return undefined;
    case "reveal-file":
      return model.revealGraphFile(row.graphId);
    case "delete-strategy":
      return model.deleteStrategy(row.graphId, row.name);
    default:
      return undefined;
  }
}
