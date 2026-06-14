export function resolveToolbarVariant(variant) {
  const isWorkspace = variant === "workspace";
  return {
    isWorkspace,
    headerClassName: `top-toolbar${isWorkspace ? " top-toolbar--workspace" : ""}`,
    layoutKind: isWorkspace ? "workspace" : "default"
  };
}

export function buildStrategyPackage(graph, exportedAt = new Date().toISOString()) {
  return {
    schema_version: "quantpilot/strategy-package/v1",
    exported_at: exportedAt,
    graph_id: graph.metadata?.graph_id || "draft_graph",
    name: graph.metadata?.name || "Untitled Strategy",
    graph
  };
}

export function buildStrategyPackageFilename(payload) {
  const safeName = String(payload.name || payload.graph_id || "strategy")
    .replace(/[^\w.-]+/g, "_")
    .slice(0, 80);
  return `${safeName || "strategy"}_strategy_package.json`;
}

export function buildToolbarLayoutProps({
  model,
  saving,
  isCompiling,
  onOpenCredentials,
  handleSaveGraph,
  handleExportQuantScript,
  handleExportRuntimeConfig,
  handleExportStrategyPackage,
  handleImportStrategyPackageClick,
  exportStrategyPackageTitle,
  importStrategyPackageTitle
}) {
  return {
    ...model,
    saving,
    isCompiling,
    onOpenCredentials,
    handleSaveGraph,
    handleExportQuantScript,
    handleExportRuntimeConfig,
    handleExportStrategyPackage,
    handleImportStrategyPackageClick,
    exportStrategyPackageTitle,
    importStrategyPackageTitle
  };
}
