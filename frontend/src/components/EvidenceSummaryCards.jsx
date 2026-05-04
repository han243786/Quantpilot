import { buildRetentionAwareEvidencePreview } from "../utils/runtimeEvidenceSummary";

function sequenceLabel(card) {
  if (!card.sequence_numbers?.length) return "无序列";
  return card.sequence_numbers.slice(0, 4).join(", ");
}

export default function EvidenceSummaryCards({ source, testId = "evidence-summary-cards" }) {
  const preview = buildRetentionAwareEvidencePreview(source || {});

  return (
    <div className="mini-list" data-testid={testId}>
      <div className="open-orders-header">
        <div>
          <div className="mini-list-title">证据摘要</div>
          <div className="muted-line">
            {preview.strategy === "compact_evidence" ? "优先使用压缩证据" : "使用当前详情窗口"} ·{" "}
            {preview.retained_event_count}/{preview.source_event_count}
          </div>
        </div>
        <strong>{preview.detail_window_required ? "详情窗口" : "压缩视图"}</strong>
      </div>
      <div className="analysis-card-grid analysis-card-grid--three">
        {preview.cards.map((card) => (
          <div
            key={card.id}
            className="history-meta-chip"
            data-testid={`${testId}-${card.id}`}
          >
            <span>{card.label}</span>
            <strong>{card.count}</strong>
            <small title={card.latest_summary}>{sequenceLabel(card)}</small>
          </div>
        ))}
      </div>
    </div>
  );
}
