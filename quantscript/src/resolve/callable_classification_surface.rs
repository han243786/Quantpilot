use super::FunctionSignature;
use crate::script::{Item, ScriptModule};
use crate::types::TypeId;
use std::collections::BTreeMap;

use super::{
    ChangeHelperKind, KnownIndicatorHelperKind, KnownUniverseHelperKind, MovingAverageHelperKind,
    ResolvedBuiltinMathKind, ResolvedCallable, ResolvedCallableKind, ResolvedChangeSmoothingKind,
    ResolvedFetchSourceKind, ResolvedMemberMutationKind, ResolvedSeriesBoundaryKind,
    ResolvedSeriesCapabilityKind, ResolvedWindowAggregateKind, RsiHelperKind,
};

pub fn classify_series_capability_name(name: &str) -> Option<ResolvedSeriesCapabilityKind> {
    match name {
        "histogram" => Some(ResolvedSeriesCapabilityKind::Histogram),
        "first" => Some(ResolvedSeriesCapabilityKind::Boundary(
            ResolvedSeriesBoundaryKind::First,
        )),
        "last" => Some(ResolvedSeriesCapabilityKind::Boundary(
            ResolvedSeriesBoundaryKind::Last,
        )),
        "sum" => Some(ResolvedSeriesCapabilityKind::WindowAggregate(
            ResolvedWindowAggregateKind::Sum,
        )),
        "mean" | "avg" => Some(ResolvedSeriesCapabilityKind::WindowAggregate(
            ResolvedWindowAggregateKind::Mean,
        )),
        "std" | "stddev" => Some(ResolvedSeriesCapabilityKind::WindowAggregate(
            ResolvedWindowAggregateKind::StdDev,
        )),
        _ => None,
    }
}

pub fn classify_builtin_math_name(name: &str) -> Option<ResolvedBuiltinMathKind> {
    match name {
        "abs" => Some(ResolvedBuiltinMathKind::Abs),
        "max" | "min" | "pow" | "sqrt" | "variance" => Some(ResolvedBuiltinMathKind::Numeric),
        _ => None,
    }
}

pub fn classify_member_mutation_name(name: &str) -> Option<ResolvedMemberMutationKind> {
    match name {
        "push" => Some(ResolvedMemberMutationKind::Push),
        _ => None,
    }
}

pub(super) fn build_resolved_callables(
    module: &ScriptModule,
    signatures: &BTreeMap<String, FunctionSignature>,
) -> BTreeMap<String, ResolvedCallable> {
    let mut callables = BTreeMap::new();

    for (name, kind, smoothing_kind, fetch_source_kind) in [
        (
            "fetch",
            ResolvedCallableKind::FetchLike,
            None,
            Some(ResolvedFetchSourceKind::KlineSeries),
        ),
        (
            "get_data",
            ResolvedCallableKind::FetchLike,
            None,
            Some(ResolvedFetchSourceKind::KlineSeries),
        ),
        ("abs", ResolvedCallableKind::BuiltinMath, None, None),
        ("avg", ResolvedCallableKind::BuiltinMath, None, None),
        ("first", ResolvedCallableKind::BuiltinMath, None, None),
        ("last", ResolvedCallableKind::BuiltinMath, None, None),
        ("max", ResolvedCallableKind::BuiltinMath, None, None),
        ("mean", ResolvedCallableKind::BuiltinMath, None, None),
        ("min", ResolvedCallableKind::BuiltinMath, None, None),
        ("pow", ResolvedCallableKind::BuiltinMath, None, None),
        ("sqrt", ResolvedCallableKind::BuiltinMath, None, None),
        ("std", ResolvedCallableKind::BuiltinMath, None, None),
        ("stddev", ResolvedCallableKind::BuiltinMath, None, None),
        ("sum", ResolvedCallableKind::BuiltinMath, None, None),
        ("variance", ResolvedCallableKind::BuiltinMath, None, None),
        (
            "sma",
            ResolvedCallableKind::IndicatorHelper(KnownIndicatorHelperKind::MovingAverage(
                MovingAverageHelperKind::Sma,
            )),
            Some(ResolvedChangeSmoothingKind::Simple),
            None,
        ),
        (
            "ema",
            ResolvedCallableKind::IndicatorHelper(KnownIndicatorHelperKind::MovingAverage(
                MovingAverageHelperKind::Ema,
            )),
            Some(ResolvedChangeSmoothingKind::Ema),
            None,
        ),
        (
            "rsi",
            ResolvedCallableKind::IndicatorHelper(KnownIndicatorHelperKind::Rsi(
                RsiHelperKind::Wilder,
            )),
            None,
            None,
        ),
        (
            "macd",
            ResolvedCallableKind::IndicatorHelper(KnownIndicatorHelperKind::Macd),
            None,
            None,
        ),
        (
            "momentum",
            ResolvedCallableKind::IndicatorHelper(KnownIndicatorHelperKind::Momentum),
            None,
            None,
        ),
        (
            "zscore",
            ResolvedCallableKind::IndicatorHelper(KnownIndicatorHelperKind::ZScore),
            None,
            None,
        ),
        (
            "z_score",
            ResolvedCallableKind::IndicatorHelper(KnownIndicatorHelperKind::ZScore),
            None,
            None,
        ),
        (
            "symbols",
            ResolvedCallableKind::UniverseHelper(KnownUniverseHelperKind::Symbols),
            None,
            None,
        ),
        (
            "universe",
            ResolvedCallableKind::UniverseHelper(KnownUniverseHelperKind::Universe),
            None,
            None,
        ),
        (
            "filter",
            ResolvedCallableKind::UniverseHelper(KnownUniverseHelperKind::Filter),
            None,
            None,
        ),
        (
            "sort_by",
            ResolvedCallableKind::UniverseHelper(KnownUniverseHelperKind::SortBy),
            None,
            None,
        ),
        (
            "top",
            ResolvedCallableKind::UniverseHelper(KnownUniverseHelperKind::Top),
            None,
            None,
        ),
        ("equal_weight", ResolvedCallableKind::Imported, None, None),
        ("fixed_weights", ResolvedCallableKind::Imported, None, None),
        ("rank_weight", ResolvedCallableKind::Imported, None, None),
        ("score_weight", ResolvedCallableKind::Imported, None, None),
        ("rebalance", ResolvedCallableKind::Imported, None, None),
        (
            "rma",
            ResolvedCallableKind::Imported,
            Some(ResolvedChangeSmoothingKind::Wilder),
            None,
        ),
        (
            "wilders",
            ResolvedCallableKind::Imported,
            Some(ResolvedChangeSmoothingKind::Wilder),
            None,
        ),
        (
            "smma",
            ResolvedCallableKind::Imported,
            Some(ResolvedChangeSmoothingKind::Wilder),
            None,
        ),
        ("field", ResolvedCallableKind::Imported, None, None),
        ("resample", ResolvedCallableKind::Imported, None, None),
        ("align", ResolvedCallableKind::Imported, None, None),
        ("align_asof", ResolvedCallableKind::Imported, None, None),
        ("spread", ResolvedCallableKind::Imported, None, None),
        (
            "gains",
            ResolvedCallableKind::ChangeHelper(ChangeHelperKind::Gain),
            None,
            None,
        ),
        (
            "gain",
            ResolvedCallableKind::ChangeHelper(ChangeHelperKind::Gain),
            None,
            None,
        ),
        (
            "up_moves",
            ResolvedCallableKind::ChangeHelper(ChangeHelperKind::Gain),
            None,
            None,
        ),
        (
            "positive_changes",
            ResolvedCallableKind::ChangeHelper(ChangeHelperKind::Gain),
            None,
            None,
        ),
        (
            "positive_deltas",
            ResolvedCallableKind::ChangeHelper(ChangeHelperKind::Gain),
            None,
            None,
        ),
        (
            "losses",
            ResolvedCallableKind::ChangeHelper(ChangeHelperKind::Loss),
            None,
            None,
        ),
        (
            "loss",
            ResolvedCallableKind::ChangeHelper(ChangeHelperKind::Loss),
            None,
            None,
        ),
        (
            "down_moves",
            ResolvedCallableKind::ChangeHelper(ChangeHelperKind::Loss),
            None,
            None,
        ),
        (
            "negative_changes",
            ResolvedCallableKind::ChangeHelper(ChangeHelperKind::Loss),
            None,
            None,
        ),
        (
            "negative_deltas",
            ResolvedCallableKind::ChangeHelper(ChangeHelperKind::Loss),
            None,
            None,
        ),
    ] {
        seed_callable(
            &mut callables,
            name,
            kind,
            smoothing_kind,
            fetch_source_kind,
            TypeId(0),
        );
    }

    for item in &module.items {
        if let Item::Import(import_decl) = item {
            if let Some(names) = &import_decl.names {
                for name in names {
                    let local_name = name.alias.clone().unwrap_or_else(|| name.name.clone());
                    callables
                        .entry(local_name.clone())
                        .or_insert(ResolvedCallable {
                            name: local_name,
                            kind: classify_imported_helper(&name.name),
                            change_smoothing_kind: classify_change_smoothing_kind(&name.name),
                            fetch_source_kind: classify_fetch_source_kind(&name.name),
                            return_type: TypeId(0),
                        });
                }
            }
        }
    }

    for (name, signature) in signatures {
        callables.insert(
            name.clone(),
            ResolvedCallable {
                name: name.clone(),
                kind: ResolvedCallableKind::UserFunction,
                change_smoothing_kind: None,
                fetch_source_kind: None,
                return_type: signature.return_type,
            },
        );
    }

    callables
}

fn seed_callable(
    callables: &mut BTreeMap<String, ResolvedCallable>,
    name: &str,
    kind: ResolvedCallableKind,
    change_smoothing_kind: Option<ResolvedChangeSmoothingKind>,
    fetch_source_kind: Option<ResolvedFetchSourceKind>,
    return_type: TypeId,
) {
    callables
        .entry(name.to_string())
        .or_insert(ResolvedCallable {
            name: name.to_string(),
            kind,
            change_smoothing_kind,
            fetch_source_kind,
            return_type,
        });
}

pub(super) fn classify_imported_helper(name: &str) -> ResolvedCallableKind {
    match name {
        "fetch" | "get_data" => ResolvedCallableKind::FetchLike,
        "sma" => ResolvedCallableKind::IndicatorHelper(KnownIndicatorHelperKind::MovingAverage(
            MovingAverageHelperKind::Sma,
        )),
        "ema" => ResolvedCallableKind::IndicatorHelper(KnownIndicatorHelperKind::MovingAverage(
            MovingAverageHelperKind::Ema,
        )),
        "rsi" => ResolvedCallableKind::IndicatorHelper(KnownIndicatorHelperKind::Rsi(
            RsiHelperKind::Wilder,
        )),
        "macd" => ResolvedCallableKind::IndicatorHelper(KnownIndicatorHelperKind::Macd),
        "momentum" => ResolvedCallableKind::IndicatorHelper(KnownIndicatorHelperKind::Momentum),
        "atr" => ResolvedCallableKind::IndicatorHelper(KnownIndicatorHelperKind::Atr),
        "bollinger" | "bb" => {
            ResolvedCallableKind::IndicatorHelper(KnownIndicatorHelperKind::BollingerBands)
        }
        "obv" => ResolvedCallableKind::IndicatorHelper(KnownIndicatorHelperKind::Obv),
        "cmf" => ResolvedCallableKind::IndicatorHelper(KnownIndicatorHelperKind::Cmf),
        "adx" => ResolvedCallableKind::IndicatorHelper(KnownIndicatorHelperKind::Adx),
        "stoch" | "stochastic" => {
            ResolvedCallableKind::IndicatorHelper(KnownIndicatorHelperKind::Stochastic)
        }
        "cci" => ResolvedCallableKind::IndicatorHelper(KnownIndicatorHelperKind::Cci),
        "psar" | "parabolic_sar" => {
            ResolvedCallableKind::IndicatorHelper(KnownIndicatorHelperKind::ParabolicSar)
        }
        "keltner" => {
            ResolvedCallableKind::IndicatorHelper(KnownIndicatorHelperKind::KeltnerChannel)
        }
        "donchian" => {
            ResolvedCallableKind::IndicatorHelper(KnownIndicatorHelperKind::DonchianChannel)
        }
        "zscore" | "z_score" => {
            ResolvedCallableKind::IndicatorHelper(KnownIndicatorHelperKind::ZScore)
        }
        "symbols" => ResolvedCallableKind::UniverseHelper(KnownUniverseHelperKind::Symbols),
        "universe" => ResolvedCallableKind::UniverseHelper(KnownUniverseHelperKind::Universe),
        "filter" => ResolvedCallableKind::UniverseHelper(KnownUniverseHelperKind::Filter),
        "sort_by" => ResolvedCallableKind::UniverseHelper(KnownUniverseHelperKind::SortBy),
        "top" => ResolvedCallableKind::UniverseHelper(KnownUniverseHelperKind::Top),
        "gains" | "gain" | "up_moves" | "positive_changes" | "positive_deltas" => {
            ResolvedCallableKind::ChangeHelper(ChangeHelperKind::Gain)
        }
        "losses" | "loss" | "down_moves" | "negative_changes" | "negative_deltas" => {
            ResolvedCallableKind::ChangeHelper(ChangeHelperKind::Loss)
        }
        "field" | "resample" | "align" | "align_asof" | "spread" => ResolvedCallableKind::Imported,
        "abs" | "avg" | "first" | "last" | "max" | "mean" | "min" | "pow" | "sqrt" | "std"
        | "stddev" | "sum" | "variance" => ResolvedCallableKind::BuiltinMath,
        _ => ResolvedCallableKind::Imported,
    }
}

pub(super) fn classify_change_smoothing_kind(name: &str) -> Option<ResolvedChangeSmoothingKind> {
    match name {
        "rma" | "wilders" | "smma" => Some(ResolvedChangeSmoothingKind::Wilder),
        "ema" => Some(ResolvedChangeSmoothingKind::Ema),
        "sma" => Some(ResolvedChangeSmoothingKind::Simple),
        _ => None,
    }
}

fn classify_fetch_source_kind(name: &str) -> Option<ResolvedFetchSourceKind> {
    match name {
        "fetch" | "get_data" => Some(ResolvedFetchSourceKind::KlineSeries),
        _ => None,
    }
}

pub(super) fn is_known_helper_function(name: &str) -> bool {
    matches!(
        name,
        "sma"
            | "ema"
            | "rma"
            | "wilders"
            | "smma"
            | "rsi"
            | "macd"
            | "momentum"
            | "zscore"
            | "gains"
            | "gain"
            | "losses"
            | "loss"
            | "up_moves"
            | "down_moves"
            | "positive_changes"
            | "negative_changes"
            | "positive_deltas"
            | "negative_deltas"
            | "field"
            | "resample"
            | "align"
            | "align_asof"
            | "spread"
            | "symbols"
            | "universe"
            | "filter"
            | "sort_by"
            | "top"
            | "equal_weight"
            | "fixed_weights"
            | "rank_weight"
            | "score_weight"
            | "rebalance"
            | "first"
            | "last"
    )
}
