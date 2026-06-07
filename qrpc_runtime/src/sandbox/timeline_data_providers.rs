#![allow(dead_code)]

use qrpc_core::{
    DataKind, DataSourceConfig, KlineSeriesSnapshot, NormalizedKline, NormalizedMarketData,
    QuoteSnapshot,
};

/// 时间轴数据提供者 trait — 按时间点提供 asof-join 后的数据快照
pub trait TimelineDataProvider: Send + Sync + std::fmt::Debug {
    fn data_id(&self) -> &str;
    fn kind(&self) -> DataKind;
    fn value_at(&self, ts_ms: u64) -> Option<NormalizedMarketData>;
    /// v1.1.1: 返回此提供者拥有的全部时间戳（用于构建统一时间轴）
    fn timestamps(&self) -> Vec<u64> {
        Vec::new()
    }
}

/// K 线数据提供者 — 从 NormalizedKline 序列按 asof 提供 KlineSeriesSnapshot
#[derive(Debug, Clone)]
pub struct KlineProvider {
    data_id: String,
    #[allow(dead_code)]
    exchange: qrpc_core::Exchange,
    #[allow(dead_code)]
    symbol: qrpc_core::Symbol,
    pub(crate) interval: String,
    bars: Vec<NormalizedKline>,
}

impl KlineProvider {
    pub fn new(source: &DataSourceConfig, bars: Vec<NormalizedKline>) -> Self {
        Self {
            data_id: source.data_id.clone(),
            exchange: source.exchange.clone(),
            symbol: source.symbol.clone(),
            interval: source.interval.clone().unwrap_or_else(|| "1d".into()),
            bars,
        }
    }

    /// 返回所有 K 线收盘时间戳（用于构建统一时间轴的慢周期触发点）
    pub fn close_timestamps(&self) -> Vec<u64> {
        self.bars.iter().map(|bar| bar.close_time_ms).collect()
    }

    /// 返回所有 K 线开盘时间戳（用于构建统一时间轴的快周期触发点）
    pub fn open_timestamps(&self) -> Vec<u64> {
        self.bars.iter().map(|bar| bar.open_time_ms).collect()
    }

    pub fn bar_count(&self) -> usize {
        self.bars.len()
    }
}

impl TimelineDataProvider for KlineProvider {
    fn data_id(&self) -> &str {
        &self.data_id
    }
    fn kind(&self) -> DataKind {
        DataKind::KlineSeries
    }
    fn timestamps(&self) -> Vec<u64> {
        self.close_timestamps()
    }

    fn value_at(&self, ts_ms: u64) -> Option<NormalizedMarketData> {
        // v1.2.0: 滑动窗口截断为最近500条，解决 O(N²) 内存
        const MAX_WINDOW_BARS: usize = 500;
        let window: Vec<NormalizedKline> = self
            .bars
            .iter()
            .take_while(|bar| bar.close_time_ms <= ts_ms)
            .cloned()
            .collect();
        if window.is_empty() {
            return None;
        }
        let skip = window.len().saturating_sub(MAX_WINDOW_BARS);
        let window: Vec<NormalizedKline> = window.into_iter().skip(skip).collect();
        let window_len = window.len();
        let last_close = window.last().map(|bar| bar.close_time_ms).unwrap_or(ts_ms);
        Some(NormalizedMarketData::KlineSeries(KlineSeriesSnapshot {
            data_id: self.data_id.clone(),
            exchange: self.exchange.clone(),
            symbol: self.symbol.clone(),
            market_type: qrpc_core::MarketType::Spot,
            interval: self.interval.clone(),
            bars: window,
            window_len,
            ts_ms: last_close,
            source_latency_ms: 0,
            source_status: qrpc_core::SourceStatus::Healthy,
            data_quality: qrpc_core::DataQualitySnapshot::default(),
        }))
    }
}

/// 报价数据提供者 — 从 QuoteSnapshot 序列按 asof 提供
#[derive(Debug, Clone)]
pub struct QuoteProvider {
    data_id: String,
    exchange: qrpc_core::Exchange,
    symbol: qrpc_core::Symbol,
    quotes: Vec<(u64, QuoteSnapshot)>, // (ts_ms, snapshot)
}

impl QuoteProvider {
    pub fn new(source: &DataSourceConfig, quotes: Vec<(u64, QuoteSnapshot)>) -> Self {
        Self {
            data_id: source.data_id.clone(),
            exchange: source.exchange.clone(),
            symbol: source.symbol.clone(),
            quotes,
        }
    }

    pub fn from_kline_fallback(
        source: &DataSourceConfig,
        kline_provider: &KlineProvider,
        _end_ms: u64,
    ) -> Self {
        // 从 K 线 close 值生成合成报价（用于回测环境无真实报价数据时）
        let quotes: Vec<(u64, QuoteSnapshot)> = kline_provider
            .bars
            .iter()
            .map(|bar| {
                let mid = bar.close;
                let ts = bar.close_time_ms;
                (
                    ts,
                    QuoteSnapshot {
                        data_id: source.data_id.clone(),
                        exchange: source.exchange.clone(),
                        symbol: source.symbol.clone(),
                        market_type: qrpc_core::MarketType::Spot,
                        best_bid: mid * 0.9999,
                        best_ask: mid * 1.0001,
                        bid_size: 100.0,
                        ask_size: 100.0,
                        mid_price: mid,
                        ts_ms: ts,
                        source_latency_ms: 0,
                        source_status: qrpc_core::SourceStatus::Healthy,
                        data_quality: qrpc_core::DataQualitySnapshot::default(),
                    },
                )
            })
            .collect();
        Self {
            data_id: source.data_id.clone(),
            exchange: source.exchange.clone(),
            symbol: source.symbol.clone(),
            quotes,
        }
    }

    pub fn timestamps(&self) -> Vec<u64> {
        self.quotes.iter().map(|(ts, _)| *ts).collect()
    }
}

impl TimelineDataProvider for QuoteProvider {
    fn data_id(&self) -> &str {
        &self.data_id
    }
    fn kind(&self) -> DataKind {
        DataKind::Quote
    }
    fn timestamps(&self) -> Vec<u64> {
        self.quotes.iter().map(|(ts, _)| *ts).collect()
    }

    fn value_at(&self, ts_ms: u64) -> Option<NormalizedMarketData> {
        self.quotes
            .iter()
            .rev()
            .find(|(quote_ts, _)| *quote_ts <= ts_ms)
            .map(|(_, quote)| NormalizedMarketData::Quote(quote.clone()))
    }
}

/// v1.1.0: 高频 K 线重采样为低频 K 线提供者
///
/// 将源提供者（如 1h BTC）的 K 线按目标时间框架聚合为低频柱（如 1d）。
/// 聚合逻辑: Open=首柱开盘, High=最高, Low=最低, Close=末柱收盘, Volume=总和。
#[derive(Debug, Clone)]
pub struct ResampleKlineProvider {
    data_id: String,
    exchange: qrpc_core::Exchange,
    symbol: qrpc_core::Symbol,
    target_interval: String,
    target_interval_ms: u64,
    /// 预计算的重采样柱列表: (close_time_ms, resampled_bar)
    pub(crate) resampled_bars: Vec<(u64, NormalizedKline)>,
}

impl ResampleKlineProvider {
    pub fn new(data_id: &str, source: &KlineProvider, target_interval: &str) -> Self {
        let target_interval_ms = bar_interval_ms(target_interval);
        let source_interval_ms = bar_interval_ms(&source.interval);

        // 如果源频率已经 ≤ 目标频率，不需要重采样（直接复制）
        if source_interval_ms >= target_interval_ms {
            let bars: Vec<(u64, NormalizedKline)> = source
                .bars
                .iter()
                .map(|b| (b.close_time_ms, b.clone()))
                .collect();
            return Self {
                data_id: data_id.to_string(),
                exchange: source.exchange.clone(),
                symbol: source.symbol.clone(),
                target_interval: target_interval.to_string(),
                target_interval_ms,
                resampled_bars: bars,
            };
        }

        // 将源 bar 按目标时间桶分组
        let mut resampled = Vec::new();
        let mut bucket_start = source.bars.first().map(|b| b.open_time_ms).unwrap_or(0);
        let mut bucket_open = 0.0;
        let mut bucket_high = f64::NEG_INFINITY;
        let mut bucket_low = f64::INFINITY;
        let mut bucket_close = 0.0;
        let mut bucket_volume = 0.0;
        let mut bar_count = 0u64;

        for bar in &source.bars {
            if bar.open_time_ms >= bucket_start + target_interval_ms {
                // 完成当前桶
                if bar_count > 0 {
                    resampled.push((
                        bucket_start + target_interval_ms,
                        NormalizedKline {
                            open_time_ms: bucket_start,
                            close_time_ms: bucket_start + target_interval_ms,
                            open: bucket_open,
                            high: bucket_high
                                .max(bucket_low)
                                .max(bucket_open)
                                .max(bucket_close),
                            low: bucket_low
                                .min(bucket_high)
                                .min(bucket_open)
                                .min(bucket_close),
                            close: bucket_close,
                            volume: bucket_volume,
                            exchange: source.exchange.clone(),
                            symbol: source.symbol.clone(),
                            market_type: qrpc_core::MarketType::Spot,
                            interval: target_interval.to_string(),
                        },
                    ));
                }
                // 开始新桶
                bucket_start = bar.open_time_ms;
                bucket_open = bar.open;
                bucket_high = bar.high;
                bucket_low = bar.low;
                bucket_close = bar.close;
                bucket_volume = bar.volume;
                bar_count = 1;
            } else {
                if bar_count == 0 {
                    bucket_open = bar.open;
                }
                bucket_high = bucket_high.max(bar.high);
                bucket_low = bucket_low.min(bar.low);
                bucket_close = bar.close;
                bucket_volume += bar.volume;
                bar_count += 1;
            }
        }
        // 处理最后一个桶
        if bar_count > 0 {
            resampled.push((
                bucket_start + target_interval_ms,
                NormalizedKline {
                    open_time_ms: bucket_start,
                    close_time_ms: bucket_start + target_interval_ms,
                    open: bucket_open,
                    high: bucket_high
                        .max(bucket_low)
                        .max(bucket_open)
                        .max(bucket_close),
                    low: bucket_low
                        .min(bucket_high)
                        .min(bucket_open)
                        .min(bucket_close),
                    close: bucket_close,
                    volume: bucket_volume,
                    exchange: source.exchange.clone(),
                    symbol: source.symbol.clone(),
                    market_type: qrpc_core::MarketType::Spot,
                    interval: target_interval.to_string(),
                },
            ));
        }

        Self {
            data_id: data_id.to_string(),
            exchange: source.exchange.clone(),
            symbol: source.symbol.clone(),
            target_interval: target_interval.to_string(),
            target_interval_ms,
            resampled_bars: resampled,
        }
    }

    pub fn close_timestamps(&self) -> Vec<u64> {
        self.resampled_bars.iter().map(|(ts, _)| *ts).collect()
    }

    pub fn bar_count(&self) -> usize {
        self.resampled_bars.len()
    }
}

fn bar_interval_ms(interval: &str) -> u64 {
    match interval {
        "1m" => 60_000,
        "5m" => 300_000,
        "15m" => 900_000,
        "30m" => 1_800_000,
        "1h" => 3_600_000,
        "4h" => 14_400_000,
        "1d" => 86_400_000,
        "1w" => 604_800_000,
        _ => 86_400_000, // 默认日线
    }
}

impl TimelineDataProvider for ResampleKlineProvider {
    fn data_id(&self) -> &str {
        &self.data_id
    }
    fn kind(&self) -> DataKind {
        DataKind::KlineSeries
    }
    fn timestamps(&self) -> Vec<u64> {
        self.close_timestamps()
    }

    fn value_at(&self, ts_ms: u64) -> Option<NormalizedMarketData> {
        // v1.2.0: 滑动窗口截断
        const MAX_WINDOW_BARS: usize = 500;
        let window: Vec<NormalizedKline> = self
            .resampled_bars
            .iter()
            .take_while(|(close_ts, _)| *close_ts <= ts_ms)
            .map(|(_, bar)| bar.clone())
            .collect();
        let window_len = window.len();
        let skip = window_len.saturating_sub(MAX_WINDOW_BARS);
        let window: Vec<NormalizedKline> = window.into_iter().skip(skip).collect();
        if window.is_empty() {
            return None;
        }
        let window_len = window.len();
        let last_close = window.last().map(|b| b.close_time_ms).unwrap_or(ts_ms);
        Some(NormalizedMarketData::KlineSeries(KlineSeriesSnapshot {
            data_id: self.data_id.clone(),
            exchange: self.exchange.clone(),
            symbol: self.symbol.clone(),
            market_type: qrpc_core::MarketType::Spot,
            interval: self.target_interval.clone(),
            bars: window,
            window_len,
            ts_ms: last_close,
            source_latency_ms: 0,
            source_status: qrpc_core::SourceStatus::Healthy,
            data_quality: qrpc_core::DataQualitySnapshot::default(),
        }))
    }
}
