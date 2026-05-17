use super::*;

pub(super) fn build_report_narrative_compare_block(
    assumptions: &BacktestExecutionAssumptionsCompareBlock,
    metrics: &BacktestMetricsCompareBlock,
    trade_ledger: &BacktestTradeLedgerCompareBlock,
    equity_curve: &BacktestEquityCurveCompareBlock,
    bundle: &BacktestCompareReportBundle,
) -> BacktestReportNarrativeCompareBlock {
    let statuses = [
        assumptions.status,
        metrics.status,
        trade_ledger.status,
        equity_curve.status,
    ];
    let status = if statuses.contains(&BacktestCompareStatus::Missing) {
        BacktestCompareStatus::Missing
    } else if statuses
        .iter()
        .all(|status| *status == BacktestCompareStatus::Same)
    {
        BacktestCompareStatus::Same
    } else {
        BacktestCompareStatus::Different
    };
    let headline = match status {
        BacktestCompareStatus::Same => {
            "Compared runs share the same execution assumptions, metrics summary, trade ledger summary, and equity curve."
                .to_string()
        }
        BacktestCompareStatus::Different => {
            "Compared runs differ across one or more execution/report dimensions.".to_string()
        }
        BacktestCompareStatus::Missing => {
            "Compared runs cannot be fully compared because one or more report modules are missing."
                .to_string()
        }
    };
    let bullets = vec![
        format!(
            "Execution assumptions: {}.",
            compare_status_label(assumptions.status)
        ),
        format!("Metrics summary: {}.", compare_status_label(metrics.status)),
        format!(
            "Trade ledger summary: {}.",
            compare_status_label(trade_ledger.status)
        ),
        format!(
            "Equity curve: {}.",
            compare_status_label(equity_curve.status)
        ),
    ];
    let highlights = vec![
        summarize_assumption_highlight(assumptions),
        summarize_metrics_highlight(metrics),
        summarize_trade_ledger_highlight(trade_ledger),
        summarize_equity_curve_highlight(equity_curve),
    ];
    let source_explanations = bundle.source_explanations.clone();
    let sections = vec![
        bundle.assumptions_section.clone(),
        bundle.metrics_section.clone(),
        bundle.trade_ledger_section.clone(),
        bundle.equity_curve_section.clone(),
    ];
    BacktestReportNarrativeCompareBlock {
        status,
        headline,
        bullets,
        highlights,
        source_explanations,
        sections,
    }
}

pub(super) fn build_compare_report_view(
    metrics: &BacktestMetricsCompareBlock,
    equity_curve: &BacktestEquityCurveCompareBlock,
    narrative: &BacktestReportNarrativeCompareBlock,
    bundle: &BacktestCompareReportBundle,
) -> BacktestCompareReportView {
    BacktestCompareReportView {
        status: narrative.status,
        headline: narrative.headline.clone(),
        overview: BacktestCompareReportOverview {
            bullets: narrative.bullets.clone(),
            highlights: narrative.highlights.clone(),
        },
        modules: BacktestCompareReportModules {
            execution_assumptions: BacktestCompareExecutionAssumptionsReportModule {
                status: bundle.assumptions_section.status,
                summary: bundle.assumptions_section.summary.clone(),
                lines: bundle.assumptions_section.lines.clone(),
                source_explanations: bundle.source_explanations.clone(),
            },
            metrics: BacktestCompareMetricsReportModule {
                status: bundle.metrics_section.status,
                summary: bundle.metrics_section.summary.clone(),
                lines: bundle.metrics_section.lines.clone(),
                drilldown: metrics.drilldown.clone(),
            },
            trade_ledger: BacktestCompareTradeLedgerReportModule {
                status: bundle.trade_ledger_section.status,
                summary: bundle.trade_ledger_section.summary.clone(),
                lines: bundle.trade_ledger_section.lines.clone(),
            },
            equity_curve: BacktestCompareEquityCurveReportModule {
                status: bundle.equity_curve_section.status,
                summary: bundle.equity_curve_section.summary.clone(),
                lines: bundle.equity_curve_section.lines.clone(),
                drilldown: equity_curve.drilldown.clone(),
            },
        },
    }
}

pub(super) fn build_compare_report_bundle(
    assumptions: &BacktestExecutionAssumptionsCompareBlock,
    metrics: &BacktestMetricsCompareBlock,
    trade_ledger: &BacktestTradeLedgerCompareBlock,
    equity_curve: &BacktestEquityCurveCompareBlock,
) -> BacktestCompareReportBundle {
    BacktestCompareReportBundle {
        source_explanations: build_assumption_source_explanations(assumptions),
        assumptions_section: build_assumptions_report_section(assumptions),
        metrics_section: build_metrics_report_section(metrics),
        trade_ledger_section: build_trade_ledger_report_section(trade_ledger),
        equity_curve_section: build_equity_curve_report_section(equity_curve),
    }
}

pub(super) fn summarize_assumption_highlight(
    assumptions: &BacktestExecutionAssumptionsCompareBlock,
) -> String {
    match assumptions.status {
        BacktestCompareStatus::Same => {
            "Execution assumptions match on all tracked fields.".to_string()
        }
        BacktestCompareStatus::Missing => {
            "Execution assumptions are unavailable on one or both runs.".to_string()
        }
        BacktestCompareStatus::Different => format!(
            "Execution assumptions differ on: {}.",
            differing_assumption_fields(assumptions).join(", ")
        ),
    }
}

pub(super) fn summarize_metrics_highlight(metrics: &BacktestMetricsCompareBlock) -> String {
    match metrics.status {
        BacktestCompareStatus::Same => "Metrics summary matches on all tracked fields.".to_string(),
        BacktestCompareStatus::Missing => {
            "Metrics summary is unavailable on one or both runs.".to_string()
        }
        BacktestCompareStatus::Different => format!(
            "Metrics summary differs on: {}.",
            differing_metrics_fields(metrics).join(", ")
        ),
    }
}

pub(super) fn summarize_trade_ledger_highlight(
    trade_ledger: &BacktestTradeLedgerCompareBlock,
) -> String {
    match trade_ledger.status {
        BacktestCompareStatus::Same => {
            "Trade ledger summary matches on all tracked fields.".to_string()
        }
        BacktestCompareStatus::Missing => {
            "Trade ledger summary is unavailable on one or both runs.".to_string()
        }
        BacktestCompareStatus::Different => format!(
            "Trade ledger summary differs on: {}.",
            differing_trade_ledger_fields(trade_ledger).join(", ")
        ),
    }
}

pub(super) fn summarize_equity_curve_highlight(
    equity_curve: &BacktestEquityCurveCompareBlock,
) -> String {
    match equity_curve.status {
        BacktestCompareStatus::Same => "Equity curve matches on all tracked fields.".to_string(),
        BacktestCompareStatus::Missing => {
            "Equity curve is unavailable on one or both runs.".to_string()
        }
        BacktestCompareStatus::Different => format!(
            "Equity curve differs on: {}.",
            differing_equity_curve_fields(equity_curve).join(", ")
        ),
    }
}

pub(super) fn build_assumption_source_explanations(
    assumptions: &BacktestExecutionAssumptionsCompareBlock,
) -> Vec<String> {
    let left_sources = assumptions
        .left
        .as_ref()
        .and_then(|value| value.summary.sources.as_ref());
    let right_sources = assumptions
        .right
        .as_ref()
        .and_then(|value| value.summary.sources.as_ref());
    vec![
        compare_source_explanation(
            "Fee",
            left_sources.map(|sources| &sources.fee_bps),
            right_sources.map(|sources| &sources.fee_bps),
        ),
        compare_source_explanation(
            "Slippage",
            left_sources.map(|sources| &sources.slippage_bps),
            right_sources.map(|sources| &sources.slippage_bps),
        ),
        compare_source_explanation(
            "Latency",
            left_sources.map(|sources| &sources.latency_ms),
            right_sources.map(|sources| &sources.latency_ms),
        ),
    ]
}

pub(super) fn compare_source_explanation(
    label: &str,
    left: Option<&ExecutionAssumptionValueSource>,
    right: Option<&ExecutionAssumptionValueSource>,
) -> String {
    match (left, right) {
        (Some(left_value), Some(right_value)) if left_value == right_value => {
            format!(
                "{} source matches on both runs: {}.",
                label,
                assumption_source_friendly_label(left_value)
            )
        }
        (Some(left_value), Some(right_value)) => {
            format!(
                "{} source differs: left={}, right={}.",
                label,
                assumption_source_friendly_label(left_value),
                assumption_source_friendly_label(right_value)
            )
        }
        _ => format!("{} source is unavailable on one or both runs.", label),
    }
}

pub(super) fn assumption_source_friendly_label(
    source: &ExecutionAssumptionValueSource,
) -> &'static str {
    match source {
        ExecutionAssumptionValueSource::RequestOverride => "Request override",
        ExecutionAssumptionValueSource::ProfileDefault => "Execution profile default",
        ExecutionAssumptionValueSource::BackendFallback => "Backend fallback",
    }
}

pub(super) fn build_assumptions_report_section(
    assumptions: &BacktestExecutionAssumptionsCompareBlock,
) -> BacktestReportNarrativeSection {
    BacktestReportNarrativeSection {
        title: "Execution assumptions".to_string(),
        status: assumptions.status,
        summary: summarize_assumption_highlight(assumptions),
        lines: vec![
            format!("Status: {}.", compare_status_label(assumptions.status)),
            format!(
                "Left resolved values: {}.",
                assumptions
                    .left
                    .as_ref()
                    .map(format_execution_assumptions_module)
                    .unwrap_or_else(|| "missing".to_string())
            ),
            format!(
                "Right resolved values: {}.",
                assumptions
                    .right
                    .as_ref()
                    .map(format_execution_assumptions_module)
                    .unwrap_or_else(|| "missing".to_string())
            ),
        ],
    }
}

pub(super) fn build_metrics_report_section(
    metrics: &BacktestMetricsCompareBlock,
) -> BacktestReportNarrativeSection {
    BacktestReportNarrativeSection {
        title: "Metrics summary".to_string(),
        status: metrics.status,
        summary: summarize_metrics_highlight(metrics),
        lines: vec![
            format!("Status: {}.", compare_status_label(metrics.status)),
            format!(
                "Performance drilldown: {}.",
                summarize_metrics_drilldown_group(&metrics.drilldown.performance)
            ),
            format!(
                "Activity drilldown: {}.",
                summarize_metrics_drilldown_group(&metrics.drilldown.activity)
            ),
            format!(
                "Cost drilldown: {}.",
                summarize_metrics_drilldown_group(&metrics.drilldown.costs)
            ),
            format!(
                "Left summary: {}.",
                metrics
                    .left
                    .as_ref()
                    .map(format_metrics_summary)
                    .unwrap_or_else(|| "missing".to_string())
            ),
            format!(
                "Right summary: {}.",
                metrics
                    .right
                    .as_ref()
                    .map(format_metrics_summary)
                    .unwrap_or_else(|| "missing".to_string())
            ),
        ],
    }
}

pub(super) fn summarize_metrics_drilldown_group(
    group: &BacktestMetricsDrilldownGroupCompare,
) -> String {
    match group.status {
        BacktestCompareStatus::Same => "same".to_string(),
        BacktestCompareStatus::Missing => "missing".to_string(),
        BacktestCompareStatus::Different => format!(
            "different on {}",
            group
                .fields
                .iter()
                .filter(|field| field.status == BacktestCompareStatus::Different)
                .map(|field| field.key.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        ),
    }
}

pub(super) fn build_trade_ledger_report_section(
    trade_ledger: &BacktestTradeLedgerCompareBlock,
) -> BacktestReportNarrativeSection {
    BacktestReportNarrativeSection {
        title: "Trade ledger summary".to_string(),
        status: trade_ledger.status,
        summary: summarize_trade_ledger_highlight(trade_ledger),
        lines: vec![
            format!("Status: {}.", compare_status_label(trade_ledger.status)),
            format!(
                "Left summary: {}.",
                trade_ledger
                    .left
                    .as_ref()
                    .map(format_trade_ledger_summary)
                    .unwrap_or_else(|| "missing".to_string())
            ),
            format!(
                "Right summary: {}.",
                trade_ledger
                    .right
                    .as_ref()
                    .map(format_trade_ledger_summary)
                    .unwrap_or_else(|| "missing".to_string())
            ),
        ],
    }
}

pub(super) fn build_equity_curve_report_section(
    equity_curve: &BacktestEquityCurveCompareBlock,
) -> BacktestReportNarrativeSection {
    BacktestReportNarrativeSection {
        title: "Equity curve".to_string(),
        status: equity_curve.status,
        summary: summarize_equity_curve_highlight(equity_curve),
        lines: vec![
            format!("Status: {}.", compare_status_label(equity_curve.status)),
            format!(
                "Sample drilldown: {}.",
                summarize_equity_curve_samples(&equity_curve.drilldown)
            ),
            format!(
                "Left summary: {}.",
                equity_curve
                    .left
                    .as_ref()
                    .map(format_equity_curve_summary)
                    .unwrap_or_else(|| "missing".to_string())
            ),
            format!(
                "Right summary: {}.",
                equity_curve
                    .right
                    .as_ref()
                    .map(format_equity_curve_summary)
                    .unwrap_or_else(|| "missing".to_string())
            ),
        ],
    }
}

pub(super) fn differing_assumption_fields(
    assumptions: &BacktestExecutionAssumptionsCompareBlock,
) -> Vec<&'static str> {
    let mut fields = Vec::new();
    if assumptions.fields.fee_bps.status == BacktestCompareStatus::Different {
        fields.push("fee_bps");
    }
    if assumptions.fields.slippage_bps.status == BacktestCompareStatus::Different {
        fields.push("slippage_bps");
    }
    if assumptions.fields.latency_ms.status == BacktestCompareStatus::Different {
        fields.push("latency_ms");
    }
    if assumptions.fields.sources.status == BacktestCompareStatus::Different {
        fields.push("sources");
    }
    fields
}

pub(super) fn differing_metrics_fields(metrics: &BacktestMetricsCompareBlock) -> Vec<&'static str> {
    let mut fields = Vec::new();
    if metrics.fields.step_count.status == BacktestCompareStatus::Different {
        fields.push("step_count");
    }
    if metrics.fields.trade_count.status == BacktestCompareStatus::Different {
        fields.push("trade_count");
    }
    if metrics.fields.total_return_ratio.status == BacktestCompareStatus::Different {
        fields.push("total_return_ratio");
    }
    if metrics.fields.max_drawdown_ratio.status == BacktestCompareStatus::Different {
        fields.push("max_drawdown_ratio");
    }
    if metrics.fields.final_equity.status == BacktestCompareStatus::Different {
        fields.push("final_equity");
    }
    if metrics.fields.net_profit.status == BacktestCompareStatus::Different {
        fields.push("net_profit");
    }
    fields
}

pub(super) fn differing_trade_ledger_fields(
    trade_ledger: &BacktestTradeLedgerCompareBlock,
) -> Vec<&'static str> {
    let mut fields = Vec::new();
    if trade_ledger.fields.trade_count.status == BacktestCompareStatus::Different {
        fields.push("trade_count");
    }
    if trade_ledger.fields.buy_fill_count.status == BacktestCompareStatus::Different {
        fields.push("buy_fill_count");
    }
    if trade_ledger.fields.sell_fill_count.status == BacktestCompareStatus::Different {
        fields.push("sell_fill_count");
    }
    if trade_ledger.fields.total_fees_paid.status == BacktestCompareStatus::Different {
        fields.push("total_fees_paid");
    }
    if trade_ledger.fields.buy_fees_paid.status == BacktestCompareStatus::Different {
        fields.push("buy_fees_paid");
    }
    if trade_ledger.fields.sell_fees_paid.status == BacktestCompareStatus::Different {
        fields.push("sell_fees_paid");
    }
    if trade_ledger.fields.total_filled_notional.status == BacktestCompareStatus::Different {
        fields.push("total_filled_notional");
    }
    if trade_ledger.fields.buy_filled_notional.status == BacktestCompareStatus::Different {
        fields.push("buy_filled_notional");
    }
    if trade_ledger.fields.sell_filled_notional.status == BacktestCompareStatus::Different {
        fields.push("sell_filled_notional");
    }
    if trade_ledger.fields.average_fill_price.status == BacktestCompareStatus::Different {
        fields.push("average_fill_price");
    }
    if trade_ledger.fields.average_buy_fill_price.status == BacktestCompareStatus::Different {
        fields.push("average_buy_fill_price");
    }
    if trade_ledger.fields.average_sell_fill_price.status == BacktestCompareStatus::Different {
        fields.push("average_sell_fill_price");
    }
    if trade_ledger.fields.average_fee_per_fill.status == BacktestCompareStatus::Different {
        fields.push("average_fee_per_fill");
    }
    if trade_ledger.fields.average_buy_fee.status == BacktestCompareStatus::Different {
        fields.push("average_buy_fee");
    }
    if trade_ledger.fields.average_sell_fee.status == BacktestCompareStatus::Different {
        fields.push("average_sell_fee");
    }
    fields
}

pub(super) fn differing_equity_curve_fields(
    equity_curve: &BacktestEquityCurveCompareBlock,
) -> Vec<&'static str> {
    let mut fields = Vec::new();
    if equity_curve.fields.point_count.status == BacktestCompareStatus::Different {
        fields.push("point_count");
    }
    if equity_curve.fields.started_at_ms.status == BacktestCompareStatus::Different {
        fields.push("started_at_ms");
    }
    if equity_curve.fields.ended_at_ms.status == BacktestCompareStatus::Different {
        fields.push("ended_at_ms");
    }
    if equity_curve.fields.first_equity.status == BacktestCompareStatus::Different {
        fields.push("first_equity");
    }
    if equity_curve.fields.final_equity.status == BacktestCompareStatus::Different {
        fields.push("final_equity");
    }
    if equity_curve.fields.min_equity.status == BacktestCompareStatus::Different {
        fields.push("min_equity");
    }
    if equity_curve.fields.max_equity.status == BacktestCompareStatus::Different {
        fields.push("max_equity");
    }
    fields
}

pub(super) fn format_execution_assumptions_module(module: &ExecutionAssumptionsModule) -> String {
    format!(
        "fee_bps={}, slippage_bps={}, latency_ms={}",
        module.summary.fee_bps, module.summary.slippage_bps, module.summary.latency_ms
    )
}

pub(super) fn format_metrics_summary(summary: &qrpc_core::BacktestSummary) -> String {
    format!(
        "steps={}, trades={}, return={}, drawdown={}, final_equity={}, net_profit={}, sharpe={}, profit_factor={}",
        summary.step_count,
        summary.trade_count,
        summary.total_return_ratio,
        summary.drawdown_analysis.max_drawdown_ratio,
        summary.final_equity,
        summary.net_profit,
        summary.risk_adjusted.sharpe_ratio,
        summary.trade_analysis.profit_factor,
    )
}

pub(super) fn format_trade_ledger_summary(
    summary: &backtest_artifacts::TradeLedgerSummary,
) -> String {
    format!(
        "trade_count={}, buy_fill_count={}, sell_fill_count={}, total_fees_paid={}, buy_fees_paid={}, sell_fees_paid={}, total_filled_notional={}, buy_filled_notional={}, sell_filled_notional={}, average_fill_price={}, average_buy_fill_price={}, average_sell_fill_price={}, average_fee_per_fill={}, average_buy_fee={}, average_sell_fee={}",
        summary.trade_count,
        summary.buy_fill_count,
        summary.sell_fill_count,
        summary.total_fees_paid,
        summary.buy_fees_paid,
        summary.sell_fees_paid,
        summary.total_filled_notional,
        summary.buy_filled_notional,
        summary.sell_filled_notional,
        summary.average_fill_price,
        summary
            .average_buy_fill_price
            .map(|value| value.to_string())
            .unwrap_or_else(|| "na".to_string()),
        summary
            .average_sell_fill_price
            .map(|value| value.to_string())
            .unwrap_or_else(|| "na".to_string()),
        summary.average_fee_per_fill,
        summary
            .average_buy_fee
            .map(|value| value.to_string())
            .unwrap_or_else(|| "na".to_string()),
        summary
            .average_sell_fee
            .map(|value| value.to_string())
            .unwrap_or_else(|| "na".to_string()),
    )
}

pub(super) fn format_equity_curve_summary(summary: &BacktestEquityCurveSummary) -> String {
    format!(
        "point_count={}, started_at_ms={}, ended_at_ms={}, first_equity={}, final_equity={}, min_equity={}, max_equity={}",
        summary.point_count,
        summary.started_at_ms,
        summary.ended_at_ms,
        summary.first_equity,
        summary.final_equity,
        summary.min_equity,
        summary.max_equity,
    )
}

pub(super) fn summarize_equity_curve_samples(drilldown: &BacktestEquityCurveDrilldown) -> String {
    if drilldown
        .samples
        .iter()
        .all(|sample| sample.status == BacktestCompareStatus::Same)
    {
        "same".to_string()
    } else if drilldown
        .samples
        .iter()
        .any(|sample| sample.status == BacktestCompareStatus::Missing)
    {
        "missing".to_string()
    } else {
        format!(
            "different on {}",
            drilldown
                .samples
                .iter()
                .filter(|sample| sample.status == BacktestCompareStatus::Different)
                .map(|sample| sample.key.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
}

pub(super) fn compare_status_label(status: BacktestCompareStatus) -> &'static str {
    match status {
        BacktestCompareStatus::Same => "same",
        BacktestCompareStatus::Different => "different",
        BacktestCompareStatus::Missing => "missing",
    }
}
