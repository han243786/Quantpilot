use qrpc_core::{
    AgentConfig, IntentConfig, IntentKind, RebalanceSchedule as RuntimeRebalanceSchedule,
};
use qrpc_core_ir::{AgentPolicy, AgentPolicyKind, RebalanceSchedule as CoreRebalanceSchedule};

pub(super) fn lower_runtime_agent_to_policy(
    agent: &AgentConfig,
    intents: &[IntentConfig],
) -> AgentPolicy {
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

fn lower_runtime_rebalance_schedule_to_core_ir(
    schedule: RuntimeRebalanceSchedule,
) -> CoreRebalanceSchedule {
    match schedule {
        RuntimeRebalanceSchedule::EverySlow => CoreRebalanceSchedule::EverySlow,
        RuntimeRebalanceSchedule::Every1d => CoreRebalanceSchedule::Every1d,
        RuntimeRebalanceSchedule::Weekly => CoreRebalanceSchedule::Weekly,
    }
}
