use super::timeline::{KlineProvider, QuoteProvider, ResampleKlineProvider, TimelineDataProvider, UnifiedTimeline};
use crate::data_module::data_sources_from_core_ir;
use anyhow::{anyhow, Result};
use qrpc_core::{CoreStrategyIr, DataKind};
use std::sync::Arc;

/// 从 CoreStrategyIr 构建统一时间轴所需的 K 线数据提供者列表
pub fn build_kline_providers(
    core_ir: &CoreStrategyIr,
    end_ms: u64,
) -> Result<Vec<KlineProvider>> {
    use crate::data_module::historical_kline_bars_for_backtest;

    let sources = data_sources_from_core_ir(core_ir);
    let kline_sources: Vec<_> = sources
        .iter()
        .filter(|s| s.enabled && matches!(s.kind, DataKind::KlineSeries))
        .cloned()
        .collect();

    if kline_sources.is_empty() {
        return Err(anyhow!("回测需要至少一个启用的 K 线数据源"));
    }

    kline_sources
        .iter()
        .map(|source| {
            let bars = historical_kline_bars_for_backtest(source, end_ms)?;
            if bars.is_empty() {
                return Err(anyhow!("数据源 {} 没有可用的历史 K 线数据", source.data_id));
            }
            Ok(KlineProvider::new(source, bars))
        })
        .collect()
}

/// 从 CoreStrategyIr 构建统一时间轴所需的报价数据提供者列表
///
/// 如果存在真实的 Quote 数据绑定且数据可用，使用真实报价；
/// 否则从对应的 K 线 close 值生成合成报价。
pub fn build_quote_providers(
    core_ir: &CoreStrategyIr,
    kline_providers: &[KlineProvider],
) -> Vec<QuoteProvider> {
    let sources = data_sources_from_core_ir(core_ir);
    let quote_sources: Vec<_> = sources
        .iter()
        .filter(|s| s.enabled && matches!(s.kind, DataKind::Quote))
        .cloned()
        .collect();

    quote_sources
        .iter()
        .map(|source| {
            // 尝试找到同一 exchange+symbol 的 K 线提供者来做 fallback
            let matching_kline = kline_providers.iter().find(|kp| {
                // 通过 data_id 前缀匹配（如 "binance_btc_quote" 匹配 "binance_btc_150d_1d"）
                let quote_base = source.data_id.replace("_quote", "");
                let kline_base = TimelineDataProvider::data_id(*kp).replace("_150d_1d", "").replace("_200d_1d", "");
                quote_base == kline_base
            });

            if let Some(kp) = matching_kline {
                QuoteProvider::from_kline_fallback(source, kp, 0)
            } else {
                // 没有匹配的 K 线数据源，使用第一个 K 线提供者（兜底）
                QuoteProvider::from_kline_fallback(source, &kline_providers[0], 0)
            }
        })
        .collect()
}

/// v1.1.1: 为亚日频 K 线源创建日频重采样提供者
fn build_resampled_providers(kline_providers: &[KlineProvider]) -> Vec<Arc<dyn TimelineDataProvider>> {
    let mut resampled: Vec<Arc<dyn TimelineDataProvider>> = Vec::new();
    for provider in kline_providers {
        let interval = &provider.interval;
        if interval == "1m" || interval == "5m" || interval == "15m"
            || interval == "30m" || interval == "1h" || interval == "4h"
        {
            let id = format!("{}_resampled_1d", TimelineDataProvider::data_id(provider));
            resampled.push(Arc::new(ResampleKlineProvider::new(&id, provider, "1d")));
        }
    }
    resampled
}

/// 从 CoreStrategyIr 构建完整的统一时间轴（包含数据提供者 + 自动日频重采样）
pub fn build_unified_timeline(
    core_ir: &CoreStrategyIr,
    end_ms: u64,
) -> Result<UnifiedTimeline> {
    let kline_providers = build_kline_providers(core_ir, end_ms)?;
    let resampled = build_resampled_providers(&kline_providers);
    let quote_providers = build_quote_providers(core_ir, &kline_providers);
    let mut all: Vec<Arc<dyn TimelineDataProvider>> = Vec::new();
    for kp in &kline_providers { all.push(Arc::new(kp.clone())); }
    all.extend(resampled);
    for qp in &quote_providers { all.push(Arc::new(qp.clone())); }
    UnifiedTimeline::from_providers(all)
}

/// 从 CoreStrategyIr 构建统一时间轴（使用 mock 数据 + 自动日频重采样）
pub fn build_mock_unified_timeline(
    core_ir: &CoreStrategyIr,
    end_ms: u64,
) -> Result<UnifiedTimeline> {
    use crate::data_module::mock_kline_bars_for_backtest;

    let sources = data_sources_from_core_ir(core_ir);
    let kline_sources: Vec<_> = sources
        .iter()
        .filter(|s| s.enabled && matches!(s.kind, DataKind::KlineSeries))
        .cloned()
        .collect();

    if kline_sources.is_empty() {
        return Err(anyhow!("回测需要至少一个启用的 K 线数据源"));
    }

    let mut kline_providers = Vec::new();
    for source in &kline_sources {
        let bars = mock_kline_bars_for_backtest(source, end_ms)?;
        kline_providers.push(KlineProvider::new(source, bars));
    }

    let resampled = build_resampled_providers(&kline_providers);
    let quote_providers = build_quote_providers(core_ir, &kline_providers);
    let mut all: Vec<Arc<dyn TimelineDataProvider>> = Vec::new();
    for kp in &kline_providers { all.push(Arc::new(kp.clone())); }
    all.extend(resampled);
    for qp in &quote_providers { all.push(Arc::new(qp.clone())); }
    UnifiedTimeline::from_providers(all)
}

#[cfg(test)]
mod tests {
    use super::*;
    use qrpc_core::{
        AgentConfig, DataSourceConfig, Exchange, IntentConfig, IntentKind, MarketType,
        RiskConfig, RuntimeProtocolCoreConfig, Symbol,
    };
    use qrpc_compiler::compile_runtime_protocol_config;
    use std::collections::BTreeMap;

    fn sample_config() -> RuntimeProtocolCoreConfig {
        RuntimeProtocolCoreConfig {
            data_sources: vec![DataSourceConfig {
                data_id: "binance_btc_150d_1d".into(),
                exchange: Exchange::Binance,
                symbol: Symbol::BtcUsdt,
                market_type: MarketType::Spot,
                kind: DataKind::KlineSeries,
                days: Some(150),
                interval: Some("1d".into()),
                ping_enabled: false,
                request_interval_ms: None,
                enabled: true,
            }],
            intents: vec![IntentConfig {
                intent_id: "intent_long_buy".into(),
                name: "Long Buy".into(),
                kind: IntentKind::LongTermBuy,
                input_data_ids: vec!["binance_btc_150d_1d".into()],
                params: BTreeMap::new(),
                enabled: true,
            }],
            agents: vec![AgentConfig {
                agent_id: "agent_long_term".into(),
                name: "Long Term Agent".into(),
                input_intent_ids: vec!["intent_long_buy".into()],
                rebalance_symbols: vec![],
                rebalance_schedule: None,
                rebalance_allocation_kind: None,
                rebalance_rank_method: None,
                rebalance_score_normalize: None,
                rebalance_target_weights: vec![],
                params: BTreeMap::new(),
                enabled: true,
            }],
            risks: vec![RiskConfig {
                risk_id: "risk_global".into(),
                name: "Global Risk".into(),
                observed_agent_ids: vec!["agent_long_term".into()],
                max_position_ratio: 0.2,
                max_single_weight: None,
                max_concentration_ratio: None,
                max_symbol_net_exposure_ratio: None,
                max_portfolio_net_exposure_ratio: None,
                max_turnover: None,
                min_trade_weight: None,
                max_new_positions_per_rebalance: None,
                max_total_leverage: 3.0,
                max_exchange_leverage: 3.0,
                min_action_interval_ms: 100,
                enabled: true,
            }],
            initial_cash_balance: 100_000.0,
            taker_fee_bps: 10.0,
            default_slippage_bps: 5.0,
            total_cost_buffer_bps: 20.0,
        }
    }

    #[test]
    fn mock_unified_timeline_produces_expected_step_count() {
        let compiled = compile_runtime_protocol_config(&sample_config()).unwrap();
        let timeline = build_mock_unified_timeline(&compiled.core_ir, 1_700_000_000_000).unwrap();
        assert!(!timeline.is_empty());
        assert!(!timeline.slow_triggers.is_empty());
    }

    #[test]
    fn unified_timeline_collects_data_at_each_step() {
        let compiled = compile_runtime_protocol_config(&sample_config()).unwrap();
        let timeline = build_mock_unified_timeline(&compiled.core_ir, 1_700_000_000_000).unwrap();
        for idx in &timeline.slow_triggers {
            let data = timeline.collect_at(*idx);
            assert!(!data.is_empty(), "每个慢周期触发点应有数据可用");
        }
    }
}
