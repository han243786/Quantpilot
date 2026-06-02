import { useEffect, useMemo, useState } from "react";
import { fetchJson } from "../store/graphStore";
import {
  backtestDetailPath,
  navigateTo,
  strategiesPath,
  strategyBacktestsPath,
  strategyWorkspacePath
} from "../router";
import { useI18n } from "../i18n";
import {
  AnalysisHero,
  AnalysisSection,
  AnalysisStatusBanner,
  comparisonMetrics,
  datasetLabelsFromDetail,
  executionAssumptionsLabelFromDetail,
  formatRatio,
  formatSharpeRatio,
  formatProfitFactor,
  formatAnnualizedReturn,
  formatTime,
  formatValue,
  maxDrawdownFromSummary,
  MetricPair
} from "./backtestViews/shared";
import {
  BacktestCompareEquityOverlayChart,
  buildBacktestCompareMeta,
  buildBacktestCompareSummary,
  buildBacktestCompareSummaryItems,
  normalizeCompareBacktestIds,
  resolveBacktestCompareStrategyId
} from "./backtestViews/comparePageAnalysis";

export default function BacktestComparePage({ backtestIds = [], strategyId = "" }) {
  const { t } = useI18n();
  const [reloadTick, setReloadTick] = useState(0);
  const [state, setState] = useState({
    status: "loading",
    details: [],
    error: ""
  });

  useEffect(() => {
    let disposed = false;
    const ids = normalizeCompareBacktestIds(backtestIds);

    if (ids.length < 2) {
      setState({
        status: "error",
        details: [],
        error: t("请先选择两条回测，再打开策略对比页。")
      });
      return undefined;
    }

    setState({
      status: "loading",
      details: [],
      error: ""
    });

    void Promise.all(ids.map((backtestId) => fetchJson(`/runtime/backtests/${backtestId}`)))
      .then((details) => {
        if (disposed) return;
        setState({
          status: "ready",
          details,
          error: ""
        });
      })
      .catch((error) => {
        if (disposed) return;
        setState({
          status: "error",
          details: [],
          error: error instanceof Error ? error.message : t("加载策略对比失败。")
        });
      });

    return () => {
      disposed = true;
    };
  }, [backtestIds, reloadTick]);

  const summary = useMemo(() => buildBacktestCompareSummary(state.details), [state.details]);

  const summaryItems = useMemo(
    () => buildBacktestCompareSummaryItems({ t, summary }),
    [summary, t]
  );

  const resolvedStrategyId = useMemo(() => {
    return resolveBacktestCompareStrategyId({ strategyId, details: state.details });
  }, [state.details, strategyId]);

  const compareMeta = buildBacktestCompareMeta(backtestIds);

  return (
    <div
      className="detail-page detail-page-compare strategy-analysis-page strategy-analysis-page--compare"
      data-testid="backtest-compare-page"
    >
      <div data-testid="backtest-compare-hero">
        <AnalysisHero
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
          { label: t("对比"), current: true }
        ]}
        kicker={t("策略研究")}
        title={t("策略回测对比")}
        subtitle={t("并排审查两次持久化实验，并把策略级差值、数据集范围与返回入口收敛在同一分析视图中。")}
        meta={t("策略") + "：" + (resolvedStrategyId || "-") + " | " + t("对比") + "：" + compareMeta}
        actions={
          <div className="toolbar-group" data-testid="backtest-compare-hero-actions">
            <button
              className="ad-btn ad-btn--ghost"
              data-testid="backtest-compare-return-button"
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
                data-testid="backtest-compare-workspace-button"
                onClick={() => navigateTo(strategyWorkspacePath(resolvedStrategyId))}
              >
                {t("打开策略工作区")}
              </button>
            ) : null}
          </div>
        }
        summaryItems={summaryItems}
      />
      </div>

      {state.status === "loading" ? (
        <AnalysisStatusBanner>{t("正在加载策略对比...")}</AnalysisStatusBanner>
      ) : null}
      {state.status === "error" ? (
        <div>
          <AnalysisStatusBanner variant="error">{state.error}</AnalysisStatusBanner>
          <div style={{ display: "flex", gap: 8, marginTop: 12, justifyContent: "center" }}>
            <button className="ad-btn ad-btn--ghost" onClick={() => setReloadTick((tick) => tick + 1)}>
              {t("重试")}
            </button>
            <button className="ad-btn ad-btn--ghost" onClick={() => navigateTo(strategiesPath())}>
              {t("返回策略中心")}
            </button>
          </div>
        </div>
      ) : null}

      {state.status === "ready" ? (
        <div className="analysis-page-grid">
          <div className="analysis-main-column">
            <AnalysisSection
              kicker={t("对比卡片")}
              title={t("实验并排视图")}
              summary={t("在同一个策略范围内对齐展示收益、回撤、成交数、数据集范围与执行假设。")}
            >
              <div
                className="analysis-card-grid analysis-card-grid--two"
                data-testid="backtest-compare-card-grid"
              >
                {state.details.map((detail) => {
                  const metrics = comparisonMetrics(detail);
                  const summaryMetrics = metrics?.summary || {};
                  const datasets = datasetLabelsFromDetail(detail);
                  return (
                    <div
                      key={detail.backtest_id}
                      className="open-orders-card analysis-compare-card"
                      data-testid={`backtest-compare-card-${detail.backtest_id}`}
                    >
                      <div className="open-orders-header">
                        <div>
                          <div className="mini-list-title">{detail.backtest_id}</div>
                          <div className="muted-line">
                            {t("策略")}：{detail.graph_id} | {t("编译")}：{detail.compile_id}
                          </div>
                        </div>
                        <button
                          className="ad-btn ad-btn--ghost compact-btn"
                          data-testid={`backtest-compare-open-detail-${detail.backtest_id}`}
                          onClick={() =>
                            navigateTo(backtestDetailPath(detail.backtest_id, resolvedStrategyId))
                          }
                        >
                          {t("打开详情")}
                        </button>
                      </div>
                      <div className="account-metric-grid">
                        <div className="account-metric-card">
                          <span>{t("收益")}</span>
                          <strong>{formatRatio(summaryMetrics.total_return_ratio)}</strong>
                        </div>
                        <div className="account-metric-card">
                          <span>{t("夏普")}</span>
                          <strong>{formatSharpeRatio(summaryMetrics.risk_adjusted?.sharpe_ratio)}</strong>
                        </div>
                        <div className="account-metric-card">
                          <span>{t("最大回撤")}</span>
                          <strong>{formatRatio(maxDrawdownFromSummary(summaryMetrics))}</strong>
                        </div>
                        <div className="account-metric-card">
                          <span>{t("盈亏比")}</span>
                          <strong>{formatProfitFactor(summaryMetrics.trade_analysis?.profit_factor)}</strong>
                        </div>
                        <div className="account-metric-card">
                          <span>{t("成交数")}</span>
                          <strong>{formatValue(summaryMetrics.trade_count)}</strong>
                        </div>
                        <div className="account-metric-card">
                          <span>{t("最终权益")}</span>
                          <strong>{formatValue(summaryMetrics.final_equity)}</strong>
                        </div>
                      </div>
                      <MetricPair label={t("年化收益")} value={formatAnnualizedReturn(summaryMetrics.annualized_return)} />
                      <MetricPair label={t("索提诺")} value={formatSharpeRatio(summaryMetrics.risk_adjusted?.sortino_ratio)} />
                      <MetricPair label={t("卡尔玛")} value={formatSharpeRatio(summaryMetrics.risk_adjusted?.calmar_ratio)} />
                      <MetricPair
                        label={t("回放来源")}
                        value={detail.backtest_artifacts?.manifest?.backtest_spec?.replay_source || "-"}
                      />
                      <MetricPair label={t("开始时间")} value={formatTime(metrics?.started_at_ms)} />
                      <MetricPair label={t("结束时间")} value={formatTime(metrics?.ended_at_ms)} />
                      <MetricPair label={t("数据集")} value={datasets.join(", ") || "-"} />
                      <MetricPair
                        label={t("执行假设")}
                        value={executionAssumptionsLabelFromDetail(detail)}
                      />
                    </div>
                  );
                })}
              </div>
            </AnalysisSection>
          </div>

          {state.details.length === 2 ? (
            <AnalysisSection
              kicker={t("权益对比")}
              title={t("叠加权益曲线")}
              summary={t("两条曲线叠加在同一时间轴，A线为鼠尾草绿，B线为Adobe蓝。")}
            >
              <BacktestCompareEquityOverlayChart details={state.details} />
            </AnalysisSection>
          ) : null}

          <div className="analysis-sidebar-column">
            <AnalysisSection
              kicker={t("策略上下文")}
              title={t("对比摘要")}
              summary={t("在对比页、详情页和工作区之间切换时，持续保留差值视图与对比范围。")}
            >
              <div className="open-orders-card" data-testid="backtest-compare-summary-card">
                <MetricPair label={t("策略 ID")} value={resolvedStrategyId || "-"} />
                <MetricPair label={t("收益差值")} value={formatRatio(summary?.returnDelta)} />
                <MetricPair label={t("回撤差值")} value={formatRatio(summary?.drawdownDelta)} />
                <MetricPair label={t("成交差值")} value={formatValue(summary?.tradeDelta)} />
                <MetricPair
                  label={t("已对比回测")}
                  value={state.details.map((detail) => detail.backtest_id).join(" vs ")}
                />
              </div>
            </AnalysisSection>
          </div>
        </div>
      ) : null}
    </div>
  );
}
