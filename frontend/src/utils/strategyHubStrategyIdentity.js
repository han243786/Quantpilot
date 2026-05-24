export function buildStrategyIdentity(strategy) {
  const name = strategy?.name || "未命名策略";
  if (!strategy?.graphId) return name;
  return `${name}（${strategy.graphId}）`;
}
