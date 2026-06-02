import { useEffect, useMemo, useState } from "react";
import { useGraphStore } from "../store/graphStore";
import {
  buildWorkspaceVersionDraftSummary,
  buildWorkspaceVersionEvidenceOptions,
  formatWorkspaceGovernanceTime as formatTime,
  formatWorkspaceVersionCountChanges as formatCountChanges,
  formatWorkspaceVersionList as formatList,
  selectWorkspaceVersionCompareEntries,
  toggleWorkspaceVersionCompareSelection,
  workspaceConfigChangeLabels as configChangeLabels,
  workspaceConfigDomainLabel as configDomainLabel
} from "./strategyWorkspaceGovernanceCardsShell";

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

function StrategyConfigVersionDiff({ diff }) {
  if (!diff) {
    return null;
  }
  return (
    <div className="workspace-version-compare-group" data-testid="workspace-version-strategy-config-diff">
      <div className="mini-list-title">策略配置契约差异</div>
      {!diff.changed ? (
        <div className="muted-line">所选版本之间没有配置域变化。</div>
      ) : (
        <>
          <div className="strategy-inspector-metrics">
            <div className="kv-line">
              <span>配置契约</span>
              <strong>
                {diff.left_artifact_id || "-"} {"->"} {diff.right_artifact_id || "-"}
              </strong>
            </div>
            <div className="kv-line">
              <span>运行边界</span>
              <strong>{diff.runtime_boundary_changed ? "已变化" : "未变化"}</strong>
            </div>
            <div className="kv-line">
              <span>来源摘要</span>
              <strong>
                {(diff.source_digest_changes || []).map((change) => change.field).filter(Boolean).join(", ") || "-"}
              </strong>
            </div>
          </div>
          {(diff.domain_changes || []).length === 0 ? (
            <div className="muted-line">配置摘要有变化，但配置域状态没有结构化变化。</div>
          ) : (
            <div className="workspace-version-config-list">
              {diff.domain_changes.map((change) => (
                <div
                  className="open-order-item"
                  key={change.domain_id}
                  data-testid={`workspace-version-strategy-config-domain-${change.domain_id}`}
                >
                  <div className="open-order-topline">
                    <strong>{configDomainLabel(change.domain_id)}</strong>
                    <span>{configChangeLabels(change)}</span>
                  </div>
                </div>
              ))}
            </div>
          )}
        </>
      )}
    </div>
  );
}

function StrategyConfigEvidenceDiff({ diff }) {
  if (!diff) {
    return null;
  }
  const diagnostics = diff.diagnostics || [];
  return (
    <div className="workspace-version-compare-group" data-testid="workspace-version-strategy-config-evidence-diff">
      <div className="mini-list-title">回测证据差异</div>
      <div className="muted-line">
        {diff.left_backtest_id || "左侧未绑定"} {"->"} {diff.right_backtest_id || "右侧未绑定"}
      </div>
      {diff.status === "missing" ? (
        <div className="workspace-version-config-list">
          {diagnostics.length === 0 ? (
            <div className="muted-line">缺少 A/B v4 回测证据，无法生成证据差异。</div>
          ) : (
            diagnostics.map((finding) => (
              <div className="open-order-item" key={`${finding.code}-${finding.message}`}>
                <div className="open-order-topline">
                  <strong>{finding.code || "evidence_missing"}</strong>
                  <span>{finding.severity || "info"}</span>
                </div>
                <div className="muted-line">{finding.message}</div>
              </div>
            ))
          )}
        </div>
      ) : (
        <>
          <div className="strategy-inspector-metrics" data-testid="workspace-version-evidence-machine">
            <div className="kv-line">
              <span>机器轨迹</span>
              <strong>
                {diff.machine_trajectory?.left_point_count || 0} / {diff.machine_trajectory?.right_point_count || 0}
              </strong>
            </div>
            <div className="kv-line">
              <span>首次分叉</span>
              <strong>{diff.machine_trajectory?.first_divergence?.index ?? "-"}</strong>
            </div>
            <div className="kv-line">
              <span>Transition</span>
              <strong>{formatCountChanges(diff.machine_trajectory?.transition_hit_changes)}</strong>
            </div>
          </div>

          <div className="strategy-inspector-metrics" data-testid="workspace-version-evidence-risk-plane">
            <div className="kv-line">
              <span>Risk Plane</span>
              <strong>
                {diff.risk_plane?.left_decision_count || 0} / {diff.risk_plane?.right_decision_count || 0}
              </strong>
            </div>
            <div className="kv-line">
              <span>Allow / Reject</span>
              <strong>{formatCountChanges(diff.risk_plane?.action_count_changes)}</strong>
            </div>
            <div className="kv-line">
              <span>原因分布</span>
              <strong>{formatCountChanges(diff.risk_plane?.reason_count_changes)}</strong>
            </div>
          </div>

          <div className="strategy-inspector-metrics" data-testid="workspace-version-evidence-execution">
            <div className="kv-line">
              <span>执行能力</span>
              <strong>
                {diff.execution_capability?.left_source_count || 0} / {diff.execution_capability?.right_source_count || 0}
              </strong>
            </div>
            <div className="kv-line">
              <span>Runtime</span>
              <strong>{formatCountChanges(diff.execution_capability?.runtime_mode_changes)}</strong>
            </div>
            <div className="kv-line">
              <span>Capability</span>
              <strong>{formatCountChanges(diff.execution_capability?.capability_source_changes)}</strong>
            </div>
          </div>

          <div className="workspace-version-config-list" data-testid="workspace-version-evidence-metrics">
            {(diff.metrics?.fields || []).filter((field) => field.status !== "same").slice(0, 6).map((field) => (
              <div className="open-order-item" key={field.key}>
                <div className="open-order-topline">
                  <strong>{field.key}</strong>
                  <span>{field.status}</span>
                </div>
                <div className="muted-line">
                  {field.left_value || "-"} {"->"} {field.right_value || "-"}
                </div>
              </div>
            ))}
          </div>
        </>
      )}
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
  const backtestHistory = useGraphStore((state) => state.runtime.backtestHistory);

  const [versionLabelDraft, setVersionLabelDraft] = useState("");
  const [saveNoteDraft, setSaveNoteDraft] = useState("");
  const [compareSelection, setCompareSelection] = useState([]);
  const [leftEvidenceBacktestId, setLeftEvidenceBacktestId] = useState("");
  const [rightEvidenceBacktestId, setRightEvidenceBacktestId] = useState("");

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
    () => buildWorkspaceVersionDraftSummary(currentGraph),
    [currentGraph]
  );
  const selectedCompareEntries = selectWorkspaceVersionCompareEntries(compareSelection, graphVersions);
  const evidenceBacktestOptions = useMemo(
    () => buildWorkspaceVersionEvidenceOptions(backtestHistory, graphId),
    [backtestHistory, graphId]
  );

  async function handleSaveVersion() {
    await saveGraph({
      versionLabel: versionLabelDraft,
      saveNote: saveNoteDraft
    });
  }

  function toggleCompareSelection(versionId) {
    setCompareSelection((current) => toggleWorkspaceVersionCompareSelection(current, versionId));
  }

  async function handleCompareVersions() {
    if (compareSelection.length !== 2) {
      return;
    }
    await compareGraphVersions(graphId, compareSelection[0], compareSelection[1], {
      leftBacktestId: leftEvidenceBacktestId,
      rightBacktestId: rightEvidenceBacktestId
    });
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
            className="ad-btn ad-btn--ghost compact-btn"
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
                  className="ad-btn ad-btn--ghost compact-btn"
                  data-testid={`workspace-version-preview-${entry.version_id}`}
                  onClick={() => loadGraphVersionPreview(graphId, entry.version_id)}
                >
                  预览
                </button>
                <button
                  type="button"
                  className="ad-btn ad-btn--ghost compact-btn"
                  data-testid={`workspace-version-restore-${entry.version_id}`}
                  onClick={() => restoreGraphVersion(graphId, entry.version_id)}
                >
                  恢复
                </button>
                <button
                  type="button"
                  className={`ad-btn ad-btn--ghost compact-btn ${isSelected ? "active" : ""}`}
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

      <div className="workspace-version-save-form" data-testid="workspace-version-evidence-binding">
        <label className="field-label">
          左侧回测证据
          <select
            className="field-input"
            value={leftEvidenceBacktestId}
            data-testid="workspace-version-left-evidence-select"
            onChange={(event) => setLeftEvidenceBacktestId(event.target.value)}
          >
            <option value="">不绑定证据</option>
            {evidenceBacktestOptions.map((entry) => (
              <option key={`left-${entry.id}`} value={entry.id}>{entry.label}</option>
            ))}
          </select>
        </label>
        <label className="field-label">
          右侧回测证据
          <select
            className="field-input"
            value={rightEvidenceBacktestId}
            data-testid="workspace-version-right-evidence-select"
            onChange={(event) => setRightEvidenceBacktestId(event.target.value)}
          >
            <option value="">不绑定证据</option>
            {evidenceBacktestOptions.map((entry) => (
              <option key={`right-${entry.id}`} value={entry.id}>{entry.label}</option>
            ))}
          </select>
        </label>
      </div>

      <div className="strategy-inspector-actions">
        <button
          type="button"
          className="ad-btn ad-btn--ghost compact-btn"
          data-testid="workspace-version-open-compare"
          disabled={compareSelection.length !== 2}
          onClick={handleCompareVersions}
        >
          对比版本
        </button>
        <button
          type="button"
          className="ad-btn ad-btn--ghost compact-btn"
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

          <StrategyConfigVersionDiff diff={graphVersionCompare.strategy_config_diff} />
          <StrategyConfigEvidenceDiff diff={graphVersionCompare.strategy_config_evidence_diff} />
        </div>
      ) : null}

      {graphVersionPreview ? (
        <div className="strategy-inspector-actions">
          <button
            type="button"
            className="ad-btn ad-btn--ghost compact-btn"
            onClick={clearGraphVersionPreview}
          >
            清空预览
          </button>
        </div>
      ) : null}
    </div>
  );
}
