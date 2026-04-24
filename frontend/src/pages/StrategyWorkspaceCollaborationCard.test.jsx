import { afterEach, beforeEach, describe, expect, it } from "vitest";
import { act, render, screen, waitFor } from "@testing-library/react";
import StrategyWorkspaceCollaborationCard from "./StrategyWorkspaceCollaborationCard";
import { useGraphStore } from "../store/graphStore";

describe("StrategyWorkspaceCollaborationCard", () => {
  const initialState = useGraphStore.getState();

  beforeEach(() => {
    act(() => {
      useGraphStore.setState(initialState, true);
    });
  });

  afterEach(() => {
    act(() => {
      useGraphStore.setState(initialState, true);
    });
  });

  it("renders owner, editors, last run actor, and audit history", async () => {
    const refreshGraphAuditHistory = async () => {
      useGraphStore.setState({
        graphAuditHistory: [
          {
            audit_id: "audit_graph_test_1",
            graph_id: "graph_test",
            action: "graph_saved",
            created_at_ms: 1_700_000_000_000,
            actor: {
              actor_id: "owner_alice",
              display_name: "Alice"
            },
            target_id: "1700000000001",
            summary: "Saved graph version 1700000000001"
          }
        ],
        graphAuditHistoryStatus: "ready",
        graphAuditHistoryMessage: ""
      });
      return useGraphStore.getState().graphAuditHistory;
    };

    act(() => {
      useGraphStore.setState({
        graphAuditHistory: [],
        graphAuditHistoryStatus: "idle",
        graphAuditHistoryMessage: "",
        refreshGraphAuditHistory
      });
    });

    render(
      <StrategyWorkspaceCollaborationCard
        graphId="graph_test"
        collaboration={{
          owner: { actor_id: "owner_alice", display_name: "Alice" },
          editors: [{ actor_id: "editor_eve", display_name: "Eve" }],
          last_saved_by: { actor_id: "owner_alice", display_name: "Alice" },
          last_run_actor: { actor_id: "editor_eve", display_name: "Eve" }
        }}
        lastRun={{
          actor: { actor_id: "editor_eve", display_name: "Eve" }
        }}
        lastBacktest={null}
      />
    );

    await waitFor(() => {
      expect(screen.getByTestId("workspace-audit-entry-audit_graph_test_1")).toBeInTheDocument();
    });

    expect(screen.getByTestId("workspace-owner-row").textContent).toContain("Alice");
    expect(screen.getByTestId("workspace-editors-row").textContent).toContain("Eve");
    expect(screen.getByTestId("workspace-last-run-row").textContent).toContain("Eve");
    expect(screen.getByTestId("workspace-audit-list").textContent).toContain("Saved graph version");
  });
});
