use anyhow::Result;
use qrpc_core::{IntentConfig, IntentKind};
use qrpc_core_ir::indicator_threshold_compare_expr;
use std::collections::BTreeMap;

use crate::script::BinaryOp;

use super::super::binding_sources::format_symbol;
use super::super::bindings::{BindingEnv, IndicatorBinding};
use super::super::semantic::rsi_method_code;
use super::super::shared::sanitize_id;
use super::{
    canonicalize_data_source_for_bindings, comparison_op_code, comparison_op_from_binary_relation,
    comparison_shape_code, normalize_relation, STRUCTURED_COMPARISON_OP_KEY,
    STRUCTURED_COMPARISON_SHAPE_KEY, STRUCTURED_COMPARISON_THRESHOLD_KEY,
};

pub(super) fn single_indicator_intent(
    indicator: IndicatorBinding,
    indicator_on_left: bool,
    op: BinaryOp,
    threshold: f64,
    action: &str,
    bindings: &BindingEnv,
    runtime_id_hint: Option<&str>,
) -> Result<Vec<IntentConfig>> {
    let relation = normalize_relation(op, indicator_on_left);
    let absolute_threshold = threshold.abs();

    let intent = match indicator {
        IndicatorBinding::Rsi {
            source,
            period,
            method,
        } => {
            let source = canonicalize_data_source_for_bindings(&source, bindings);
            let instrument = format_symbol(&source.symbol);
            let comparison_op = match action {
                "BUY" if matches!(relation, BinaryOp::Less | BinaryOp::LessEqual) => {
                    comparison_op_from_binary_relation(&relation)
                }
                "SELL" if matches!(relation, BinaryOp::Greater | BinaryOp::GreaterEqual) => {
                    comparison_op_from_binary_relation(&relation)
                }
                _ => None,
            };
            let Some(comparison_op) = comparison_op else {
                return Ok(Vec::new());
            };
            let mut params = BTreeMap::from([
                ("period".into(), period as f64),
                ("smoothing_method".into(), rsi_method_code(method)),
                (
                    STRUCTURED_COMPARISON_OP_KEY.into(),
                    comparison_op_code(comparison_op),
                ),
                (
                    STRUCTURED_COMPARISON_SHAPE_KEY.into(),
                    comparison_shape_code(action).unwrap_or_default(),
                ),
            ]);
            match action {
                "BUY" if matches!(relation, BinaryOp::Less | BinaryOp::LessEqual) => {
                    if indicator_threshold_compare_expr(
                        runtime_id_hint
                            .map(str::to_string)
                            .unwrap_or_else(|| format!("intent_{}_rsi", sanitize_id(instrument))),
                        comparison_op,
                        threshold,
                    )
                    .is_none()
                    {
                        return Ok(Vec::new());
                    }
                    params.insert("oversold_threshold".into(), threshold);
                    params.insert("overbought_threshold".into(), 70.0);
                }
                "SELL" if matches!(relation, BinaryOp::Greater | BinaryOp::GreaterEqual) => {
                    if indicator_threshold_compare_expr(
                        runtime_id_hint
                            .map(str::to_string)
                            .unwrap_or_else(|| format!("intent_{}_rsi", sanitize_id(instrument))),
                        comparison_op,
                        threshold,
                    )
                    .is_none()
                    {
                        return Ok(Vec::new());
                    }
                    params.insert("oversold_threshold".into(), 30.0);
                    params.insert("overbought_threshold".into(), threshold);
                }
                _ => return Ok(Vec::new()),
            }
            IntentConfig {
                intent_id: runtime_id_hint
                    .map(str::to_string)
                    .unwrap_or_else(|| format!("intent_{}_rsi", sanitize_id(instrument))),
                name: format!("{instrument} RSI"),
                kind: IntentKind::Rsi,
                input_data_ids: vec![source.data_id.clone()],
                params,
                enabled: true,
            }
        }
        IndicatorBinding::Macd {
            source,
            fast_period,
            slow_period,
            signal_period,
        } => {
            let source = canonicalize_data_source_for_bindings(&source, bindings);
            if !matches!(
                (action, relation),
                ("BUY", BinaryOp::Greater | BinaryOp::GreaterEqual)
                    | ("SELL", BinaryOp::Less | BinaryOp::LessEqual)
            ) {
                return Ok(Vec::new());
            }
            let instrument = format_symbol(&source.symbol);
            IntentConfig {
                intent_id: runtime_id_hint
                    .map(str::to_string)
                    .unwrap_or_else(|| format!("intent_{}_macd", sanitize_id(instrument))),
                name: format!("{instrument} MACD"),
                kind: IntentKind::Macd,
                input_data_ids: vec![source.data_id.clone()],
                params: BTreeMap::from([
                    ("fast_period".into(), fast_period as f64),
                    ("slow_period".into(), slow_period as f64),
                    ("signal_period".into(), signal_period as f64),
                    ("histogram_threshold".into(), absolute_threshold),
                ]),
                enabled: true,
            }
        }
        IndicatorBinding::Momentum { source, lookback } => {
            let source = canonicalize_data_source_for_bindings(&source, bindings);
            let comparison_op = match action {
                "BUY" if matches!(relation, BinaryOp::Greater | BinaryOp::GreaterEqual) => {
                    comparison_op_from_binary_relation(&relation)
                }
                "SELL" if matches!(relation, BinaryOp::Less | BinaryOp::LessEqual) => {
                    comparison_op_from_binary_relation(&relation)
                }
                _ => None,
            };
            let Some(comparison_op) = comparison_op else {
                return Ok(Vec::new());
            };
            if !matches!(
                (action, relation),
                ("BUY", BinaryOp::Greater | BinaryOp::GreaterEqual)
                    | ("SELL", BinaryOp::Less | BinaryOp::LessEqual)
            ) {
                return Ok(Vec::new());
            }
            let instrument = format_symbol(&source.symbol);
            if indicator_threshold_compare_expr(
                runtime_id_hint
                    .map(str::to_string)
                    .unwrap_or_else(|| format!("intent_{}_momentum", sanitize_id(instrument))),
                comparison_op,
                threshold,
            )
            .is_none()
            {
                return Ok(Vec::new());
            }
            IntentConfig {
                intent_id: runtime_id_hint
                    .map(str::to_string)
                    .unwrap_or_else(|| format!("intent_{}_momentum", sanitize_id(instrument))),
                name: format!("{instrument} Momentum"),
                kind: IntentKind::Momentum,
                input_data_ids: vec![source.data_id.clone()],
                params: BTreeMap::from([
                    ("lookback".into(), lookback as f64),
                    ("threshold_ratio".into(), absolute_threshold),
                    (
                        STRUCTURED_COMPARISON_OP_KEY.into(),
                        comparison_op_code(comparison_op),
                    ),
                    (
                        STRUCTURED_COMPARISON_SHAPE_KEY.into(),
                        comparison_shape_code(action).unwrap_or_default(),
                    ),
                    (STRUCTURED_COMPARISON_THRESHOLD_KEY.into(), threshold),
                ]),
                enabled: true,
            }
        }
        IndicatorBinding::ZScore { source, window } => {
            let source = canonicalize_data_source_for_bindings(&source, bindings);
            let comparison_op = match action {
                "BUY" if matches!(relation, BinaryOp::Less | BinaryOp::LessEqual) => {
                    comparison_op_from_binary_relation(&relation)
                }
                "SELL" if matches!(relation, BinaryOp::Greater | BinaryOp::GreaterEqual) => {
                    comparison_op_from_binary_relation(&relation)
                }
                _ => None,
            };
            let Some(comparison_op) = comparison_op else {
                return Ok(Vec::new());
            };
            if !matches!(
                (action, relation),
                ("BUY", BinaryOp::Less | BinaryOp::LessEqual)
                    | ("SELL", BinaryOp::Greater | BinaryOp::GreaterEqual)
            ) {
                return Ok(Vec::new());
            }
            let instrument = format_symbol(&source.symbol);
            if indicator_threshold_compare_expr(
                runtime_id_hint
                    .map(str::to_string)
                    .unwrap_or_else(|| format!("intent_{}_zscore", sanitize_id(instrument))),
                comparison_op,
                threshold,
            )
            .is_none()
            {
                return Ok(Vec::new());
            }
            IntentConfig {
                intent_id: runtime_id_hint
                    .map(str::to_string)
                    .unwrap_or_else(|| format!("intent_{}_zscore", sanitize_id(instrument))),
                name: format!("{instrument} ZScore"),
                kind: IntentKind::ZScore,
                input_data_ids: vec![source.data_id.clone()],
                params: BTreeMap::from([
                    ("window".into(), window as f64),
                    ("entry_z".into(), absolute_threshold.max(0.1)),
                    (
                        STRUCTURED_COMPARISON_OP_KEY.into(),
                        comparison_op_code(comparison_op),
                    ),
                    (
                        STRUCTURED_COMPARISON_SHAPE_KEY.into(),
                        comparison_shape_code(action).unwrap_or_default(),
                    ),
                    (STRUCTURED_COMPARISON_THRESHOLD_KEY.into(), threshold),
                ]),
                enabled: true,
            }
        }
        IndicatorBinding::MovingAverage { .. }
        | IndicatorBinding::MacdLine { .. }
        | IndicatorBinding::MacdSignal { .. }
        | IndicatorBinding::Atr { .. }
        | IndicatorBinding::BollingerBands { .. }
        | IndicatorBinding::Obv { .. }
        | IndicatorBinding::Cmf { .. }
        | IndicatorBinding::Adx { .. }
        | IndicatorBinding::Stochastic { .. }
        | IndicatorBinding::Cci { .. }
        | IndicatorBinding::ParabolicSar { .. }
        | IndicatorBinding::KeltnerChannel { .. }
        | IndicatorBinding::DonchianChannel { .. } => return Ok(Vec::new()),
    };

    Ok(vec![intent])
}
