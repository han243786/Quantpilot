import { useI18n } from "../../../i18n";
import {
  AnalysisSection,
  comparisonMetrics,
  datasetLabelsFromDetail,
  executionAssumptionsLabelFromDetail,
  formatAnnualizedReturn,
  formatProfitFactor,
  formatRatio,
  formatSharpeRatio,
  formatTime,
  formatValue,
  maxDrawdownFromSummary,
  MetricPair
} from "../shared";

export function buildBacktestCompareCardModel(detail = {}) {
  const metrics = comparisonMetrics(detail);
  const summaryMetrics = metrics?.summary || {};

  return {
    backtestId: detail.backtest_id || "",
    graphId: detail.graph_id || "",
    compileId: detail.compile_id || "",
    metrics,
    summaryMetrics,
    datasetLabel: datasetLabelsFromDetail(detail).join(", ") || "-",
    executionAssumptionsLabel: executionAssumptionsLabelFromDetail(detail),
    replaySource: detail.backtest_artifacts?.manifest?.backtest_spec?.replay_source || "-"
  };
}

export function buildBacktestCompareSummaryCardModel({
  details = [],
  resolvedStrategyId = "",
  summary = null
} = {}) {
  return {
    strategyId: resolvedStrategyId || "-",
    returnDelta: summary?.returnDelta,
    drawdownDelta: summary?.drawdownDelta,
    tradeDelta: summary?.tradeDelta,
    comparedBacktests: details.map((detail) => detail.backtest_id).join(" vs ")
  };
}

export function BacktestCompareCardsSection({
  details = [],
  onOpenDetail = () => {}
}) {
  const { t } = useI18n();

  return (
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
          {details.map((detail) => {
            const card = buildBacktestCompareCardModel(detail);

            return (
              <div
                key={card.backtestId}
                className="open-orders-card analysis-compare-card"
                data-testid={`backtest-compare-card-${card.backtestId}`}
              >
                <div className="open-orders-header">
                  <div>
                    <div className="mini-list-title">{card.backtestId}</div>
                    <div className="muted-line">
                      {t("策略")}：{card.graphId} | {t("编译")}：{card.compileId}
                    </div>
                  </div>
                  <button
                    className="ad-btn ad-btn--ghost compact-btn"
                    data-testid={`backtest-compare-open-detail-${card.backtestId}`}
                    onClick={() => onOpenDetail(card.backtestId)}
                  >
                    {t("打开详情")}
                  </button>
                </div>
                <div className="account-metric-grid">
                  <div className="account-metric-card">
                    <span>{t("收益")}</span>
                    <strong>{formatRatio(card.summaryMetrics.total_return_ratio)}</strong>
                  </div>
                  <div className="account-metric-card">
                    <span>{t("夏普")}</span>
                    <strong>{formatSharpeRatio(card.summaryMetrics.risk_adjusted?.sharpe_ratio)}</strong>
                  </div>
                  <div className="account-metric-card">
                    <span>{t("最大回撤")}</span>
                    <strong>{formatRatio(maxDrawdownFromSummary(card.summaryMetrics))}</strong>
                  </div>
                  <div className="account-metric-card">
                    <span>{t("盈亏比")}</span>
                    <strong>{formatProfitFactor(card.summaryMetrics.trade_analysis?.profit_factor)}</strong>
                  </div>
                  <div className="account-metric-card">
                    <span>{t("成交数")}</span>
                    <strong>{formatValue(card.summaryMetrics.trade_count)}</strong>
                  </div>
                  <div className="account-metric-card">
                    <span>{t("最终权益")}</span>
                    <strong>{formatValue(card.summaryMetrics.final_equity)}</strong>
                  </div>
                </div>
                <MetricPair label={t("年化收益")} value={formatAnnualizedReturn(card.summaryMetrics.annualized_return)} />
                <MetricPair label={t("索提诺")} value={formatSharpeRatio(card.summaryMetrics.risk_adjusted?.sortino_ratio)} />
                <MetricPair label={t("卡尔玛")} value={formatSharpeRatio(card.summaryMetrics.risk_adjusted?.calmar_ratio)} />
                <MetricPair label={t("回放来源")} value={card.replaySource} />
                <MetricPair label={t("开始时间")} value={formatTime(card.metrics?.started_at_ms)} />
                <MetricPair label={t("结束时间")} value={formatTime(card.metrics?.ended_at_ms)} />
                <MetricPair label={t("数据集")} value={card.datasetLabel} />
                <MetricPair label={t("执行假设")} value={card.executionAssumptionsLabel} />
              </div>
            );
          })}
        </div>
      </AnalysisSection>
    </div>
  );
}

export function BacktestCompareSummarySidebar({
  details = [],
  resolvedStrategyId = "",
  summary = null
}) {
  const { t } = useI18n();
  const model = buildBacktestCompareSummaryCardModel({
    details,
    resolvedStrategyId,
    summary
  });

  return (
    <div className="analysis-sidebar-column">
      <AnalysisSection
        kicker={t("策略上下文")}
        title={t("对比摘要")}
        summary={t("在对比页、详情页和工作区之间切换时，持续保留差值视图与对比范围。")}
      >
        <div className="open-orders-card" data-testid="backtest-compare-summary-card">
          <MetricPair label={t("策略 ID")} value={model.strategyId} />
          <MetricPair label={t("收益差值")} value={formatRatio(model.returnDelta)} />
          <MetricPair label={t("回撤差值")} value={formatRatio(model.drawdownDelta)} />
          <MetricPair label={t("成交差值")} value={formatValue(model.tradeDelta)} />
          <MetricPair label={t("已对比回测")} value={model.comparedBacktests} />
        </div>
      </AnalysisSection>
    </div>
  );
}
