use super::backtest_compare_core::*;
use super::backtest_compare_narrative::*;
use super::*;

pub(super) async fn compare_backtests(
    user_id: auth::UserId,
    State(state): State<AppState>,
    Json(request): Json<BacktestCompareRequest>,
) -> Result<Json<BacktestCompareResponse>, (StatusCode, String)> {
    if request.backtest_ids.len() != 2 {
        return Err(json_bad_request(
            "bad_request",
            "回测比较需要恰好两个 backtest_id",
        ));
    }
    let left_backtest_id = request.backtest_ids[0].clone();
    let right_backtest_id = request.backtest_ids[1].clone();
    let left_record = load_backtest_record_from_state(&state, &user_id, &left_backtest_id).await?;
    let right_record = load_backtest_record_from_state(&state, &user_id, &right_backtest_id).await?;
    let left_execution_assumptions = left_record
        .backtest_artifacts
        .as_ref()
        .and_then(|artifacts| artifacts.metrics.execution_assumptions.clone());
    let right_execution_assumptions = right_record
        .backtest_artifacts
        .as_ref()
        .and_then(|artifacts| artifacts.metrics.execution_assumptions.clone());
    let left_metrics = left_record
        .backtest_artifacts
        .as_ref()
        .map(|artifacts| artifacts.metrics.summary.clone());
    let right_metrics = right_record
        .backtest_artifacts
        .as_ref()
        .map(|artifacts| artifacts.metrics.summary.clone());
    let left_trade_ledger = left_record
        .backtest_artifacts
        .as_ref()
        .map(|artifacts| summarize_trade_ledger(&artifacts.trade_ledger));
    let right_trade_ledger = right_record
        .backtest_artifacts
        .as_ref()
        .map(|artifacts| summarize_trade_ledger(&artifacts.trade_ledger));
    let left_equity_curve = left_record
        .backtest_artifacts
        .as_ref()
        .map(|artifacts| artifacts.equity_curve.points.clone());
    let right_equity_curve = right_record
        .backtest_artifacts
        .as_ref()
        .map(|artifacts| artifacts.equity_curve.points.clone());
    let execution_assumptions = compare_execution_assumptions_modules(
        left_execution_assumptions,
        right_execution_assumptions,
    );
    let metrics = compare_metrics_summaries(left_metrics, right_metrics);
    let trade_ledger = compare_trade_ledger_summaries(left_trade_ledger, right_trade_ledger);
    let equity_curve = compare_equity_curve_points(left_equity_curve, right_equity_curve);
    let report_bundle = build_compare_report_bundle(
        &execution_assumptions,
        &metrics,
        &trade_ledger,
        &equity_curve,
    );
    let report_narrative = build_report_narrative_compare_block(
        &execution_assumptions,
        &metrics,
        &trade_ledger,
        &equity_curve,
        &report_bundle,
    );
    let compare_report =
        build_compare_report_view(&metrics, &equity_curve, &report_narrative, &report_bundle);

    Ok(Json(BacktestCompareResponse {
        left_backtest_id,
        right_backtest_id,
        execution_assumptions,
        metrics,
        trade_ledger,
        equity_curve,
        report_narrative,
        compare_report,
    }))
}
