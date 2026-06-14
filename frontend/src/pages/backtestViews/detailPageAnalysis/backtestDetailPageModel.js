const PREVIEW_EDGE_COUNT = 4;
const TRADE_PREVIEW_LIMIT = 8;

export function previewEquityCurve(equityCurve, edgeCount = PREVIEW_EDGE_COUNT) {
  if (!Array.isArray(equityCurve) || equityCurve.length === 0) return [];
  const visibleCount = edgeCount * 2;
  if (equityCurve.length <= visibleCount) return equityCurve;
  return [...equityCurve.slice(0, edgeCount), ...equityCurve.slice(-edgeCount)];
}

export function previewTrades(trades, limit = TRADE_PREVIEW_LIMIT) {
  return Array.isArray(trades) ? trades.slice(0, limit) : [];
}

export function buildBacktestDetailPageModel({ runtime = {}, strategyId = "", backtestId = "" }) {
  const selectedBacktestId = runtime.selectedBacktestId || backtestId;
  const backtestHistory = Array.isArray(runtime.backtestHistory) ? runtime.backtestHistory : [];
  const selectedSummary = runtime.selectedBacktestId
    ? backtestHistory.find((item) => item.backtest_id === runtime.selectedBacktestId) || null
    : null;
  const artifacts = runtime.backtestArtifacts || null;
  const metrics = artifacts?.metrics || null;
  const manifest = artifacts?.manifest || null;
  const equityCurve = artifacts?.equity_curve?.points || [];
  const trades = artifacts?.trade_ledger?.trades || [];
  const v4Artifact = artifacts?.v4_artifact || null;
  const summary = metrics?.summary || null;

  return {
    selectedBacktestId,
    selectedSummary,
    artifacts,
    metrics,
    manifest,
    equityCurve,
    trades,
    outputArtifacts: manifest?.output_artifacts || [],
    v4Artifact,
    v4MicroMetrics: v4Artifact?.microstructure_metrics || null,
    summary,
    startedAt: metrics?.started_at_ms || null,
    endedAt: metrics?.ended_at_ms || null,
    resolvedStrategyId: strategyId || selectedSummary?.graph_id || artifacts?.graph_id || "",
    curvePreview: previewEquityCurve(equityCurve),
    tradePreview: previewTrades(trades),
    timelineSource: {
      timeline: runtime.timeline,
      events: runtime.events,
      retained_key_event_index: runtime.retainedKeyEventIndex,
      compact_evidence: runtime.compactEvidence
    }
  };
}
