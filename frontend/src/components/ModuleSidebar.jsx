import { useMemo, useState } from "react";
import { useI18n } from "../i18n";
import { isCapabilitySyncBlocked } from "../capabilities/supportMatrix";
import { StrategyCardNote } from "../pages/StrategyHubSharedComponents";
import { useGraphStore } from "../store/graphStore";

const categoryOrder = ["data", "intent", "agent", "risk", "execution", "runtime"];
const initialExpandedGroups = Object.fromEntries(categoryOrder.map((category) => [category, true]));

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

function buildPrioritizedCategories(laneId, selectedNodeType = null) {
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

function laneRecommendation(laneId, laneLabel, selectedNodeType = null) {
  if (laneId === "diagnostics") {
    return selectedNodeType
      ? `处理阻塞问题时，先关注执行、风控与运行时模块。当前选中项锚定在 ${selectedNodeType}，因此也请把这一类模块放在附近。`
      : "处理阻塞问题时，先关注执行、风控与运行时模块。";
  }
  if (laneId === "code") {
    return selectedNodeType
      ? `优先处理通常与源码工件或 Strategy IR 联动的模块，再补上与当前选中项相邻的 ${selectedNodeType} 模块。`
      : "优先处理通常与源码工件或 Strategy IR 联动的模块。";
  }
  if (selectedNodeType) {
    return `调整结构时，把最贴近${laneLabel || "当前构建路径"}的模块放在顶部，并让 ${selectedNodeType} 模块贴近当前选中项。`;
  }
  return `调整结构时，把最贴近${laneLabel || "当前构建路径"}的模块放在顶部。`;
}

function moduleAvailabilityTone(status) {
  if (status === "unsupported") return "warning";
  return "success";
}

function moduleAvailabilityLabel(status, t) {
  if (status === "unsupported") return t("\u5df2\u9501\u5b9a");
  return t("\u53ef\u7528");
}

function buildCategoryLabels(t) {
  return {
    data: t("\u6570\u636e\u6a21\u5757"),
    intent: t("\u610f\u56fe\u6a21\u5757"),
    agent: t("\u4ee3\u7406\u6a21\u5757"),
    risk: t("\u98ce\u63a7\u6a21\u5757"),
    execution: t("\u6267\u884c\u6a21\u5757"),
    runtime: t("\u8fd0\u884c\u65f6\u6a21\u5757")
  };
}

export default function ModuleSidebar({ workspaceContext = null }) {
  const { t } = useI18n();
  const registry = useGraphStore((state) => state.registry);
  const graph = useGraphStore((state) => state.graph);
  const selectedNodeId = useGraphStore((state) => state.selectedNodeId);
  const createNode = useGraphStore((state) => state.createNode);
  const capabilityStatus = useGraphStore((state) => state.capabilityStatus);
  const capabilitySource = useGraphStore((state) => state.capabilitySource);
  const capabilityMessage = useGraphStore((state) => state.capabilityMessage);
  const [keyword, setKeyword] = useState("");
  const [expandedGroups, setExpandedGroups] = useState(initialExpandedGroups);
  const capabilitySyncBlocked = isCapabilitySyncBlocked(capabilityStatus, capabilitySource);
  const hasSearch = keyword.trim().length > 0;
  const sidebarTitle = t("\u6a21\u5757\u6a21\u677f");
  const sidebarNote = t(
    "\u53ea\u5c55\u793a\u5f53\u524d\u58f0\u660e\u7684\u6a21\u5757\u8fb9\u754c\uff1b\u4e0d\u53ef\u7528\u6a21\u5757\u4fdd\u7559\u5361\u7247\uff0c\u5e76\u660e\u786e\u8bf4\u660e\u9501\u5b9a\u539f\u56e0\u3002"
  );

  const localizedCategoryLabels = useMemo(() => buildCategoryLabels(t), [t]);
  const allModules = useMemo(() => registry.getAll(), [registry]);
  const nodeMap = useMemo(
    () => new Map((graph.nodes || []).map((node) => [node.id, node])),
    [graph.nodes]
  );
  const selectedNode = selectedNodeId ? nodeMap.get(selectedNodeId) || null : null;
  const workspaceLaneId = workspaceContext?.laneId || null;
  const prioritizedCategories = useMemo(
    () => buildPrioritizedCategories(workspaceLaneId, selectedNode?.type || null),
    [selectedNode?.type, workspaceLaneId]
  );
  const promotedCategories = useMemo(
    () => new Set(prioritizedCategories.slice(0, workspaceLaneId ? 3 : 1)),
    [prioritizedCategories, workspaceLaneId]
  );

  const grouped = useMemo(() => {
    const normalizedKeyword = keyword.trim().toLowerCase();
    const filtered = allModules.filter((item) => {
      if (!normalizedKeyword) return true;
      return `${item.display_name} ${item.description} ${item.module_key}`
        .toLowerCase()
        .includes(normalizedKeyword);
    });

    return prioritizedCategories
      .map((category) => {
        const items = filtered.filter((item) => item.category === category);
        return {
          category,
          items,
          isExpanded: hasSearch ? items.length > 0 : expandedGroups[category] !== false
        };
      })
      .filter((group) => group.items.length > 0);
  }, [allModules, expandedGroups, hasSearch, keyword, prioritizedCategories]);

  const structureCounts = useMemo(
    () =>
      categoryOrder.map((category) => ({
        category,
        count: (graph.nodes || []).filter((node) => node.type === category).length
      })),
    [graph.nodes]
  );

  const recentModules = useMemo(() => {
    const moduleMap = new Map(allModules.map((item) => [item.module_key, item]));
    const recentNodeIds = Array.isArray(graph.metadata?.editor?.recent_node_ids)
      ? graph.metadata.editor.recent_node_ids
      : [];
    const seen = new Set();

    return recentNodeIds
      .map((nodeId) => nodeMap.get(nodeId))
      .filter(Boolean)
      .map((node) => node.module_key)
      .filter((moduleKey) => {
        if (!moduleKey || seen.has(moduleKey)) return false;
        seen.add(moduleKey);
        return true;
      })
      .map((moduleKey) => moduleMap.get(moduleKey))
      .filter(Boolean)
      .slice(0, 4);
  }, [allModules, graph.metadata, nodeMap]);

  const recommendedModules = useMemo(() => {
    if (!workspaceContext) return [];

    const recentModuleKeys = new Set(recentModules.map((item) => item.module_key));
    const selectedNodeType = selectedNode?.type || null;
    const selectedModuleKey = selectedNode?.module_key || null;
    const selectedModuleDef = selectedModuleKey
      ? allModules.find((moduleDef) => moduleDef.module_key === selectedModuleKey) || null
      : null;
    const recommended = [];

    prioritizedCategories.forEach((category) => {
      allModules.forEach((moduleDef) => {
        if (
          moduleDef.category !== category ||
          moduleDef.availability?.status === "unsupported" ||
          (recentModuleKeys.has(moduleDef.module_key) && moduleDef.category !== selectedNodeType) ||
          recommended.some((item) => item.module_key === moduleDef.module_key)
        ) {
          return;
        }
        recommended.push(moduleDef);
      });
    });

    const recommendedSlice = recommended.slice(0, 4);
    if (
      selectedModuleDef &&
      selectedModuleDef.availability?.status !== "unsupported" &&
      !recommendedSlice.some((item) => item.module_key === selectedModuleDef.module_key)
    ) {
      const insertIndex = workspaceLaneId ? Math.min(1, recommendedSlice.length) : 0;
      recommendedSlice.splice(insertIndex, 0, selectedModuleDef);
    }

    return recommendedSlice
      .filter(
        (moduleDef, index, list) =>
          list.findIndex((candidate) => candidate.module_key === moduleDef.module_key) === index
      )
      .slice(0, 4);
  }, [
    allModules,
    prioritizedCategories,
    recentModules,
    selectedNode?.module_key,
    selectedNode?.type,
    workspaceContext,
    workspaceLaneId
  ]);

  function moduleBlockReason(moduleDef) {
    if (capabilityStatus === "loading") {
      return t("\u524d\u7aef\u6b63\u5728\u540c\u6b65\u540e\u7aef\u80fd\u529b\u5feb\u7167\uff0c\u6682\u65f6\u65e0\u6cd5\u521b\u5efa\u6a21\u5757\u3002");
    }
    if (capabilitySource === "safe_fallback") {
      return (
        capabilityMessage ||
        t("\u5b89\u5168\u56de\u9000\u6a21\u5f0f\u4e0b\uff0c\u6a21\u5757\u521b\u5efa\u4f1a\u4fdd\u6301\u9501\u5b9a\uff0c\u76f4\u5230\u80fd\u529b\u6821\u9a8c\u6062\u590d\u3002")
      );
    }
    if (moduleDef.availability?.status === "unsupported") {
      return moduleDef.availability.reason || t("\u8be5\u6a21\u5757\u8d85\u51fa\u4e86\u5f53\u524d\u540e\u7aef\u80fd\u529b\u8fb9\u754c\u3002");
    }
    return "";
  }

  const hasVisibleModules = grouped.length > 0;
  const visibleModuleCount = grouped.reduce((count, group) => count + group.items.length, 0);

  function toggleGroup(category) {
    if (hasSearch) return;
    setExpandedGroups((current) => ({
      ...current,
      [category]: current[category] === false
    }));
  }

  function setAllGroups(expanded) {
    setExpandedGroups(
      Object.fromEntries(categoryOrder.map((category) => [category, expanded]))
    );
  }

  function renderModuleCard(moduleDef, variant = "default") {
    const blockReason = moduleBlockReason(moduleDef);
    const isBlocked = Boolean(blockReason) || capabilitySyncBlocked;
    const availabilityStatus = moduleDef.availability?.status || "supported";

    return (
      <button
        key={moduleDef.module_key}
        data-testid={`module-card-${moduleDef.module_key}`}
        className={`module-card module-card-${moduleDef.category}${
          variant === "compact" ? " module-card--compact" : ""
        }${isBlocked ? " module-card-disabled" : ""}`}
        disabled={isBlocked}
        title={blockReason}
        onClick={() => createNode(moduleDef.module_key)}
      >
        <div className="module-card-header">
          <div>
            <div className="module-card-title">{moduleDef.display_name}</div>
            <div className="module-card-key">{moduleDef.module_key}</div>
          </div>
          <span className={`status-pill ${moduleAvailabilityTone(availabilityStatus)}`}>
            {variant === "compact" && availabilityStatus !== "unsupported"
              ? "复用"
              : moduleAvailabilityLabel(availabilityStatus, t)}
          </span>
        </div>
        <div className="module-card-desc">{moduleDef.description}</div>
        {blockReason ? (
          <div className="module-card-note" data-testid={`module-card-note-${moduleDef.module_key}`}>
            {blockReason}
          </div>
        ) : null}
      </button>
    );
  }

  return (
    <aside className="module-sidebar">
      <div className="sidebar-header">
        <div className="panel-title strategy-card-title-note">
          <StrategyCardNote label={sidebarTitle} note={sidebarNote} />
        </div>
        <label htmlFor="module-sidebar-search" style={{position:"absolute",width:"1px",height:"1px",overflow:"hidden",clip:"rect(0,0,0,0)",whiteSpace:"nowrap"}}>{t("\u641c\u7d22\u6a21\u5757")}</label>
        <input
          id="module-sidebar-search"
          className="sidebar-search"
          data-testid="module-sidebar-search"
          placeholder={t("\u641c\u7d22\u6a21\u5757")}
          value={keyword}
          onChange={(event) => setKeyword(event.target.value)}
        />
        <div className="module-sidebar-toolbar">
          <div className="module-sidebar-toolbar__meta">
            {hasSearch
              ? t("\u641c\u7d22\u547d\u4e2d {count} \u4e2a\u6a21\u5757", { count: visibleModuleCount })
              : t("\u5171 {count} \u4e2a\u5206\u7ec4", { count: grouped.length })}
          </div>
          <div className="module-sidebar-toolbar__actions">
            <button
              type="button"
              className="ghost-btn compact-btn"
              onClick={() => setAllGroups(true)}
              disabled={hasSearch}
            >
              {t("\u5168\u90e8\u5c55\u5f00")}
            </button>
            <button
              type="button"
              className="ghost-btn compact-btn"
              onClick={() => setAllGroups(false)}
              disabled={hasSearch}
            >
              {t("\u5168\u90e8\u6298\u53e0")}
            </button>
          </div>
        </div>

      </div>

      <div className="sidebar-scroll">
        {!hasVisibleModules ? (
          <div className="empty-state module-sidebar-empty">
            {t("\u5f53\u524d\u7b5b\u9009\u6761\u4ef6\u4e0b\u6ca1\u6709\u53ef\u663e\u793a\u7684\u6a21\u5757\u3002")}
          </div>
        ) : null}

        {!hasSearch && workspaceContext && recommendedModules.length > 0 ? (
          <section
            className="module-sidebar-section"
            data-testid="module-sidebar-recommended-section"
          >
              <div className="module-group-title">{`${workspaceContext.laneLabel}推荐模块`}</div>
              <div className="module-sidebar-section__note">
              {laneRecommendation(
                workspaceLaneId,
                workspaceContext.laneLabel,
                selectedNode?.type || null
              )}
              </div>
              <div className="module-sidebar-quick-picks">
                {recommendedModules.map((moduleDef) => renderModuleCard(moduleDef, "compact"))}
            </div>
          </section>
        ) : null}

        {!hasSearch && recentModules.length > 0 ? (
          <section
            className="module-sidebar-section"
            data-testid="module-sidebar-recent-section"
          >
            <div className="module-group-title">最近使用</div>
            <div className="module-sidebar-quick-picks">
              {recentModules.map((moduleDef) => renderModuleCard(moduleDef, "compact"))}
            </div>
          </section>
        ) : null}

        {!hasSearch ? (
          <section
            className="module-sidebar-section"
            data-testid="module-sidebar-structure-section"
          >
            <div className="module-group-title">结构泳道</div>
            <div className="module-sidebar-lanes">
              {structureCounts.map((lane) => (
                <div
                  key={lane.category}
                  className={`module-sidebar-lane${
                    selectedNode?.type === lane.category ? " module-sidebar-lane--active" : ""
                  }`}
                >
                  <span>{localizedCategoryLabels[lane.category]}</span>
                  <strong>{lane.count}</strong>
                </div>
              ))}
            </div>
          </section>
        ) : null}

        {grouped.map((group) => (
          <section
            key={group.category}
            className={`module-group ${
              group.isExpanded ? "module-group--expanded" : "module-group--collapsed"
            }`}
          >
            <button
              type="button"
              className="module-group-toggle"
              onClick={() => toggleGroup(group.category)}
              disabled={hasSearch}
              aria-expanded={group.isExpanded}
            >
              <span className="module-group-toggle__label">
                {localizedCategoryLabels[group.category]}
                {selectedNode?.type === group.category ? (
                  <span className="module-group-toggle__badge module-group-toggle__badge--selection">
                    当前选中
                  </span>
                ) : null}
                {promotedCategories.has(group.category) && workspaceContext ? (
                  <span className="module-group-toggle__badge">推荐</span>
                ) : null}
              </span>
              <span className="module-group-toggle__meta">{group.items.length}</span>
              <span className="module-group-toggle__chevron">{group.isExpanded ? "-" : "+"}</span>
            </button>

            {group.isExpanded ? (
              <div className="module-group-body">
                {group.items.map((moduleDef) => renderModuleCard(moduleDef))}
              </div>
            ) : (
              <div className="module-group-collapsed-note">
                {t("\u5206\u7ec4\u5df2\u6298\u53e0\uff0c\u5c55\u5f00\u540e\u53ef\u67e5\u770b\u5168\u90e8\u6a21\u5757\u3002")}
              </div>
            )}
          </section>
        ))}
      </div>
    </aside>
  );
}

