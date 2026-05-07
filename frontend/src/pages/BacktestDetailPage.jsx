import { useEffect, useMemo, useState } from "react";
import EventStreamPanel from "../components/EventStreamPanel";
import GovernedTimelinePanel from "../components/GovernedTimelinePanel";
import RuntimeReportPanel from "../components/RuntimeReportPanel";
import { useGraphStore } from "../store/graphStore";
import {
  navigateTo,
  strategiesPath,
  strategyBacktestsPath,
  strategyWorkspacePath
} from "../router";
import { useI18n } from "../i18n";
import { AnalysisHero, AnalysisSection } from "./BacktestAnalysisLayout";
import {
  formatRatio,
  formatTime,
  formatValue,
  MetricPair
} from "./backtestAnalysisShared";
import { buildDiagnosticsExplanationEntries } from "../utils/runtimeExplanation";
import {
  buildGovernanceIdentityRows,
  governanceFromRuntime
} from "../utils/runtimeGovernance";

function ExplanationDetailCard({ title, summary, entries, testId, emptyText }) {
  return (
    <div className="open-orders-card" data-testid={testId}>
      <div className="open-orders-header">
        <div>
          <div className="mini-list-title">{title}</div>
          <div className="muted-line">{summary}</div>
        </div>
        <strong>{entries.length}</strong>
      </div>
      {entries.length === 0 ? <div className="muted-line">{emptyText}</div> : null}
      {entries.map((entry) => (
        <div
          key={entry.nodeId}
          className="open-order-item"
          data-testid={`${testId}-entry-${entry.nodeId}`}
        >
          <div className="open-order-topline">
            <strong>{entry.nodeName}</strong>
            <span>{entry.nodeId}</span>
          </div>
          {entry.explanationSummary ? <div className="muted-line">{entry.explanationSummary}</div> : null}
          <div className="open-order-grid">
            {entry.rows.map((row) => (
              <div key={`${entry.nodeId}_${row.key}`}>
                <span>{row.label}</span>
                <strong>{row.value}</strong>
              </div>
            ))}
          </div>
        </div>
      ))}
    </div>
  );
}

export default function BacktestDetailPage({ backtestId, strategyId = "" }) {
  const { t } = useI18n();
  const runtime = useGraphStore((state) => state.runtime);
  const graph = useGraphStore((state) => state.graph);
  const loadBacktestDetail = useGraphStore((state) => state.loadBacktestDetail);

  useEffect(() => {
    void loadBacktestDetail(backtestId);
  }, [backtestId, loadBacktestDetail]);

  const selectedSummary = runtime.selectedBacktestId
    ? runtime.backtestHistory.find(
        (item) => item.backtest_id === runtime.selectedBacktestId
      ) || null
    : null;

  const metrics = runtime.backtestArtifacts?.metrics || null;
  const manifest = runtime.backtestArtifacts?.manifest || null;
  const equityCurve = runtime.backtestArtifacts?.equity_curve?.points || [];
  const trades = runtime.backtestArtifacts?.trade_ledger?.trades || [];
  const outputArtifacts = manifest?.output_artifacts || [];
  const summary = metrics?.summary || null;
  const startedAt = metrics?.started_at_ms || null;
  const endedAt = metrics?.ended_at_ms || null;
  const resolvedStrategyId =
    strategyId || selectedSummary?.graph_id || runtime.backtestArtifacts?.graph_id || "";
  const governanceRows = useMemo(
    () => buildGovernanceIdentityRows(governanceFromRuntime(runtime)),
    [runtime]
  );

  const curvePreview = useMemo(() => {
    if (!Array.isArray(equityCurve) || equityCurve.length === 0) return [];
    if (equityCurve.length <= 8) return equityCurve;
    return [...equityCurve.slice(0, 4), ...equityCurve.slice(-4)];
  }, [equityCurve]);

  const tradePreview = useMemo(() => trades.slice(0, 8), [trades]);
  const riskExplanationEntries = useMemo(
    () => buildDiagnosticsExplanationEntries(graph, runtime.diagnostics, "risk"),
    [graph, runtime.diagnostics]
  );
  const orderExplanationEntries = useMemo(
    () => buildDiagnosticsExplanationEntries(graph, runtime.diagnostics, "order"),
    [graph, runtime.diagnostics]
  );
  const timelineSource = useMemo(
    () => ({
      timeline: runtime.timeline,
      events: runtime.events,
      retained_key_event_index: runtime.retainedKeyEventIndex,
      compact_evidence: runtime.compactEvidence
    }),
    [runtime.compactEvidence, runtime.events, runtime.retainedKeyEventIndex, runtime.timeline]
  );

  const [summaryExpanded, setSummaryExpanded] = useState(false);

  const visibleSummaryItems = [
    {
      label: t("收益"),
      value: formatRatio(summary?.total_return_ratio)
    },
    {
      label: t("最大回撤"),
      value: formatRatio(summary?.max_drawdown_ratio)
    },
    {
      label: t("胜率"),
      value: summary?.win_rate != null ? `${(summary.win_rate * 100).toFixed(1)}%` : "-"
    },
    {
      label: t("成交数"),
      value: formatValue(summary?.trade_count ?? trades.length)
    }
  ];

  const foldedSummaryItems = [
    {
      label: t("协议"),
      value: manifest?.protocol_name || selectedSummary?.protocol_name || "-"
    },
    {
      label: t("配置哈希"),
      value: manifest?.config_hash || selectedSummary?.config_hash || "-"
    },
    {
      label: t("最终权益"),
      value: formatValue(summary?.final_equity || metrics?.final_account?.equity_estimate)
    }
  ];

  const summaryItems = summaryExpanded
    ? [...visibleSummaryItems, ...foldedSummaryItems]
    : visibleSummaryItems;

  if (runtime.backendError) {
    return (
      <div className="qp-page">
        <div className="qp-error" role="alert">
          <span>{t("加载回测失败:")} {runtime.backendError}</span>
        </div>
        <div style={{ display: "flex", gap: 8, marginTop: 12, justifyContent: "center" }}>
          <button className="ghost-btn" onClick={() => loadBacktestDetail(backtestId)}>
            {t("重试")}
          </button>
          <button className="ghost-btn" onClick={() => navigateTo(strategiesPath())}>
            {t("返回策略中心")}
          </button>
        </div>
      </div>
    );
  }

  if (!metrics && !summary && !runtime.backendError) {
    return (
      <div className="qp-page">
        <div className="qp-loading">{t("加载回测数据...")}</div>
      </div>
    );
  }

  return (
    <div className="detail-page strategy-analysis-page">
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
              className="ghost-btn"
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
                className="ghost-btn"
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
          className="ghost-btn compact-btn"
          onClick={() => setSummaryExpanded(!summaryExpanded)}
        >
          {summaryExpanded ? t("收起") : t("展开详情")}
        </button>
      </div>

      <div className="analysis-page-grid">
        <div className="analysis-main-column">
          <AnalysisSection
            testId="backtest-detail-core-artifacts"
            kicker={t("工件概览")}
            title={t("核心工件")}
            summary={t("先以持久化的 manifest 和 metrics 工件作为主要审查入口，再按需展开回放预览或完整事件流。")}
          >
            <div className="analysis-card-grid analysis-card-grid--two">
              <div className="open-orders-card" data-testid="backtest-detail-manifest-card">
                <div className="open-orders-header">
                  <div>
                    <div className="mini-list-title">{t("清单工件")}</div>
                    <div className="muted-line">
                      {t("与策略关联的 manifest 上下文、编译链路与回放来源。")}
                    </div>
                  </div>
                  <strong>{manifest?.manifest_id || "-"}</strong>
                </div>
                <MetricPair label={t("创建时间")} value={formatTime(manifest?.created_at_ms)} />
                <MetricPair
                  label={t("编译 ID")}
                  value={manifest?.compile_id || selectedSummary?.compile_id || "-"}
                />
                <MetricPair
                  label={t("运行规格")}
                  value={manifest?.backtest_spec?.run_spec?.schema_version || "-"}
                />
                <MetricPair
                  label={t("回放来源")}
                  value={manifest?.backtest_spec?.replay_source || "-"}
                />
                <MetricPair
                  label={t("策略工件")}
                  value={manifest?.compile_artifacts?.strategy?.artifact_id || "-"}
                  testId="backtest-detail-manifest-strategy-artifact"
                />
                <MetricPair
                  label={t("编译工件")}
                  value={manifest?.compile_artifacts?.compile?.artifact_id || "-"}
                  testId="backtest-detail-manifest-compile-artifact"
                />
                <MetricPair
                  label={t("Core IR 工件")}
                  value={manifest?.compile_artifacts?.core_ir?.artifact_id || "-"}
                  testId="backtest-detail-manifest-core-ir-artifact"
                />
                <div className="mini-list" data-testid="backtest-detail-governance-card">
                  <div className="mini-list-title">{t("治理身份")}</div>
                  {governanceRows.map((row) => (
                    <MetricPair
                      key={row.key}
                      label={row.label}
                      value={row.value}
                      fullValue={row.fullValue}
                      testId={`backtest-detail-governance-${row.key}`}
                    />
                  ))}
                </div>
              </div>

              <div className="open-orders-card" data-testid="backtest-detail-metrics-card">
                <div className="open-orders-header">
                  <div>
                    <div className="mini-list-title">{t("指标工件")}</div>
                    <div className="muted-line">
                      {t("当前策略实验的持久化结果摘要。")}
                    </div>
                  </div>
                  <strong>{runtime.backtestArtifacts?.metrics?.artifact_id || "-"}</strong>
                </div>
                <MetricPair label={t("开始时间")} value={formatTime(startedAt)} />
                <MetricPair label={t("结束时间")} value={formatTime(endedAt)} />
                <MetricPair
                  label={t("步数")}
                  value={formatValue(summary?.step_count)}
                  testId="backtest-detail-metrics-step-count"
                />
                <MetricPair label={t("会话数")} value={formatValue(metrics?.session_count)} />
                <MetricPair
                  label={t("事件数")}
                  value={formatValue(metrics?.event_count || runtime.events.length)}
                  testId="backtest-detail-metrics-event-count"
                />
              </div>
            </div>
          </AnalysisSection>

          <AnalysisSection
            testId="backtest-detail-governed-timeline"
            kicker={t("证据链")}
            title={t("治理时间轴")}
            summary={t("按 envelope 阶段、保留级别和模块查看回测证据，并优先保留关键事件。")}
          >
            <GovernedTimelinePanel
              source={timelineSource}
              title={t("回测证据时间轴")}
              summary={t("同一 timeline item 同时服务详情、回放、压缩证据和后续报告输入。")}
              testId="backtest-detail-timeline"
            />
          </AnalysisSection>

          <AnalysisSection
            testId="backtest-detail-report-lifecycle"
            kicker={t("报告生命周期")}
            title={t("证据报告")}
            summary={t("从压缩证据生成可导出的报告，报告只链接来源证据和治理身份，不复制完整原始日志。")}
          >
            <RuntimeReportPanel
              sourceKind="backtest"
              sourceId={runtime.selectedBacktestId || backtestId}
              evidenceSource={timelineSource}
              title={t("回测证据报告")}
              summary={t("生成、打开和导出当前回测的治理报告。")}
            />
          </AnalysisSection>

          <AnalysisSection
            testId="backtest-detail-replay-preview"
            kicker={t("回放预览")}
            title={t("权益曲线与成交样本")}
            summary={t("详情页只保留高信号的回放切片，让它保持为策略分析视图，而不是原始日志堆叠。")}
          >
            <div className="analysis-card-grid analysis-card-grid--two">
              <div className="open-orders-card" data-testid="backtest-detail-equity-card">
                <div className="open-orders-header">
                  <div>
                    <div className="mini-list-title">{t("权益曲线工件")}</div>
                    <div className="muted-line">
                      {t("预览曲线首尾片段，以便快速确认策略层面的权益表现。")}
                    </div>
                  </div>
                  <strong>{runtime.backtestArtifacts?.equity_curve?.artifact_id || "-"}</strong>
                </div>
                {curvePreview.length === 0 ? (
                  <div className="muted-line">{t("这次回测没有可用的权益曲线样本。")}</div>
                ) : null}
                {curvePreview.map((point, index) => (
                  <div key={`${point.ts_ms}_${index}`} className="open-order-item">
                    <div className="open-order-topline">
                      <strong>{formatTime(point.ts_ms)}</strong>
                    </div>
                    <div className="open-order-grid">
                      <div>
                        <span>{t("权益")}</span>
                        <strong>{formatValue(point.equity)}</strong>
                      </div>
                      <div>
                        <span>{t("现金")}</span>
                        <strong>{formatValue(point.cash_balance)}</strong>
                      </div>
                      <div>
                        <span>{t("净名义价值")}</span>
                        <strong>{formatValue(point.net_notional)}</strong>
                      </div>
                    </div>
                  </div>
                ))}
              </div>

              <div className="open-orders-card" data-testid="backtest-detail-trade-card">
                <div className="open-orders-header">
                  <div>
                    <div className="mini-list-title">{t("成交账本工件")}</div>
                    <div className="muted-line">
                      {t("抽样展示已执行成交，便于审计与回放交叉核验。")}
                    </div>
                  </div>
                  <strong>{runtime.backtestArtifacts?.trade_ledger?.artifact_id || "-"}</strong>
                </div>
                {tradePreview.length === 0 ? (
                  <div className="muted-line">{t("这次回测没有记录成交。")}</div>
                ) : null}
                {tradePreview.map((trade) => (
                  <div key={trade.fill_id} className="open-order-item">
                    <div className="open-order-topline">
                      <strong>{trade.fill_id}</strong>
                      <span>{trade.cycle_name}</span>
                    </div>
                    <div className="open-order-grid">
                      <div>
                        <span>{t("方向")}</span>
                        <strong>{trade.side}</strong>
                      </div>
                      <div>
                        <span>{t("数量")}</span>
                        <strong>{formatValue(trade.filled_qty)}</strong>
                      </div>
                      <div>
                        <span>{t("价格")}</span>
                        <strong>{formatValue(trade.filled_price)}</strong>
                      </div>
                      <div>
                        <span>{t("手续费")}</span>
                        <strong>{formatValue(trade.fee_paid)}</strong>
                      </div>
                    </div>
                  </div>
                ))}
              </div>
            </div>
          </AnalysisSection>

          <AnalysisSection
            testId="backtest-detail-output-artifacts"
            kicker={t("输出引用")}
            title={t("持久化输出文件")}
            summary={t("保留文件级可追溯性，但不把页面变成纯存储列表。")}
          >
            <div className="open-orders-card" data-testid="backtest-detail-output-card">
              <div className="open-orders-header">
                <div>
                  <div className="mini-list-title">{t("输出文件")}</div>
                  <div className="muted-line">{t("记录在当前策略实验 manifest 下的文件列表。")}</div>
                </div>
                <strong>{outputArtifacts.length}</strong>
              </div>
              {outputArtifacts.length === 0 ? (
                <div className="muted-line">{t("这次回测没有记录任何输出文件引用。")}</div>
              ) : null}
              {outputArtifacts.map((artifact) => (
                <MetricPair key={artifact.artifact_id} label={artifact.kind} value={artifact.file_name} />
              ))}
            </div>
          </AnalysisSection>

          <AnalysisSection
            testId="backtest-detail-explanations"
            kicker={t("执行解释")}
            title={t("风控与订单解释")}
            summary={t("复用同一套 runtime_diagnostics explanation rows，在详情页直接展示风控裁剪和订单执行语义。")}
          >
            <div className="analysis-card-grid analysis-card-grid--two">
              <ExplanationDetailCard
                title={t("风控详情")}
                summary={t("选取 detail payload 中已结构化的 risk_detail_rows，不再重新拼第二套解释协议。")}
                entries={riskExplanationEntries}
                testId="backtest-detail-risk-card"
                emptyText={t("当前回测详情还没有可展示的风控解释。")}
              />
              <ExplanationDetailCard
                title={t("订单详情")}
                summary={t("沿用同一 explanation rows 展示下单来源、生命周期和订单语义。")}
                entries={orderExplanationEntries}
                testId="backtest-detail-order-card"
                emptyText={t("当前回测详情还没有可展示的订单解释。")}
              />
            </div>
          </AnalysisSection>
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
    </div>
  );
}
