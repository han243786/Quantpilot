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
  AnalysisStatusBanner
} from "./BacktestAnalysisLayout";
import {
  comparisonMetrics,
  datasetLabelsFromDetail,
  executionAssumptionsLabelFromDetail,
  formatRatio,
  formatTime,
  formatValue,
  MetricPair
} from "./backtestAnalysisShared";

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
    const ids = [...new Set((backtestIds || []).filter(Boolean))].slice(0, 2);

    if (ids.length < 2) {
      setState({
        status: "error",
        details: [],
        error: "请先选择两条回测，再打开策略对比页。"
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
          error: error instanceof Error ? error.message : "加载策略对比失败。"
        });
      });

    return () => {
      disposed = true;
    };
  }, [backtestIds, reloadTick]);

  const summary = useMemo(() => {
    if (state.details.length < 2) return null;
    const [left, right] = state.details;
    const leftSummary = comparisonMetrics(left)?.summary || {};
    const rightSummary = comparisonMetrics(right)?.summary || {};
    return {
      returnDelta:
        (leftSummary.total_return_ratio || 0) - (rightSummary.total_return_ratio || 0),
      drawdownDelta:
        (leftSummary.max_drawdown_ratio || 0) - (rightSummary.max_drawdown_ratio || 0),
      tradeDelta: (leftSummary.trade_count || 0) - (rightSummary.trade_count || 0)
    };
  }, [state.details]);

  const summaryItems = summary
    ? [
        { label: "收益差值", value: formatRatio(summary.returnDelta) },
        { label: "回撤差值", value: formatRatio(summary.drawdownDelta) },
        { label: "成交差值", value: formatValue(summary.tradeDelta) }
      ]
    : [];

  const resolvedStrategyId = useMemo(() => {
    if (strategyId) return strategyId;
    if (state.details.length !== 2) return "";
    const leftGraphId = state.details[0]?.graph_id || "";
    const rightGraphId = state.details[1]?.graph_id || "";
    return leftGraphId && leftGraphId === rightGraphId ? leftGraphId : "";
  }, [state.details, strategyId]);

  const compareMeta =
    (backtestIds || []).filter(Boolean).slice(0, 2).join(" vs ") || "-";

  return (
    <div
      className="detail-page detail-page-compare strategy-analysis-page strategy-analysis-page--compare"
      data-testid="backtest-compare-page"
    >
      <div data-testid="backtest-compare-hero">
        <AnalysisHero
        routeItems={[
          { label: "策略", onClick: () => navigateTo(strategiesPath()) },
          resolvedStrategyId
            ? {
                label: resolvedStrategyId,
                onClick: () => navigateTo(strategyWorkspacePath(resolvedStrategyId))
              }
            : null,
          resolvedStrategyId
            ? {
                label: "回测",
                onClick: () => navigateTo(strategyBacktestsPath(resolvedStrategyId))
              }
            : null,
          { label: "对比", current: true }
        ]}
        kicker="策略研究"
        title="策略回测对比"
        subtitle="并排审查两次持久化实验，并把策略级差值、数据集范围与返回入口收敛在同一分析视图中。"
        meta={`策略：${resolvedStrategyId || "-"} | 对比：${compareMeta}`}
        actions={
          <div className="toolbar-group" data-testid="backtest-compare-hero-actions">
            <button
              className="ghost-btn"
              data-testid="backtest-compare-return-button"
              onClick={() =>
                navigateTo(
                  resolvedStrategyId ? strategyBacktestsPath(resolvedStrategyId) : strategiesPath()
                )
              }
            >
              {resolvedStrategyId ? "返回策略回测页" : "返回策略列表"}
            </button>
            {resolvedStrategyId ? (
              <button
                className="ghost-btn"
                data-testid="backtest-compare-workspace-button"
                onClick={() => navigateTo(strategyWorkspacePath(resolvedStrategyId))}
              >
                打开策略工作区
              </button>
            ) : null}
          </div>
        }
        summaryItems={summaryItems}
      />
      </div>

      {state.status === "loading" ? (
        <AnalysisStatusBanner>正在加载策略对比...</AnalysisStatusBanner>
      ) : null}
      {state.status === "error" ? (
        <div>
          <AnalysisStatusBanner variant="error">{state.error}</AnalysisStatusBanner>
          <div style={{ display: "flex", gap: 8, marginTop: 12, justifyContent: "center" }}>
            <button className="ghost-btn" onClick={() => setReloadTick((t) => t + 1)}>
              {t("重试")}
            </button>
            <button className="ghost-btn" onClick={() => navigateTo(strategiesPath())}>
              {t("返回策略中心")}
            </button>
          </div>
        </div>
      ) : null}

      {state.status === "ready" ? (
        <div className="analysis-page-grid">
          <div className="analysis-main-column">
            <AnalysisSection
              kicker="对比卡片"
              title="实验并排视图"
              summary="在同一个策略范围内对齐展示收益、回撤、成交数、数据集范围与执行假设。"
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
                            策略：{detail.graph_id} | 编译：{detail.compile_id}
                          </div>
                        </div>
                        <button
                          className="ghost-btn compact-btn"
                          data-testid={`backtest-compare-open-detail-${detail.backtest_id}`}
                          onClick={() =>
                            navigateTo(backtestDetailPath(detail.backtest_id, resolvedStrategyId))
                          }
                        >
                          打开详情
                        </button>
                      </div>
                      <div className="account-metric-grid">
                        <div className="account-metric-card">
                          <span>收益</span>
                          <strong>{formatRatio(summaryMetrics.total_return_ratio)}</strong>
                        </div>
                        <div className="account-metric-card">
                          <span>最大回撤</span>
                          <strong>{formatRatio(summaryMetrics.max_drawdown_ratio)}</strong>
                        </div>
                        <div className="account-metric-card">
                          <span>成交数</span>
                          <strong>{formatValue(summaryMetrics.trade_count)}</strong>
                        </div>
                        <div className="account-metric-card">
                          <span>最终权益</span>
                          <strong>{formatValue(summaryMetrics.final_equity)}</strong>
                        </div>
                      </div>
                      <MetricPair
                        label="回放来源"
                        value={detail.backtest_artifacts?.manifest?.backtest_spec?.replay_source || "-"}
                      />
                      <MetricPair label="开始时间" value={formatTime(metrics?.started_at_ms)} />
                      <MetricPair label="结束时间" value={formatTime(metrics?.ended_at_ms)} />
                      <MetricPair label="数据集" value={datasets.join(", ") || "-"} />
                      <MetricPair
                        label="执行假设"
                        value={executionAssumptionsLabelFromDetail(detail)}
                      />
                    </div>
                  );
                })}
              </div>
            </AnalysisSection>
          </div>

          <div className="analysis-sidebar-column">
            <AnalysisSection
              kicker="策略上下文"
              title="对比摘要"
              summary="在对比页、详情页和工作区之间切换时，持续保留差值视图与对比范围。"
            >
              <div className="open-orders-card" data-testid="backtest-compare-summary-card">
                <MetricPair label="策略 ID" value={resolvedStrategyId || "-"} />
                <MetricPair label="收益差值" value={formatRatio(summary?.returnDelta)} />
                <MetricPair label="回撤差值" value={formatRatio(summary?.drawdownDelta)} />
                <MetricPair label="成交差值" value={formatValue(summary?.tradeDelta)} />
                <MetricPair
                  label="已对比回测"
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
