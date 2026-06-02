export const categoryOrder = ["data", "intent", "agent", "risk", "execution", "runtime"];
export const initialExpandedGroups = Object.fromEntries(categoryOrder.map((category) => [category, true]));

function uniqueCategoryOrder(priority = []) {
  const seen = new Set();
  return [...priority, ...categoryOrder].filter((category) => {
    if (seen.has(category)) return false;
    seen.add(category);
    return true;
  });
}

function lanePriorityCategories(laneId) {
  if (laneId === "diagnostics") {
    return uniqueCategoryOrder(["execution", "risk", "runtime", "agent"]);
  }
  if (laneId === "code") {
    return uniqueCategoryOrder(["intent", "agent", "execution", "runtime"]);
  }
  return uniqueCategoryOrder(["data", "intent", "agent"]);
}

export function buildPrioritizedCategories(laneId, selectedNodeType = null) {
  const laneCategories = lanePriorityCategories(laneId);
  if (!selectedNodeType) {
    return laneCategories;
  }
  if (!laneId) {
    return uniqueCategoryOrder([selectedNodeType, ...laneCategories]);
  }
  if (laneCategories.includes(selectedNodeType)) {
    return laneCategories;
  }
  return uniqueCategoryOrder([
    laneCategories[0],
    selectedNodeType,
    ...laneCategories.slice(1)
  ]);
}

export function laneRecommendation(laneId, laneLabel, selectedNodeType = null) {
  if (laneId === "diagnostics") {
    return selectedNodeType
      ? `处理阻塞问题时，先关注执行、风控与运行时模块。当前选中项锚定在 ${selectedNodeType}，因此也请把这一类模块放在附近。`
      : "处理阻塞问题时，先关注执行、风控与运行时模块。";
  }
  if (laneId === "code") {
    return selectedNodeType
      ? `优先处理通常与源码工件或策略中间表示联动的模块，再补上与当前选中项相邻的 ${selectedNodeType} 模块。`
      : "优先处理通常与源码工件或策略中间表示联动的模块。";
  }
  if (selectedNodeType) {
    return `调整结构时，把最贴近${laneLabel || "当前构建路径"}的模块放在顶部，并让 ${selectedNodeType} 模块贴近当前选中项。`;
  }
  return `调整结构时，把最贴近${laneLabel || "当前构建路径"}的模块放在顶部。`;
}

export function moduleAvailabilityTone(status) {
  if (status === "unsupported") return "warning";
  return "success";
}

export function moduleAvailabilityLabel(status, t) {
  if (status === "unsupported") return t("\u5df2\u9501\u5b9a");
  return t("\u53ef\u7528");
}

export function buildCategoryLabels(t) {
  return {
    data: t("\u6570\u636e\u6a21\u5757"),
    intent: t("\u610f\u56fe\u6a21\u5757"),
    agent: t("\u4ee3\u7406\u6a21\u5757"),
    risk: t("\u98ce\u63a7\u6a21\u5757"),
    execution: t("\u6267\u884c\u6a21\u5757"),
    runtime: t("\u8fd0\u884c\u65f6\u6a21\u5757")
  };
}
