import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { act, fireEvent, render, screen, waitFor } from "@testing-library/react";
import StrategyWorkspaceVersionHistoryCard from "./StrategyWorkspaceVersionHistoryCard";
import { useGraphStore } from "../store/graphStore";
import { buildValidatedSampleGraph } from "../test/fixtures/runtime/buildValidatedSampleGraph";

function buildGraph(registry, graphId, name, updatedAt = 1_700_000_000_000) {
  return buildValidatedSampleGraph(registry, (graph) => {
    graph.metadata.graph_id = graphId;
    graph.metadata.name = name;
    graph.metadata.updated_at = updatedAt;
  });
}

describe("StrategyWorkspaceVersionHistoryCard", () => {
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

  it("captures version label / note on save and renders compare diff output", async () => {
    const saveGraph = vi.fn().mockResolvedValue(undefined);
    const compareGraphVersions = vi.fn().mockResolvedValue(undefined);
    const clearGraphVersionCompare = vi.fn();
    const loadGraphVersionPreview = vi.fn();
    const restoreGraphVersion = vi.fn();
    const clearGraphVersionPreview = vi.fn();
    const registry = initialState.registry;
    const currentGraph = buildGraph(registry, "alpha_strategy", "Working draft");

    act(() => {
      useGraphStore.setState({
        graphVersions: [
          {
            graph_id: "alpha_strategy",
            version_id: "1700000000001",
            name: "Persisted V1",
            updated_at: 1_700_000_000_001,
            version_label: "baseline",
            save_note: "Initial persisted strategy snapshot.",
            node_count: 7,
            edge_count: 6,
            is_latest: false
          },
          {
            graph_id: "alpha_strategy",
            version_id: "1700000000002",
            name: "Persisted V2",
            updated_at: 1_700_000_000_002,
            version_label: "tuned",
            save_note: "Raised window size and removed one edge.",
            node_count: 7,
            edge_count: 5,
            is_latest: true
          }
        ],
        graphVersionsStatus: "ready",
        graphVersionPreview: null,
        graphVersionPreviewStatus: "idle",
        graphVersionPreviewMessage: "",
        graphVersionCompare: {
          graph_id: "alpha_strategy",
          left: {
            graph_id: "alpha_strategy",
            version_id: "1700000000001",
            name: "Persisted V1",
            updated_at: 1_700_000_000_001,
            version_label: "baseline",
            save_note: "Initial persisted strategy snapshot.",
            node_count: 7,
            edge_count: 6,
            is_latest: false
          },
          right: {
            graph_id: "alpha_strategy",
            version_id: "1700000000002",
            name: "Persisted V2",
            updated_at: 1_700_000_000_002,
            version_label: "tuned",
            save_note: "Raised window size and removed one edge.",
            node_count: 7,
            edge_count: 5,
            is_latest: true
          },
          metadata_rows: [
            {
              key: "version_label",
              label: "Version label",
              status: "different",
              left_value: "baseline",
              right_value: "tuned"
            }
          ],
          node_diff: {
            left_count: 7,
            right_count: 7,
            added_ids: [],
            removed_ids: [],
            changed_ids: ["data_data_1"]
          },
          edge_diff: {
            left_count: 6,
            right_count: 5,
            added_ids: [],
            removed_ids: ["edge_data_data_1_intent_intent_2_market_data_out_data_input"],
            changed_ids: []
          },
          config_diffs: [
            {
              node_id: "data_data_1",
              node_name: "Persisted data source",
              field_path: "window_size",
              status: "different",
              left_value: "20",
              right_value: "55"
            }
          ],
          strategy_config_diff: {
            schema_version: "quantpilot/v4-strategy-config-diff/v1",
            left_artifact_id: "strategy_config_left",
            right_artifact_id: "strategy_config_right",
            source_digest_changes: [{ field: "graph_digest" }],
            domain_changes: [
              {
                domain_id: "risk",
                lifecycle_changed: false,
                readiness_changed: true,
                source_refs_changed: true,
                findings_changed: true
              }
            ],
            runtime_boundary_changed: false,
            changed: true
          },
          strategy_config_evidence_diff: {
            schema_version: "quantpilot/v4-strategy-config-evidence-diff/v1",
            left_backtest_id: "bt_left",
            right_backtest_id: "bt_right",
            status: "different",
            changed: true,
            diagnostics: [],
            machine_trajectory: {
              status: "different",
              left_point_count: 2,
              right_point_count: 3,
              left_visited_states: ["machine:observe"],
              right_visited_states: ["machine:observe", "machine:trade"],
              transition_hit_changes: [{ key: "machine:observe->trade:*", left_count: 0, right_count: 1 }],
              left_terminal_state: "machine:observe:*",
              right_terminal_state: "machine:trade:*",
              first_divergence: { index: 1, left: "machine:observe:ok:*", right: "machine:trade:ok:*" }
            },
            risk_plane: {
              status: "different",
              left_decision_count: 1,
              right_decision_count: 1,
              left_approved_count: 1,
              right_approved_count: 0,
              left_rejected_count: 0,
              right_rejected_count: 1,
              action_count_changes: [
                { key: "allow", left_count: 1, right_count: 0 },
                { key: "reject", left_count: 0, right_count: 1 }
              ],
              reason_count_changes: [{ key: "risk_limit", left_count: 0, right_count: 1 }],
              first_divergence: { index: 0, left: "risk:allow", right: "risk:reject" }
            },
            execution_capability: {
              status: "same",
              left_source_count: 1,
              right_source_count: 1,
              left_accepted_count: 1,
              right_accepted_count: 1,
              left_rejected_count: 0,
              right_rejected_count: 0,
              runtime_mode_changes: [],
              capability_kind_changes: [],
              capability_source_changes: [],
              status_changes: [],
              first_divergence: null
            },
            metrics: {
              status: "different",
              fields: [
                { key: "total_return_ratio", status: "different", left_value: "0.01000000", right_value: "0.02000000" }
              ]
            }
          },
          has_changes: true
        },
        runtime: {
          ...initialState.runtime,
          backtestHistory: [
            { backtest_id: "bt_left", graph_id: "alpha_strategy", created_at_ms: 1_700_000_000_101 },
            { backtest_id: "bt_right", graph_id: "alpha_strategy", created_at_ms: 1_700_000_000_202 }
          ],
          backtestHistoryStatus: "ready"
        },
        graphVersionCompareStatus: "ready",
        graphVersionCompareMessage: "",
        saveGraph,
        compareGraphVersions,
        clearGraphVersionCompare,
        loadGraphVersionPreview,
        restoreGraphVersion,
        clearGraphVersionPreview
      });
    });

    await act(async () => {
      render(
        <StrategyWorkspaceVersionHistoryCard graphId="alpha_strategy" currentGraph={currentGraph} />
      );
      await Promise.resolve();
    });

    await act(async () => {
      fireEvent.change(screen.getByTestId("workspace-version-label-input"), {
        target: { value: "release_candidate" }
      });
      fireEvent.change(screen.getByTestId("workspace-version-note-input"), {
        target: { value: "Captured before compare/diff rollout." }
      });
      fireEvent.click(screen.getByTestId("workspace-version-save-action"));
    });

    expect(saveGraph).toHaveBeenCalledWith({
      versionLabel: "release_candidate",
      saveNote: "Captured before compare/diff rollout."
    });

    await act(async () => {
      fireEvent.click(screen.getByTestId("workspace-version-compare-toggle-1700000000001"));
      fireEvent.click(screen.getByTestId("workspace-version-compare-toggle-1700000000002"));
    });
    fireEvent.change(screen.getByTestId("workspace-version-left-evidence-select"), {
      target: { value: "bt_left" }
    });
    fireEvent.change(screen.getByTestId("workspace-version-right-evidence-select"), {
      target: { value: "bt_right" }
    });

    await waitFor(() =>
      expect(screen.getByTestId("workspace-version-compare-selection").textContent).toContain(
        "1700000000001, 1700000000002"
      )
    );

    await act(async () => {
      fireEvent.click(screen.getByTestId("workspace-version-open-compare"));
    });

    await waitFor(() =>
      expect(compareGraphVersions).toHaveBeenCalledWith(
        "alpha_strategy",
        "1700000000001",
        "1700000000002",
        {
          leftBacktestId: "bt_left",
          rightBacktestId: "bt_right"
        }
      )
    );
    expect(screen.getByTestId("workspace-version-compare-card")).toBeInTheDocument();
    expect(screen.getByTestId("workspace-version-metadata-row-version_label").textContent).toContain(
      "baseline"
    );
    expect(screen.getByTestId("workspace-version-config-row-data_data_1-0").textContent).toContain(
      "window_size"
    );
    expect(screen.getByTestId("workspace-version-strategy-config-diff")).toBeInTheDocument();
    expect(screen.getByTestId("workspace-version-strategy-config-domain-risk").textContent).toContain(
      "Risk Plane"
    );
    expect(screen.getByTestId("workspace-version-strategy-config-evidence-diff")).toBeInTheDocument();
    expect(screen.getByTestId("workspace-version-evidence-risk-plane").textContent).toContain(
      "reject: 0->1"
    );
  });
});
