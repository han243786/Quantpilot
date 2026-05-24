import { describe, expect, it } from "vitest";
import {
  projectCapabilityView,
  projectUiActions,
  projectWorkspaceSurfaces
} from "./capabilityProjection";
import { backendCapabilitiesFixture } from "../test/fixtures/capabilities/capabilityFallbacks";

function cloneFixture(patch = {}) {
  return JSON.parse(JSON.stringify({ ...backendCapabilitiesFixture, ...patch }));
}

describe("capability projection", () => {
  it("projects workspace surfaces from backend capability entries", () => {
    const surfaces = projectWorkspaceSurfaces(backendCapabilitiesFixture);

    expect(surfaces.dashboard).toMatchObject({
      key: "dashboard",
      label: "总览",
      status: "supported",
      visible: true,
      enabled: true,
      source: "backend:/api/capabilities.workspace.surfaces"
    });
  });

  it("hides workspace entries that are not declared by backend capabilities", () => {
    const capabilities = cloneFixture({
      workspace: {
        surfaces: backendCapabilitiesFixture.workspace.surfaces.filter(
          (entry) => entry.key !== "monitor"
        )
      }
    });
    const surfaces = projectWorkspaceSurfaces(capabilities);

    expect(surfaces.monitor.visible).toBe(false);
    expect(surfaces.monitor.enabled).toBe(false);
    expect(surfaces.monitor.reason).toContain("未声明");
  });

  it("keeps declared-only workspace entries visible but disabled", () => {
    const capabilities = cloneFixture({
      workspace: {
        surfaces: backendCapabilitiesFixture.workspace.surfaces.map((entry) =>
          entry.key === "research"
            ? { ...entry, status: "declared_only", reason: "等待回测服务恢复。" }
            : entry
        )
      }
    });
    const surfaces = projectWorkspaceSurfaces(capabilities);

    expect(surfaces.research.visible).toBe(true);
    expect(surfaces.research.enabled).toBe(false);
    expect(surfaces.research.reason).toBe("等待回测服务恢复。");
  });

  it("disables UI actions that backend capabilities do not declare", () => {
    const capabilities = cloneFixture({
      ui_actions: {
        actions: backendCapabilitiesFixture.ui_actions.actions.filter(
          (entry) => entry.key !== "compile"
        )
      }
    });
    const actions = projectUiActions({ capabilities });

    expect(actions.compile.visible).toBe(false);
    expect(actions.compile.enabled).toBe(false);
    expect(actions.compile.blockReason).toContain("未声明");
  });

  it("keeps cache-mode supported actions enabled while preserving a warning reason", () => {
    const actions = projectUiActions({
      capabilities: backendCapabilitiesFixture,
      capabilityStatus: "degraded",
      capabilitySource: "cache",
      capabilityMessage: "使用缓存"
    });

    expect(actions.compile.enabled).toBe(true);
    expect(actions.compile.blockReason).toContain("缓存能力快照");
  });

  it("builds one projection object for workspace and toolbar consumers", () => {
    const projection = projectCapabilityView({
      capabilities: backendCapabilitiesFixture,
      capabilityStatus: "ready",
      capabilitySource: "remote",
      capabilityMessage: ""
    });

    expect(projection.workspace.surfaces.code.enabled).toBe(true);
    expect(projection.uiActions.actions.export_runtime_config.enabled).toBe(true);
  });
});
