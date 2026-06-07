use super::{KlineProvider, QuoteProvider, TimelineDataProvider};
use anyhow::{anyhow, Result};
use qrpc_core::{DataKind, NormalizedMarketData};
use std::collections::BTreeSet;
use std::sync::Arc;

/// 统一时间轴 — 合并所有数据源的时间戳，按时间顺序回放
#[derive(Debug, Clone)]
pub struct UnifiedTimeline {
    /// 全部去重排序后的时间戳
    pub timestamps: Vec<u64>,
    /// 慢周期触发索引（由 K 线收盘时间驱动）
    pub slow_triggers: Vec<usize>,
    /// 快周期触发索引（由报价时间驱动）
    #[allow(dead_code)]
    pub fast_triggers: Vec<usize>,
    /// 数据提供者列表
    pub providers: Vec<Arc<dyn TimelineDataProvider>>,
}

impl UnifiedTimeline {
    /// 从 K 线提供者和报价提供者构建统一时间轴
    pub fn new(
        kline_providers: &[KlineProvider],
        quote_providers: &[QuoteProvider],
    ) -> Result<Self> {
        if kline_providers.is_empty() && quote_providers.is_empty() {
            return Err(anyhow!("统一时间轴需要至少一个数据提供者"));
        }

        // 收集所有 K 线收盘时间戳（慢周期）
        let mut kline_close_set = BTreeSet::new();
        for provider in kline_providers {
            for ts in provider.close_timestamps() {
                kline_close_set.insert(ts);
            }
        }

        // 收集所有报价时间戳（快周期）
        let mut quote_ts_set = BTreeSet::new();
        for provider in quote_providers {
            for ts in provider.timestamps() {
                quote_ts_set.insert(ts);
            }
        }

        // 合并所有时间戳
        let mut all_ts: BTreeSet<u64> = BTreeSet::new();
        all_ts.extend(&kline_close_set);
        all_ts.extend(&quote_ts_set);

        let timestamps: Vec<u64> = all_ts.into_iter().collect();

        // 确定慢/快周期触发索引
        let mut slow_triggers = Vec::new();
        let mut fast_triggers = Vec::new();
        for (idx, ts) in timestamps.iter().enumerate() {
            if kline_close_set.contains(ts) {
                slow_triggers.push(idx);
            }
            if quote_ts_set.contains(ts) {
                fast_triggers.push(idx);
            }
        }

        // 构建提供者列表
        let mut providers: Vec<Arc<dyn TimelineDataProvider>> = Vec::new();
        for provider in kline_providers {
            providers.push(Arc::new(provider.clone()));
        }
        for provider in quote_providers {
            providers.push(Arc::new(provider.clone()));
        }

        Ok(Self {
            timestamps,
            slow_triggers,
            fast_triggers,
            providers,
        })
    }

    /// v1.1.1: 从统一提供者列表构建时间轴（支持 Kline + ResampleKline + Quote）
    pub fn from_providers(providers: Vec<Arc<dyn TimelineDataProvider>>) -> Result<Self> {
        if providers.is_empty() {
            return Err(anyhow!("统一时间轴需要至少一个数据提供者"));
        }

        let mut kline_close_set = BTreeSet::new();
        let mut quote_ts_set = BTreeSet::new();

        for p in &providers {
            match p.kind() {
                DataKind::KlineSeries => {
                    for ts in p.timestamps() {
                        kline_close_set.insert(ts);
                    }
                }
                DataKind::Quote => {
                    for ts in p.timestamps() {
                        quote_ts_set.insert(ts);
                    }
                }
            }
        }

        let mut all_ts: BTreeSet<u64> = BTreeSet::new();
        all_ts.extend(&kline_close_set);
        all_ts.extend(&quote_ts_set);
        let timestamps: Vec<u64> = all_ts.into_iter().collect();

        let mut slow_triggers = Vec::new();
        let mut fast_triggers = Vec::new();
        for (idx, ts) in timestamps.iter().enumerate() {
            if kline_close_set.contains(ts) {
                slow_triggers.push(idx);
            }
            if quote_ts_set.contains(ts) {
                fast_triggers.push(idx);
            }
        }

        Ok(Self {
            timestamps,
            slow_triggers,
            fast_triggers,
            providers,
        })
    }

    /// 获取某个时间索引的全部数据快照
    pub fn collect_at(&self, ts_idx: usize) -> Vec<NormalizedMarketData> {
        let ts_ms = self.timestamps.get(ts_idx).copied().unwrap_or(0);
        self.providers
            .iter()
            .filter_map(|p| p.value_at(ts_ms))
            .collect()
    }

    pub fn len(&self) -> usize {
        self.timestamps.len()
    }

    pub fn is_empty(&self) -> bool {
        self.timestamps.is_empty()
    }
}
