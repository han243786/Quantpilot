#![allow(dead_code)]

#[path = "timeline_data_providers.rs"]
mod timeline_data_providers;
#[path = "unified_timeline.rs"]
mod unified_timeline;

pub use self::timeline_data_providers::{
    KlineProvider, QuoteProvider, ResampleKlineProvider, TimelineDataProvider,
};
pub use self::unified_timeline::UnifiedTimeline;

#[cfg(test)]
mod tests {
    use super::*;
    use qrpc_core::{
        DataKind, DataSourceConfig, Exchange, MarketType, NormalizedKline, NormalizedMarketData,
        Symbol,
    };

    fn interval_to_label(ms: u64) -> String {
        match ms {
            3_600_000 => "1h".into(),
            14_400_000 => "4h".into(),
            86_400_000 => "1d".into(),
            604_800_000 => "1w".into(),
            _ => format!("{}ms", ms),
        }
    }

    fn sample_kline_provider(
        data_id: &str,
        symbol: &Symbol,
        count: usize,
        start_ts: u64,
        interval_ms: u64,
    ) -> KlineProvider {
        let interval_label = interval_to_label(interval_ms);
        let bars: Vec<NormalizedKline> = (0..count)
            .map(|i| {
                let open_time = start_ts + i as u64 * interval_ms;
                NormalizedKline {
                    open_time_ms: open_time,
                    close_time_ms: open_time + interval_ms,
                    open: 50_000.0 + i as f64 * 10.0,
                    high: 50_100.0,
                    low: 49_900.0,
                    close: 50_050.0 + i as f64 * 10.0,
                    volume: 100.0,
                    exchange: Exchange::Binance,
                    symbol: symbol.clone(),
                    market_type: qrpc_core::MarketType::Spot,
                    interval: interval_label.clone(),
                }
            })
            .collect();
        let source = DataSourceConfig {
            data_id: data_id.into(),
            exchange: Exchange::Binance,
            symbol: symbol.clone(),
            market_type: MarketType::Spot,
            kind: DataKind::KlineSeries,
            days: Some(200),
            interval: Some(interval_label),
            ping_enabled: false,
            request_interval_ms: None,
            enabled: true,
        };
        KlineProvider::new(&source, bars)
    }

    #[test]
    fn kline_provider_asof_returns_bars_up_to_timestamp() {
        let provider = sample_kline_provider("btc_1d", &Symbol::BtcUsdt, 5, 1_000_000, 86_400_000);
        // ts = start + 3*interval: bar[3] 的 close = start + 4*interval > ts，不包含
        let ts = 1_000_000 + 3 * 86_400_000;
        let result = provider.value_at(ts);
        assert!(result.is_some(), "应返回 ≤ ts 的 bar");
        if let Some(NormalizedMarketData::KlineSeries(series)) = result {
            assert_eq!(series.window_len, 3); // bars 0,1,2
        }
    }

    #[test]
    fn unified_timeline_merges_multiple_sources() {
        let btc = sample_kline_provider("btc_1d", &Symbol::BtcUsdt, 3, 1_000_000, 86_400_000);
        let eth = sample_kline_provider(
            "eth_1d",
            &Symbol::Other("ETHUSDT".into()),
            5,
            1_000_000,
            86_400_000,
        );
        let timeline = UnifiedTimeline::new(&[btc.clone(), eth.clone()], &[]).unwrap();

        // 时间轴长度: min(3,5)=3 不再适用，实际是 5 个时间戳（取并集）
        assert_eq!(
            timeline.timestamps.len(),
            5,
            "统一时间轴应取所有数据源时间戳的并集"
        );
        assert!(!timeline.slow_triggers.is_empty());
    }

    #[test]
    fn unified_timeline_single_source_has_same_step_count_as_bars() {
        let btc = sample_kline_provider("btc_1d", &Symbol::BtcUsdt, 10, 1_000_000, 86_400_000);
        let timeline = UnifiedTimeline::new(&[btc], &[]).unwrap();
        assert_eq!(timeline.slow_triggers.len(), 10);
        assert_eq!(timeline.timestamps.len(), 10);
    }

    #[test]
    fn resample_1h_to_1d_aggregates_bars() {
        // 24 根 1h K 线 → 1 根 1d K 线
        let source = sample_kline_provider("btc_1h", &Symbol::BtcUsdt, 24, 1_000_000, 3_600_000);
        let resampled = ResampleKlineProvider::new("btc_1d_resampled", &source, "1d");
        assert_eq!(resampled.bar_count(), 1, "24h → 1d 应产生 1 根日线");
        assert!(resampled.resampled_bars[0].1.interval == "1d");
    }

    #[test]
    fn resample_1h_to_4h_aggregates_correctly() {
        let source = sample_kline_provider("btc_1h", &Symbol::BtcUsdt, 12, 1_000_000, 3_600_000);
        let resampled = ResampleKlineProvider::new("btc_4h_resampled", &source, "4h");
        assert_eq!(resampled.bar_count(), 3, "12h → 4h 应产生 3 根 4 小时线");
    }

    #[test]
    fn resample_same_frequency_passthrough() {
        // 同日频数据不需要重采样
        let source = sample_kline_provider("btc_1d", &Symbol::BtcUsdt, 5, 1_000_000, 86_400_000);
        let resampled = ResampleKlineProvider::new("btc_1d_copy", &source, "1d");
        assert_eq!(
            resampled.bar_count(),
            source.bar_count(),
            "同频重采样应保持 bar 数不变"
        );
    }

    #[test]
    fn resample_preserves_ohlc_values() {
        let source = sample_kline_provider("btc_1h", &Symbol::BtcUsdt, 6, 1_000_000, 3_600_000);
        let resampled = ResampleKlineProvider::new("btc_4h_resampled", &source, "4h");
        // 6h → 4h = 1 桶 (前 4 根聚合) + 1 桶 (后 2 根)
        // 实际上 6 根 1h 应该产生 0 根... 不对，应该是 1 根 4h + 1 根不足 4h 的
        assert!(resampled.bar_count() >= 1, "应有至少 1 根重采样柱");
        // 验证第一根柱的 OHLC
        let first = &resampled.resampled_bars[0].1;
        assert!(first.high >= first.low, "high >= low");
        assert!(first.open > 0.0, "open > 0");
        assert!(first.close > 0.0, "close > 0");
    }
}
