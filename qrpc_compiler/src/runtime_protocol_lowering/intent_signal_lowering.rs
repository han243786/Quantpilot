mod condition_lowering;

use anyhow::Result;
use qrpc_core::{IntentConfig, IntentKind};
use qrpc_core_ir::{
    CoreIndicatorKind, IndicatorNode, ScalarExpr, SeriesExpr, SignalKind, SignalRule,
};
use std::collections::BTreeMap;

use super::{
    decode_align_direction, decode_series_field, decode_spread_output, lower_runtime_spread_spec,
    runtime_intent_is_spread,
};

pub(super) fn lower_runtime_intent_to_indicator(intent: &IntentConfig) -> Result<IndicatorNode> {
    let mut params = intent
        .params
        .iter()
        .map(|(key, value)| (key.clone(), serde_json::Value::from(*value)))
        .collect::<BTreeMap<_, _>>();
    if matches!(intent.kind, IntentKind::LongTermBuy) {
        params.insert(
            "intent_variant".to_string(),
            serde_json::Value::String("long_term_buy".to_string()),
        );
    }
    if matches!(intent.kind, IntentKind::LongTermSell) {
        params.insert(
            "intent_variant".to_string(),
            serde_json::Value::String("long_term_sell".to_string()),
        );
    }
    if matches!(intent.kind, IntentKind::SmaCrossover) {
        params.insert(
            "intent_variant".to_string(),
            serde_json::Value::String("sma_crossover".to_string()),
        );
    }
    Ok(IndicatorNode {
        indicator_id: intent.intent_id.clone(),
        kind: if runtime_intent_is_spread(intent) {
            CoreIndicatorKind::Spread
        } else {
            lower_runtime_intent_kind(&intent.kind)?
        },
        inputs: intent
            .input_data_ids
            .iter()
            .map(|data_id| SeriesExpr::DataRef {
                data_id: data_id.clone(),
            })
            .collect(),
        spread_spec: lower_runtime_spread_spec(intent),
        custom_expr: None,
        params,
    })
}

pub(super) fn lower_runtime_intent_to_signal_rule(intent: &IntentConfig) -> SignalRule {
    SignalRule {
        signal_id: format!("{}_signal", intent.intent_id),
        indicator_id: intent.intent_id.clone(),
        signal_kind: match intent.kind {
            IntentKind::LongTermSell => SignalKind::Short,
            IntentKind::QuoteObserve => SignalKind::Observe,
            _ => SignalKind::Long,
        },
        condition: condition_lowering::lower_runtime_intent_condition(intent).unwrap_or_else(
            || ScalarExpr::RawText {
                source: describe_runtime_intent_condition(intent),
            },
        ),
    }
}

fn describe_runtime_intent_condition(intent: &IntentConfig) -> String {
    match intent.kind {
        IntentKind::LongTermBuy | IntentKind::SmaCrossover => format!(
            "ma_cross(fast={}, slow={}, entry_ratio={})",
            intent
                .params
                .get("fast_period")
                .copied()
                .unwrap_or_default(),
            intent
                .params
                .get("slow_period")
                .copied()
                .unwrap_or_default(),
            intent
                .params
                .get("entry_ratio")
                .copied()
                .unwrap_or_default()
        ),
        IntentKind::LongTermSell => format!(
            "ma_deviation(lookback={}, baseline_period={}, threshold_ratio={})",
            intent.params.get("lookback").copied().unwrap_or_default(),
            intent
                .params
                .get("baseline_period")
                .copied()
                .unwrap_or_default(),
            intent
                .params
                .get("threshold_ratio")
                .copied()
                .unwrap_or_default()
        ),
        IntentKind::Rsi => format!(
            "rsi(period={}, oversold={}, overbought={})",
            intent.params.get("period").copied().unwrap_or_default(),
            intent
                .params
                .get("oversold_threshold")
                .copied()
                .unwrap_or_default(),
            intent
                .params
                .get("overbought_threshold")
                .copied()
                .unwrap_or_default()
        ),
        IntentKind::Macd => format!(
            "macd(fast={}, slow={}, signal={})",
            intent
                .params
                .get("fast_period")
                .copied()
                .unwrap_or_default(),
            intent
                .params
                .get("slow_period")
                .copied()
                .unwrap_or_default(),
            intent
                .params
                .get("signal_period")
                .copied()
                .unwrap_or_default()
        ),
        IntentKind::Momentum => format!(
            "momentum(lookback={}, threshold_ratio={})",
            intent.params.get("lookback").copied().unwrap_or_default(),
            intent
                .params
                .get("threshold_ratio")
                .copied()
                .unwrap_or_default()
        ),
        IntentKind::ZScore => format!(
            "zscore(window={}, entry_z={})",
            intent.params.get("window").copied().unwrap_or_default(),
            intent.params.get("entry_z").copied().unwrap_or_default()
        ),
        IntentKind::QuoteObserve => {
            if runtime_intent_is_spread(intent) {
                let field = decode_series_field(
                    intent.params.get("field_code").copied().unwrap_or_default() as u64,
                );
                let align = decode_align_direction(
                    intent
                        .params
                        .get("align_direction_code")
                        .copied()
                        .unwrap_or_default() as u64,
                );
                let output = decode_spread_output(
                    intent
                        .params
                        .get("spread_output_code")
                        .copied()
                        .unwrap_or_default() as u64,
                );
                format!(
                    "spread_observe(inputs={}, field={:?}, align={:?}, resample_ms={}, window={}, output={:?}, max_time_diff_ms={})",
                    intent.input_data_ids.len(),
                    field,
                    align,
                    intent
                        .params
                        .get("resample_period_ms")
                        .copied()
                        .unwrap_or_default()
                        .round() as u64,
                    intent
                        .params
                        .get("window_size")
                        .copied()
                        .unwrap_or_default()
                        .round() as usize,
                    output,
                    intent.params.get("max_time_diff_ms").copied().unwrap_or(5_000.0)
                )
            } else {
                "quote_observe(mid_price_delta)".to_string()
            }
        }
    }
}

fn lower_runtime_intent_kind(kind: &IntentKind) -> Result<CoreIndicatorKind> {
    match kind {
        IntentKind::LongTermBuy | IntentKind::LongTermSell | IntentKind::SmaCrossover => {
            Ok(CoreIndicatorKind::MaCross)
        }
        IntentKind::Rsi => Ok(CoreIndicatorKind::Rsi),
        IntentKind::Macd => Ok(CoreIndicatorKind::Macd),
        IntentKind::Momentum => Ok(CoreIndicatorKind::Momentum),
        IntentKind::ZScore => Ok(CoreIndicatorKind::ZScore),
        IntentKind::QuoteObserve => Ok(CoreIndicatorKind::QuoteObserve),
    }
}
