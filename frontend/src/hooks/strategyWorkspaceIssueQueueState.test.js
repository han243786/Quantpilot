import { afterEach, describe, expect, it, vi } from "vitest";

import {
  WORKSPACE_ISSUE_FILTERS_STORAGE_KEY,
  buildWorkspaceIssueQueueFilterModel,
  normalizeWorkspaceIssueFilters,
  persistWorkspaceIssueFilters,
  readStoredWorkspaceIssueFilters,
  workspaceIssueFiltersStorageScope,
  workspaceIssueFiltersSummary
} from "./strategyWorkspaceIssueQueueState";

describe("strategyWorkspaceIssueQueueState", () => {
  afterEach(() => {
    vi.restoreAllMocks();
    window.localStorage.clear();
  });

  it("resolves storage scope and persists scoped filters with defaults", () => {
    expect(workspaceIssueFiltersStorageScope("strategy_a", "graph_a")).toBe("strategy_a");
    expect(workspaceIssueFiltersStorageScope("", "graph_a")).toBe("graph_a");
    expect(workspaceIssueFiltersStorageScope("", "")).toBe("draft_graph");

    persistWorkspaceIssueFilters("strategy_a", {
      severityFilter: "warning",
      actionableOnly: true
    });

    expect(readStoredWorkspaceIssueFilters("strategy_a")).toMatchObject({
      severityFilter: "warning",
      actionableOnly: true,
      sourceFilter: "all",
      nodeTypeFilter: "all"
    });
    expect(readStoredWorkspaceIssueFilters("missing")).toMatchObject({
      severityFilter: "all",
      actionableOnly: false
    });
    expect(JSON.parse(window.localStorage.getItem(WORKSPACE_ISSUE_FILTERS_STORAGE_KEY))).toHaveProperty(
      "strategy_a"
    );
  });

  it("falls back to defaults for malformed storage payloads", () => {
    vi.spyOn(console, "warn").mockImplementation(() => {});
    window.localStorage.setItem(WORKSPACE_ISSUE_FILTERS_STORAGE_KEY, "{bad json");

    expect(readStoredWorkspaceIssueFilters("strategy_a")).toMatchObject({
      severityFilter: "all",
      sourceFilter: "all"
    });
  });

  it("normalizes stale source and node-type filters against current queue items", () => {
    const items = [
      {
        id: "runtime_agent",
        severity: "error",
        source: "runtime",
        nodeType: "agent",
        actionable: true
      },
      {
        id: "validation_intent",
        severity: "warning",
        source: "validation",
        nodeType: "intent",
        actionable: false
      }
    ];

    expect(
      normalizeWorkspaceIssueFilters(
        {
          severityFilter: "error",
          sourceFilter: "runtime",
          nodeTypeFilter: "intent"
        },
        items
      )
    ).toMatchObject({
      severityFilter: "error",
      sourceFilter: "runtime",
      nodeTypeFilter: "all"
    });

    expect(
      normalizeWorkspaceIssueFilters(
        {
          sourceFilter: "missing",
          nodeTypeFilter: "agent"
        },
        items
      )
    ).toMatchObject({
      sourceFilter: "all",
      nodeTypeFilter: "all"
    });
  });

  it("summarizes active filters without requiring React state", () => {
    expect(workspaceIssueFiltersSummary({})).toBe("\u65e0\u6d3b\u52a8\u7b5b\u9009");
    expect(
      workspaceIssueFiltersSummary({
        severityFilter: "warning",
        actionableOnly: true,
        sourceFilter: "runtime",
        nodeTypeFilter: "agent"
      })
    ).toBe("\u8b66\u544a / \u4ec5\u53ef\u64cd\u4f5c\u9879 / runtime / agent");
  });

  it("builds the issue queue card filter model from one pure boundary", () => {
    const items = [
      {
        id: "runtime_agent",
        severity: "error",
        source: "runtime",
        nodeType: "agent",
        actionable: true
      },
      {
        id: "runtime_intent",
        severity: "warning",
        source: "runtime",
        nodeType: "intent",
        actionable: true
      },
      {
        id: "validation_graph",
        severity: "warning",
        source: "validation",
        nodeType: null,
        actionable: false
      }
    ];

    const model = buildWorkspaceIssueQueueFilterModel(items, {
      severityFilter: "warning",
      actionableOnly: true,
      showSourceFilters: true,
      sourceFilter: "runtime",
      nodeTypeFilter: "intent"
    });

    expect(model.isDirty).toBe(true);
    expect(model.counts).toMatchObject({ error: 1, warning: 2, actionable: 2 });
    expect(model.orderedSources).toEqual(["runtime", "validation"]);
    expect(model.sourceCounts).toMatchObject({ runtime: 2, validation: 1 });
    expect(model.nodeTypeCounts).toMatchObject({ intent: 1 });
    expect(model.orderedNodeTypes).toEqual(["intent"]);
    expect(model.filteredItems.map((item) => item.id)).toEqual(["runtime_intent"]);
  });
});
