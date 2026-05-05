use super::*;
use qrpc_core::{RebalanceSchedule, Symbol};

pub(super) fn compile_runtime_targets_from_mapped(
    mapped: &MappedRuntimeConfig,
) -> CompileRuntimeTargets {
    CompileRuntimeTargets {
        source_to_node: mapped.source_to_node.clone(),
        runtime_node_id: mapped.runtime_node_id.clone(),
        execution_node_id: mapped.execution_node_id.clone(),
    }
}

pub(super) fn merge_runtime_targets(
    provided: &CompileRuntimeTargets,
    mapped: &MappedRuntimeConfig,
) -> CompileRuntimeTargets {
    let fallback = compile_runtime_targets_from_mapped(mapped);
    let mut source_to_node = fallback.source_to_node;
    source_to_node.extend(provided.source_to_node.clone());
    CompileRuntimeTargets {
        source_to_node,
        runtime_node_id: provided
            .runtime_node_id
            .clone()
            .or(fallback.runtime_node_id),
        execution_node_id: provided
            .execution_node_id
            .clone()
            .or(fallback.execution_node_id),
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
            "threshold_ratio": intent.params.get("threshold_ratio").copied().unwrap_or(0.02)
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

pub(super) struct MappedRuntimeConfig {
    pub(super) runtime_protocol: RuntimeProtocolCoreConfig,
    pub(super) source_to_node: BTreeMap<String, String>,
    pub(super) runtime_node_id: Option<String>,
    pub(super) execution_node_id: Option<String>,
}

pub(super) fn map_frontend_runtime_config(
    input: &FrontendRuntimeConfig,
) -> anyhow::Result<MappedRuntimeConfig> {
    let _ = (&input.metadata.name, &input.metadata.version);
    if !matches!(input.metadata.mode.as_str(), "paper") {
        anyhow::bail!("不支持的运行时模式: {}", input.metadata.mode);
    }
    if input.runtime_control.is_none() {
        anyhow::bail!("缺少运行时控制节点");
    }
    if input.executions.len() != 1 {
        anyhow::bail!("当前 beta 版本需要且仅需要一个执行节点");
    }
    let mut source_to_node = BTreeMap::new();
    let mut data_sources = Vec::new();

    for data in &input.data_sources {
        let _ = &data.name;
        let exchange = parse_exchange(config_str(&data.config, "exchange").unwrap_or("binance"))?;
        let symbol = parse_symbol(config_str(&data.config, "instrument").unwrap_or("BTCUSDT"))?;
        let ping_enabled = config_bool(&data.config, "ping_enabled").unwrap_or(false);
        let request_interval_ms =
            config_u64(&data.config, "request_interval_ms").filter(|value| *value > 0);
        let source = match data.module_key.as_str() {
            "builtin.data.kline" => DataSourceConfig {
                data_id: data.id.clone(),
                exchange: exchange.clone(),
                symbol,
                market_type: MarketType::Spot,
                kind: DataKind::KlineSeries,
                days: Some(
                    config_u32(&data.config, "window_size")
                        .unwrap_or(200)
                        .max(150),
                ),
                interval: Some(
                    config_str(&data.config, "timeframe")
                        .unwrap_or("1d")
                        .to_string(),
                ),
                ping_enabled,
                request_interval_ms,
                enabled: true,
            },
            "builtin.data.quote" => DataSourceConfig {
                data_id: data.id.clone(),
                exchange: exchange.clone(),
                symbol,
                market_type: MarketType::Spot,
                kind: DataKind::Quote,
                days: None,
                interval: None,
                ping_enabled,
                request_interval_ms,
                enabled: true,
            },
            _ => anyhow::bail!("不支持的数据模块: {}", data.module_key),
        };
        source_to_node.insert(source.data_id.clone(), original_node_id(&data.id));
        data_sources.push(source);
    }

    let mut frontend_intent_to_runtime_id = BTreeMap::new();
    let mut intents = Vec::new();

    for intent in &input.intent_generators {
        let _ = (
            &intent.config,
            intent
                .input_refs
                .iter()
                .map(|item| (&item.source_port, &item.target_port))
                .collect::<Vec<_>>(),
        );
        let input_data_ids = intent
            .input_refs
            .iter()
            .map(|item| item.source_id.clone())
            .collect::<Vec<_>>();
        let kind = match intent.module_key.as_str() {
            "builtin.intent.double_ma" => IntentKind::LongTermBuy,
            "builtin.intent.ma_deviation" => IntentKind::LongTermSell,
            "builtin.intent.rsi" => IntentKind::Rsi,
            "builtin.intent.macd" => IntentKind::Macd,
            "builtin.intent.momentum" => IntentKind::Momentum,
            "builtin.intent.zscore" => IntentKind::ZScore,
            "builtin.intent.spread_observer" => IntentKind::QuoteObserve,
            _ => anyhow::bail!("不支持的意图模块: {}", intent.module_key),
        };
        let intent_id = intent.id.clone();
        let mut params = config_number_map(&intent.config);
        augment_frontend_intent_params(&intent.module_key, &mut params);

        frontend_intent_to_runtime_id.insert(intent.id.clone(), intent_id.clone());
        source_to_node.insert(intent_id.clone(), original_node_id(&intent.id));
        intents.push(IntentConfig {
            intent_id,
            name: intent.name.clone(),
            kind,
            input_data_ids,
            params,
            enabled: true,
        });
    }

    let mut frontend_agent_to_runtime_id = BTreeMap::new();
    let mut agents = Vec::new();
    for agent in &input.agents {
        let _ = &agent.config;
        let agent_id = match agent.module_key.as_str() {
            "builtin.agent.weighted" => agent.id.clone(),
            "builtin.agent.arbitrage" => agent.id.clone(),
            _ => anyhow::bail!("不支持的代理模块: {}", agent.module_key),
        };

        let input_intent_ids = agent
            .intent_refs
            .iter()
            .filter_map(|item| frontend_intent_to_runtime_id.get(item).cloned())
            .collect::<Vec<_>>();

        frontend_agent_to_runtime_id.insert(agent.id.clone(), agent_id.clone());
        source_to_node.insert(agent_id.clone(), original_node_id(&agent.id));
        agents.push(AgentConfig {
            agent_id,
            name: agent.name.clone(),
            input_intent_ids,
            rebalance_symbols: config_symbol_list(&agent.config, "rebalance_symbols")?,
            rebalance_schedule: parse_rebalance_schedule(&agent.config)?,
            rebalance_allocation_kind: config_optional_string(
                &agent.config,
                "rebalance_allocation_kind",
            ),
            rebalance_rank_method: config_optional_string(&agent.config, "rebalance_rank_method"),
            rebalance_score_normalize: config_optional_string(
                &agent.config,
                "rebalance_score_normalize",
            ),
            rebalance_target_weights: config_number_list(
                &agent.config,
                "rebalance_target_weights",
            )?,
            params: config_number_map(&agent.config),
            enabled: true,
        });
    }

    let mut risks = Vec::new();
    for risk in &input.risk_controls {
        let risk_id = match risk.module_key.as_str() {
            "builtin.risk.global" => risk.id.clone(),
            _ => anyhow::bail!("不支持的风险模块: {}", risk.module_key),
        };
        if let Some(profile_id) = config_string(&risk.config, "profile_id") {
            if profile_id != GLOBAL_RISK_PROFILE_ID {
                anyhow::bail!("不支持的风险配置 ID: {}", profile_id);
            }
        }

        let observed_agent_ids = risk
            .agent_refs
            .iter()
            .filter_map(|item| frontend_agent_to_runtime_id.get(item).cloned())
            .collect::<Vec<_>>();

        source_to_node.insert(risk_id.clone(), original_node_id(&risk.id));
        risks.push(RiskConfig {
            risk_id,
            name: risk.name.clone(),
            observed_agent_ids,
            max_position_ratio: config_f64(&risk.config, "max_position")
                .unwrap_or(GLOBAL_RISK_PROFILE_DEFAULT_MAX_POSITION),
            max_single_weight: config_f64(&risk.config, "max_single_weight"),
            max_concentration_ratio: config_f64(&risk.config, "max_concentration"),
            max_symbol_net_exposure_ratio: config_f64(&risk.config, "max_symbol_net_exposure"),
            max_portfolio_net_exposure_ratio: config_f64(
                &risk.config,
                "max_portfolio_net_exposure",
            ),
            max_turnover: config_f64(&risk.config, "max_turnover"),
            min_trade_weight: config_f64(&risk.config, "min_trade_weight"),
            max_new_positions_per_rebalance: config_u64(
                &risk.config,
                "max_new_positions_per_rebalance",
            )
            .map(|value| value as u32),
            max_total_leverage: config_f64(&risk.config, "max_total_leverage")
                .unwrap_or(GLOBAL_RISK_PROFILE_DEFAULT_MAX_TOTAL_LEVERAGE),
            max_exchange_leverage: config_f64(&risk.config, "max_exchange_leverage")
                .unwrap_or(GLOBAL_RISK_PROFILE_DEFAULT_MAX_EXCHANGE_LEVERAGE),
            min_action_interval_ms: config_u64(&risk.config, "min_action_interval_ms")
                .unwrap_or(GLOBAL_RISK_PROFILE_DEFAULT_MIN_ACTION_INTERVAL_MS),
            enabled: true,
        });
    }

    let execution_node_id = input
        .executions
        .first()
        .map(|item| original_node_id(&item.id));
    let runtime_node_id = input
        .runtime_control
        .as_ref()
        .map(|item| original_node_id(&item.id));

    for execution in &input.executions {
        let _ = (&execution.name, &execution.config, &execution.risk_ref);
        if execution.module_key != "builtin.execution.paper" {
            anyhow::bail!("不支持的执行模块: {}", execution.module_key);
        }
        if execution.risk_ref.is_none() {
            anyhow::bail!("执行节点 {} 缺少风险输入", execution.id);
        }
        if let Some(profile_id) = config_string(&execution.config, "profile_id") {
            if profile_id != PAPER_EXECUTION_PROFILE_ID {
                anyhow::bail!("不支持的执行配置 ID: {}", profile_id);
            }
        }
    }

    Ok(MappedRuntimeConfig {
        runtime_protocol: RuntimeProtocolCoreConfig {
            data_sources,
            intents,
            agents,
            risks,
            initial_cash_balance: 100_000.0,
            taker_fee_bps: first_number(&input.executions, "fee_bps")
                .unwrap_or(PAPER_EXECUTION_PROFILE_DEFAULT_FEE_BPS),
            default_slippage_bps: first_number(&input.executions, "slippage_bps")
                .unwrap_or(PAPER_EXECUTION_PROFILE_DEFAULT_SLIPPAGE_BPS),
            total_cost_buffer_bps: 20.0,
        },
        source_to_node,
        runtime_node_id,
        execution_node_id,
    })
}

fn augment_frontend_intent_params(module_key: &str, params: &mut BTreeMap<String, f64>) {
    if module_key == "builtin.intent.rsi" {
        let oversold = params.get("oversold_threshold").copied().unwrap_or(30.0);
        let overbought = params.get("overbought_threshold").copied().unwrap_or(70.0);
        if (overbought - 70.0).abs() <= f64::EPSILON && (oversold - 30.0).abs() > f64::EPSILON {
            params
                .entry("comparison_shape_code".to_string())
                .or_insert(1.0);
            params
                .entry("comparison_op_code".to_string())
                .or_insert(0.0);
        } else if (oversold - 30.0).abs() <= f64::EPSILON
            && (overbought - 70.0).abs() > f64::EPSILON
        {
            params
                .entry("comparison_shape_code".to_string())
                .or_insert(2.0);
            params
                .entry("comparison_op_code".to_string())
                .or_insert(2.0);
        }
    } else if module_key == "builtin.intent.momentum" {
        if let Some(threshold) = params
            .get("threshold_ratio")
            .copied()
            .filter(|value| *value > 0.0)
        {
            params
                .entry("comparison_shape_code".to_string())
                .or_insert(1.0);
            params
                .entry("comparison_op_code".to_string())
                .or_insert(2.0);
            params
                .entry("comparison_threshold".to_string())
                .or_insert(threshold);
        }
    } else if module_key == "builtin.intent.zscore" {
        if let Some(entry_z) = params.get("entry_z").copied().filter(|value| *value > 0.0) {
            params
                .entry("comparison_shape_code".to_string())
                .or_insert(1.0);
            params
                .entry("comparison_op_code".to_string())
                .or_insert(0.0);
            params
                .entry("comparison_threshold".to_string())
                .or_insert(-entry_z);
        }
    }
}

pub(super) fn config_str<'a>(config: &'a Value, key: &str) -> Option<&'a str> {
    config.get(key)?.as_str()
}

fn config_u32(config: &Value, key: &str) -> Option<u32> {
    config.get(key)?.as_u64().map(|item| item as u32)
}

fn config_u64(config: &Value, key: &str) -> Option<u64> {
    config.get(key)?.as_u64()
}

fn config_bool(config: &Value, key: &str) -> Option<bool> {
    config.get(key)?.as_bool()
}

pub(super) fn config_f64(config: &Value, key: &str) -> Option<f64> {
    config.get(key)?.as_f64()
}

fn config_string(config: &Value, key: &str) -> Option<String> {
    config.get(key)?.as_str().map(|value| value.to_string())
}

fn config_optional_string(config: &Value, key: &str) -> Option<String> {
    config_string(config, key).and_then(|value| {
        let trimmed = value.trim();
        (!trimmed.is_empty()).then(|| trimmed.to_string())
    })
}

fn config_string_list(config: &Value, key: &str) -> Vec<String> {
    config_string(config, key)
        .map(|value| {
            value
                .split(',')
                .map(|item| item.trim())
                .filter(|item| !item.is_empty())
                .map(|item| item.to_string())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default()
}

fn config_symbol_list(config: &Value, key: &str) -> anyhow::Result<Vec<Symbol>> {
    config_string_list(config, key)
        .into_iter()
        .map(|value| parse_symbol(&value))
        .collect()
}

fn config_number_list(config: &Value, key: &str) -> anyhow::Result<Vec<f64>> {
    let Some(value) = config_optional_string(config, key) else {
        return Ok(Vec::new());
    };
    value
        .split(',')
        .map(|item| {
            let trimmed = item.trim();
            trimmed.parse::<f64>().map_err(|_| {
                anyhow::anyhow!(
                    "frontend agent config field `{}` must contain comma-separated numbers",
                    key
                )
            })
        })
        .collect()
}

fn parse_rebalance_schedule(config: &Value) -> anyhow::Result<Option<RebalanceSchedule>> {
    match config_optional_string(config, "rebalance_schedule").as_deref() {
        None => Ok(None),
        Some("every_slow") => Ok(Some(RebalanceSchedule::EverySlow)),
        Some("every_1d") => Ok(Some(RebalanceSchedule::Every1d)),
        Some("weekly") => Ok(Some(RebalanceSchedule::Weekly)),
        Some(other) => anyhow::bail!("不支持的重新平衡计划: {}", other),
    }
}

fn config_number_map(config: &Value) -> BTreeMap<String, f64> {
    let mut params = BTreeMap::new();
    if let Some(object) = config.as_object() {
        for (key, value) in object {
            if let Some(number) = value.as_f64() {
                params.insert(key.clone(), number);
            }
        }
    }
    params
}

fn first_number(items: &[FrontendExecutionConfig], key: &str) -> Option<f64> {
    items.first()?.config.get(key)?.as_f64()
}

fn parse_exchange(input: &str) -> anyhow::Result<Exchange> {
    match input.to_ascii_lowercase().as_str() {
        "binance" => Ok(Exchange::Binance),
        "okx" => Ok(Exchange::Okx),
        _ => anyhow::bail!("不支持的交易所: {}", input),
    }
}

fn parse_symbol(input: &str) -> anyhow::Result<Symbol> {
    let normalized = input.trim();
    if normalized.is_empty() {
        anyhow::bail!("标的不能为空");
    }
    Ok(Symbol::parse(normalized))
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

fn original_node_id(compiled_id: &str) -> String {
    compiled_id
        .split_once("_")
        .map(|(_, rest)| rest.to_string())
        .unwrap_or_else(|| compiled_id.to_string())
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
    fn map_frontend_runtime_config_threads_multi_symbol_rebalance_fields() {
        let mapped =
            map_frontend_runtime_config(&sample_runtime_config()).expect("mapping should succeed");

        let agent = &mapped.runtime_protocol.agents[0];
        assert_eq!(
            agent.rebalance_symbols,
            vec![
                Symbol::parse("BTCUSDT"),
                Symbol::parse("ETHUSDT"),
                Symbol::parse("SOLUSDT")
            ]
        );
        assert_eq!(agent.rebalance_schedule, Some(RebalanceSchedule::Weekly));
        assert_eq!(
            agent.rebalance_allocation_kind.as_deref(),
            Some("rank_weight")
        );
        assert_eq!(agent.rebalance_rank_method.as_deref(), Some("inverse_rank"));
        assert_eq!(agent.rebalance_score_normalize.as_deref(), Some("sum"));
        assert_eq!(agent.rebalance_target_weights, vec![0.5, 0.3, 0.2]);
        let risk = &mapped.runtime_protocol.risks[0];
        assert_eq!(risk.max_concentration_ratio, Some(0.25));
        assert_eq!(risk.max_symbol_net_exposure_ratio, Some(0.22));
        assert_eq!(risk.max_portfolio_net_exposure_ratio, Some(0.45));
        assert_eq!(risk.max_turnover, Some(0.4));
        assert_eq!(risk.min_trade_weight, Some(0.02));
        assert_eq!(risk.max_new_positions_per_rebalance, Some(2));
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
