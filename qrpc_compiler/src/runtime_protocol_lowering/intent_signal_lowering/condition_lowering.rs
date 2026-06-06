use qrpc_core::{IntentConfig, IntentKind};
use qrpc_core_ir::{
    indicator_threshold_compare_expr, moving_average_compare_expr, ComparisonOp, ScalarExpr,
    SpreadValueKind,
};

use super::super::lower_runtime_spread_spec;

pub(super) fn lower_runtime_intent_condition(intent: &IntentConfig) -> Option<ScalarExpr> {
    match intent.kind {
        IntentKind::LongTermBuy | IntentKind::SmaCrossover => {
            let data_id = intent.input_data_ids.first()?;
            let fast_period = intent.params.get("fast_period")?.round() as usize;
            let slow_period = intent.params.get("slow_period")?.round() as usize;
            let entry_ratio = intent
                .params
                .get("entry_ratio")
                .copied()
                .unwrap_or_default();
            if (entry_ratio - 1.0).abs() > f64::EPSILON {
                return None;
            }
            let op = decode_runtime_comparison_op(
                intent.params.get("comparison_op_code").copied(),
                ComparisonOp::Gt,
            )?;
            if !matches!(op, ComparisonOp::Gt | ComparisonOp::Gte) {
                return None;
            }
            moving_average_compare_expr(data_id.clone(), fast_period, op, slow_period)
        }
        IntentKind::LongTermSell => {
            let data_id = intent.input_data_ids.first()?;
            let fast_period = intent.params.get("lookback")?.round() as usize;
            let slow_period = intent.params.get("baseline_period")?.round() as usize;
            let threshold_ratio = intent
                .params
                .get("threshold_ratio")
                .copied()
                .unwrap_or_default();
            if (threshold_ratio - 1.0).abs() > f64::EPSILON {
                return None;
            }
            let op = decode_runtime_comparison_op(
                intent.params.get("comparison_op_code").copied(),
                ComparisonOp::Lt,
            )?;
            if !matches!(op, ComparisonOp::Lt | ComparisonOp::Lte) {
                return None;
            }
            moving_average_compare_expr(data_id.clone(), fast_period, op, slow_period)
        }
        IntentKind::Rsi => {
            let indicator_id = intent.intent_id.clone();
            let oversold = intent.params.get("oversold_threshold").copied()?;
            let overbought = intent.params.get("overbought_threshold").copied()?;
            let shape = decode_runtime_comparison_shape(
                intent.params.get("comparison_shape_code").copied(),
            )?;
            let (default_op, threshold) = if (overbought - 70.0).abs() <= f64::EPSILON
                && (oversold - 30.0).abs() > f64::EPSILON
                && matches!(shape, RuntimeComparisonShape::Buy)
            {
                (ComparisonOp::Lt, oversold)
            } else if (oversold - 30.0).abs() <= f64::EPSILON
                && (overbought - 70.0).abs() > f64::EPSILON
                && matches!(shape, RuntimeComparisonShape::Sell)
            {
                (ComparisonOp::Gt, overbought)
            } else {
                return None;
            };
            let op = decode_runtime_comparison_op(
                intent.params.get("comparison_op_code").copied(),
                default_op,
            )?;
            match op {
                ComparisonOp::Lt | ComparisonOp::Lte | ComparisonOp::Gt | ComparisonOp::Gte => {
                    indicator_threshold_compare_expr(indicator_id, op, threshold)
                }
                ComparisonOp::Eq => None,
            }
        }
        IntentKind::Momentum => {
            let indicator_id = intent.intent_id.clone();
            let shape = decode_runtime_comparison_shape(
                intent.params.get("comparison_shape_code").copied(),
            )?;
            let threshold = intent.params.get("comparison_threshold").copied()?;
            let default_op = match shape {
                RuntimeComparisonShape::Buy => ComparisonOp::Gt,
                RuntimeComparisonShape::Sell => ComparisonOp::Lt,
            };
            let op = decode_runtime_comparison_op(
                intent.params.get("comparison_op_code").copied(),
                default_op,
            )?;
            match (shape, op) {
                (RuntimeComparisonShape::Buy, ComparisonOp::Gt | ComparisonOp::Gte)
                | (RuntimeComparisonShape::Sell, ComparisonOp::Lt | ComparisonOp::Lte) => {
                    indicator_threshold_compare_expr(indicator_id, op, threshold)
                }
                _ => None,
            }
        }
        IntentKind::ZScore => {
            let indicator_id = intent.intent_id.clone();
            let shape = decode_runtime_comparison_shape(
                intent.params.get("comparison_shape_code").copied(),
            )?;
            let threshold = intent.params.get("comparison_threshold").copied()?;
            let default_op = match shape {
                RuntimeComparisonShape::Buy => ComparisonOp::Lt,
                RuntimeComparisonShape::Sell => ComparisonOp::Gt,
            };
            let op = decode_runtime_comparison_op(
                intent.params.get("comparison_op_code").copied(),
                default_op,
            )?;
            match (shape, op) {
                (RuntimeComparisonShape::Buy, ComparisonOp::Lt | ComparisonOp::Lte)
                | (RuntimeComparisonShape::Sell, ComparisonOp::Gt | ComparisonOp::Gte) => {
                    indicator_threshold_compare_expr(indicator_id, op, threshold)
                }
                _ => None,
            }
        }
        IntentKind::QuoteObserve => lower_runtime_spread_threshold_condition(intent),
        _ => None,
    }
}

fn lower_runtime_spread_threshold_condition(intent: &IntentConfig) -> Option<ScalarExpr> {
    let spread = lower_runtime_spread_spec(intent)?;
    if !matches!(spread.output, SpreadValueKind::Bps) {
        return None;
    }
    if spread.align.tolerance_ms == 0 {
        return None;
    }
    let shape =
        decode_runtime_comparison_shape(intent.params.get("comparison_shape_code").copied())?;
    if !matches!(shape, RuntimeComparisonShape::Buy) {
        return None;
    }
    let threshold = intent.params.get("comparison_threshold").copied()?;
    let op = decode_runtime_comparison_op(
        intent.params.get("comparison_op_code").copied(),
        ComparisonOp::Gt,
    )?;
    match op {
        ComparisonOp::Gt | ComparisonOp::Gte => {
            indicator_threshold_compare_expr(intent.intent_id.clone(), op, threshold)
        }
        ComparisonOp::Lt | ComparisonOp::Lte | ComparisonOp::Eq => None,
    }
}

fn decode_runtime_comparison_op(code: Option<f64>, default: ComparisonOp) -> Option<ComparisonOp> {
    match code.unwrap_or(comparison_op_code(default)).round() as i64 {
        0 => Some(ComparisonOp::Lt),
        1 => Some(ComparisonOp::Lte),
        2 => Some(ComparisonOp::Gt),
        3 => Some(ComparisonOp::Gte),
        4 => Some(ComparisonOp::Eq),
        _ => None,
    }
}

fn comparison_op_code(op: ComparisonOp) -> f64 {
    match op {
        ComparisonOp::Lt => 0.0,
        ComparisonOp::Lte => 1.0,
        ComparisonOp::Gt => 2.0,
        ComparisonOp::Gte => 3.0,
        ComparisonOp::Eq => 4.0,
    }
}

enum RuntimeComparisonShape {
    Buy,
    Sell,
}

fn decode_runtime_comparison_shape(code: Option<f64>) -> Option<RuntimeComparisonShape> {
    match code?.round() as i64 {
        1 => Some(RuntimeComparisonShape::Buy),
        2 => Some(RuntimeComparisonShape::Sell),
        _ => None,
    }
}
