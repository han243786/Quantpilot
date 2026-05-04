import { useEffect } from "react";
import { useGraphStore } from "../store/graphStore";

function formatTime(value) {
  return value ? new Date(value).toLocaleString() : "-";
}

function formatActor(actor, fallback = "未分配") {
  return actor?.display_name || actor?.actor_id || fallback;
}

export default function StrategyWorkspaceCollaborationCard({ graphId, collaboration, lastRun, lastBacktest }) {
  const graphAuditHistory = useGraphStore((state) => state.graphAuditHistory);
  const graphAuditHistoryStatus = useGraphStore((state) => state.graphAuditHistoryStatus);
  const graphAuditHistoryMessage = useGraphStore((state) => state.graphAuditHistoryMessage);
  const refreshGraphAuditHistory = useGraphStore((state) => state.refreshGraphAuditHistory);

  useEffect(() => {
    if (!graphId || graphId === "draft_graph") return;
    void refreshGraphAuditHistory(graphId);
  }, [graphId, refreshGraphAuditHistory]);

  return (
    <div className="open-orders-card" data-testid="workspace-collaboration-card">
      <div className="open-orders-header">
        <div>
          <div className="mini-list-title">协作与审计</div>
          <div className="muted-line">
            在工作区内保留策略图所有权、编辑权限和近期审计操作。
          </div>
        </div>
        <strong>{graphAuditHistory.length}</strong>
      </div>

      <div className="strategy-inspector-metrics">
        <div className="kv-line" data-testid="workspace-owner-row">
          <span>所有者</span>
          <strong>{formatActor(collaboration?.owner)}</strong>
        </div>
        <div className="kv-line" data-testid="workspace-editors-row">
          <span>协作者</span>
          <strong>
            {Array.isArray(collaboration?.editors) && collaboration.editors.length > 0
              ? collaboration.editors.map((actor) => formatActor(actor)).join(", ")
              : "未分配协作者"}
          </strong>
        </div>
        <div className="kv-line" data-testid="workspace-last-saved-row">
          <span>最近保存人</span>
          <strong>{formatActor(collaboration?.last_saved_by, "-")}</strong>
        </div>
        <div className="kv-line" data-testid="workspace-last-run-row">
          <span>最近执行人</span>
          <strong>
            {formatActor(
              lastRun?.actor || lastBacktest?.actor || collaboration?.last_run_actor,
              "-"
            )}
          </strong>
        </div>
      </div>

      {graphAuditHistoryStatus === "loading" ? <div className="muted-line">正在加载审计历史...</div> : null}
      {graphAuditHistoryStatus === "error" ? (
        <div className="history-note history-note-warning">{graphAuditHistoryMessage}</div>
      ) : null}

      <div className="workspace-collaboration-audit-list" data-testid="workspace-audit-list">
        {graphAuditHistory.length === 0 && graphAuditHistoryStatus !== "loading" ? (
          <div className="muted-line">该策略图尚未记录审计条目。</div>
        ) : null}
        {graphAuditHistory.map((entry) => (
          <div
            key={entry.audit_id}
            className="open-order-item"
            data-testid={`workspace-audit-entry-${entry.audit_id}`}
          >
            <div className="open-order-topline">
              <strong>{entry.action}</strong>
              <span>{formatTime(entry.created_at_ms)}</span>
            </div>
            <div className="muted-line">{entry.summary}</div>
            <div className="muted-line">
              {formatActor(entry.actor)}{entry.target_id ? ` / ${entry.target_id}` : ""}
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}
