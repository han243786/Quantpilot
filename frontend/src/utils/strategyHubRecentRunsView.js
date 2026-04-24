export function projectStrategyHubRecentRunsView(items = []) {
  return {
    title: "近期模拟",
    emptyText: "这条策略暂无近期模拟。",
    items: items.map((item) => ({
      ...item,
      statusTone: "info"
    }))
  };
}
