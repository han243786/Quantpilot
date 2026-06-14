import { formatCount, formatTime } from "./strategyFormatters";

export function buildStrategyHubMetricCards(hubSummary) {
  return [
    {
      label: "策略文件",
      value: formatCount(hubSummary.trackedCount),
      note: "当前由后端真实追踪、可在策略中心加载的策略文件数量。"
    },
    {
      label: "可运行策略",
      value: formatCount(hubSummary.runnableCount),
      note: "当前没有明显阻塞，可以直接编译或启动模拟的策略数量。"
    },
    {
      label: "可研究策略",
      value: formatCount(hubSummary.researchReadyCount),
      note: "至少已经有一条持久化回测，可继续查看或对比的策略数量。"
    },
    {
      label: "最近活动",
      value: hubSummary.latestActivityAt ? formatTime(hubSummary.latestActivityAt) : "暂无活动",
      note: `对比队列：已选 ${formatCount(hubSummary.compareCount)} 项`
    }
  ];
}

export function buildStrategyHubOpsCards({ hubSummary, compareSelection = [], selectedStrategyCount = 0 }) {
  return [
    {
      title: "待修复",
      value: formatCount(hubSummary.issueCount),
      note: "当前仍被编译或校验问题阻塞的策略数量。",
      tone: "danger"
    },
    {
      title: "运行就绪",
      value: formatCount(hubSummary.runnableCount),
      note: "可直接进入模拟或回测复盘的策略数量。",
      tone: "success"
    },
    {
      title: "对比队列",
      value: formatCount(compareSelection.length),
      note: "进入对比页前，这里应刚好保留两条回测。",
      tone: "info"
    },
    {
      title: "已选策略",
      value: formatCount(selectedStrategyCount),
      note: "进入工作区前，可先用勾选把管理范围收敛到一小组策略。",
      tone: "muted"
    }
  ];
}
