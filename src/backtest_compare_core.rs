use super::*;

pub(super) fn compare_execution_assumptions_modules(
    left: Option<ExecutionAssumptionsModule>,
    right: Option<ExecutionAssumptionsModule>,
) -> BacktestExecutionAssumptionsCompareBlock {
    let fields = BacktestExecutionAssumptionsFieldDiffs {
        fee_bps: compare_optional_field(
            left.as_ref().map(|value| value.summary.fee_bps),
            right.as_ref().map(|value| value.summary.fee_bps),
        ),
        slippage_bps: compare_optional_field(
            left.as_ref().map(|value| value.summary.slippage_bps),
            right.as_ref().map(|value| value.summary.slippage_bps),
        ),
        latency_ms: compare_optional_field(
            left.as_ref().map(|value| value.summary.latency_ms),
            right.as_ref().map(|value| value.summary.latency_ms),
        ),
        sources: compare_optional_field(
            left.as_ref()
                .and_then(|value| value.summary.sources.clone()),
            right
                .as_ref()
                .and_then(|value| value.summary.sources.clone()),
        ),
    };
    let status = match (&left, &right) {
        (Some(left_value), Some(right_value)) if left_value == right_value => {
            BacktestCompareStatus::Same
        }
        (Some(_), Some(_)) => BacktestCompareStatus::Different,
        _ => BacktestCompareStatus::Missing,
    };
    BacktestExecutionAssumptionsCompareBlock {
        status,
        left,
        right,
        fields,
    }
}

pub(super) fn compare_optional_field<T: PartialEq>(
    left: Option<T>,
    right: Option<T>,
) -> BacktestExecutionAssumptionsFieldDiff {
    let status = match (left, right) {
        (Some(left_value), Some(right_value)) if left_value == right_value => {
            BacktestCompareStatus::Same
        }
        (Some(_), Some(_)) => BacktestCompareStatus::Different,
        _ => BacktestCompareStatus::Missing,
    };
    BacktestExecutionAssumptionsFieldDiff { status }
}

pub(super) fn compare_metrics_summaries(
    left: Option<qrpc_core::BacktestSummary>,
    right: Option<qrpc_core::BacktestSummary>,
) -> BacktestMetricsCompareBlock {
    let fields = BacktestMetricsFieldDiffs {
        step_count: BacktestMetricsFieldDiff {
            status: compare_optional_status(
                left.as_ref().map(|value| value.step_count),
                right.as_ref().map(|value| value.step_count),
            ),
        },
        trade_count: BacktestMetricsFieldDiff {
            status: compare_optional_status(
                left.as_ref().map(|value| value.trade_count),
                right.as_ref().map(|value| value.trade_count),
            ),
        },
        total_return_ratio: BacktestMetricsFieldDiff {
            status: compare_optional_status(
                left.as_ref()
                    .map(|value| value.total_return_ratio.to_bits()),
                right
                    .as_ref()
                    .map(|value| value.total_return_ratio.to_bits()),
            ),
        },
        max_drawdown_ratio: BacktestMetricsFieldDiff {
            status: compare_optional_status(
                left.as_ref()
                    .map(|value| value.max_drawdown_ratio.to_bits()),
                right
                    .as_ref()
                    .map(|value| value.max_drawdown_ratio.to_bits()),
            ),
        },
        final_equity: BacktestMetricsFieldDiff {
            status: compare_optional_status(
                left.as_ref().map(|value| value.final_equity.to_bits()),
                right.as_ref().map(|value| value.final_equity.to_bits()),
            ),
        },
        net_profit: BacktestMetricsFieldDiff {
            status: compare_optional_status(
                left.as_ref().map(|value| value.net_profit.to_bits()),
                right.as_ref().map(|value| value.net_profit.to_bits()),
            ),
        },
        turnover_ratio: BacktestMetricsFieldDiff {
            status: compare_optional_status(
                left.as_ref().map(|value| value.turnover_ratio.to_bits()),
                right.as_ref().map(|value| value.turnover_ratio.to_bits()),
            ),
        },
        average_trade_notional: BacktestMetricsFieldDiff {
            status: compare_optional_status(
                left.as_ref()
                    .map(|value| value.average_trade_notional.to_bits()),
                right
                    .as_ref()
                    .map(|value| value.average_trade_notional.to_bits()),
            ),
        },
        fee_drag_ratio: BacktestMetricsFieldDiff {
            status: compare_optional_status(
                left.as_ref().map(|value| value.fee_drag_ratio.to_bits()),
                right.as_ref().map(|value| value.fee_drag_ratio.to_bits()),
            ),
        },
    };
    let metric_statuses = [
        fields.step_count.status,
        fields.trade_count.status,
        fields.total_return_ratio.status,
        fields.max_drawdown_ratio.status,
        fields.final_equity.status,
        fields.net_profit.status,
        fields.turnover_ratio.status,
        fields.average_trade_notional.status,
        fields.fee_drag_ratio.status,
    ];
    let status = if metric_statuses.contains(&BacktestCompareStatus::Missing) {
        BacktestCompareStatus::Missing
    } else if metric_statuses
        .iter()
        .all(|status| *status == BacktestCompareStatus::Same)
    {
        BacktestCompareStatus::Same
    } else {
        BacktestCompareStatus::Different
    };
    let drilldown = BacktestMetricsDrilldownCompare {
        performance: build_metrics_drilldown_group(&[
            (
                "total_return_ratio",
                fields.total_return_ratio.status,
                left.as_ref()
                    .map(|value| value.total_return_ratio.to_string()),
                right
                    .as_ref()
                    .map(|value| value.total_return_ratio.to_string()),
            ),
            (
                "net_profit",
                fields.net_profit.status,
                left.as_ref().map(|value| value.net_profit.to_string()),
                right.as_ref().map(|value| value.net_profit.to_string()),
            ),
            (
                "final_equity",
                fields.final_equity.status,
                left.as_ref().map(|value| value.final_equity.to_string()),
                right.as_ref().map(|value| value.final_equity.to_string()),
            ),
            (
                "max_drawdown_ratio",
                fields.max_drawdown_ratio.status,
                left.as_ref()
                    .map(|value| value.max_drawdown_ratio.to_string()),
                right
                    .as_ref()
                    .map(|value| value.max_drawdown_ratio.to_string()),
            ),
        ]),
        activity: build_metrics_drilldown_group(&[
            (
                "step_count",
                fields.step_count.status,
                left.as_ref().map(|value| value.step_count.to_string()),
                right.as_ref().map(|value| value.step_count.to_string()),
            ),
            (
                "trade_count",
                fields.trade_count.status,
                left.as_ref().map(|value| value.trade_count.to_string()),
                right.as_ref().map(|value| value.trade_count.to_string()),
            ),
            (
                "turnover_ratio",
                fields.turnover_ratio.status,
                left.as_ref().map(|value| value.turnover_ratio.to_string()),
                right.as_ref().map(|value| value.turnover_ratio.to_string()),
            ),
            (
                "average_trade_notional",
                fields.average_trade_notional.status,
                left.as_ref()
                    .map(|value| value.average_trade_notional.to_string()),
                right
                    .as_ref()
                    .map(|value| value.average_trade_notional.to_string()),
            ),
        ]),
        costs: build_metrics_drilldown_group(&[(
            "fee_drag_ratio",
            fields.fee_drag_ratio.status,
            left.as_ref().map(|value| value.fee_drag_ratio.to_string()),
            right.as_ref().map(|value| value.fee_drag_ratio.to_string()),
        )]),
    };
    BacktestMetricsCompareBlock {
        status,
        left,
        right,
        fields,
        drilldown,
    }
}

pub(super) fn build_metrics_drilldown_group(
    fields: &[(
        &'static str,
        BacktestCompareStatus,
        Option<String>,
        Option<String>,
    )],
) -> BacktestMetricsDrilldownGroupCompare {
    let status = if fields
        .iter()
        .any(|(_, status, _, _)| *status == BacktestCompareStatus::Missing)
    {
        BacktestCompareStatus::Missing
    } else if fields
        .iter()
        .all(|(_, status, _, _)| *status == BacktestCompareStatus::Same)
    {
        BacktestCompareStatus::Same
    } else {
        BacktestCompareStatus::Different
    };
    BacktestMetricsDrilldownGroupCompare {
        status,
        fields: fields
            .iter()
            .map(
                |(key, status, left_value, right_value)| BacktestMetricsDrilldownFieldCompare {
                    key: (*key).to_string(),
                    status: *status,
                    left_value: left_value.clone(),
                    right_value: right_value.clone(),
                },
            )
            .collect(),
    }
}

pub(super) fn summarize_trade_ledger(
    artifact: &backtest_artifacts::TradeLedgerArtifact,
) -> backtest_artifacts::TradeLedgerSummary {
    artifact.summary.clone().unwrap_or_else(|| {
        let buy_fills = artifact
            .trades
            .iter()
            .filter(|trade| trade.side == qrpc_core::OrderSide::Buy)
            .collect::<Vec<_>>();
        let sell_fills = artifact
            .trades
            .iter()
            .filter(|trade| trade.side == qrpc_core::OrderSide::Sell)
            .collect::<Vec<_>>();
        let total_fees_paid = artifact
            .trades
            .iter()
            .map(|trade| trade.fee_paid)
            .sum::<f64>();
        let buy_fees_paid = buy_fills.iter().map(|trade| trade.fee_paid).sum::<f64>();
        let sell_fees_paid = sell_fills.iter().map(|trade| trade.fee_paid).sum::<f64>();
        let total_filled_notional = artifact
            .trades
            .iter()
            .map(|trade| trade.filled_qty * trade.filled_price)
            .sum::<f64>();
        let buy_notional = buy_fills
            .iter()
            .map(|trade| trade.filled_qty * trade.filled_price)
            .sum::<f64>();
        let sell_notional = sell_fills
            .iter()
            .map(|trade| trade.filled_qty * trade.filled_price)
            .sum::<f64>();
        let total_qty = artifact
            .trades
            .iter()
            .map(|trade| trade.filled_qty)
            .sum::<f64>();
        let buy_qty = buy_fills.iter().map(|trade| trade.filled_qty).sum::<f64>();
        let sell_qty = sell_fills.iter().map(|trade| trade.filled_qty).sum::<f64>();

        backtest_artifacts::TradeLedgerSummary {
            trade_count: artifact.trade_count,
            buy_fill_count: buy_fills.len(),
            sell_fill_count: sell_fills.len(),
            total_fees_paid,
            buy_fees_paid,
            sell_fees_paid,
            total_filled_notional,
            buy_filled_notional: buy_notional,
            sell_filled_notional: sell_notional,
            average_fill_price: average_or_zero(total_filled_notional, total_qty),
            average_buy_fill_price: average_or_option(buy_notional, buy_qty),
            average_sell_fill_price: average_or_option(sell_notional, sell_qty),
            average_fee_per_fill: average_or_zero(total_fees_paid, artifact.trade_count as f64),
            average_buy_fee: average_or_option(buy_fees_paid, buy_fills.len() as f64),
            average_sell_fee: average_or_option(sell_fees_paid, sell_fills.len() as f64),
        }
    })
}

pub(super) fn compare_trade_ledger_summaries(
    left: Option<backtest_artifacts::TradeLedgerSummary>,
    right: Option<backtest_artifacts::TradeLedgerSummary>,
) -> BacktestTradeLedgerCompareBlock {
    let fields = BacktestTradeLedgerFieldDiffs {
        trade_count: BacktestTradeLedgerFieldDiff {
            status: compare_optional_status(
                left.as_ref().map(|value| value.trade_count),
                right.as_ref().map(|value| value.trade_count),
            ),
        },
        buy_fill_count: BacktestTradeLedgerFieldDiff {
            status: compare_optional_status(
                left.as_ref().map(|value| value.buy_fill_count),
                right.as_ref().map(|value| value.buy_fill_count),
            ),
        },
        sell_fill_count: BacktestTradeLedgerFieldDiff {
            status: compare_optional_status(
                left.as_ref().map(|value| value.sell_fill_count),
                right.as_ref().map(|value| value.sell_fill_count),
            ),
        },
        total_fees_paid: BacktestTradeLedgerFieldDiff {
            status: compare_optional_status(
                left.as_ref().map(|value| value.total_fees_paid.to_bits()),
                right.as_ref().map(|value| value.total_fees_paid.to_bits()),
            ),
        },
        buy_fees_paid: BacktestTradeLedgerFieldDiff {
            status: compare_optional_status(
                left.as_ref().map(|value| value.buy_fees_paid.to_bits()),
                right.as_ref().map(|value| value.buy_fees_paid.to_bits()),
            ),
        },
        sell_fees_paid: BacktestTradeLedgerFieldDiff {
            status: compare_optional_status(
                left.as_ref().map(|value| value.sell_fees_paid.to_bits()),
                right.as_ref().map(|value| value.sell_fees_paid.to_bits()),
            ),
        },
        total_filled_notional: BacktestTradeLedgerFieldDiff {
            status: compare_optional_status(
                left.as_ref()
                    .map(|value| value.total_filled_notional.to_bits()),
                right
                    .as_ref()
                    .map(|value| value.total_filled_notional.to_bits()),
            ),
        },
        buy_filled_notional: BacktestTradeLedgerFieldDiff {
            status: compare_optional_status(
                left.as_ref()
                    .map(|value| value.buy_filled_notional.to_bits()),
                right
                    .as_ref()
                    .map(|value| value.buy_filled_notional.to_bits()),
            ),
        },
        sell_filled_notional: BacktestTradeLedgerFieldDiff {
            status: compare_optional_status(
                left.as_ref()
                    .map(|value| value.sell_filled_notional.to_bits()),
                right
                    .as_ref()
                    .map(|value| value.sell_filled_notional.to_bits()),
            ),
        },
        average_fill_price: BacktestTradeLedgerFieldDiff {
            status: compare_optional_status(
                left.as_ref()
                    .map(|value| value.average_fill_price.to_bits()),
                right
                    .as_ref()
                    .map(|value| value.average_fill_price.to_bits()),
            ),
        },
        average_buy_fill_price: BacktestTradeLedgerFieldDiff {
            status: compare_optional_status(
                left.as_ref()
                    .and_then(|value| value.average_buy_fill_price.map(f64::to_bits)),
                right
                    .as_ref()
                    .and_then(|value| value.average_buy_fill_price.map(f64::to_bits)),
            ),
        },
        average_sell_fill_price: BacktestTradeLedgerFieldDiff {
            status: compare_optional_status(
                left.as_ref()
                    .and_then(|value| value.average_sell_fill_price.map(f64::to_bits)),
                right
                    .as_ref()
                    .and_then(|value| value.average_sell_fill_price.map(f64::to_bits)),
            ),
        },
        average_fee_per_fill: BacktestTradeLedgerFieldDiff {
            status: compare_optional_status(
                left.as_ref()
                    .map(|value| value.average_fee_per_fill.to_bits()),
                right
                    .as_ref()
                    .map(|value| value.average_fee_per_fill.to_bits()),
            ),
        },
        average_buy_fee: BacktestTradeLedgerFieldDiff {
            status: compare_optional_status(
                left.as_ref()
                    .and_then(|value| value.average_buy_fee.map(f64::to_bits)),
                right
                    .as_ref()
                    .and_then(|value| value.average_buy_fee.map(f64::to_bits)),
            ),
        },
        average_sell_fee: BacktestTradeLedgerFieldDiff {
            status: compare_optional_status(
                left.as_ref()
                    .and_then(|value| value.average_sell_fee.map(f64::to_bits)),
                right
                    .as_ref()
                    .and_then(|value| value.average_sell_fee.map(f64::to_bits)),
            ),
        },
    };
    let status = compare_optional_status(left.as_ref(), right.as_ref());
    BacktestTradeLedgerCompareBlock {
        status,
        left,
        right,
        fields,
    }
}

pub(super) fn summarize_equity_curve_points(
    points: &[qrpc_core::BacktestEquityPoint],
) -> Option<BacktestEquityCurveSummary> {
    let first = points.first()?;
    let last = points.last()?;
    let min_equity = points
        .iter()
        .map(|point| point.equity)
        .fold(f64::INFINITY, f64::min);
    let max_equity = points
        .iter()
        .map(|point| point.equity)
        .fold(f64::NEG_INFINITY, f64::max);

    Some(BacktestEquityCurveSummary {
        point_count: points.len(),
        started_at_ms: first.ts_ms,
        ended_at_ms: last.ts_ms,
        first_equity: first.equity,
        final_equity: last.equity,
        min_equity,
        max_equity,
    })
}

pub(super) fn compare_equity_curve_points(
    left: Option<Vec<qrpc_core::BacktestEquityPoint>>,
    right: Option<Vec<qrpc_core::BacktestEquityPoint>>,
) -> BacktestEquityCurveCompareBlock {
    let left_summary = left
        .as_ref()
        .and_then(|points| summarize_equity_curve_points(points));
    let right_summary = right
        .as_ref()
        .and_then(|points| summarize_equity_curve_points(points));
    let fields = BacktestEquityCurveFieldDiffs {
        point_count: BacktestEquityCurveFieldDiff {
            status: compare_optional_status(
                left_summary.as_ref().map(|value| value.point_count),
                right_summary.as_ref().map(|value| value.point_count),
            ),
        },
        started_at_ms: BacktestEquityCurveFieldDiff {
            status: compare_optional_status(
                left_summary.as_ref().map(|value| value.started_at_ms),
                right_summary.as_ref().map(|value| value.started_at_ms),
            ),
        },
        ended_at_ms: BacktestEquityCurveFieldDiff {
            status: compare_optional_status(
                left_summary.as_ref().map(|value| value.ended_at_ms),
                right_summary.as_ref().map(|value| value.ended_at_ms),
            ),
        },
        first_equity: BacktestEquityCurveFieldDiff {
            status: compare_optional_status(
                left_summary
                    .as_ref()
                    .map(|value| value.first_equity.to_bits()),
                right_summary
                    .as_ref()
                    .map(|value| value.first_equity.to_bits()),
            ),
        },
        final_equity: BacktestEquityCurveFieldDiff {
            status: compare_optional_status(
                left_summary
                    .as_ref()
                    .map(|value| value.final_equity.to_bits()),
                right_summary
                    .as_ref()
                    .map(|value| value.final_equity.to_bits()),
            ),
        },
        min_equity: BacktestEquityCurveFieldDiff {
            status: compare_optional_status(
                left_summary
                    .as_ref()
                    .map(|value| value.min_equity.to_bits()),
                right_summary
                    .as_ref()
                    .map(|value| value.min_equity.to_bits()),
            ),
        },
        max_equity: BacktestEquityCurveFieldDiff {
            status: compare_optional_status(
                left_summary
                    .as_ref()
                    .map(|value| value.max_equity.to_bits()),
                right_summary
                    .as_ref()
                    .map(|value| value.max_equity.to_bits()),
            ),
        },
    };
    let drilldown = BacktestEquityCurveDrilldown {
        samples: vec![
            compare_equity_curve_sample("start", left.as_deref(), right.as_deref(), |points| {
                points.first()
            }),
            compare_equity_curve_sample("middle", left.as_deref(), right.as_deref(), |points| {
                points.get(points.len() / 2)
            }),
            compare_equity_curve_sample("end", left.as_deref(), right.as_deref(), |points| {
                points.last()
            }),
        ],
    };
    let status = if left_summary.is_none() || right_summary.is_none() {
        BacktestCompareStatus::Missing
    } else if fields.point_count.status == BacktestCompareStatus::Same
        && fields.started_at_ms.status == BacktestCompareStatus::Same
        && fields.ended_at_ms.status == BacktestCompareStatus::Same
        && fields.first_equity.status == BacktestCompareStatus::Same
        && fields.final_equity.status == BacktestCompareStatus::Same
        && fields.min_equity.status == BacktestCompareStatus::Same
        && fields.max_equity.status == BacktestCompareStatus::Same
        && drilldown
            .samples
            .iter()
            .all(|sample| sample.status == BacktestCompareStatus::Same)
    {
        BacktestCompareStatus::Same
    } else {
        BacktestCompareStatus::Different
    };
    BacktestEquityCurveCompareBlock {
        status,
        left: left_summary,
        right: right_summary,
        fields,
        drilldown,
    }
}

pub(super) fn compare_equity_curve_sample(
    key: &str,
    left: Option<&[qrpc_core::BacktestEquityPoint]>,
    right: Option<&[qrpc_core::BacktestEquityPoint]>,
    selector: impl Fn(&[qrpc_core::BacktestEquityPoint]) -> Option<&qrpc_core::BacktestEquityPoint>,
) -> BacktestEquityCurveSampleCompare {
    let left_sample = left
        .and_then(&selector)
        .map(backtest_equity_curve_sample_value);
    let right_sample = right
        .and_then(selector)
        .map(backtest_equity_curve_sample_value);
    let status = compare_optional_status(left_sample.as_ref(), right_sample.as_ref());
    BacktestEquityCurveSampleCompare {
        key: key.to_string(),
        status,
        left: left_sample,
        right: right_sample,
    }
}

pub(super) fn backtest_equity_curve_sample_value(
    point: &qrpc_core::BacktestEquityPoint,
) -> BacktestEquityCurveSampleValue {
    BacktestEquityCurveSampleValue {
        ts_ms: point.ts_ms,
        equity: point.equity,
        cash_balance: point.cash_balance,
        net_notional: point.net_notional,
    }
}

pub(super) fn compare_optional_status<T: PartialEq>(
    left: Option<T>,
    right: Option<T>,
) -> BacktestCompareStatus {
    match (left, right) {
        (Some(left_value), Some(right_value)) if left_value == right_value => {
            BacktestCompareStatus::Same
        }
        (Some(_), Some(_)) => BacktestCompareStatus::Different,
        _ => BacktestCompareStatus::Missing,
    }
}

pub(super) fn average_or_zero(total: f64, qty: f64) -> f64 {
    average_or_option(total, qty).unwrap_or(0.0)
}

pub(super) fn average_or_option(total: f64, qty: f64) -> Option<f64> {
    if qty.abs() > f64::EPSILON {
        Some(total / qty)
    } else {
        None
    }
}
