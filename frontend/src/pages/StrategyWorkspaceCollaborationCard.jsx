import { useEffect } from "react";
import { useGraphStore } from "../store/graphStore";
import {
  buildWorkspaceCollaborationRows,
  formatWorkspaceAuditActorLine,
  formatWorkspaceGovernanceTime as formatTime,
  shouldRefreshWorkspaceAuditHistory
} from "./strategyWorkspaceGovernanceCardsShell";

export default function StrategyWorkspaceCollaborationCard({ graphId, collaboration, lastRun, lastBacktest }) {
  const graphAuditHistory = useGraphStore((state) => state.graphAuditHistory);
  const graphAuditHistoryStatus = useGraphStore((state) => state.graphAuditHistoryStatus);
  const graphAuditHistoryMessage = useGraphStore((state) => state.graphAuditHistoryMessage);
  const refreshGraphAuditHistory = useGraphStore((state) => state.refreshGraphAuditHistory);
  const collaborationRows = buildWorkspaceCollaborationRows({
    collaboration,
    lastRun,
    lastBacktest
  });

  useEffect(() => {
    if (!shouldRefreshWorkspaceAuditHistory(graphId)) return;
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
        {collaborationRows.map((row) => (
          <div className="kv-line" data-testid={row.testId} key={row.testId}>
            <span>{row.label}</span>
            <strong>{row.value}</strong>
          </div>
        ))}
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
              {formatWorkspaceAuditActorLine(entry)}
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}
