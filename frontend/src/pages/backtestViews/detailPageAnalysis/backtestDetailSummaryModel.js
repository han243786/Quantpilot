import {
  benchmarkComparisonFromSummary,
  drawdownAnalysisFromSummary,
  formatAnnualizedReturn,
  formatDays,
  formatProfitFactor,
  formatRatio,
  formatSharpeRatio,
  formatValue,
  maxDrawdownFromSummary,
  profitFactorColor,
  riskAdjustedFromSummary,
  sharpeColor,
  tradeAnalysisFromSummary
} from "../shared";

export function buildBacktestDetailSummaryModel({
  t = (value) => value,
  summary = null,
  metrics = null,
  manifest = null,
  selectedSummary = null,
  trades = [],
  summaryExpanded = false
}) {
  const riskAdj = riskAdjustedFromSummary(summary);
  const tradeAnaly = tradeAnalysisFromSummary(summary);
  const drawdownAnaly = drawdownAnalysisFromSummary(summary);
  const benchComp = benchmarkComparisonFromSummary(summary);
  const tradeCount = Array.isArray(trades) ? trades.length : 0;

  const visibleSummaryItems = [
    {
      label: t("收益"),
      value: formatRatio(summary?.total_return_ratio)
    },
    {
      label: t("年化收益"),
      value: formatAnnualizedReturn(summary?.annualized_return)
    },
    {
      label: t("夏普"),
      value: formatSharpeRatio(riskAdj.sharpe_ratio),
      color: sharpeColor(riskAdj.sharpe_ratio),
      tooltip: t("风险调整后收益。>1 良好, >2 优秀, <0 表示收益低于无风险利率")
    },
    {
      label: t("最大回撤"),
      value: formatRatio(maxDrawdownFromSummary(summary))
    },
    {
      label: t("盈亏比"),
      value: formatProfitFactor(tradeAnaly.profit_factor),
      color: profitFactorColor(tradeAnaly.profit_factor)
    }
  ];

  const foldedSummaryItems = [
    {
      label: t("索提诺"),
      value: formatSharpeRatio(riskAdj.sortino_ratio),
      tooltip: t("下行风险调整收益。仅惩罚负波动, 更适合评估下跌风险")
    },
    {
      label: t("卡尔玛"),
      value: formatSharpeRatio(riskAdj.calmar_ratio),
      tooltip: t("年化收益÷最大回撤。衡量每单位最大亏损能产生多少收益")
    },
    {
      label: t("年化波动率"),
      value: formatAnnualizedReturn(summary?.annualized_volatility)
    },
    {
      label: t("最大回撤持续"),
      value: formatDays(drawdownAnaly.max_drawdown_duration_days)
    },
    benchComp
      ? {
          label: "Alpha",
          value: formatRatio(benchComp.alpha)
        }
      : null,
    benchComp
      ? {
          label: "Beta",
          value: benchComp.beta?.toFixed(2) ?? "-"
        }
      : null,
    {
      label: t("胜率"),
      value:
        summary?.win_rate != null && Number.isFinite(summary.win_rate)
          ? `${(summary.win_rate * 100).toFixed(1)}%`
          : "-"
    },
    {
      label: t("成交数"),
      value: formatValue(summary?.trade_count ?? tradeCount)
    },
    {
      label: t("协议"),
      value: manifest?.protocol_name || selectedSummary?.protocol_name || "-"
    },
    {
      label: t("最终权益"),
      value: formatValue(summary?.final_equity || metrics?.final_account?.equity_estimate)
    }
  ].filter(Boolean);

  return {
    visibleSummaryItems,
    foldedSummaryItems,
    summaryItems: summaryExpanded
      ? [...visibleSummaryItems, ...foldedSummaryItems]
      : visibleSummaryItems
  };
}
