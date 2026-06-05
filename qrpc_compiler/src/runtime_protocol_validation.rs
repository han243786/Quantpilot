use anyhow::{bail, Result};
use qrpc_core::{
    AgentConfig, DataKind, DataSourceConfig, IntentConfig, IntentKind, RiskConfig,
    RuntimeProtocolCoreConfig,
};
use std::collections::{BTreeMap, BTreeSet};

use super::runtime_intent_is_spread;

pub(super) fn validate_runtime_protocol_config(config: &RuntimeProtocolCoreConfig) -> Result<()> {
    if !config.initial_cash_balance.is_finite() || config.initial_cash_balance <= 0.0 {
        bail!("initial_cash_balance 必须大于 0");
    }
    if !config.taker_fee_bps.is_finite()
        || !config.default_slippage_bps.is_finite()
        || !config.total_cost_buffer_bps.is_finite()
        || config.taker_fee_bps < 0.0
        || config.default_slippage_bps < 0.0
        || config.total_cost_buffer_bps < 0.0
    {
        bail!("手续费和滑点参数必须是有限数且大于等于 0");
    }

    ensure_unique_ids(config)?;
    validate_data_sources(&config.data_sources)?;
    validate_intents(config)?;
    validate_agents(config)?;
    validate_risks(config)?;

    Ok(())
}

fn ensure_unique_ids(config: &RuntimeProtocolCoreConfig) -> Result<()> {
    let mut data_ids = BTreeSet::new();
    for source in &config.data_sources {
        if !data_ids.insert(source.data_id.clone()) {
            bail!("重复的 data_id: {}", source.data_id);
        }
    }

    let mut intent_ids = BTreeSet::new();
    for intent in &config.intents {
        if !intent_ids.insert(intent.intent_id.clone()) {
            bail!("重复的 intent_id: {}", intent.intent_id);
        }
    }

    let mut agent_ids = BTreeSet::new();
    for agent in &config.agents {
        if !agent_ids.insert(agent.agent_id.clone()) {
            bail!("重复的 agent_id: {}", agent.agent_id);
        }
    }

    let mut risk_ids = BTreeSet::new();
    for risk in &config.risks {
        if !risk_ids.insert(risk.risk_id.clone()) {
            bail!("重复的 risk_id: {}", risk.risk_id);
        }
    }

    Ok(())
}

fn validate_data_sources(data_sources: &[DataSourceConfig]) -> Result<()> {
    if data_sources.is_empty() {
        bail!("至少需要一个数据源");
    }

    for source in data_sources.iter().filter(|item| item.enabled) {
        if source.data_id.trim().is_empty() {
            bail!("data_id 不能为空");
        }
        match source.kind {
            DataKind::KlineSeries => {
                if source.days.unwrap_or_default() == 0 {
                    bail!("kline 数据源 {} 必须声明 days > 0", source.data_id);
                }
                if source
                    .interval
                    .as_deref()
                    .unwrap_or_default()
                    .trim()
                    .is_empty()
                {
                    bail!("kline 数据源 {} 必须声明 interval", source.data_id);
                }
            }
            DataKind::Quote => {}
        }
    }

    Ok(())
}

fn validate_intents(config: &RuntimeProtocolCoreConfig) -> Result<()> {
    let enabled_sources = config
        .data_sources
        .iter()
        .filter(|item| item.enabled)
        .map(|item| (&item.data_id, item.kind.clone()))
        .collect::<BTreeMap<_, _>>();

    if config.intents.is_empty() {
        bail!("至少需要一个意图");
    }

    for intent in config.intents.iter().filter(|item| item.enabled) {
        validate_intent(intent, &enabled_sources)?;
    }

    Ok(())
}

fn validate_intent(
    intent: &IntentConfig,
    enabled_sources: &BTreeMap<&String, DataKind>,
) -> Result<()> {
    if intent.intent_id.trim().is_empty() || intent.name.trim().is_empty() {
        bail!("意图 ID 和名称不能为空");
    }
    if intent.input_data_ids.is_empty() {
        bail!("意图 {} 必须声明 input_data_ids", intent.intent_id);
    }

    for data_id in &intent.input_data_ids {
        let Some(kind) = enabled_sources.get(data_id) else {
            bail!("意图 {} 引用了缺失的数据源 {}", intent.intent_id, data_id);
        };
        match intent.kind {
            IntentKind::LongTermBuy
            | IntentKind::LongTermSell
            | IntentKind::SmaCrossover
            | IntentKind::Rsi
            | IntentKind::Macd
            | IntentKind::Momentum
            | IntentKind::ZScore => {
                if !matches!(kind, DataKind::KlineSeries) {
                    bail!("意图 {} 期望 KlineSeries 输入", intent.intent_id);
                }
            }
            IntentKind::QuoteObserve => {
                if runtime_intent_is_spread(intent) {
                    if !matches!(kind, DataKind::Quote | DataKind::KlineSeries) {
                        bail!(
                            "意图 {} 期望 Quote 或 KlineSeries 输入用于 spread",
                            intent.intent_id
                        );
                    }
                } else if !matches!(kind, DataKind::Quote) {
                    bail!("意图 {} 期望 Quote 输入", intent.intent_id);
                }
            }
        }
    }

    if runtime_intent_is_spread(intent) && intent.input_data_ids.len() < 2 {
        bail!("意图 {} spread 观察需要至少两个输入", intent.intent_id);
    }

    Ok(())
}

fn validate_agents(config: &RuntimeProtocolCoreConfig) -> Result<()> {
    let intent_ids = config
        .intents
        .iter()
        .filter(|item| item.enabled)
        .map(|item| item.intent_id.as_str())
        .collect::<BTreeSet<_>>();

    if config.agents.is_empty() {
        bail!("至少需要一个代理");
    }

    for agent in config.agents.iter().filter(|item| item.enabled) {
        validate_agent(agent, &intent_ids)?;
    }

    Ok(())
}

fn validate_agent(agent: &AgentConfig, intent_ids: &BTreeSet<&str>) -> Result<()> {
    if agent.agent_id.trim().is_empty() || agent.name.trim().is_empty() {
        bail!("代理 ID 和名称不能为空");
    }
    if agent.input_intent_ids.is_empty() {
        bail!("代理 {} 必须声明 input_intent_ids", agent.agent_id);
    }
    for intent_id in &agent.input_intent_ids {
        if !intent_ids.contains(intent_id.as_str()) {
            bail!("代理 {} 引用了缺失的意图 {}", agent.agent_id, intent_id);
        }
    }
    Ok(())
}

fn validate_risks(config: &RuntimeProtocolCoreConfig) -> Result<()> {
    let agent_ids = config
        .agents
        .iter()
        .filter(|item| item.enabled)
        .map(|item| item.agent_id.as_str())
        .collect::<BTreeSet<_>>();

    if config.risks.is_empty() {
        bail!("至少需要一个风险配置");
    }

    let mut observed_once = BTreeMap::<&str, u32>::new();
    for risk in config.risks.iter().filter(|item| item.enabled) {
        validate_risk(risk, &agent_ids)?;
        for agent_id in &risk.observed_agent_ids {
            *observed_once.entry(agent_id.as_str()).or_default() += 1;
        }
    }

    for agent_id in agent_ids {
        if observed_once.get(agent_id).copied().unwrap_or_default() != 1 {
            bail!("已启用的代理 {} 必须被恰好一个已启用的风险观察", agent_id);
        }
    }

    Ok(())
}

fn validate_risk(risk: &RiskConfig, agent_ids: &BTreeSet<&str>) -> Result<()> {
    if risk.risk_id.trim().is_empty() || risk.name.trim().is_empty() {
        bail!("风险 ID 和名称不能为空");
    }
    if risk.observed_agent_ids.is_empty() {
        bail!("风险 {} 必须至少观察一个代理", risk.risk_id);
    }
    if !risk.max_total_leverage.is_finite()
        || risk.max_total_leverage <= 0.0
        || !risk.max_exchange_leverage.is_finite()
        || risk.max_exchange_leverage <= 0.0
    {
        bail!("风险 {} 杠杆限制必须大于 0", risk.risk_id);
    }
    for agent_id in &risk.observed_agent_ids {
        if !agent_ids.contains(agent_id.as_str()) {
            bail!("风险 {} 引用了缺失的代理 {}", risk.risk_id, agent_id);
        }
    }
    Ok(())
}
