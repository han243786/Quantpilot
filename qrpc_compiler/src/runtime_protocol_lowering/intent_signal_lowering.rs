mod condition_lowering;
mod fallback_description;

use anyhow::Result;
use qrpc_core::{IntentConfig, IntentKind};
use qrpc_core_ir::{
    CoreIndicatorKind, IndicatorNode, ScalarExpr, SeriesExpr, SignalKind, SignalRule,
};
use std::collections::BTreeMap;

use super::{lower_runtime_spread_spec, runtime_intent_is_spread};

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
                source: fallback_description::describe_runtime_intent_condition(intent),
            },
        ),
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
