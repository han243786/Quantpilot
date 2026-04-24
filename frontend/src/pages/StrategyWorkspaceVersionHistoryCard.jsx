import { useEffect, useMemo, useState } from "react";
import { useGraphStore } from "../store/graphStore";

function formatTime(value) {
  return value ? new Date(value).toLocaleString() : "-";
}

function formatList(items = []) {
  return items.length ? items.join(", ") : "-";
}

function DiffList({ title, diff, testId }) {
  return (
    <div className="workspace-version-compare-group" data-testid={testId}>
      <div className="mini-list-title">{title}</div>
      <div className="strategy-inspector-metrics">
        <div className="kv-line">
          <span>Left / right</span>
          <strong>
            {diff.left_count} / {diff.right_count}
          </strong>
        </div>
        <div className="kv-line">
          <span>Added</span>
          <strong>{formatList(diff.added_ids)}</strong>
        </div>
        <div className="kv-line">
          <span>Removed</span>
          <strong>{formatList(diff.removed_ids)}</strong>
        </div>
        <div className="kv-line">
          <span>Changed</span>
          <strong>{formatList(diff.changed_ids)}</strong>
        </div>
      </div>
    </div>
  );
}

export default function StrategyWorkspaceVersionHistoryCard({ graphId, currentGraph }) {
  const graphVersions = useGraphStore((state) => state.graphVersions);
  const graphVersionsStatus = useGraphStore((state) => state.graphVersionsStatus);
  const graphVersionsMessage = useGraphStore((state) => state.graphVersionsMessage);
  const graphVersionPreview = useGraphStore((state) => state.graphVersionPreview);
  const graphVersionPreviewStatus = useGraphStore((state) => state.graphVersionPreviewStatus);
  const graphVersionPreviewMessage = useGraphStore((state) => state.graphVersionPreviewMessage);
  const graphVersionCompare = useGraphStore((state) => state.graphVersionCompare);
  const graphVersionCompareStatus = useGraphStore((state) => state.graphVersionCompareStatus);
  const graphVersionCompareMessage = useGraphStore((state) => state.graphVersionCompareMessage);
  const loadGraphVersionPreview = useGraphStore((state) => state.loadGraphVersionPreview);
  const restoreGraphVersion = useGraphStore((state) => state.restoreGraphVersion);
  const clearGraphVersionPreview = useGraphStore((state) => state.clearGraphVersionPreview);
  const compareGraphVersions = useGraphStore((state) => state.compareGraphVersions);
  const clearGraphVersionCompare = useGraphStore((state) => state.clearGraphVersionCompare);
  const saveGraph = useGraphStore((state) => state.saveGraph);

  const [versionLabelDraft, setVersionLabelDraft] = useState("");
  const [saveNoteDraft, setSaveNoteDraft] = useState("");
  const [compareSelection, setCompareSelection] = useState([]);

  useEffect(() => {
    setVersionLabelDraft(currentGraph?.metadata?.version_label || "");
    setSaveNoteDraft(currentGraph?.metadata?.save_note || "");
  }, [currentGraph?.metadata?.graph_id, currentGraph?.metadata?.version_label, currentGraph?.metadata?.save_note]);

  useEffect(() => {
    setCompareSelection((current) =>
      current.filter((versionId) => graphVersions.some((entry) => entry.version_id === versionId))
    );
  }, [graphVersions]);

  const previewMeta = graphVersionPreview?.graph?.metadata || null;
  const draftSummary = useMemo(
    () => ({
      graphId: currentGraph?.metadata?.graph_id || "draft_graph",
      updatedAt: currentGraph?.metadata?.updated_at || null,
      nodeCount: currentGraph?.nodes?.length || 0,
      edgeCount: currentGraph?.edges?.length || 0
    }),
    [currentGraph]
  );
  const selectedCompareEntries = compareSelection
    .map((versionId) => graphVersions.find((entry) => entry.version_id === versionId))
    .filter(Boolean);

  async function handleSaveVersion() {
    await saveGraph({
      versionLabel: versionLabelDraft,
      saveNote: saveNoteDraft
    });
  }

  function toggleCompareSelection(versionId) {
    setCompareSelection((current) => {
      if (current.includes(versionId)) {
        return current.filter((item) => item !== versionId);
      }
      if (current.length >= 2) {
        return [current[1], versionId];
      }
      return [...current, versionId];
    });
  }

  async function handleCompareVersions() {
    if (compareSelection.length !== 2) {
      return;
    }
    await compareGraphVersions(graphId, compareSelection[0], compareSelection[1]);
  }

  return (
    <div className="open-orders-card" data-testid="workspace-version-history-card">
      <div className="open-orders-header">
        <div>
          <div className="mini-list-title">Version history</div>
          <div className="muted-line">
            Keep the working draft separate from persisted versions. Preview first, restore only
            when you want the saved version to become the current graph.
          </div>
        </div>
        <strong>{graphVersions.length}</strong>
      </div>

      <div className="strategy-inspector-metrics">
        <div className="kv-line">
          <span>Working draft</span>
          <strong>{draftSummary.graphId}</strong>
        </div>
        <div className="kv-line">
          <span>Draft updated</span>
          <strong>{formatTime(draftSummary.updatedAt)}</strong>
        </div>
        <div className="kv-line">
          <span>Draft shape</span>
          <strong>
            {draftSummary.nodeCount} nodes / {draftSummary.edgeCount} edges
          </strong>
        </div>
      </div>

      <div className="workspace-version-save-form" data-testid="workspace-version-save-form">
        <label className="field-label">
          Version label
          <input
            type="text"
            className="field-input"
            value={versionLabelDraft}
            data-testid="workspace-version-label-input"
            onChange={(event) => setVersionLabelDraft(event.target.value)}
            placeholder="baseline, tuned, pre-risk-refactor"
          />
        </label>
        <label className="field-label">
          Save note
          <textarea
            className="field-input field-input--multiline"
            value={saveNoteDraft}
            data-testid="workspace-version-note-input"
            onChange={(event) => setSaveNoteDraft(event.target.value)}
            placeholder="What changed in this persisted version?"
            rows={3}
          />
        </label>
        <div className="strategy-inspector-actions">
          <button
            type="button"
            className="ghost-btn compact-btn"
            data-testid="workspace-version-save-action"
            onClick={handleSaveVersion}
          >
            Save persisted version
          </button>
        </div>
      </div>

      {previewMeta ? (
        <div className="history-note history-note-info" data-testid="workspace-version-preview">
          Previewing persisted version {graphVersionPreview.versionId}: {previewMeta.name || previewMeta.graph_id} (
          {previewMeta.graph_id}, {formatTime(previewMeta.updated_at)}).
        </div>
      ) : null}

      {graphVersionCompare ? (
        <div className="history-note history-note-info" data-testid="workspace-version-compare-summary">
          Comparing {graphVersionCompare.left.version_id} against {graphVersionCompare.right.version_id}.
        </div>
      ) : null}

      {graphVersionPreviewStatus === "error" ? (
        <div className="history-note history-note-warning">{graphVersionPreviewMessage}</div>
      ) : null}
      {graphVersionCompareStatus === "error" ? (
        <div className="history-note history-note-warning">{graphVersionCompareMessage}</div>
      ) : null}
      {graphVersionsStatus === "error" ? (
        <div className="history-note history-note-warning">{graphVersionsMessage}</div>
      ) : null}
      {graphVersionsStatus === "loading" ? (
        <div className="muted-line">Loading persisted versions...</div>
      ) : null}
      {graphVersions.length === 0 && graphVersionsStatus !== "loading" ? (
        <div className="muted-line">No persisted version has been recorded for this graph yet.</div>
      ) : null}

      <div className="workspace-version-history-list">
        {graphVersions.map((entry) => {
          const isSelected = compareSelection.includes(entry.version_id);
          return (
            <div
              key={entry.version_id}
              className="open-order-item"
              data-testid={`workspace-version-entry-${entry.version_id}`}
            >
              <div className="open-order-topline">
                <strong>{entry.version_id}</strong>
                <span>{entry.is_latest ? "Latest" : "Persisted"}</span>
              </div>
              <div className="muted-line">
                {entry.name} at {formatTime(entry.updated_at)}
              </div>
              {entry.version_label ? (
                <div className="muted-line" data-testid={`workspace-version-label-${entry.version_id}`}>
                  Label: {entry.version_label}
                </div>
              ) : null}
              {entry.save_note ? (
                <div className="muted-line" data-testid={`workspace-version-note-${entry.version_id}`}>
                  Note: {entry.save_note}
                </div>
              ) : null}
              <div className="muted-line" data-testid={`workspace-version-shape-${entry.version_id}`}>
                {entry.node_count} nodes / {entry.edge_count} edges
              </div>
              <div className="strategy-inspector-actions">
                <button
                  type="button"
                  className="ghost-btn compact-btn"
                  data-testid={`workspace-version-preview-${entry.version_id}`}
                  onClick={() => loadGraphVersionPreview(graphId, entry.version_id)}
                >
                  Preview
                </button>
                <button
                  type="button"
                  className="ghost-btn compact-btn"
                  data-testid={`workspace-version-restore-${entry.version_id}`}
                  onClick={() => restoreGraphVersion(graphId, entry.version_id)}
                >
                  Restore
                </button>
                <button
                  type="button"
                  className={`ghost-btn compact-btn ${isSelected ? "active" : ""}`}
                  data-testid={`workspace-version-compare-toggle-${entry.version_id}`}
                  onClick={() => toggleCompareSelection(entry.version_id)}
                >
                  {isSelected ? "Selected" : "Compare"}
                </button>
              </div>
            </div>
          );
        })}
      </div>

      {selectedCompareEntries.length > 0 ? (
        <div className="history-note" data-testid="workspace-version-compare-selection">
          Compare queue: {selectedCompareEntries.map((entry) => entry.version_id).join(", ")}
        </div>
      ) : null}

      <div className="strategy-inspector-actions">
        <button
          type="button"
          className="ghost-btn compact-btn"
          data-testid="workspace-version-open-compare"
          disabled={compareSelection.length !== 2}
          onClick={handleCompareVersions}
        >
          Compare versions
        </button>
        <button
          type="button"
          className="ghost-btn compact-btn"
          data-testid="workspace-version-clear-compare"
          onClick={() => {
            setCompareSelection([]);
            clearGraphVersionCompare();
          }}
        >
          Clear compare
        </button>
      </div>

      {graphVersionCompare ? (
        <div className="workspace-version-compare-card" data-testid="workspace-version-compare-card">
          <div className="open-orders-header">
            <div>
              <div className="mini-list-title">Version diff</div>
              <div className="muted-line">
                Left: {graphVersionCompare.left.version_id} / Right: {graphVersionCompare.right.version_id}
              </div>
            </div>
            <strong>{graphVersionCompare.has_changes ? "Changed" : "Same"}</strong>
          </div>

          <div className="workspace-version-compare-grid">
            <DiffList
              title="Node diff"
              diff={graphVersionCompare.node_diff}
              testId="workspace-version-node-diff"
            />
            <DiffList
              title="Edge diff"
              diff={graphVersionCompare.edge_diff}
              testId="workspace-version-edge-diff"
            />
          </div>

          <div className="workspace-version-compare-group" data-testid="workspace-version-metadata-diff">
            <div className="mini-list-title">Metadata diff</div>
            <div className="strategy-inspector-metrics">
              {graphVersionCompare.metadata_rows.map((row) => (
                <div className="kv-line" key={row.key} data-testid={`workspace-version-metadata-row-${row.key}`}>
                  <span>{row.label}</span>
                  <strong>
                    {row.left_value || "-"} → {row.right_value || "-"}
                  </strong>
                </div>
              ))}
            </div>
          </div>

          <div className="workspace-version-compare-group" data-testid="workspace-version-config-diff">
            <div className="mini-list-title">Config diff</div>
            {graphVersionCompare.config_diffs.length === 0 ? (
              <div className="muted-line">No config field changed between the selected versions.</div>
            ) : (
              <div className="workspace-version-config-list">
                {graphVersionCompare.config_diffs.map((row, index) => (
                  <div
                    className="open-order-item"
                    key={`${row.node_id}-${row.field_path}-${index}`}
                    data-testid={`workspace-version-config-row-${row.node_id}-${index}`}
                  >
                    <div className="open-order-topline">
                      <strong>{row.node_name || row.node_id}</strong>
                      <span>{row.field_path}</span>
                    </div>
                    <div className="muted-line">
                      {row.left_value || "-"} → {row.right_value || "-"}
                    </div>
                  </div>
                ))}
              </div>
            )}
          </div>
        </div>
      ) : null}

      {graphVersionPreview ? (
        <div className="strategy-inspector-actions">
          <button
            type="button"
            className="ghost-btn compact-btn"
            onClick={clearGraphVersionPreview}
          >
            Clear preview
          </button>
        </div>
      ) : null}
    </div>
  );
}
