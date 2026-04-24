use super::*;

#[derive(Debug, Deserialize)]
pub(super) struct BacktestCompareRequest {
    pub(super) backtest_ids: Vec<String>,
}

#[derive(Debug, Serialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(super) enum BacktestCompareStatus {
    Same,
    Different,
    Missing,
}

#[derive(Debug, Serialize, Clone, PartialEq)]
pub(super) struct BacktestExecutionAssumptionsCompareBlock {
    pub(super) status: BacktestCompareStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) left: Option<ExecutionAssumptionsModule>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) right: Option<ExecutionAssumptionsModule>,
    pub(super) fields: BacktestExecutionAssumptionsFieldDiffs,
}

#[derive(Debug, Serialize, Clone, PartialEq, Eq)]
pub(super) struct BacktestExecutionAssumptionsFieldDiff {
    pub(super) status: BacktestCompareStatus,
}

#[derive(Debug, Serialize, Clone, PartialEq, Eq)]
pub(super) struct BacktestExecutionAssumptionsFieldDiffs {
    pub(super) fee_bps: BacktestExecutionAssumptionsFieldDiff,
    pub(super) slippage_bps: BacktestExecutionAssumptionsFieldDiff,
    pub(super) latency_ms: BacktestExecutionAssumptionsFieldDiff,
    pub(super) sources: BacktestExecutionAssumptionsFieldDiff,
}

#[derive(Debug, Serialize)]
pub(super) struct BacktestCompareResponse {
    pub(super) left_backtest_id: String,
    pub(super) right_backtest_id: String,
    pub(super) execution_assumptions: BacktestExecutionAssumptionsCompareBlock,
    pub(super) metrics: BacktestMetricsCompareBlock,
    pub(super) trade_ledger: BacktestTradeLedgerCompareBlock,
    pub(super) equity_curve: BacktestEquityCurveCompareBlock,
    pub(super) report_narrative: BacktestReportNarrativeCompareBlock,
    pub(super) compare_report: BacktestCompareReportView,
}

#[derive(Debug, Serialize, Clone)]
pub(super) struct BacktestMetricsCompareBlock {
    pub(super) status: BacktestCompareStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) left: Option<qrpc_core::BacktestSummary>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) right: Option<qrpc_core::BacktestSummary>,
    pub(super) fields: BacktestMetricsFieldDiffs,
    pub(super) drilldown: BacktestMetricsDrilldownCompare,
}

#[derive(Debug, Serialize, Clone, PartialEq, Eq)]
pub(super) struct BacktestMetricsFieldDiff {
    pub(super) status: BacktestCompareStatus,
}

#[derive(Debug, Serialize, Clone, PartialEq, Eq)]
pub(super) struct BacktestMetricsFieldDiffs {
    pub(super) step_count: BacktestMetricsFieldDiff,
    pub(super) trade_count: BacktestMetricsFieldDiff,
    pub(super) total_return_ratio: BacktestMetricsFieldDiff,
    pub(super) max_drawdown_ratio: BacktestMetricsFieldDiff,
    pub(super) final_equity: BacktestMetricsFieldDiff,
    pub(super) net_profit: BacktestMetricsFieldDiff,
    pub(super) turnover_ratio: BacktestMetricsFieldDiff,
    pub(super) average_trade_notional: BacktestMetricsFieldDiff,
    pub(super) fee_drag_ratio: BacktestMetricsFieldDiff,
}

#[derive(Debug, Serialize, Clone, PartialEq, Eq)]
pub(super) struct BacktestMetricsDrilldownCompare {
    pub(super) performance: BacktestMetricsDrilldownGroupCompare,
    pub(super) activity: BacktestMetricsDrilldownGroupCompare,
    pub(super) costs: BacktestMetricsDrilldownGroupCompare,
}

#[derive(Debug, Serialize, Clone, PartialEq, Eq)]
pub(super) struct BacktestMetricsDrilldownGroupCompare {
    pub(super) status: BacktestCompareStatus,
    pub(super) fields: Vec<BacktestMetricsDrilldownFieldCompare>,
}

#[derive(Debug, Serialize, Clone, PartialEq, Eq)]
pub(super) struct BacktestMetricsDrilldownFieldCompare {
    pub(super) key: String,
    pub(super) status: BacktestCompareStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) left_value: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) right_value: Option<String>,
}

#[derive(Debug, Serialize, Clone, PartialEq)]
pub(super) struct BacktestTradeLedgerCompareBlock {
    pub(super) status: BacktestCompareStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) left: Option<backtest_artifacts::TradeLedgerSummary>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) right: Option<backtest_artifacts::TradeLedgerSummary>,
    pub(super) fields: BacktestTradeLedgerFieldDiffs,
}

#[derive(Debug, Serialize, Clone, PartialEq, Eq)]
pub(super) struct BacktestTradeLedgerFieldDiff {
    pub(super) status: BacktestCompareStatus,
}

#[derive(Debug, Serialize, Clone, PartialEq, Eq)]
pub(super) struct BacktestTradeLedgerFieldDiffs {
    pub(super) trade_count: BacktestTradeLedgerFieldDiff,
    pub(super) buy_fill_count: BacktestTradeLedgerFieldDiff,
    pub(super) sell_fill_count: BacktestTradeLedgerFieldDiff,
    pub(super) total_fees_paid: BacktestTradeLedgerFieldDiff,
    pub(super) buy_fees_paid: BacktestTradeLedgerFieldDiff,
    pub(super) sell_fees_paid: BacktestTradeLedgerFieldDiff,
    pub(super) total_filled_notional: BacktestTradeLedgerFieldDiff,
    pub(super) buy_filled_notional: BacktestTradeLedgerFieldDiff,
    pub(super) sell_filled_notional: BacktestTradeLedgerFieldDiff,
    pub(super) average_fill_price: BacktestTradeLedgerFieldDiff,
    pub(super) average_buy_fill_price: BacktestTradeLedgerFieldDiff,
    pub(super) average_sell_fill_price: BacktestTradeLedgerFieldDiff,
    pub(super) average_fee_per_fill: BacktestTradeLedgerFieldDiff,
    pub(super) average_buy_fee: BacktestTradeLedgerFieldDiff,
    pub(super) average_sell_fee: BacktestTradeLedgerFieldDiff,
}

#[derive(Debug, Serialize, Clone, PartialEq)]
pub(super) struct BacktestEquityCurveCompareBlock {
    pub(super) status: BacktestCompareStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) left: Option<BacktestEquityCurveSummary>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) right: Option<BacktestEquityCurveSummary>,
    pub(super) fields: BacktestEquityCurveFieldDiffs,
    pub(super) drilldown: BacktestEquityCurveDrilldown,
}

#[derive(Debug, Serialize, Clone, PartialEq)]
pub(super) struct BacktestEquityCurveSummary {
    pub(super) point_count: usize,
    pub(super) started_at_ms: u64,
    pub(super) ended_at_ms: u64,
    pub(super) first_equity: f64,
    pub(super) final_equity: f64,
    pub(super) min_equity: f64,
    pub(super) max_equity: f64,
}

#[derive(Debug, Serialize, Clone, PartialEq, Eq)]
pub(super) struct BacktestEquityCurveFieldDiff {
    pub(super) status: BacktestCompareStatus,
}

#[derive(Debug, Serialize, Clone, PartialEq, Eq)]
pub(super) struct BacktestEquityCurveFieldDiffs {
    pub(super) point_count: BacktestEquityCurveFieldDiff,
    pub(super) started_at_ms: BacktestEquityCurveFieldDiff,
    pub(super) ended_at_ms: BacktestEquityCurveFieldDiff,
    pub(super) first_equity: BacktestEquityCurveFieldDiff,
    pub(super) final_equity: BacktestEquityCurveFieldDiff,
    pub(super) min_equity: BacktestEquityCurveFieldDiff,
    pub(super) max_equity: BacktestEquityCurveFieldDiff,
}

#[derive(Debug, Serialize, Clone, PartialEq)]
pub(super) struct BacktestEquityCurveDrilldown {
    pub(super) samples: Vec<BacktestEquityCurveSampleCompare>,
}

#[derive(Debug, Serialize, Clone, PartialEq)]
pub(super) struct BacktestEquityCurveSampleCompare {
    pub(super) key: String,
    pub(super) status: BacktestCompareStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) left: Option<BacktestEquityCurveSampleValue>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) right: Option<BacktestEquityCurveSampleValue>,
}

#[derive(Debug, Serialize, Clone, PartialEq)]
pub(super) struct BacktestEquityCurveSampleValue {
    pub(super) ts_ms: u64,
    pub(super) equity: f64,
    pub(super) cash_balance: f64,
    pub(super) net_notional: f64,
}

#[derive(Debug, Serialize, Clone, PartialEq, Eq)]
pub(super) struct BacktestReportNarrativeCompareBlock {
    pub(super) status: BacktestCompareStatus,
    pub(super) headline: String,
    pub(super) bullets: Vec<String>,
    pub(super) highlights: Vec<String>,
    pub(super) source_explanations: Vec<String>,
    pub(super) sections: Vec<BacktestReportNarrativeSection>,
}

#[derive(Debug, Serialize, Clone, PartialEq, Eq)]
pub(super) struct BacktestReportNarrativeSection {
    pub(super) title: String,
    pub(super) status: BacktestCompareStatus,
    pub(super) summary: String,
    pub(super) lines: Vec<String>,
}

#[derive(Debug, Serialize, Clone, PartialEq)]
pub(super) struct BacktestCompareReportView {
    pub(super) status: BacktestCompareStatus,
    pub(super) headline: String,
    pub(super) overview: BacktestCompareReportOverview,
    pub(super) modules: BacktestCompareReportModules,
}

#[derive(Debug, Serialize, Clone, PartialEq)]
pub(super) struct BacktestCompareReportOverview {
    pub(super) bullets: Vec<String>,
    pub(super) highlights: Vec<String>,
}

#[derive(Debug, Serialize, Clone, PartialEq)]
pub(super) struct BacktestCompareReportModules {
    pub(super) execution_assumptions: BacktestCompareExecutionAssumptionsReportModule,
    pub(super) metrics: BacktestCompareMetricsReportModule,
    pub(super) trade_ledger: BacktestCompareTradeLedgerReportModule,
    pub(super) equity_curve: BacktestCompareEquityCurveReportModule,
}

#[derive(Debug, Serialize, Clone, PartialEq)]
pub(super) struct BacktestCompareExecutionAssumptionsReportModule {
    pub(super) status: BacktestCompareStatus,
    pub(super) summary: String,
    pub(super) lines: Vec<String>,
    pub(super) source_explanations: Vec<String>,
}

#[derive(Debug, Serialize, Clone, PartialEq)]
pub(super) struct BacktestCompareMetricsReportModule {
    pub(super) status: BacktestCompareStatus,
    pub(super) summary: String,
    pub(super) lines: Vec<String>,
    pub(super) drilldown: BacktestMetricsDrilldownCompare,
}

#[derive(Debug, Serialize, Clone, PartialEq)]
pub(super) struct BacktestCompareTradeLedgerReportModule {
    pub(super) status: BacktestCompareStatus,
    pub(super) summary: String,
    pub(super) lines: Vec<String>,
}

#[derive(Debug, Serialize, Clone, PartialEq)]
pub(super) struct BacktestCompareEquityCurveReportModule {
    pub(super) status: BacktestCompareStatus,
    pub(super) summary: String,
    pub(super) lines: Vec<String>,
    pub(super) drilldown: BacktestEquityCurveDrilldown,
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct BacktestCompareReportBundle {
    pub(super) source_explanations: Vec<String>,
    pub(super) assumptions_section: BacktestReportNarrativeSection,
    pub(super) metrics_section: BacktestReportNarrativeSection,
    pub(super) trade_ledger_section: BacktestReportNarrativeSection,
    pub(super) equity_curve_section: BacktestReportNarrativeSection,
}

