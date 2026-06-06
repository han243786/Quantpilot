mod intent_signal_lowering;

use anyhow::Result;
use qrpc_core::{
    AgentConfig, DataKind, IntentConfig, IntentKind, RebalanceSchedule as RuntimeRebalanceSchedule,
    RiskConfig, RuntimeProtocolCoreConfig,
};
use qrpc_core_ir::{
    AgentPolicy, AgentPolicyKind, CoreMetadata, CoreStrategyIr, CoreTimeInForce, DataBinding,
    DataBindingKind, ExecutionRule, ExecutionSizingKind,
    RebalanceSchedule as CoreRebalanceSchedule, RiskPolicy,
};
use std::collections::BTreeMap;

use super::{
    decode_align_direction, decode_series_field, decode_spread_output, lower_runtime_spread_spec,
    runtime_intent_is_spread, validate_runtime_protocol_config,
};

pub(super) fn lower_runtime_protocol_to_core_ir_with_metadata(
    config: &RuntimeProtocolCoreConfig,
    metadata: CoreMetadata,
) -> Result<CoreStrategyIr> {
    validate_runtime_protocol_config(config)?;

    let mut core_ir = CoreStrategyIr::new(
        metadata,
        ExecutionRule {
            execution_id: "execution.paper".to_string(),
            venue_kind: "paper".to_string(),
            sizing_kind: ExecutionSizingKind::EquityNotionalRatio,
            slippage_bps: config.default_slippage_bps,
            taker_fee_bps: config.taker_fee_bps,
            total_cost_buffer_bps: config.total_cost_buffer_bps,
            time_in_force: CoreTimeInForce::Gtc,
            params: BTreeMap::from([
                (
                    "initial_cash_balance".to_string(),
                    serde_json::Value::from(config.initial_cash_balance),
                ),
                (
                    "taker_fee_bps".to_string(),
                    serde_json::Value::from(config.taker_fee_bps),
                ),
                (
                    "default_slippage_bps".to_string(),
                    serde_json::Value::from(config.default_slippage_bps),
                ),
                (
                    "total_cost_buffer_bps".to_string(),
                    serde_json::Value::from(config.total_cost_buffer_bps),
                ),
            ]),
        },
    );

    core_ir.data_bindings = config
        .data_sources
        .iter()
        .map(|source| {
            let mut source_hints = BTreeMap::from([
                (
                    "exchange".to_string(),
                    format!("{:?}", source.exchange).to_lowercase(),
                ),
                (
                    "symbol".to_string(),
                    format!("{:?}", source.symbol).to_uppercase(),
                ),
                (
                    "timeframe".to_string(),
                    source.interval.clone().unwrap_or_else(|| "1d".to_string()),
                ),
            ]);
            if source.ping_enabled {
                source_hints.insert("ping_enabled".to_string(), "true".to_string());
            }
            if let Some(request_interval_ms) = source.request_interval_ms {
                source_hints.insert(
                    "request_interval_ms".to_string(),
                    request_interval_ms.to_string(),
                );
            }
            DataBinding {
                data_id: source.data_id.clone(),
                kind: match source.kind {
                    DataKind::KlineSeries => DataBindingKind::KlineSeries,
                    DataKind::Quote => DataBindingKind::Quote,
                },
                source_hints,
            }
        })
        .collect();

    core_ir.indicators = config
        .intents
        .iter()
        .filter(|intent| intent.enabled)
        .map(intent_signal_lowering::lower_runtime_intent_to_indicator)
        .collect::<Result<Vec<_>>>()?;

    core_ir.signal_rules = config
        .intents
        .iter()
        .filter(|intent| intent.enabled)
        .map(intent_signal_lowering::lower_runtime_intent_to_signal_rule)
        .collect::<Vec<_>>();
    core_ir.agent_policies = config
        .agents
        .iter()
        .filter(|agent| agent.enabled)
        .map(|agent| lower_runtime_agent_to_policy(agent, &config.intents))
        .collect();
    core_ir.risk_policies = config
        .risks
        .iter()
        .filter(|risk| risk.enabled)
        .map(lower_runtime_risk_to_policy)
        .collect();

    // v1.0.1: DAG 环检测 — 编译前校验策略图无环
    core_ir
        .validate_dag()
        .map_err(|errs| anyhow::anyhow!("策略图 DAG 校验失败: {:?}", errs))?;

    Ok(core_ir)
}

fn lower_runtime_risk_to_policy(risk: &RiskConfig) -> RiskPolicy {
    RiskPolicy {
        policy_id: risk.risk_id.clone(),
        name: risk.name.clone(),
        observed_agent_ids: risk.observed_agent_ids.clone(),
        max_position_ratio: risk.max_position_ratio,
        max_single_weight: risk.max_single_weight,
        max_concentration_ratio: risk.max_concentration_ratio,
        max_symbol_net_exposure_ratio: risk.max_symbol_net_exposure_ratio,
        max_portfolio_net_exposure_ratio: risk.max_portfolio_net_exposure_ratio,
        max_turnover: risk.max_turnover,
        min_trade_weight: risk.min_trade_weight,
        max_new_positions_per_rebalance: risk.max_new_positions_per_rebalance,
        max_total_leverage: risk.max_total_leverage,
        max_exchange_leverage: risk.max_exchange_leverage,
        min_action_interval_ms: risk.min_action_interval_ms,
        enabled: risk.enabled,
        max_cross_symbol_leverage: None,
    }
}

fn lower_runtime_rebalance_schedule_to_core_ir(
    schedule: RuntimeRebalanceSchedule,
) -> CoreRebalanceSchedule {
    match schedule {
        RuntimeRebalanceSchedule::EverySlow => CoreRebalanceSchedule::EverySlow,
        RuntimeRebalanceSchedule::Every1d => CoreRebalanceSchedule::Every1d,
        RuntimeRebalanceSchedule::Weekly => CoreRebalanceSchedule::Weekly,
    }
}

fn lower_runtime_agent_to_policy(agent: &AgentConfig, intents: &[IntentConfig]) -> AgentPolicy {
    let referenced_intents = intents
        .iter()
        .filter(|intent| agent.input_intent_ids.contains(&intent.intent_id))
        .collect::<Vec<_>>();
    let all_observe = !referenced_intents.is_empty()
        && referenced_intents
            .iter()
            .all(|intent| matches!(intent.kind, IntentKind::QuoteObserve));
    let portfolio_rebalance = agent
        .params
        .get("portfolio_rebalance")
        .copied()
        .unwrap_or_default()
        > 0.5;

    AgentPolicy {
        agent_id: agent.agent_id.clone(),
        name: agent.name.clone(),
        kind: if all_observe {
            AgentPolicyKind::CrossVenueArbitrage
        } else if portfolio_rebalance {
            AgentPolicyKind::PortfolioRebalance
        } else {
            AgentPolicyKind::WeightedSignals
        },
        input_signal_ids: agent.input_intent_ids.clone(),
        rebalance_symbols: agent
            .rebalance_symbols
            .iter()
            .map(|symbol| symbol.as_str().to_string())
            .collect(),
        rebalance_schedule: agent
            .rebalance_schedule
            .clone()
            .map(lower_runtime_rebalance_schedule_to_core_ir),
        rebalance_allocation_kind: agent.rebalance_allocation_kind.clone(),
        rebalance_rank_method: agent.rebalance_rank_method.clone(),
        rebalance_score_normalize: agent.rebalance_score_normalize.clone(),
        rebalance_target_weights: agent.rebalance_target_weights.clone(),
        decision_threshold: (!all_observe).then(|| {
            agent
                .params
                .get("decision_threshold")
                .copied()
                .unwrap_or(0.05)
        }),
        max_quantity_ratio: agent.params.get("max_quantity_ratio").copied().unwrap_or(
            if all_observe {
                0.5
            } else if portfolio_rebalance {
                1.0
            } else {
                0.8
            },
        ),
        spread_trigger_bps: all_observe.then(|| {
            agent
                .params
                .get("spread_trigger_bps")
                .copied()
                .unwrap_or(50.0)
        }),
        enabled: agent.enabled,
    }
}
