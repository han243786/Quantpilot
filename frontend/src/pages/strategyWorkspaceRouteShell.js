export const WORKSPACE_TAB_DEFS = [
  { id: "dashboard", labelKey: "总览", kickerKey: "任务总览" },
  { id: "code", labelKey: "构建", kickerKey: "构建工作区" },
  { id: "research", labelKey: "研究回测", kickerKey: "研究回测工作区" },
  { id: "monitor", labelKey: "运行监控", kickerKey: "运行监控工作区" },
  { id: "source", labelKey: "源码", kickerKey: "源码工作区" }
];

export const CODE_INSPECTOR_DEFS = [
  { id: "params", label: "配置" },
  { id: "diagnostics", label: "检查" },
  { id: "code", label: "源码" }
];

export function buildWorkspaceTabs(capabilityView) {
  return WORKSPACE_TAB_DEFS.map((tab) => ({
    ...tab,
    capability: capabilityView.workspace.surfaces[tab.id]
  })).filter((tab) => tab.capability?.visible);
}

export function isWorkspaceSurfaceVisible(capabilityView, surfaceKey) {
  return capabilityView.workspace.surfaces[surfaceKey]?.visible === true;
}

export function shouldMountWorkspaceTab({
  capabilityView,
  activeTab,
  visitedTabs,
  surfaceKey
}) {
  return (
    isWorkspaceSurfaceVisible(capabilityView, surfaceKey) &&
    (activeTab === surfaceKey || visitedTabs.has(surfaceKey))
  );
}

export function buildWorkspaceTabPanelProps(activeTab, surfaceKey) {
  return {
    className: "workspace-tab-panel",
    style: { display: activeTab === surfaceKey ? "block" : "none" },
    "aria-hidden": activeTab !== surfaceKey
  };
}
