import { backtestComparePath, navigateTo } from "../router";

export function projectStrategyHubCompareQueueView(compareQueue = {}) {
  const selectedIds = compareQueue.selectedIds || [];

  return {
    title: "对比队列",
    description: "先在这里保留两条回测，再直接进入对比页，无需离开策略中心。",
    chips: selectedIds.length === 0 ? [] : selectedIds,
    emptyLabel: "未选择回测",
    actions: [
      {
        key: "clear-selection",
        label: "清空选择",
        ariaLabel: "清空当前对比选择",
        tone: "ghost",
        disabled: selectedIds.length === 0
      },
      {
        key: "open-compare",
        label: "打开对比",
        ariaLabel: "打开当前回测对比",
        tone: "primary",
        disabled: !compareQueue.canCompare
      }
    ]
  };
}

export function runStrategyHubCompareQueueAction(
  graphId,
  compareQueue,
  actionKey,
  onClearSelection
) {
  switch (actionKey) {
    case "clear-selection":
      return onClearSelection();
    case "open-compare":
      navigateTo(backtestComparePath(compareQueue.selectedIds, graphId));
      return undefined;
    default:
      return undefined;
  }
}
