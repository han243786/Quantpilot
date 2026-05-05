use crate::core_ir_evaluator::evaluate_indicator_signal;
use qrpc_core::{
    CoreStrategyIr, IntentKind, IntentSignal, NormalizedMarketData, RuntimeEvent, RuntimeEventType,
};
use qrpc_core_ir::{CoreIndicatorKind, IndicatorNode, SignalKind};
use serde_json::json;

#[derive(Debug, Clone)]
pub struct IntentEvaluationRequest<'a> {
    pub intent_kinds: &'a [IntentKind],
    pub core_ir: &'a CoreStrategyIr,
    pub normalized_data: &'a [NormalizedMarketData],
    pub now_ms: u64,
    pub trace_id: &'a str,
}

#[derive(Debug, Clone)]
pub struct IntentEvaluationOutput {
    pub signals: Vec<IntentSignal>,
    pub events: Vec<RuntimeEvent>,
}

pub trait IntentModuleProvider: Send + Sync {
    fn provider_key(&self) -> &'static str {
        "builtin.intent.default"
    }

    fn evaluate_intents(&self, request: IntentEvaluationRequest<'_>) -> IntentEvaluationOutput;
}

#[derive(Debug, Clone, Default)]
pub struct BuiltinIntentModule;

impl IntentModuleProvider for BuiltinIntentModule {
    fn evaluate_intents(&self, request: IntentEvaluationRequest<'_>) -> IntentEvaluationOutput {
        let mut signals = Vec::new();
        let mut events = Vec::new();

        let intents = resolved_intents(&request);
        for intent in intents {
            if let Some(signal) = evaluate_intent(
                &intent,
                request.core_ir,
                request.normalized_data,
                request.now_ms,
                request.trace_id,
            ) {
                events.push(RuntimeEvent {
                    event_id: format!(
                        "evt-intent-evaluated-{}-{}",
                        intent.intent_id, request.now_ms
                    ),
                    event_type: RuntimeEventType::IntentEvaluated,
                    trace_id: request.trace_id.to_string(),
                    source_id: intent.intent_id.clone(),
                    ts_ms: request.now_ms,
                    payload: json!({
                        "provider_key": self.provider_key(),
                        "kind": format!("{:?}", signal.kind),
                        "strength": signal.strength,
                        "confidence": signal.confidence,
                    }),
                });
                if signal.strength.abs() > 0.0 || matches!(signal.kind, IntentKind::QuoteObserve) {
                    events.push(RuntimeEvent {
                        event_id: format!(
                            "evt-intent-triggered-{}-{}",
                            signal.signal_id, request.now_ms
                        ),
                        event_type: RuntimeEventType::IntentTriggered,
                        trace_id: request.trace_id.to_string(),
                        source_id: signal.intent_id.clone(),
                        ts_ms: request.now_ms,
                        payload: json!({
                            "provider_key": self.provider_key(),
                            "side": format!("{:?}", signal.side),
                            "strength": signal.strength,
                            "confidence": signal.confidence,
                            "reference_price": signal.reference_price,
                            "reason": signal.reason,
                        }),
                    });
                }
                signals.push(signal);
            }
        }

        IntentEvaluationOutput { signals, events }
    }
}

#[derive(Debug, Clone)]
struct ResolvedIntentConfig {
    intent_id: String,
    kind: IntentKind,
}

fn resolved_intents(request: &IntentEvaluationRequest<'_>) -> Vec<ResolvedIntentConfig> {
    request
        .core_ir
        .indicators
        .iter()
        .filter_map(|indicator| {
            let kind = intent_kind_from_indicator(indicator)?;
            request
                .intent_kinds
                .contains(&kind)
                .then(|| ResolvedIntentConfig {
                    intent_id: indicator.indicator_id.clone(),
                    kind,
                })
        })
        .collect()
}

fn evaluate_intent(
    intent: &ResolvedIntentConfig,
    core_ir: &CoreStrategyIr,
    normalized_data: &[NormalizedMarketData],
    now_ms: u64,
    trace_id: &str,
) -> Option<IntentSignal> {
    let indicator = core_ir
        .indicators
        .iter()
        .find(|item| item.indicator_id == intent.intent_id)?;
    let signal_rule = core_ir
        .signal_rules
        .iter()
        .find(|rule| rule.indicator_id == indicator.indicator_id);
    let evaluation = evaluate_indicator_signal(indicator, signal_rule, normalized_data).ok()?;
    Some(IntentSignal {
        signal_id: format!("signal-{}-{now_ms}", intent.intent_id),
        intent_id: intent.intent_id.clone(),
        kind: intent.kind.clone(),
        exchange_scope: evaluation.exchange_scope,
        symbol_scope: evaluation.symbol_scope,
        side: evaluation.side,
        strength: evaluation.strength,
        confidence: evaluation.confidence,
        reference_price: evaluation.reference_price,
        derived_metrics: evaluation.derived_metrics,
        reason: evaluation.reason,
        triggered_at_ms: now_ms,
        ttl_ms: evaluation.ttl_ms,
        trace_id: trace_id.to_string(),
    })
}

fn intent_kind_from_indicator(indicator: &IndicatorNode) -> Option<IntentKind> {
    match indicator.kind {
        CoreIndicatorKind::MaCross => {
            let variant = indicator
                .params
                .get("intent_variant")
                .and_then(|value| value.as_str())
                .unwrap_or("long_term_buy");
            Some(if variant == "long_term_sell" {
                IntentKind::LongTermSell
            } else if variant == "sma_crossover" {
                IntentKind::SmaCrossover
            } else {
                IntentKind::LongTermBuy
            })
        }
        CoreIndicatorKind::Rsi => Some(IntentKind::Rsi),
        CoreIndicatorKind::Macd => Some(IntentKind::Macd),
        CoreIndicatorKind::Momentum => Some(IntentKind::Momentum),
        CoreIndicatorKind::Spread => Some(IntentKind::QuoteObserve),
        CoreIndicatorKind::ZScore => Some(IntentKind::ZScore),
        CoreIndicatorKind::Custom => {
            indicator
                .custom_expr
                .as_ref()
                .map(|spec| match spec.signal_kind {
                    SignalKind::Short => IntentKind::LongTermSell,
                    SignalKind::Observe | SignalKind::Raw => IntentKind::QuoteObserve,
                    SignalKind::Long => IntentKind::LongTermBuy,
                })
        }
        CoreIndicatorKind::QuoteObserve => Some(IntentKind::QuoteObserve),
        CoreIndicatorKind::Atr => None,
        CoreIndicatorKind::BollingerBands => Some(IntentKind::Rsi),
        CoreIndicatorKind::Obv => Some(IntentKind::Rsi),
        CoreIndicatorKind::Cmf => Some(IntentKind::Rsi),
        CoreIndicatorKind::Adx => Some(IntentKind::Momentum),
        CoreIndicatorKind::Stochastic => Some(IntentKind::Rsi),
        CoreIndicatorKind::Cci => Some(IntentKind::Rsi),
        CoreIndicatorKind::ParabolicSar => Some(IntentKind::Momentum),
        CoreIndicatorKind::KeltnerChannel => Some(IntentKind::Rsi),
        CoreIndicatorKind::DonchianChannel => Some(IntentKind::Momentum),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use qrpc_core::{
        DataQualitySnapshot, Exchange, KlineSeriesSnapshot, MarketType, NormalizedKline,
        QuoteSnapshot, SignalSide, SourceStatus, Symbol,
    };
    use qrpc_core_ir::{
        CoreIndicatorKind, CoreMetadata, CoreSourceKind, CoreStrategyIr, CoreTimeInForce,
        CustomExprSpec, CustomPredicateExpr, CustomValueExpr, DataBinding, DataBindingKind,
        ExecutionRule, ExecutionSizingKind, IndicatorNode, ScalarExpr, SeriesExpr, SeriesField,
        SignalKind, SignalRule, CUSTOM_EXPR_V1_VERSION,
    };
    use std::collections::BTreeMap;

    fn sample_kline_series(data_id: &str, prices: &[f64]) -> NormalizedMarketData {
        NormalizedMarketData::KlineSeries(KlineSeriesSnapshot {
            data_id: data_id.into(),
            exchange: Exchange::Binance,
            symbol: Symbol::BtcUsdt,
            market_type: MarketType::Spot,
            interval: "1d".into(),
            bars: prices
                .iter()
                .enumerate()
                .map(|(index, close)| NormalizedKline {
                    exchange: Exchange::Binance,
                    symbol: Symbol::BtcUsdt,
                    market_type: MarketType::Spot,
                    interval: "1d".into(),
                    open_time_ms: index as u64 * 60_000,
                    close_time_ms: (index as u64 + 1) * 60_000,
                    open: *close,
                    high: *close,
                    low: *close,
                    close: *close,
                    volume: 1.0,
                })
                .collect(),
            window_len: prices.len(),
            ts_ms: prices.len() as u64 * 60_000,
            source_latency_ms: 0,
            source_status: SourceStatus::Healthy,
            data_quality: DataQualitySnapshot::default(),
        })
    }

    fn sample_execution_rule() -> ExecutionRule {
        ExecutionRule {
            execution_id: "exec".into(),
            venue_kind: "paper".into(),
            sizing_kind: ExecutionSizingKind::EquityNotionalRatio,
            slippage_bps: 5.0,
            taker_fee_bps: 10.0,
            total_cost_buffer_bps: 20.0,
            time_in_force: CoreTimeInForce::Gtc,
            params: BTreeMap::new(),
        }
    }

    fn sample_core_ir(
        indicator_id: &str,
        kind: CoreIndicatorKind,
        data_id: &str,
        data_kind: DataBindingKind,
        signal_kind: SignalKind,
    ) -> CoreStrategyIr {
        CoreStrategyIr {
            ir_version: qrpc_core::CORE_IR_V1_VERSION.to_string(),
            metadata: CoreMetadata {
                strategy_id: "intent_test".into(),
                name: "Intent Test".into(),
                source_kind: CoreSourceKind::RuntimeProtocol,
            },
            data_bindings: vec![DataBinding {
                data_id: data_id.into(),
                kind: data_kind,
                source_hints: BTreeMap::new(),
            }],
            indicators: vec![IndicatorNode {
                indicator_id: indicator_id.into(),
                kind,
                inputs: vec![SeriesExpr::DataRef {
                    data_id: data_id.into(),
                }],
                spread_spec: None,
                custom_expr: None,
                params: BTreeMap::new(),
            }],
            signal_rules: vec![SignalRule {
                signal_id: format!("{indicator_id}_signal"),
                indicator_id: indicator_id.into(),
                signal_kind,
                condition: ScalarExpr::RawText {
                    source: indicator_id.into(),
                },
            }],
            agent_policies: vec![],
            risk_policies: vec![],
            execution: sample_execution_rule(),
        }
    }

    #[test]
    fn builtin_intent_module_emits_quote_observe_signal() {
        let module = BuiltinIntentModule;
        let core_ir = sample_core_ir(
            "intent_binance_quote",
            CoreIndicatorKind::QuoteObserve,
            "binance_btc_quote",
            DataBindingKind::Quote,
            SignalKind::Observe,
        );
        let output = module.evaluate_intents(IntentEvaluationRequest {
            intent_kinds: &[IntentKind::QuoteObserve],
            core_ir: &core_ir,
            normalized_data: &[NormalizedMarketData::Quote(QuoteSnapshot {
                data_id: "binance_btc_quote".into(),
                exchange: Exchange::Binance,
                symbol: Symbol::BtcUsdt,
                market_type: MarketType::Spot,
                best_bid: 49_995.0,
                best_ask: 50_005.0,
                bid_size: 10.0,
                ask_size: 10.0,
                mid_price: 50_000.0,
                ts_ms: 10,
                source_latency_ms: 0,
                source_status: SourceStatus::Healthy,
                data_quality: DataQualitySnapshot::default(),
            })],
            now_ms: 10,
            trace_id: "trace",
        });

        assert_eq!(output.signals.len(), 1);
        assert_eq!(output.events.len(), 2);
        assert_eq!(
            output.events[0].payload["provider_key"],
            "builtin.intent.default"
        );
    }

    #[test]
    fn builtin_intent_module_emits_rsi_signal_for_oversold_series() {
        let module = BuiltinIntentModule;
        let prices = (0..30).map(|idx| 100.0 - idx as f64).collect::<Vec<_>>();
        let core_ir = sample_core_ir(
            "intent_rsi",
            CoreIndicatorKind::Rsi,
            "btc_kline",
            DataBindingKind::KlineSeries,
            SignalKind::Long,
        );
        let output = module.evaluate_intents(IntentEvaluationRequest {
            intent_kinds: &[IntentKind::Rsi],
            core_ir: &core_ir,
            normalized_data: &[sample_kline_series("btc_kline", &prices)],
            now_ms: 10,
            trace_id: "trace",
        });

        assert_eq!(output.signals.len(), 1);
        assert_eq!(output.signals[0].kind, IntentKind::Rsi);
        assert_eq!(output.signals[0].side, SignalSide::Long);
        assert!(output.signals[0].derived_metrics.contains_key("rsi"));
    }

    #[test]
    fn builtin_intent_module_emits_macd_signal_for_uptrend_series() {
        let module = BuiltinIntentModule;
        let mut prices = vec![100.0; 40];
        prices.extend((0..20).map(|idx| 100.0 + (idx as f64).powi(2)));
        let core_ir = sample_core_ir(
            "intent_macd",
            CoreIndicatorKind::Macd,
            "btc_kline",
            DataBindingKind::KlineSeries,
            SignalKind::Long,
        );
        let output = module.evaluate_intents(IntentEvaluationRequest {
            intent_kinds: &[IntentKind::Macd],
            core_ir: &core_ir,
            normalized_data: &[sample_kline_series("btc_kline", &prices)],
            now_ms: 10,
            trace_id: "trace",
        });

        assert_eq!(output.signals.len(), 1);
        assert_eq!(output.signals[0].kind, IntentKind::Macd);
        assert_eq!(output.signals[0].side, SignalSide::Long);
        assert!(output.signals[0].derived_metrics.contains_key("histogram"));
    }

    #[test]
    fn builtin_intent_module_prefers_core_ir_evaluator_when_available() {
        let module = BuiltinIntentModule;
        let prices = (0..30).map(|idx| 100.0 - idx as f64).collect::<Vec<_>>();
        let core_ir = sample_core_ir(
            "intent_rsi",
            CoreIndicatorKind::Rsi,
            "btc_kline",
            DataBindingKind::KlineSeries,
            SignalKind::Long,
        );
        let output = module.evaluate_intents(IntentEvaluationRequest {
            intent_kinds: &[IntentKind::Rsi],
            core_ir: &core_ir,
            normalized_data: &[sample_kline_series("btc_kline", &prices)],
            now_ms: 10,
            trace_id: "trace-core-ir",
        });

        assert_eq!(output.signals.len(), 1);
        assert_eq!(output.signals[0].side, SignalSide::Long);
        assert!(output.signals[0].derived_metrics.contains_key("rsi"));
    }

    #[test]
    fn builtin_intent_module_maps_custom_indicator_to_buy_intent() {
        let module = BuiltinIntentModule;
        let mut core_ir = sample_core_ir(
            "intent_custom",
            CoreIndicatorKind::Custom,
            "btc_kline",
            DataBindingKind::KlineSeries,
            SignalKind::Long,
        );
        core_ir.indicators[0].custom_expr = Some(CustomExprSpec {
            schema_version: CUSTOM_EXPR_V1_VERSION.into(),
            signal_kind: SignalKind::Long,
            predicate: CustomPredicateExpr {
                left: CustomValueExpr::Input {
                    data_id: "btc_kline".into(),
                    field: SeriesField::Close,
                },
                op: qrpc_core_ir::ComparisonOp::Gt,
                right: CustomValueExpr::Number { value: 90.0 },
            },
            strength: None,
            confidence: 0.8,
        });

        let output = module.evaluate_intents(IntentEvaluationRequest {
            intent_kinds: &[IntentKind::LongTermBuy],
            core_ir: &core_ir,
            normalized_data: &[sample_kline_series("btc_kline", &[95.0, 100.0, 110.0])],
            now_ms: 10,
            trace_id: "trace-custom",
        });

        assert_eq!(output.signals.len(), 1);
        assert_eq!(output.signals[0].kind, IntentKind::LongTermBuy);
        assert_eq!(output.signals[0].side, SignalSide::Long);
    }
}
