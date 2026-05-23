import { useGraphStore } from "../store/graphStore";
import { StrategyCardNote } from "../pages/StrategyHubSharedComponents";
import { getRuntimeStatusMeta, runtimeStatusLabel } from "../utils/runtimeStatus";
import { buildRuntimeDiagnosticsProjection } from "../utils/runtimeDiagnosticsProjection";
import GovernedTimelinePanel from "./GovernedTimelinePanel";
import RuntimeReportPanel from "./RuntimeReportPanel";
import V4RuntimeEvidencePanel from "./V4RuntimeEvidencePanel";

function DiagnosticsRows({ rows, emptyText }) {
  if (!rows || rows.length === 0) {
    return <div className="muted-line">{emptyText}</div>;
  }

  return rows.map((row) => (
    <div key={row.key} className="kv-line">
      <span>{row.label}</span>
      <strong title={row.fullValue || row.value}>{row.value}</strong>
    </div>
  ));
}

export default function RuntimeDiagnosticsPanel({
  graph,
  runtime,
  selectedNodeId = null,
  onSelectNode = null,
  title = "运行时诊断",
  subtitle = "按节点收拢最近事件、输入输出快照、数据质量与订单/风控解释，避免在事件流里反复扫读。"
}) {
  const setSelectedNode = useGraphStore((state) => state.setSelectedNode);
  const projection = buildRuntimeDiagnosticsProjection(graph, runtime, selectedNodeId);
  const selectNode = onSelectNode || setSelectedNode;
  const reportSourceKind = runtime?.selectedBacktestId ? "backtest" : "run";
  const reportSourceId = runtime?.selectedBacktestId || runtime?.selectedHistoryRunId || runtime?.runId;

  if (!projection?.selectedNode) {
    return (
      <div className="property-card runtime-diagnostics-card">
        <div className="property-card-heading">
          <div className="property-card-title strategy-card-title-note">
            <StrategyCardNote label={title} note={subtitle} />
          </div>
        </div>
        <div className="muted-line">
          当前还没有可归属到节点的运行时事件。先启动模拟、加载回测详情，或在画布中选中一个活跃节点。
        </div>
      </div>
    );
  }

  const runtimeMeta = getRuntimeStatusMeta(projection.selectedNode.runtime_state?.status || "idle");

  return (
    <div className="property-card runtime-diagnostics-card" data-testid="runtime-diagnostics-panel">
      <div className="property-card-heading">
        <div className="property-card-title strategy-card-title-note">
          <StrategyCardNote label={title} note={subtitle} />
        </div>
      </div>

      {projection.activeNodes.length > 1 ? (
        <div
          className="strategy-inspector-actions"
          aria-label="运行诊断节点"
          data-testid="runtime-diagnostics-node-switcher"
        >
          {projection.activeNodes.map((node) => (
            <button
              key={node.nodeId}
              type="button"
              className={`ghost-btn compact-btn${
                node.nodeId === projection.selectedNodeId ? " is-active" : ""
              }`}
              aria-pressed={node.nodeId === projection.selectedNodeId}
              onClick={() => selectNode(node.nodeId)}
            >
              {node.nodeName}
            </button>
          ))}
        </div>
      ) : null}

      <div className="kv-line">
        <span>节点</span>
        <strong>{projection.selectedNode.name || projection.selectedNode.id}</strong>
      </div>
      <div className="kv-line">
        <span>运行状态</span>
        <strong>
          <span className={`status-pill ${runtimeMeta.tone}`}>
            {runtimeStatusLabel(projection.selectedNode.runtime_state?.status || "idle")}
          </span>
        </strong>
      </div>
      <div className="kv-line">
        <span>最近事件</span>
        <strong>
          {projection.latestEvent?.summary || projection.selectedNode.runtime_state?.last_message || "-"}
        </strong>
      </div>
      <div className="kv-line">
        <span>事件总数</span>
        <strong>{projection.eventCount}</strong>
      </div>

      {projection.explanationSummary ? (
        <div
          className="issue-hint history-note history-note-info"
          data-testid="runtime-diagnostics-explanation-summary"
        >
          {projection.explanationSummary}
        </div>
      ) : null}

      {projection.latestNotice ? (
        <div className={`issue-hint history-note history-note-${projection.latestNotice.tone}`}>
          {projection.latestNotice.label} · {projection.latestNotice.timeLabel} ·{" "}
          {projection.latestNotice.summary}
        </div>
      ) : null}

      {projection.governanceRows?.length > 0 ? (
        <div className="mini-list" data-testid="runtime-diagnostics-governance">
          <div className="mini-list-title">治理身份</div>
          <DiagnosticsRows rows={projection.governanceRows} emptyText="当前没有治理身份。" />
        </div>
      ) : null}

      <V4RuntimeEvidencePanel source={runtime} testId="runtime-diagnostics-v4-evidence" />

      <GovernedTimelinePanel
        source={runtime}
        title="证据时间轴"
        summary="按 envelope 阶段、保留级别和模块过滤当前运行证据。"
        testId="runtime-diagnostics-timeline"
      />

      <RuntimeReportPanel
        sourceKind={reportSourceKind}
        sourceId={reportSourceId}
        evidenceSource={runtime}
        title="运行证据报告"
        summary="从当前运行或回测证据生成治理报告，保留 source id、序列范围和治理身份。"
      />

      <div className="mini-list" data-testid="runtime-diagnostics-input-snapshot">
        <div className="mini-list-title">最新输入快照</div>
        <DiagnosticsRows
          rows={projection.latestInputRows}
          emptyText="当前事件没有结构化输入字段。"
        />
      </div>

      <div className="mini-list" data-testid="runtime-diagnostics-output-snapshot">
        <div className="mini-list-title">最新输出快照</div>
        <DiagnosticsRows
          rows={projection.latestOutputRows}
          emptyText="当前事件没有可展示的结构化输出字段。"
        />
      </div>

      {projection.dataQualityRows.length > 0 ? (
        <div className="mini-list" data-testid="runtime-diagnostics-data-quality">
          <div className="mini-list-title">数据质量</div>
          <DiagnosticsRows
            rows={projection.dataQualityRows}
            emptyText="当前节点还没有可展示的数据质量字段。"
          />
        </div>
      ) : null}

      {projection.explanationRows.length > 0 ? (
        <div className="mini-list" data-testid="runtime-diagnostics-explanation-rows">
          <div className="mini-list-title">解释字段</div>
          <DiagnosticsRows rows={projection.explanationRows} emptyText="-" />
        </div>
      ) : null}

      {projection.riskDetailRows.length > 0 ? (
        <div className="mini-list" data-testid="runtime-diagnostics-risk-detail">
          <div className="mini-list-title">风控详情</div>
          <DiagnosticsRows
            rows={projection.riskDetailRows}
            emptyText="当前节点没有可展示的风控详情。"
          />
        </div>
      ) : null}

      {projection.orderDetailRows.length > 0 ? (
        <div className="mini-list" data-testid="runtime-diagnostics-order-detail">
          <div className="mini-list-title">订单详情</div>
          <DiagnosticsRows
            rows={projection.orderDetailRows}
            emptyText="当前节点没有可展示的订单详情。"
          />
        </div>
      ) : null}

      <div className="mini-list" data-testid="runtime-diagnostics-recent-events">
        <div className="mini-list-title">最近节点事件</div>
        {projection.recentEvents.length === 0 ? (
          <div className="muted-line">当前节点还没有近期事件。</div>
        ) : (
          projection.recentEvents.map((event) => (
            <div key={event.eventId || `${event.label}_${event.timeLabel}`} className="mini-item">
              <div className="kv-line">
                <span>{event.label}</span>
                <strong>
                  <span className={`status-pill ${event.tone}`}>{event.timeLabel}</span>
                </strong>
              </div>
              <div className="muted-line">{event.summary}</div>
            </div>
          ))
        )}
      </div>
    </div>
  );
}
