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
          <span>左侧 / 右侧</span>
          <strong>
            {diff.left_count} / {diff.right_count}
          </strong>
        </div>
        <div className="kv-line">
          <span>新增</span>
          <strong>{formatList(diff.added_ids)}</strong>
        </div>
        <div className="kv-line">
          <span>移除</span>
          <strong>{formatList(diff.removed_ids)}</strong>
        </div>
        <div className="kv-line">
          <span>变更</span>
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
          <div className="mini-list-title">版本历史</div>
          <div className="muted-line">
            保持工作草稿与持久化版本分离。先预览，只有需要让保存版本成为当前策略图时才恢复。
          </div>
        </div>
        <strong>{graphVersions.length}</strong>
      </div>

      <div className="strategy-inspector-metrics">
        <div className="kv-line">
          <span>工作草稿</span>
          <strong>{draftSummary.graphId}</strong>
        </div>
        <div className="kv-line">
          <span>草稿更新时间</span>
          <strong>{formatTime(draftSummary.updatedAt)}</strong>
        </div>
        <div className="kv-line">
          <span>草稿结构</span>
          <strong>
            {draftSummary.nodeCount} 节点 / {draftSummary.edgeCount} 连线
          </strong>
        </div>
      </div>

      <div className="workspace-version-save-form" data-testid="workspace-version-save-form">
        <label className="field-label">
          版本标签
          <input
            type="text"
            className="field-input"
            value={versionLabelDraft}
            data-testid="workspace-version-label-input"
            onChange={(event) => setVersionLabelDraft(event.target.value)}
            placeholder="基线、调参后、风控重构前"
          />
        </label>
        <label className="field-label">
          保存说明
          <textarea
            className="field-input field-input--multiline"
            value={saveNoteDraft}
            data-testid="workspace-version-note-input"
            onChange={(event) => setSaveNoteDraft(event.target.value)}
            placeholder="这个持久化版本有哪些变化？"
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
            保存持久化版本
          </button>
        </div>
      </div>

      {previewMeta ? (
        <div className="history-note history-note-info" data-testid="workspace-version-preview">
          正在预览持久化版本 {graphVersionPreview.versionId}：{previewMeta.name || previewMeta.graph_id} (
          {previewMeta.graph_id}, {formatTime(previewMeta.updated_at)}).
        </div>
      ) : null}

      {graphVersionCompare ? (
        <div className="history-note history-note-info" data-testid="workspace-version-compare-summary">
          正在对比 {graphVersionCompare.left.version_id} 与 {graphVersionCompare.right.version_id}。
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
        <div className="muted-line">正在加载持久化版本...</div>
      ) : null}
      {graphVersions.length === 0 && graphVersionsStatus !== "loading" ? (
        <div className="muted-line">该策略图尚未记录持久化版本。</div>
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
                <span>{entry.is_latest ? "最新" : "已保存"}</span>
              </div>
              <div className="muted-line">
                {entry.name}，{formatTime(entry.updated_at)}
              </div>
              {entry.version_label ? (
                <div className="muted-line" data-testid={`workspace-version-label-${entry.version_id}`}>
                  标签：{entry.version_label}
                </div>
              ) : null}
              {entry.save_note ? (
                <div className="muted-line" data-testid={`workspace-version-note-${entry.version_id}`}>
                  说明：{entry.save_note}
                </div>
              ) : null}
              <div className="muted-line" data-testid={`workspace-version-shape-${entry.version_id}`}>
                {entry.node_count} 节点 / {entry.edge_count} 连线
              </div>
              <div className="strategy-inspector-actions">
                <button
                  type="button"
                  className="ghost-btn compact-btn"
                  data-testid={`workspace-version-preview-${entry.version_id}`}
                  onClick={() => loadGraphVersionPreview(graphId, entry.version_id)}
                >
                  预览
                </button>
                <button
                  type="button"
                  className="ghost-btn compact-btn"
                  data-testid={`workspace-version-restore-${entry.version_id}`}
                  onClick={() => restoreGraphVersion(graphId, entry.version_id)}
                >
                  恢复
                </button>
                <button
                  type="button"
                  className={`ghost-btn compact-btn ${isSelected ? "active" : ""}`}
                  data-testid={`workspace-version-compare-toggle-${entry.version_id}`}
                  onClick={() => toggleCompareSelection(entry.version_id)}
                >
                  {isSelected ? "已选择" : "对比"}
                </button>
              </div>
            </div>
          );
        })}
      </div>

      {selectedCompareEntries.length > 0 ? (
        <div className="history-note" data-testid="workspace-version-compare-selection">
          对比队列：{selectedCompareEntries.map((entry) => entry.version_id).join(", ")}
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
          对比版本
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
          清空对比
        </button>
      </div>

      {graphVersionCompare ? (
        <div className="workspace-version-compare-card" data-testid="workspace-version-compare-card">
          <div className="open-orders-header">
            <div>
              <div className="mini-list-title">版本差异</div>
              <div className="muted-line">
                左侧：{graphVersionCompare.left.version_id} / 右侧：{graphVersionCompare.right.version_id}
              </div>
            </div>
            <strong>{graphVersionCompare.has_changes ? "有变更" : "相同"}</strong>
          </div>

          <div className="workspace-version-compare-grid">
            <DiffList
              title="节点差异"
              diff={graphVersionCompare.node_diff}
              testId="workspace-version-node-diff"
            />
            <DiffList
              title="连线差异"
              diff={graphVersionCompare.edge_diff}
              testId="workspace-version-edge-diff"
            />
          </div>

          <div className="workspace-version-compare-group" data-testid="workspace-version-metadata-diff">
            <div className="mini-list-title">元数据差异</div>
            <div className="strategy-inspector-metrics">
              {graphVersionCompare.metadata_rows.map((row) => (
                <div className="kv-line" key={row.key} data-testid={`workspace-version-metadata-row-${row.key}`}>
                  <span>{row.label}</span>
                  <strong>
                    {row.left_value || "-"} {"->"} {row.right_value || "-"}
                  </strong>
                </div>
              ))}
            </div>
          </div>

          <div className="workspace-version-compare-group" data-testid="workspace-version-config-diff">
            <div className="mini-list-title">配置差异</div>
            {graphVersionCompare.config_diffs.length === 0 ? (
              <div className="muted-line">所选版本之间没有配置字段变化。</div>
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
                      {row.left_value || "-"} {"->"} {row.right_value || "-"}
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
            清空预览
          </button>
        </div>
      ) : null}
    </div>
  );
}
