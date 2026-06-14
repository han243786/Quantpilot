import { describe, expect, it } from "vitest";

import {
  buildWorkspaceTabPanelProps,
  buildWorkspaceTabs,
  isWorkspaceSurfaceVisible,
  shouldMountWorkspaceTab
} from "./strategyWorkspaceRouteShell";

function createCapabilityView() {
  return {
    workspace: {
      surfaces: {
        dashboard: { visible: true, enabled: true },
        code: { visible: true, enabled: true },
        research: { visible: false, enabled: false },
        monitor: { visible: true, enabled: false },
        source: { visible: true, enabled: true }
      }
    }
  };
}

describe("strategyWorkspaceRouteShell", () => {
  it("builds route tabs from visible capability surfaces only", () => {
    const tabs = buildWorkspaceTabs(createCapabilityView());

    expect(tabs.map((tab) => tab.id)).toEqual(["dashboard", "code", "monitor", "source"]);
    expect(tabs.find((tab) => tab.id === "monitor")?.capability.enabled).toBe(false);
  });

  it("resolves tab mount state from capability visibility and visited tabs", () => {
    const capabilityView = createCapabilityView();

    expect(isWorkspaceSurfaceVisible(capabilityView, "code")).toBe(true);
    expect(isWorkspaceSurfaceVisible(capabilityView, "research")).toBe(false);
    expect(
      shouldMountWorkspaceTab({
        capabilityView,
        activeTab: "dashboard",
        visitedTabs: new Set(["dashboard", "code"]),
        surfaceKey: "code"
      })
    ).toBe(true);
    expect(
      shouldMountWorkspaceTab({
        capabilityView,
        activeTab: "dashboard",
        visitedTabs: new Set(["dashboard"]),
        surfaceKey: "code"
      })
    ).toBe(false);
  });

  it("builds stable tab panel props without layout branching in the page", () => {
    expect(buildWorkspaceTabPanelProps("code", "code")).toEqual({
      className: "workspace-tab-panel",
      style: { display: "block" },
      "aria-hidden": false
    });
    expect(buildWorkspaceTabPanelProps("dashboard", "code")).toEqual({
      className: "workspace-tab-panel",
      style: { display: "none" },
      "aria-hidden": true
    });
  });
});
