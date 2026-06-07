use anyhow::Result;
use qrpc_core::{DataSourceConfig, Exchange, NormalizedKline, RawKline, RawQuote, SourceStatus};

use super::normalize_kline_series;

/// Configurable mock volatility — set via TestRunner before backtest
pub static MOCK_VOLATILITY: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

const DEFAULT_MOCK_VOLATILITY: f64 = 0.015;

fn get_mock_volatility() -> f64 {
    let bits = MOCK_VOLATILITY.load(std::sync::atomic::Ordering::Relaxed);
    if bits == 0 {
        return DEFAULT_MOCK_VOLATILITY;
    }
    let vol = f64::from_bits(bits);
    // v2.1.x: reject NaN/Inf injection; v2.4.0 P1-C3: clamp extreme values.
    if !vol.is_finite() {
        DEFAULT_MOCK_VOLATILITY
    } else {
        vol.clamp(1e-6, 1.0)
    }
}

pub(super) fn mock_raw_klines(source: &DataSourceConfig, now_ms: u64) -> Result<Vec<RawKline>> {
    let days = source.days.unwrap_or(150);
    let mut bars = Vec::new();
    let interval_ms = 86_400_000_u64;
    let symbol_bytes = format!("{:?}", source.symbol).into_bytes();
    let symbol_seed = symbol_bytes
        .iter()
        .fold(0u64, |acc, &b| acc.wrapping_mul(31).wrapping_add(b as u64));
    let seed = symbol_seed.wrapping_add(days as u64);

    for idx in 0..days {
        let day_index = idx as f64;
        let trend_close = match source.exchange {
            Exchange::Binance => {
                if idx < 50 {
                    42_000.0 + day_index * 20.0
                } else if idx < 100 {
                    43_000.0 + (idx - 50) as f64 * 180.0
                } else if idx < 140 {
                    52_000.0 - (idx - 100) as f64 * 50.0
                } else {
                    50_000.0 + (idx - 140) as f64 * 300.0
                }
            }
            Exchange::Okx => 42_100.0 + day_index * 22.0,
        };

        let vol = get_mock_volatility();
        let noise = pseudo_random(idx as u64, seed) * trend_close * vol;
        let close = trend_close + noise;
        let daily_range = close * (0.002 + pseudo_random(idx as u64 + 1, seed).abs() * 0.008);
        let open = close - daily_range * pseudo_random(idx as u64 + 2, seed);
        let high = close.max(open) + daily_range * pseudo_random(idx as u64 + 3, seed).abs() * 0.5;
        let low = close.min(open) - daily_range * pseudo_random(idx as u64 + 4, seed).abs() * 0.5;
        let close_time = now_ms.saturating_sub(interval_ms * (days - idx) as u64);
        let open_time = close_time.saturating_sub(interval_ms);
        bars.push(RawKline {
            open_time,
            open,
            high,
            low,
            close,
            volume: 1000.0 + idx as f64 * 10.0 + pseudo_random(idx as u64 + 5, seed).abs() * 500.0,
            close_time,
        });
    }

    Ok(bars)
}

pub(super) fn pseudo_random(idx: u64, seed: u64) -> f64 {
    let val = idx
        .wrapping_mul(6364136223846793005)
        .wrapping_add(seed.wrapping_mul(1442695040888963407))
        .wrapping_add(1);
    let mixed = (val ^ (val >> 33)).wrapping_mul(0xFF51AFD7ED558CCD);
    let mixed = (mixed ^ (mixed >> 33)).wrapping_mul(0xC4CEB9FE1A85EC53);
    let mixed = mixed ^ (mixed >> 33);
    (mixed as f64 / u64::MAX as f64) * 2.0 - 1.0
}

pub(super) fn mock_raw_quote(source: &DataSourceConfig, now_ms: u64) -> Result<RawQuote> {
    let mid = match source.exchange {
        Exchange::Binance => 50_000.0,
        Exchange::Okx => 50_350.0,
    };
    Ok(RawQuote {
        best_bid: mid - 5.0,
        best_ask: mid + 5.0,
        bid_size: 10.0,
        ask_size: 10.0,
        ts: now_ms.saturating_sub(10),
    })
}

pub(crate) fn mock_kline_bars_for_backtest(
    source: &DataSourceConfig,
    now_ms: u64,
) -> Result<Vec<NormalizedKline>> {
    Ok(normalize_kline_series(
        source,
        mock_raw_klines(source, now_ms)?,
        now_ms,
        0,
        SourceStatus::Healthy,
    )
    .bars)
}
