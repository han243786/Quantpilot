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
          has_changes: true
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
        "1700000000002"
      )
    );
    expect(screen.getByTestId("workspace-version-compare-card")).toBeInTheDocument();
    expect(screen.getByTestId("workspace-version-metadata-row-version_label").textContent).toContain(
      "baseline"
    );
    expect(screen.getByTestId("workspace-version-config-row-data_data_1-0").textContent).toContain(
      "window_size"
    );
  });
});
