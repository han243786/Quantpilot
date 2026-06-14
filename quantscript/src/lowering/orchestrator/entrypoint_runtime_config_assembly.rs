use crate::analysis::analyze_script_module;
use crate::evaluator::normalize_script_module;
use crate::resolve::lower_script_to_typed_hir;
use crate::script::{Item, ScriptModule};
use anyhow::{anyhow, bail, Context, Result};
use qrpc_core::{
    AgentConfig, RiskConfig, RuntimeProtocolCoreConfig,
    GLOBAL_RISK_PROFILE_DEFAULT_MAX_EXCHANGE_LEVERAGE, GLOBAL_RISK_PROFILE_DEFAULT_MAX_POSITION,
    GLOBAL_RISK_PROFILE_DEFAULT_MAX_TOTAL_LEVERAGE,
    GLOBAL_RISK_PROFILE_DEFAULT_MIN_ACTION_INTERVAL_MS, PAPER_EXECUTION_PROFILE_DEFAULT_FEE_BPS,
    PAPER_EXECUTION_PROFILE_DEFAULT_SLIPPAGE_BPS,
};
use std::collections::BTreeSet;

use super::super::binding_sources::infer_data_sources;
use super::super::bindings::collect_bindings;
use super::super::context::LoweringContext;
use super::super::diagnostics::format_diagnostics;
use super::super::intents::{canonicalize_data_sources, infer_intents, inferred_agent_params};
use super::super::profile_detection_surface::{
    detect_global_risk_profile, detect_paper_execution_profile,
};
use super::super::universe::{detect_portfolio_rebalance_directive, expand_universe_constructs};

const ERR_MISSING_STRATEGY_FN: &str = "QPQSLOW006 QuantScript 必须声明 fn strategy() 入口函数。请在 .qs 文件中添加: fn strategy() { ... }";
const ERR_NO_FETCH_CALLS: &str = "QPQSLOW007 策略编译需要至少一个 fetch/get_data 调用";

pub(super) fn lower_script_to_runtime_config_with_context(
    module: &ScriptModule,
    context: &LoweringContext,
) -> Result<RuntimeProtocolCoreConfig> {
    let normalized_module = normalize_script_module(module).context("QPQSLOW000 脚本标准化失败")?;
    let rebalance_directive = detect_portfolio_rebalance_directive(&normalized_module, context)?;
    let expanded_module = expand_universe_constructs(&normalized_module, context)?;
    let resolved = lower_script_to_typed_hir(&expanded_module);
    let analysis = analyze_script_module(&expanded_module, &resolved);
    let mut diagnostics = resolved.diagnostics.clone();
    diagnostics.extend(analysis.diagnostics);
    let has_errors = diagnostics
        .iter()
        .any(|diagnostic| diagnostic.severity == crate::DiagnosticSeverity::Error);
    if has_errors {
        bail!(
            "QuantScript 语义分析失败:\n{}",
            format_diagnostics(&diagnostics)
        );
    }
    let strategy = expanded_module
        .items
        .iter()
        .find_map(|item| match item {
            Item::Function(function) if function.name == "strategy" => Some(function),
            _ => None,
        })
        .ok_or_else(|| anyhow!(ERR_MISSING_STRATEGY_FN))?;
    let risk_profile = detect_global_risk_profile(strategy)?;
    let execution_profile = detect_paper_execution_profile(strategy)?;

    let mut inferred_data_sources = infer_data_sources(strategy, &resolved.callables)?;
    // B1-13: fetch 去重
    {
        let mut seen = BTreeSet::new();
        inferred_data_sources.retain(|ds| {
            let key = format!(
                "{}:{:?}:{}",
                ds.symbol.as_str(),
                ds.exchange,
                ds.interval.as_deref().unwrap_or("")
            );
            seen.insert(key)
        });
    }
    if inferred_data_sources.is_empty() {
        bail!(ERR_NO_FETCH_CALLS);
    }

    let (bindings, binding_diagnostics) = collect_bindings(
        strategy,
        &inferred_data_sources,
        resolved.functions.clone(),
        resolved.expr_semantics.clone(),
        resolved.callables.clone(),
    )?;
    diagnostics.extend(binding_diagnostics);
    let data_sources = canonicalize_data_sources(&inferred_data_sources, &bindings);
    let intents = infer_intents(strategy, &bindings, &data_sources)?;

    let agent = AgentConfig {
        agent_id: "agent_script_main".into(),
        name: "Script Main Agent".into(),
        input_intent_ids: intents
            .iter()
            .map(|intent| intent.intent_id.clone())
            .collect(),
        rebalance_symbols: rebalance_directive
            .as_ref()
            .map(|directive| directive.symbols.clone())
            .unwrap_or_default(),
        rebalance_schedule: rebalance_directive
            .as_ref()
            .and_then(|directive| directive.schedule.clone()),
        rebalance_allocation_kind: rebalance_directive
            .as_ref()
            .map(|directive| directive.allocation_kind.clone()),
        rebalance_rank_method: rebalance_directive
            .as_ref()
            .and_then(|directive| directive.rank_method.clone()),
        rebalance_score_normalize: rebalance_directive
            .as_ref()
            .and_then(|directive| directive.score_normalize.clone()),
        rebalance_target_weights: rebalance_directive
            .as_ref()
            .map(|directive| directive.target_weights.clone())
            .unwrap_or_default(),
        params: inferred_agent_params(&intents, rebalance_directive.as_ref()),
        enabled: true,
    };

    let max_position_ratio = risk_profile
        .as_ref()
        .map(|profile| profile.max_position_ratio)
        .unwrap_or_else(|| {
            rebalance_directive
                .as_ref()
                .map(|_| 1.0)
                .unwrap_or(GLOBAL_RISK_PROFILE_DEFAULT_MAX_POSITION)
        });
    let risk = RiskConfig {
        risk_id: "risk_script_global".into(),
        name: "Script Global Risk".into(),
        observed_agent_ids: vec![agent.agent_id.clone()],
        max_position_ratio,
        max_single_weight: None,
        max_concentration_ratio: None,
        max_symbol_net_exposure_ratio: None,
        max_portfolio_net_exposure_ratio: None,
        max_turnover: None,
        min_trade_weight: None,
        max_new_positions_per_rebalance: None,
        max_total_leverage: risk_profile
            .as_ref()
            .map(|profile| profile.max_total_leverage)
            .unwrap_or(GLOBAL_RISK_PROFILE_DEFAULT_MAX_TOTAL_LEVERAGE),
        max_exchange_leverage: risk_profile
            .as_ref()
            .map(|profile| profile.max_exchange_leverage)
            .unwrap_or(GLOBAL_RISK_PROFILE_DEFAULT_MAX_EXCHANGE_LEVERAGE),
        min_action_interval_ms: risk_profile
            .as_ref()
            .map(|profile| profile.min_action_interval_ms)
            .unwrap_or(GLOBAL_RISK_PROFILE_DEFAULT_MIN_ACTION_INTERVAL_MS),
        enabled: true,
    };

    Ok(RuntimeProtocolCoreConfig {
        data_sources,
        intents,
        agents: vec![agent],
        risks: vec![risk],
        initial_cash_balance: 100_000.0,
        taker_fee_bps: execution_profile
            .as_ref()
            .map(|profile| profile.fee_bps)
            .unwrap_or(PAPER_EXECUTION_PROFILE_DEFAULT_FEE_BPS),
        default_slippage_bps: execution_profile
            .as_ref()
            .map(|profile| profile.slippage_bps)
            .unwrap_or(PAPER_EXECUTION_PROFILE_DEFAULT_SLIPPAGE_BPS),
        total_cost_buffer_bps: 20.0,
    })
}
