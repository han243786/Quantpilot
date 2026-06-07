use qrpc_core::{
    DataQualitySnapshot, DataSourceConfig, KlineSeriesSnapshot, NormalizedKline, QuoteSnapshot,
    RawKline, RawQuote, SourceStatus,
};

pub(super) fn normalize_kline_series(
    source: &DataSourceConfig,
    raw: Vec<RawKline>,
    now_ms: u64,
    source_latency_ms: u64,
    source_status: SourceStatus,
) -> KlineSeriesSnapshot {
    // v1.1.14: OHLC relation validation, reject invalid bars.
    let bars = raw
        .into_iter()
        .filter_map(|bar| {
            let high = bar.high.max(bar.open).max(bar.close).max(bar.low);
            let low = bar.low.min(bar.open).min(bar.close).min(bar.high);
            if high != bar.high || low != bar.low {
                eprintln!("[data_module] 修正K线 OHLC 关系: open_time={}, O={:.2} H={:.2} L={:.2} C={:.2} (high clamped from {} to {}, low clamped from {} to {})",
                    bar.open_time, bar.open, bar.high, bar.low, bar.close, bar.high, high, bar.low, low);
            }
            if bar.high < bar.open
                || bar.high < bar.close
                || bar.low > bar.open
                || bar.low > bar.close
                || bar.low > bar.high
                || bar.volume < 0.0
            {
                eprintln!("[data_module] 跳过非法K线: open_time={}, O={:.2} H={:.2} L={:.2} C={:.2} V={:.2}",
                    bar.open_time, bar.open, bar.high, bar.low, bar.close, bar.volume);
                None
            } else {
                Some(NormalizedKline {
                    exchange: source.exchange.clone(),
                    symbol: source.symbol.clone(),
                    market_type: source.market_type.clone(),
                    interval: source.interval.clone().unwrap_or_else(|| "1d".into()),
                    open_time_ms: bar.open_time,
                    close_time_ms: bar.close_time,
                    open: bar.open,
                    high,
                    low,
                    close: bar.close,
                    volume: bar.volume.max(0.0),
                })
            }
        })
        .collect::<Vec<_>>();

    KlineSeriesSnapshot {
        data_id: source.data_id.clone(),
        exchange: source.exchange.clone(),
        symbol: source.symbol.clone(),
        market_type: source.market_type.clone(),
        interval: source.interval.clone().unwrap_or_else(|| "1d".into()),
        window_len: bars.len(),
        bars,
        ts_ms: now_ms,
        source_latency_ms,
        source_status,
        data_quality: DataQualitySnapshot::default(),
    }
}

pub(super) fn normalize_quote(
    source: &DataSourceConfig,
    raw: RawQuote,
    source_latency_ms: u64,
    source_status: SourceStatus,
) -> QuoteSnapshot {
    QuoteSnapshot {
        data_id: source.data_id.clone(),
        exchange: source.exchange.clone(),
        symbol: source.symbol.clone(),
        market_type: source.market_type.clone(),
        best_bid: raw.best_bid,
        best_ask: raw.best_ask,
        bid_size: raw.bid_size,
        ask_size: raw.ask_size,
        mid_price: (raw.best_bid + raw.best_ask) / 2.0,
        ts_ms: raw.ts,
        source_latency_ms,
        source_status,
        data_quality: DataQualitySnapshot::default(),
    }
}

#[allow(dead_code)]
pub(crate) fn quote_snapshot_from_price(
    source: &DataSourceConfig,
    mid_price: f64,
    ts_ms: u64,
) -> QuoteSnapshot {
    normalize_quote(
        source,
        RawQuote {
            best_bid: mid_price - 5.0,
            best_ask: mid_price + 5.0,
            bid_size: 10.0,
            ask_size: 10.0,
            ts: ts_ms,
        },
        0,
        SourceStatus::Healthy,
    )
}
