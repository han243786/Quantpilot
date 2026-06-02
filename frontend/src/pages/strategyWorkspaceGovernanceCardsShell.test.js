import { describe, expect, it } from "vitest";

import {
  buildWorkspaceCollaborationRows,
  buildWorkspaceExperimentStartPayload,
  buildWorkspaceVersionDraftSummary,
  buildWorkspaceVersionEvidenceOptions,
  formatWorkspaceActor,
  formatWorkspaceAuditActorLine,
  formatWorkspaceExperimentPercent,
  formatWorkspaceGovernanceTime,
  formatWorkspaceVersionCountChanges,
  formatWorkspaceVersionList,
  parseWorkspaceExperimentNumberList,
  selectWorkspaceActiveExperiment,
  selectWorkspaceGraphExperiments,
  selectWorkspaceVersionCompareEntries,
  shouldRefreshWorkspaceAuditHistory,
  toggleWorkspaceVersionCompareSelection,
  workspaceConfigChangeLabels,
  workspaceConfigDomainLabel
} from "./strategyWorkspaceGovernanceCardsShell";

describe("strategyWorkspaceGovernanceCardsShell", () => {
  it("projects version history helpers and compare selection state", () => {
    expect(formatWorkspaceGovernanceTime(null)).toBe("-");
    expect(formatWorkspaceVersionList(["a", "b"])).toBe("a, b");
    expect(formatWorkspaceVersionList([])).toBe("-");
    expect(workspaceConfigDomainLabel("risk")).toBe("Risk Plane");
    expect(workspaceConfigDomainLabel("custom")).toBe("custom");
    expect(
      workspaceConfigChangeLabels({
        lifecycle_changed: true,
        readiness_changed: true,
        source_refs_changed: true,
        findings_changed: false
      })
    ).toBe("生命周期 / 就绪状态 / 来源证据");
    expect(workspaceConfigChangeLabels({})).toBe("-");
    expect(formatWorkspaceVersionCountChanges([{ key: "allow", left_count: 1, right_count: 0 }])).toBe(
      "allow: 1->0"
    );

    const graph = {
      metadata: { graph_id: "graph_a", updated_at: 123 },
      nodes: [{ id: "n1" }],
      edges: [{ id: "e1" }, { id: "e2" }]
    };
    expect(buildWorkspaceVersionDraftSummary(graph)).toEqual({
      graphId: "graph_a",
      updatedAt: 123,
      nodeCount: 1,
      edgeCount: 2
    });
    expect(
      selectWorkspaceVersionCompareEntries(["v1", "missing", "v2"], [
        { version_id: "v1" },
        { version_id: "v2" }
      ])
    ).toEqual([{ version_id: "v1" }, { version_id: "v2" }]);
    expect(toggleWorkspaceVersionCompareSelection(["v1"], "v2")).toEqual(["v1", "v2"]);
    expect(toggleWorkspaceVersionCompareSelection(["v1", "v2"], "v3")).toEqual(["v2", "v3"]);
    expect(toggleWorkspaceVersionCompareSelection(["v1", "v2"], "v1")).toEqual(["v2"]);
  });

  it("builds graph-scoped evidence options and experiment payloads", () => {
    expect(
      buildWorkspaceVersionEvidenceOptions(
        [
          { backtest_id: "bt_a", graph_id: "graph_a", created_at_ms: 123 },
          { backtest_id: "bt_b", graph_id: "graph_b" },
          { backtest_id: "bt_any" },
          { graph_id: "graph_a" }
        ],
        "graph_a"
      ).map((entry) => entry.id)
    ).toEqual(["bt_a", "bt_any"]);

    expect(parseWorkspaceExperimentNumberList("5, nope, 10")).toEqual([5, 10]);
    expect(parseWorkspaceExperimentNumberList("0, 250", (value) => Number.parseInt(value, 10))).toEqual([
      0,
      250
    ]);
    expect(formatWorkspaceExperimentPercent(0.12)).toBe("+12.00%");
    expect(formatWorkspaceExperimentPercent(-0.03)).toBe("-3.00%");
    expect(
      selectWorkspaceGraphExperiments(
        [
          { experiment_id: "exp_a", graph_id: "graph_a" },
          { experiment_id: "exp_b", graph_id: "graph_b" }
        ],
        "graph_a"
      )
    ).toEqual([{ experiment_id: "exp_a", graph_id: "graph_a" }]);
    expect(selectWorkspaceActiveExperiment({ experiment_id: "exp_a", graph_id: "graph_a" }, "graph_a")).toEqual({
      experiment_id: "exp_a",
      graph_id: "graph_a"
    });
    expect(selectWorkspaceActiveExperiment({ experiment_id: "exp_b", graph_id: "graph_b" }, "graph_a")).toBeNull();
    expect(
      buildWorkspaceExperimentStartPayload({
        experimentName: "Sweep",
        feeGridDraft: "5, 15",
        slippageGridDraft: "5",
        latencyGridDraft: "0, 250"
      })
    ).toEqual({
      experimentName: "Sweep",
      feeBps: [5, 15],
      slippageBps: [5],
      latencyMs: [0, 250]
    });
  });

  it("projects collaboration rows and audit actor lines", () => {
    const collaboration = {
      owner: { actor_id: "owner_alice", display_name: "Alice" },
      editors: [{ actor_id: "editor_eve", display_name: "Eve" }],
      last_saved_by: { actor_id: "owner_alice", display_name: "Alice" },
      last_run_actor: { actor_id: "runner_ray" }
    };

    expect(formatWorkspaceActor({ actor_id: "owner_alice", display_name: "Alice" })).toBe("Alice");
    expect(formatWorkspaceActor(null)).toBe("未分配");
    expect(
      buildWorkspaceCollaborationRows({
        collaboration,
        lastRun: null,
        lastBacktest: { actor: { actor_id: "backtest_bob", display_name: "Bob" } }
      })
    ).toMatchObject([
      { testId: "workspace-owner-row", label: "所有者", value: "Alice" },
      { testId: "workspace-editors-row", label: "协作者", value: "Eve" },
      { testId: "workspace-last-saved-row", label: "最近保存人", value: "Alice" },
      { testId: "workspace-last-run-row", label: "最近执行人", value: "Bob" }
    ]);
    expect(shouldRefreshWorkspaceAuditHistory("graph_a")).toBe(true);
    expect(shouldRefreshWorkspaceAuditHistory("draft_graph")).toBe(false);
    expect(shouldRefreshWorkspaceAuditHistory("")).toBe(false);
    expect(
      formatWorkspaceAuditActorLine({
        actor: { actor_id: "owner_alice", display_name: "Alice" },
        target_id: "v1"
      })
    ).toBe("Alice / v1");
  });
});
