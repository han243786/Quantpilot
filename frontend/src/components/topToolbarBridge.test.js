import { describe, expect, it, vi } from "vitest";

import {
  buildStrategyPackage,
  buildStrategyPackageFilename,
  buildToolbarLayoutProps,
  resolveToolbarVariant
} from "./topToolbarBridge";

describe("topToolbarBridge", () => {
  it("resolves toolbar variants into stable layout metadata", () => {
    expect(resolveToolbarVariant("workspace")).toEqual({
      isWorkspace: true,
      headerClassName: "top-toolbar top-toolbar--workspace",
      layoutKind: "workspace"
    });
    expect(resolveToolbarVariant("default")).toEqual({
      isWorkspace: false,
      headerClassName: "top-toolbar",
      layoutKind: "default"
    });
    expect(resolveToolbarVariant(undefined)).toEqual({
      isWorkspace: false,
      headerClassName: "top-toolbar",
      layoutKind: "default"
    });
  });

  it("builds strategy package payloads and safe filenames", () => {
    const graph = {
      metadata: {
        graph_id: "graph_1",
        name: "Mean Reversion / BTC"
      },
      nodes: [{ id: "node_1" }]
    };

    expect(buildStrategyPackage(graph, "2026-06-02T00:00:00.000Z")).toEqual({
      schema_version: "quantpilot/strategy-package/v1",
      exported_at: "2026-06-02T00:00:00.000Z",
      graph_id: "graph_1",
      name: "Mean Reversion / BTC",
      graph
    });
    expect(buildStrategyPackageFilename({ name: "Mean Reversion / BTC" })).toBe(
      "Mean_Reversion_BTC_strategy_package.json"
    );
    expect(buildStrategyPackageFilename({ graph_id: "draft_graph" })).toBe(
      "draft_graph_strategy_package.json"
    );
  });

  it("bridges model state and guarded handlers into layout props", () => {
    const handleSaveGraph = vi.fn();
    const handleExportQuantScript = vi.fn();
    const handleExportRuntimeConfig = vi.fn();
    const handleExportStrategyPackage = vi.fn();
    const handleImportStrategyPackageClick = vi.fn();
    const onOpenCredentials = vi.fn();
    const model = {
      graph: { metadata: { graph_id: "graph_1" } },
      canCompile: true,
      notice: null
    };

    expect(
      buildToolbarLayoutProps({
        model,
        saving: true,
        isCompiling: false,
        onOpenCredentials,
        handleSaveGraph,
        handleExportQuantScript,
        handleExportRuntimeConfig,
        handleExportStrategyPackage,
        handleImportStrategyPackageClick,
        exportStrategyPackageTitle: "export title",
        importStrategyPackageTitle: "import title"
      })
    ).toEqual({
      ...model,
      saving: true,
      isCompiling: false,
      onOpenCredentials,
      handleSaveGraph,
      handleExportQuantScript,
      handleExportRuntimeConfig,
      handleExportStrategyPackage,
      handleImportStrategyPackageClick,
      exportStrategyPackageTitle: "export title",
      importStrategyPackageTitle: "import title"
    });
  });
});
