use super::*;
use qrpc_core::{RebalanceSchedule, Symbol};

/// 动量指标默认阈值比率 (§5.1)
const DEFAULT_MOMENTUM_THRESHOLD: f64 = 0.02;


/// v0.5.2: 改为接受两个 CompileRuntimeTargets, 消除对 MappedRuntimeConfig 的依赖。
/// provided 优先级高于 fallback。
pub(super) fn merge_runtime_targets(
    provided: &CompileRuntimeTargets,
    fallback: &CompileRuntimeTargets,
) -> CompileRuntimeTargets {
    let mut source_to_node = fallback.source_to_node.clone();
    source_to_node.extend(provided.source_to_node.clone());
    CompileRuntimeTargets {
        source_to_node,
        runtime_node_id: provided
            .runtime_node_id
            .clone()
            .or_else(|| fallback.runtime_node_id.clone()),
        execution_node_id: provided
            .execution_node_id
            .clone()
            .or_else(|| fallback.execution_node_id.clone()),
    }
}

pub(super) fn frontend_runtime_config_from_core_with_template(
    runtime_protocol: &RuntimeProtocolCoreConfig,
    template: &FrontendRuntimeConfig,
    runtime_targets: &CompileRuntimeTargets,
    graph_id: &str,
    compile_id: &str,
) -> FrontendRuntimeConfig {
    FrontendRuntimeConfig {
        metadata: FrontendMetadata {
            graph_id: graph_id.to_string(),
            compile_id: compile_id.to_string(),
            name: template.metadata.name.clone(),
            version: template.metadata.version.clone(),
            mode: template.metadata.mode.clone(),
        },
        data_sources: runtime_protocol
            .data_sources
            .iter()
            .map(|source| frontend_data_node_from_core(source, template, runtime_targets))
            .collect(),
        intent_generators: runtime_protocol
            .intents
            .iter()
            .map(|intent| frontend_intent_node_from_core(intent, template, runtime_targets))
            .collect(),
        agents: runtime_protocol
            .agents
            .iter()
            .map(|agent| frontend_agent_node_from_core(agent, template, runtime_targets))
            .collect(),
        risk_controls: runtime_protocol
            .risks
            .iter()
            .map(|risk| frontend_risk_node_from_core(risk, template, runtime_targets))
            .collect(),
        executions: frontend_execution_nodes_from_core(runtime_protocol, template, runtime_targets),
        runtime_control: Some(frontend_runtime_node_from_core(
            template,
            runtime_targets,
            &template.metadata.mode,
        )),
    }
}

fn frontend_data_node_from_core(
    source: &DataSourceConfig,
    template: &FrontendRuntimeConfig,
    runtime_targets: &CompileRuntimeTargets,
) -> FrontendNodeConfig {
    let node_id = runtime_target_node_id(runtime_targets, &source.data_id, &source.data_id);
    let template_node = template.data_sources.iter().find(|node| node.id == node_id);
    FrontendNodeConfig {
        id: node_id.clone(),
        module_key: match source.kind {
            DataKind::KlineSeries => "builtin.data.kline".to_string(),
            DataKind::Quote => "builtin.data.quote".to_string(),
        },
        name: template_node
            .map(|node| node.name.clone())
            .unwrap_or_else(|| node_id.clone()),
        config: match source.kind {
            DataKind::KlineSeries => serde_json::json!({
                "exchange": format_exchange_name(&source.exchange),
                "instrument": format_symbol_name(&source.symbol),
                "timeframe": source.interval.clone().unwrap_or_else(|| "1d".to_string()),
                "window_size": source.days.unwrap_or(200),
                "ping_enabled": source.ping_enabled,
                "request_interval_ms": source.request_interval_ms.unwrap_or(0)
            }),
            DataKind::Quote => serde_json::json!({
                "exchange": format_exchange_name(&source.exchange),
                "instrument": format_symbol_name(&source.symbol),
                "ping_enabled": source.ping_enabled,
                "request_interval_ms": source.request_interval_ms.unwrap_or(0)
            }),
        },
    }
}

fn frontend_intent_node_from_core(
    intent: &IntentConfig,
    template: &FrontendRuntimeConfig,
    runtime_targets: &CompileRuntimeTargets,
) -> FrontendIntentConfig {
    let node_id = runtime_target_node_id(runtime_targets, &intent.intent_id, &intent.intent_id);
    let template_node = template
        .intent_generators
        .iter()
        .find(|node| node.id == node_id);
    FrontendIntentConfig {
        id: node_id.clone(),
        module_key: frontend_intent_module_key(&intent.kind).to_string(),
        name: template_node
            .map(|node| node.name.clone())
            .unwrap_or_else(|| intent.name.clone()),
        config: frontend_intent_config_value(intent),
        input_refs: intent
            .input_data_ids
            .iter()
            .map(|source_id| FrontendInputRef {
                source_id: runtime_target_node_id(runtime_targets, source_id, source_id),
                source_port: "market_data_out".to_string(),
                target_port: "data_input".to_string(),
            })
            .collect(),
    }
}

fn frontend_agent_node_from_core(
    agent: &AgentConfig,
    template: &FrontendRuntimeConfig,
    runtime_targets: &CompileRuntimeTargets,
) -> FrontendAgentConfig {
    let node_id = runtime_target_node_id(runtime_targets, &agent.agent_id, &agent.agent_id);
    let template_node = template.agents.iter().find(|node| node.id == node_id);
    FrontendAgentConfig {
        id: node_id.clone(),
        module_key: "builtin.agent.weighted".to_string(),
        name: template_node
            .map(|node| node.name.clone())
            .unwrap_or_else(|| agent.name.clone()),
        config: serde_json::json!({
            "decision_threshold": agent.params.get("decision_threshold").copied().unwrap_or(0.05),
            "max_quantity_ratio": agent.params.get("max_quantity_ratio").copied().unwrap_or(0.8),
            "rebalance_symbols": format_rebalance_symbols(&agent.rebalance_symbols),
            "rebalance_schedule": format_rebalance_schedule(agent.rebalance_schedule.as_ref()),
            "rebalance_allocation_kind": agent.rebalance_allocation_kind.clone().unwrap_or_default(),
            "rebalance_rank_method": agent.rebalance_rank_method.clone().unwrap_or_default(),
            "rebalance_score_normalize": agent.rebalance_score_normalize.clone().unwrap_or_default(),
            "rebalance_target_weights": format_rebalance_target_weights(&agent.rebalance_target_weights)
        }),
        intent_refs: agent
            .input_intent_ids
            .iter()
            .map(|intent_id| runtime_target_node_id(runtime_targets, intent_id, intent_id))
            .collect(),
    }
}

fn frontend_risk_node_from_core(
    risk: &RiskConfig,
    template: &FrontendRuntimeConfig,
    runtime_targets: &CompileRuntimeTargets,
) -> FrontendRiskConfig {
    let node_id = runtime_target_node_id(runtime_targets, &risk.risk_id, &risk.risk_id);
    let template_node = template
        .risk_controls
        .iter()
        .find(|node| node.id == node_id);
    FrontendRiskConfig {
        id: node_id.clone(),
        module_key: "builtin.risk.global".to_string(),
        name: template_node
            .map(|node| node.name.clone())
            .unwrap_or_else(|| risk.name.clone()),
        config: serde_json::json!({
            "profile_id": GLOBAL_RISK_PROFILE_ID,
            "max_position": risk.max_position_ratio,
            "max_single_weight": risk.max_single_weight,
            "max_concentration": risk.max_concentration_ratio,
            "max_symbol_net_exposure": risk.max_symbol_net_exposure_ratio,
            "max_portfolio_net_exposure": risk.max_portfolio_net_exposure_ratio,
            "max_turnover": risk.max_turnover,
            "min_trade_weight": risk.min_trade_weight,
            "max_new_positions_per_rebalance": risk.max_new_positions_per_rebalance,
            "max_total_leverage": risk.max_total_leverage,
            "max_exchange_leverage": risk.max_exchange_leverage,
            "min_action_interval_ms": risk.min_action_interval_ms
        }),
        agent_refs: risk
            .observed_agent_ids
            .iter()
            .map(|agent_id| runtime_target_node_id(runtime_targets, agent_id, agent_id))
            .collect(),
    }
}

fn frontend_execution_nodes_from_core(
    runtime_protocol: &RuntimeProtocolCoreConfig,
    template: &FrontendRuntimeConfig,
    runtime_targets: &CompileRuntimeTargets,
) -> Vec<FrontendExecutionConfig> {
    let node_id = runtime_targets
        .execution_node_id
        .clone()
        .or_else(|| template.executions.first().map(|node| node.id.clone()))
        .unwrap_or_else(|| "execution_script_main".to_string());
    let template_node = template.executions.iter().find(|node| node.id == node_id);
    let risk_ref = runtime_protocol
        .risks
        .first()
        .map(|risk| runtime_target_node_id(runtime_targets, &risk.risk_id, &risk.risk_id));

    vec![FrontendExecutionConfig {
        id: node_id.clone(),
        module_key: "builtin.execution.paper".to_string(),
        name: template_node
            .map(|node| node.name.clone())
            .unwrap_or_else(|| node_id.clone()),
        config: serde_json::json!({
            "profile_id": PAPER_EXECUTION_PROFILE_ID,
            "mode": "paper",
            "fee_bps": runtime_protocol.taker_fee_bps,
            "slippage_bps": runtime_protocol.default_slippage_bps
        }),
        risk_ref,
    }]
}

fn frontend_runtime_node_from_core(
    template: &FrontendRuntimeConfig,
    runtime_targets: &CompileRuntimeTargets,
    mode: &str,
) -> FrontendNodeConfig {
    let node_id = runtime_targets
        .runtime_node_id
        .clone()
        .or_else(|| {
            template
                .runtime_control
                .as_ref()
                .map(|node| node.id.clone())
        })
        .unwrap_or_else(|| "runtime_script_main".to_string());
    let template_node = template
        .runtime_control
        .as_ref()
        .filter(|node| node.id == node_id);

    FrontendNodeConfig {
        id: node_id.clone(),
        module_key: "builtin.runtime.control".to_string(),
        name: template_node
            .map(|node| node.name.clone())
            .unwrap_or_else(|| node_id),
        config: serde_json::json!({
            "mode": mode
        }),
    }
}

fn frontend_intent_module_key(kind: &IntentKind) -> &'static str {
    match kind {
        IntentKind::LongTermBuy => "builtin.intent.double_ma",
        IntentKind::LongTermSell => "builtin.intent.ma_deviation",
        IntentKind::Rsi => "builtin.intent.rsi",
        IntentKind::Macd => "builtin.intent.macd",
        IntentKind::Momentum => "builtin.intent.momentum",
        IntentKind::ZScore => "builtin.intent.zscore",
        IntentKind::QuoteObserve => "builtin.intent.spread_observer",
        IntentKind::SmaCrossover => "builtin.intent.double_ma",
    }
}

fn frontend_intent_config_value(intent: &IntentConfig) -> Value {
    match intent.kind {
        IntentKind::LongTermBuy => serde_json::json!({
            "fast_period": intent.params.get("fast_period").copied().unwrap_or(20.0),
            "slow_period": intent.params.get("slow_period").copied().unwrap_or(50.0),
            "entry_ratio": intent.params.get("entry_ratio").copied().unwrap_or(1.0)
        }),
        IntentKind::LongTermSell => serde_json::json!({
            "lookback": intent.params.get("lookback").copied().unwrap_or(20.0),
            "baseline_period": intent.params.get("baseline_period").copied().unwrap_or(50.0),
            "threshold_ratio": intent.params.get("threshold_ratio").copied().unwrap_or(1.0)
        }),
        IntentKind::Rsi => serde_json::json!({
            "period": intent.params.get("period").copied().unwrap_or(14.0),
            "oversold_threshold": intent.params.get("oversold_threshold").copied().unwrap_or(30.0),
            "overbought_threshold": intent.params.get("overbought_threshold").copied().unwrap_or(70.0)
        }),
        IntentKind::Macd => serde_json::json!({
            "fast_period": intent.params.get("fast_period").copied().unwrap_or(12.0),
            "slow_period": intent.params.get("slow_period").copied().unwrap_or(26.0),
            "signal_period": intent.params.get("signal_period").copied().unwrap_or(9.0),
            "histogram_threshold": intent.params.get("histogram_threshold").copied().unwrap_or(0.0)
        }),
        IntentKind::Momentum => serde_json::json!({
            "lookback": intent.params.get("lookback").copied().unwrap_or(10.0),
            "threshold_ratio": intent.params.get("threshold_ratio").copied().unwrap_or(DEFAULT_MOMENTUM_THRESHOLD)
        }),
        IntentKind::ZScore => serde_json::json!({
            "window": intent.params.get("window").copied().unwrap_or(20.0),
            "entry_z": intent.params.get("entry_z").copied().unwrap_or(2.0)
        }),
        IntentKind::QuoteObserve => serde_json::json!({
            "max_time_diff_ms": intent.params.get("max_time_diff_ms").copied().unwrap_or(5_000.0),
            "field_code": intent.params.get("field_code").copied().unwrap_or(0.0),
            "align_direction_code": intent.params.get("align_direction_code").copied().unwrap_or(0.0),
            "resample_period_ms": intent.params.get("resample_period_ms").copied().unwrap_or(0.0),
            "resample_agg_code": intent.params.get("resample_agg_code").copied().unwrap_or(0.0),
            "window_size": intent.params.get("window_size").copied().unwrap_or(1.0),
            "window_agg_code": intent.params.get("window_agg_code").copied().unwrap_or(1.0),
            "spread_output_code": intent.params.get("spread_output_code").copied().unwrap_or(0.0),
            "comparison_shape_code": intent.params.get("comparison_shape_code").copied().unwrap_or(0.0),
            "comparison_op_code": intent.params.get("comparison_op_code").copied().unwrap_or(0.0),
            "comparison_threshold": intent.params.get("comparison_threshold").copied().unwrap_or(0.0)
        }),
        IntentKind::SmaCrossover => serde_json::json!({
            "fast_period": intent.params.get("fast_period").copied().unwrap_or(20.0),
            "slow_period": intent.params.get("slow_period").copied().unwrap_or(50.0),
            "entry_ratio": intent.params.get("entry_ratio").copied().unwrap_or(0.2)
        }),
    }
}

fn runtime_target_node_id(
    runtime_targets: &CompileRuntimeTargets,
    runtime_id: &str,
    fallback: &str,
) -> String {
    runtime_targets
        .source_to_node
        .get(runtime_id)
        .cloned()
        .unwrap_or_else(|| fallback.to_string())
}

fn format_exchange_name(exchange: &Exchange) -> &'static str {
    match exchange {
        Exchange::Binance => "binance",
        Exchange::Okx => "okx",
    }
}

fn format_symbol_name(symbol: &Symbol) -> String {
    symbol.as_str().to_string()
}




pub(super) fn config_str<'a>(config: &'a Value, key: &str) -> Option<&'a str> {
    config.get(key)?.as_str()
}


pub(super) fn config_f64(config: &Value, key: &str) -> Option<f64> {
    config.get(key)?.as_f64()
}


fn format_rebalance_symbols(symbols: &[Symbol]) -> String {
    symbols
        .iter()
        .map(|symbol| symbol.as_str().to_string())
        .collect::<Vec<_>>()
        .join(", ")
}

fn format_rebalance_schedule(schedule: Option<&RebalanceSchedule>) -> String {
    match schedule {
        Some(RebalanceSchedule::EverySlow) => "every_slow".to_string(),
        Some(RebalanceSchedule::Every1d) => "every_1d".to_string(),
        Some(RebalanceSchedule::Weekly) => "weekly".to_string(),
        None => String::new(),
    }
}

fn format_rebalance_target_weights(weights: &[f64]) -> String {
    weights
        .iter()
        .map(|value| {
            let mut rendered = format!("{value}");
            if rendered.contains('.') {
                while rendered.ends_with('0') {
                    rendered.pop();
                }
                if rendered.ends_with('.') {
                    rendered.push('0');
                }
            }
            rendered
        })
        .collect::<Vec<_>>()
        .join(", ")
}


#[cfg(test)]
mod tests {
    use super::*;
    use qrpc_core::{AgentConfig, RebalanceSchedule, Symbol};
    use serde_json::json;

    fn sample_runtime_config() -> FrontendRuntimeConfig {
        FrontendRuntimeConfig {
            metadata: FrontendMetadata {
                graph_id: "graph_multi".to_string(),
                compile_id: "compile_multi".to_string(),
                name: "Multi Symbol".to_string(),
                version: "1.0.0".to_string(),
                mode: "paper".to_string(),
            },
            data_sources: vec![FrontendNodeConfig {
                id: "data_data_1".to_string(),
                module_key: "builtin.data.kline".to_string(),
                name: "Data".to_string(),
                config: json!({
                    "exchange": "binance",
                    "instrument": "BTCUSDT",
                    "timeframe": "1d",
                    "window_size": 200
                }),
            }],
            intent_generators: vec![FrontendIntentConfig {
                id: "intent_intent_1".to_string(),
                module_key: "builtin.intent.double_ma".to_string(),
                name: "Intent".to_string(),
                config: json!({
                    "fast_period": 20.0,
                    "slow_period": 50.0,
                    "entry_ratio": 0.2
                }),
                input_refs: vec![FrontendInputRef {
                    source_id: "data_data_1".to_string(),
                    source_port: "market_data_out".to_string(),
                    target_port: "data_input".to_string(),
                }],
            }],
            agents: vec![FrontendAgentConfig {
                id: "agent_agent_1".to_string(),
                module_key: "builtin.agent.weighted".to_string(),
                name: "Agent".to_string(),
                config: json!({
                    "decision_threshold": 0.05,
                    "max_quantity_ratio": 0.4,
                    "rebalance_symbols": "BTCUSDT, ETHUSDT, SOLUSDT",
                    "rebalance_schedule": "weekly",
                    "rebalance_allocation_kind": "rank_weight",
                    "rebalance_rank_method": "inverse_rank",
                    "rebalance_score_normalize": "sum",
                    "rebalance_target_weights": "0.5, 0.3, 0.2"
                }),
                intent_refs: vec!["intent_intent_1".to_string()],
            }],
            risk_controls: vec![FrontendRiskConfig {
                id: "risk_risk_1".to_string(),
                module_key: "builtin.risk.global".to_string(),
                name: "Risk".to_string(),
                config: json!({
                    "max_position": 0.2,
                    "max_concentration": 0.25,
                    "max_symbol_net_exposure": 0.22,
                    "max_portfolio_net_exposure": 0.45,
                    "max_turnover": 0.4,
                    "min_trade_weight": 0.02,
                    "max_new_positions_per_rebalance": 2,
                    "max_total_leverage": 3.0,
                    "max_exchange_leverage": 3.0,
                    "min_action_interval_ms": 100
                }),
                agent_refs: vec!["agent_agent_1".to_string()],
            }],
            executions: vec![FrontendExecutionConfig {
                id: "execution_execution_1".to_string(),
                module_key: "builtin.execution.paper".to_string(),
                name: "Execution".to_string(),
                config: json!({
                    "profile_id": "paper",
                    "mode": "paper",
                    "slippage_bps": 5.0
                }),
                risk_ref: Some("risk_risk_1".to_string()),
            }],
            runtime_control: Some(FrontendNodeConfig {
                id: "runtime_runtime_1".to_string(),
                module_key: "builtin.runtime.control".to_string(),
                name: "Runtime".to_string(),
                config: json!({
                    "mode": "paper"
                }),
            }),
        }
    }


    #[test]
    fn frontend_runtime_config_from_core_restores_multi_symbol_agent_fields() {
        let template = sample_runtime_config();
        let runtime_targets = CompileRuntimeTargets::default();
        let runtime_protocol = RuntimeProtocolCoreConfig {
            data_sources: vec![],
            intents: vec![],
            agents: vec![AgentConfig {
                agent_id: "agent_agent_1".to_string(),
                name: "Agent".to_string(),
                input_intent_ids: vec![],
                rebalance_symbols: vec![Symbol::parse("BTCUSDT"), Symbol::parse("ETHUSDT")],
                rebalance_schedule: Some(RebalanceSchedule::Every1d),
                rebalance_allocation_kind: Some("score_weight".to_string()),
                rebalance_rank_method: Some("linear".to_string()),
                rebalance_score_normalize: Some("sum".to_string()),
                rebalance_target_weights: vec![0.6, 0.4],
                params: BTreeMap::from([
                    ("decision_threshold".to_string(), 0.05),
                    ("max_quantity_ratio".to_string(), 0.8),
                ]),
                enabled: true,
            }],
            risks: vec![RiskConfig {
                risk_id: "risk_risk_1".to_string(),
                name: "Risk".to_string(),
                observed_agent_ids: vec!["agent_agent_1".to_string()],
                max_position_ratio: 0.2,
                max_single_weight: Some(0.2),
                max_concentration_ratio: Some(0.25),
                max_symbol_net_exposure_ratio: Some(0.22),
                max_portfolio_net_exposure_ratio: Some(0.45),
                max_turnover: Some(0.4),
                min_trade_weight: Some(0.02),
                max_new_positions_per_rebalance: Some(2),
                max_total_leverage: 3.0,
                max_exchange_leverage: 3.0,
                min_action_interval_ms: 100,
                enabled: true,
            }],
            initial_cash_balance: 100_000.0,
            taker_fee_bps: 0.0,
            default_slippage_bps: 5.0,
            total_cost_buffer_bps: 20.0,
        };

        let frontend = frontend_runtime_config_from_core_with_template(
            &runtime_protocol,
            &template,
            &runtime_targets,
            "graph_multi",
            "compile_multi",
        );

        let config = &frontend.agents[0].config;
        assert_eq!(config["rebalance_symbols"], "BTCUSDT, ETHUSDT");
        assert_eq!(config["rebalance_schedule"], "every_1d");
        assert_eq!(config["rebalance_allocation_kind"], "score_weight");
        assert_eq!(config["rebalance_rank_method"], "linear");
        assert_eq!(config["rebalance_score_normalize"], "sum");
        assert_eq!(config["rebalance_target_weights"], "0.6, 0.4");
        let risk_config = &frontend.risk_controls[0].config;
        assert_eq!(risk_config["max_concentration"], 0.25);
        assert_eq!(risk_config["max_symbol_net_exposure"], 0.22);
        assert_eq!(risk_config["max_portfolio_net_exposure"], 0.45);
        assert_eq!(risk_config["max_turnover"], 0.4);
        assert_eq!(risk_config["min_trade_weight"], 0.02);
        assert_eq!(risk_config["max_new_positions_per_rebalance"], 2);
    }
}
