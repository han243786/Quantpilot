import { useEffect, useMemo, useState } from "react";
import EventStreamPanel from "../components/EventStreamPanel";
import { useGraphStore } from "../store/graphStore";
import {
  navigateTo,
  strategiesPath,
  strategyBacktestsPath,
  strategyWorkspacePath
} from "../router";
import { useI18n } from "../i18n";
import {
  AnalysisHero,
  AnalysisSection,
  formatTime,
  MetricPair
} from "./backtestViews/shared";
import {
  BacktestDetailCoreArtifactSections,
  BacktestDetailGovernedTimelineSection,
  BacktestDetailReplayOutputExplanationSections,
  BacktestDetailReportLifecycleSection,
  BacktestDetailV4ArtifactSection,
  buildBacktestDetailPageModel,
  buildBacktestDetailSummaryModel
} from "./backtestViews/detailPageAnalysis";
import { buildDiagnosticsExplanationEntries } from "../utils/runtimeExplanation";
import {
  buildGovernanceIdentityRows,
  governanceFromRuntime
} from "../utils/runtimeGovernance";

export default function BacktestDetailPage({ backtestId, strategyId = "" }) {
  const { t } = useI18n();
  const runtime = useGraphStore((state) => state.runtime);
  const graph = useGraphStore((state) => state.graph);
  const loadBacktestDetail = useGraphStore((state) => state.loadBacktestDetail);

  useEffect(() => {
    loadBacktestDetail(backtestId)?.catch((err) => {
      console.warn("[BacktestDetail] 加载回测详情失败:", err.message);
    });
  }, [backtestId, loadBacktestDetail]);

  const {
    selectedSummary,
    metrics,
    manifest,
    equityCurve,
    trades,
    outputArtifacts,
    v4Artifact,
    v4MicroMetrics,
    summary,
    startedAt,
    endedAt,
    resolvedStrategyId,
    curvePreview,
    tradePreview,
    timelineSource
  } = useMemo(
    () => buildBacktestDetailPageModel({ runtime, strategyId, backtestId }),
    [backtestId, runtime, strategyId]
  );
  const governanceRows = useMemo(
    () => buildGovernanceIdentityRows(governanceFromRuntime(runtime)),
    [runtime]
  );

  const riskExplanationEntries = useMemo(
    () => buildDiagnosticsExplanationEntries(graph, runtime.diagnostics, "risk"),
    [graph, runtime.diagnostics]
  );
  const orderExplanationEntries = useMemo(
    () => buildDiagnosticsExplanationEntries(graph, runtime.diagnostics, "order"),
    [graph, runtime.diagnostics]
  );
  const [summaryExpanded, setSummaryExpanded] = useState(false);

  const { summaryItems } = useMemo(
    () =>
      buildBacktestDetailSummaryModel({
        t,
        summary,
        metrics,
        manifest,
        selectedSummary,
        trades,
        summaryExpanded
      }),
    [manifest, metrics, selectedSummary, summary, summaryExpanded, t, trades]
  );

  if (runtime.backendError) {
    return (
      <div className="qp-page">
        <div className="qp-error" role="alert">
          <span>{t("加载回测失败:")} {runtime.backendError}</span>
        </div>
        <div style={{ display: "flex", gap: 8, marginTop: 12, justifyContent: "center" }}>
          <button className="ad-btn ad-btn--ghost" onClick={() => loadBacktestDetail(backtestId)}>
            {t("重试")}
          </button>
          <button className="ad-btn ad-btn--ghost" onClick={() => navigateTo(strategiesPath())}>
            {t("返回策略中心")}
          </button>
        </div>
      </div>
    );
  }

  if (!metrics && !summary && !runtime.backendError) {
    const isLoaded = runtime.selectedBacktestId && runtime.backtestArtifacts !== undefined;
    if (isLoaded) {
      return (
        <div className="qp-page">
          <div className="qp-empty">{t("回测数据为空，请尝试重新运行回测。")}</div>
        </div>
      );
    }
    return (
      <div className="qp-page">
        <div className="qp-loading">{t("加载回测数据...")}</div>
      </div>
    );
  }

  return (
    <main className="detail-page strategy-analysis-page">
      <h1 style={{position:"absolute",width:"1px",height:"1px",overflow:"hidden",clip:"rect(0,0,0,0)",whiteSpace:"nowrap"}}>回测详情</h1>
      <AnalysisHero
        testId="backtest-detail-hero"
        routeItems={[
          { label: t("策略"), onClick: () => navigateTo(strategiesPath()) },
          resolvedStrategyId
            ? {
                label: resolvedStrategyId,
                onClick: () => navigateTo(strategyWorkspacePath(resolvedStrategyId))
              }
            : null,
          resolvedStrategyId
            ? {
                label: t("回测"),
                onClick: () => navigateTo(strategyBacktestsPath(resolvedStrategyId))
              }
            : null,
          { label: t("详情"), current: true }
        ]}
        kicker={t("策略研究")}
        title={t("策略回测详情")}
        subtitle={t("在策略上下文中查看单次持久化实验，并在审查结果时持续保留回放输出、工件链路与返回入口。")}
        meta={`策略：${resolvedStrategyId || graph.metadata?.graph_id || "-"} | 回测：${
          runtime.selectedBacktestId || backtestId
        }`}
        actions={
          <div className="toolbar-group">
            <button
              className="ad-btn ad-btn--ghost"
              onClick={() =>
                navigateTo(
                  resolvedStrategyId ? strategyBacktestsPath(resolvedStrategyId) : strategiesPath()
                )
              }
            >
              {resolvedStrategyId ? t("返回策略回测页") : t("返回策略列表")}
            </button>
            {resolvedStrategyId ? (
              <button
                className="ad-btn ad-btn--ghost"
                onClick={() => navigateTo(strategyWorkspacePath(resolvedStrategyId))}
              >
                {t("打开策略工作区")}
              </button>
            ) : null}
          </div>
        }
        summaryItems={summaryItems}
      />
      <div style={{ display: "flex", justifyContent: "flex-end", padding: "0 0 12px" }}>
        <button
          className="ad-btn ad-btn--ghost compact-btn"
          onClick={() => setSummaryExpanded(!summaryExpanded)}
        >
          {summaryExpanded ? t("收起") : t("展开详情")}
        </button>
      </div>

      <div className="analysis-page-grid">
        <div className="analysis-main-column">
          <BacktestDetailCoreArtifactSections
            t={t}
            selectedSummary={selectedSummary}
            manifest={manifest}
            metrics={metrics}
            summary={summary}
            startedAt={startedAt}
            endedAt={endedAt}
            governanceRows={governanceRows}
            equityCurve={equityCurve}
            periodReturns={runtime.backtestArtifacts?.period_returns || []}
            metricsArtifactId={runtime.backtestArtifacts?.metrics?.artifact_id || "-"}
            eventsLength={runtime.events.length}
          />

          <BacktestDetailGovernedTimelineSection
            t={t}
            timelineSource={timelineSource}
          />

          <BacktestDetailV4ArtifactSection
            v4Artifact={v4Artifact}
            v4MicroMetrics={v4MicroMetrics}
          />

          <BacktestDetailReportLifecycleSection
            t={t}
            sourceId={runtime.selectedBacktestId || backtestId}
            timelineSource={timelineSource}
          />

          <BacktestDetailReplayOutputExplanationSections
            t={t}
            curvePreview={curvePreview}
            tradePreview={tradePreview}
            equityCurveArtifactId={runtime.backtestArtifacts?.equity_curve?.artifact_id || "-"}
            tradeLedgerArtifactId={runtime.backtestArtifacts?.trade_ledger?.artifact_id || "-"}
            outputArtifacts={outputArtifacts}
            riskExplanationEntries={riskExplanationEntries}
            orderExplanationEntries={orderExplanationEntries}
          />
        </div>

        <div className="analysis-sidebar-column">
          <AnalysisSection
            testId="backtest-detail-context"
            kicker={t("策略上下文")}
            title={t("策略实验上下文")}
            summary={t("在详情、对比和工作区之间切换时，把策略 ID、实验 ID 与回放时序集中展示在一个位置。")}
          >
            <div className="open-orders-card" data-testid="backtest-detail-context-card">
              <MetricPair label={t("策略 ID")} value={resolvedStrategyId || graph.metadata?.graph_id || "-"} />
              <MetricPair label={t("图 ID")} value={graph.metadata?.graph_id || "-"} />
              <MetricPair label={t("回测 ID")} value={runtime.selectedBacktestId || backtestId} />
              <MetricPair label={t("协议")} value={manifest?.protocol_name || "-"} />
              <MetricPair label={t("配置哈希")} value={manifest?.config_hash || "-"} />
              <MetricPair label={t("开始时间")} value={formatTime(startedAt)} />
              <MetricPair label={t("结束时间")} value={formatTime(endedAt)} />
            </div>
          </AnalysisSection>
        </div>
      </div>

      <div className="analysis-followup-section">
        <EventStreamPanel detailMode />
      </div>
    </main>
  );
}
