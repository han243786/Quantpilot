import { formatCount, formatPercent, formatTime } from "./strategyHubFormatters";

export function getStrategyInspectorNextMove(selectedStrategy) {
  if (!selectedStrategy) {
    return {
      title: "选择一条策略查看驾驶舱",
      description: "从左侧策略列表选择一条策略后，这里会展示近期研究、模拟和对比状态。"
    };
  }

  if (selectedStrategy.health.tone === "danger") {
    return {
      title: "进入诊断并修复问题",
      description: "当前策略仍有明显阻塞，建议先进入工作区处理诊断，再继续研究或模拟。"
    };
  }

  if (selectedStrategy.backtestCount > 0) {
    return {
      title: "查看近期研究结果",
      description: "这条策略已经有持久化回测，可以先查看详情或加入对比，再决定是否回到工作区继续迭代。"
    };
  }

  return {
    title: "打开工作区继续推进",
    description: "当前还没有形成足够的研究轨迹，进入工作区后可以继续编译、模拟或发起首个回测。"
  };
}

export function projectStrategyHubInspectorOverview(selectedStrategy) {
  if (!selectedStrategy) {
    return {
      emptyText: "选择一条策略以查看详情。"
    };
  }

  return {
    routeItems: [
      { label: "策略", current: false },
      { label: selectedStrategy.name, current: true }
    ],
    title: "策略驾驶舱",
    subtitle: "在保持清单与活动面板可见的同时，持续聚焦当前选中的这一条策略。",
    healthTone: selectedStrategy.health.tone,
    healthLabel: selectedStrategy.health.label,
    strategyName: selectedStrategy.name,
    strategyId: selectedStrategy.graphId,
    summaryItems: [
      {
        label: "当前压力",
        value: selectedStrategy.issueCount > 0 ? `${selectedStrategy.issueCount} 个问题` : "已清空"
      },
      {
        label: "研究深度",
        value: `${formatCount(selectedStrategy.backtestCount)} 条回测`
      },
      {
        label: "模拟轨迹",
        value: `${formatCount(selectedStrategy.runCount)} 次运行`
      }
    ],
    metrics: [
      { label: "最近编译", value: selectedStrategy.lastCompileId || "-" },
      { label: "协议", value: selectedStrategy.protocolName || "-" },
      { label: "配置哈希", value: selectedStrategy.lastConfigHash || "-" },
      { label: "数据集", value: selectedStrategy.datasetLabels.join(", ") || "-" }
    ],
    nextMove: getStrategyInspectorNextMove(selectedStrategy)
  };
}

export function projectInspectorBacktests(selectedStrategy, compareSelection = []) {
  if (!selectedStrategy) return [];

  return selectedStrategy.recentBacktests.map((item) => ({
    backtestId: item.backtest_id,
    createdAtLabel: formatTime(item.created_at_ms),
    returnLabel: formatPercent(item.summary?.total_return_ratio),
    checked: compareSelection.includes(item.backtest_id)
  }));
}

export function projectInspectorRuns(selectedStrategy) {
  if (!selectedStrategy) return [];

  return selectedStrategy.recentRuns.map((item) => ({
    runId: item.run_id,
    createdAtLabel: formatTime(item.created_at_ms),
    compileIdLabel: item.compile_id || "无编译 ID"
  }));
}

export function projectInspectorCompareQueue(compareSelection = []) {
  return {
    selectedIds: compareSelection,
    canCompare: compareSelection.length === 2
  };
}
