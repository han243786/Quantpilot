use serde::{Deserialize, Serialize};

use crate::{DataQualitySnapshot, Exchange, MarketType, SourceStatus, Symbol};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawKline {
    pub open_time: u64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
    pub close_time: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawQuote {
    pub best_bid: f64,
    pub best_ask: f64,
    pub bid_size: f64,
    pub ask_size: f64,
    pub ts: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NormalizedKline {
    pub exchange: Exchange,
    pub symbol: Symbol,
    pub market_type: MarketType,
    pub interval: String,
    pub open_time_ms: u64,
    pub close_time_ms: u64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KlineSeriesSnapshot {
    pub data_id: String,
    pub exchange: Exchange,
    pub symbol: Symbol,
    pub market_type: MarketType,
    pub interval: String,
    pub bars: Vec<NormalizedKline>,
    pub window_len: usize,
    pub ts_ms: u64,
    pub source_latency_ms: u64,
    pub source_status: SourceStatus,
    #[serde(default)]
    pub data_quality: DataQualitySnapshot,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuoteSnapshot {
    pub data_id: String,
    pub exchange: Exchange,
    pub symbol: Symbol,
    pub market_type: MarketType,
    pub best_bid: f64,
    pub best_ask: f64,
    pub bid_size: f64,
    pub ask_size: f64,
    pub mid_price: f64,
    pub ts_ms: u64,
    pub source_latency_ms: u64,
    pub source_status: SourceStatus,
    #[serde(default)]
    pub data_quality: DataQualitySnapshot,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NormalizedMarketData {
    KlineSeries(KlineSeriesSnapshot),
    Quote(QuoteSnapshot),
}
