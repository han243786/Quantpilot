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
} from "./backtestViews/shared";
import {
  BacktestCompareCardsSection,
  BacktestCompareEquityOverlayChart,
  BacktestCompareSummarySidebar,
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
          <BacktestCompareCardsSection
            details={state.details}
            onOpenDetail={(backtestId) =>
              navigateTo(backtestDetailPath(backtestId, resolvedStrategyId))
            }
          />

          {state.details.length === 2 ? (
            <AnalysisSection
              kicker={t("权益对比")}
              title={t("叠加权益曲线")}
              summary={t("两条曲线叠加在同一时间轴，A线为鼠尾草绿，B线为Adobe蓝。")}
            >
              <BacktestCompareEquityOverlayChart details={state.details} />
            </AnalysisSection>
          ) : null}

          <BacktestCompareSummarySidebar
            details={state.details}
            resolvedStrategyId={resolvedStrategyId}
            summary={summary}
          />
        </div>
      ) : null}
    </div>
  );
}
