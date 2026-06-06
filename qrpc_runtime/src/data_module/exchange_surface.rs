use anyhow::{anyhow, bail, Context, Result};
use qrpc_core::{DataKind, DataSourceConfig, Exchange, MarketType, RawKline, RawQuote, Symbol};
use serde_json::Value;

const OKX_BASE_URL: &str = "https://www.okx.com";
const BINANCE_BASE_URL: &str = "https://api.binance.com";

pub(super) fn provider_key_for_source(source: &DataSourceConfig) -> &'static str {
    match source.exchange {
        Exchange::Okx => "builtin.data.okx_v5_http",
        Exchange::Binance => "builtin.data.mock",
    }
}

pub(super) fn endpoint_for_source(source: &DataSourceConfig) -> String {
    match source.exchange {
        Exchange::Okx => okx_endpoint_for_source(source),
        Exchange::Binance => binance_endpoint_for_source(source),
    }
}

pub(super) fn ping_endpoint_for_source(source: &DataSourceConfig) -> String {
    match source.exchange {
        Exchange::Okx => format!("{OKX_BASE_URL}/api/v5/public/time"),
        Exchange::Binance => format!("{BINANCE_BASE_URL}/api/v3/ping"),
    }
}

pub(super) fn binance_endpoint_for_source(source: &DataSourceConfig) -> String {
    let symbol = binance_symbol(source.symbol.clone());
    match source.kind {
        DataKind::KlineSeries => {
            let interval = source.interval.as_deref().unwrap_or("1d");
            let limit = source.days.unwrap_or(200).clamp(1, 1000);
            format!(
                "{BINANCE_BASE_URL}/api/v3/klines?symbol={symbol}&interval={interval}&limit={limit}"
            )
        }
        DataKind::Quote => {
            format!("{BINANCE_BASE_URL}/api/v3/ticker/bookTicker?symbol={symbol}")
        }
    }
}

pub(super) fn okx_endpoint_for_source(source: &DataSourceConfig) -> String {
    let inst_id = okx_inst_id(source.symbol.clone(), source.market_type.clone());
    match source.kind {
        DataKind::KlineSeries => {
            let bar = okx_bar(source.interval.as_deref().unwrap_or("1d"));
            let limit = source.days.unwrap_or(200).min(300);
            format!(
                "{OKX_BASE_URL}/api/v5/market/history-candles?instId={inst_id}&bar={bar}&limit={limit}"
            )
        }
        DataKind::Quote => {
            format!("{OKX_BASE_URL}/api/v5/market/ticker?instId={inst_id}")
        }
    }
}

fn okx_inst_id(symbol: Symbol, market_type: MarketType) -> String {
    match market_type {
        MarketType::Spot => okx_spot_symbol(symbol.as_str()),
    }
}

pub(super) fn binance_symbol(symbol: Symbol) -> String {
    symbol.as_str().to_string()
}

fn okx_spot_symbol(symbol: &str) -> String {
    const KNOWN_QUOTES: [&str; 4] = ["USDT", "USDC", "BTC", "ETH"];
    for quote in KNOWN_QUOTES {
        if let Some(base) = symbol.strip_suffix(quote) {
            if !base.is_empty() {
                return format!("{base}-{quote}");
            }
        }
    }
    symbol.to_string()
}

fn okx_bar(interval: &str) -> &'static str {
    match interval {
        "1m" => "1m",
        "5m" => "5m",
        "1h" => "1H",
        "1d" => "1Dutc",
        _ => "1Dutc",
    }
}

pub(super) fn bar_interval_ms(interval: &str) -> u64 {
    match interval {
        "1m" => 60_000,
        "5m" => 300_000,
        "1h" => 3_600_000,
        "1d" => 86_400_000,
        _ => 86_400_000,
    }
}

pub(super) fn parse_okx_candles(
    payload: &Value,
    source: &DataSourceConfig,
) -> Result<Vec<RawKline>> {
    ensure_okx_success(payload)?;
    let rows = payload
        .get("data")
        .and_then(Value::as_array)
        .ok_or_else(|| anyhow!("OKX K 线响应缺少 data 数组"))?;
    let interval = source.interval.as_deref().unwrap_or("1d");
    let interval_ms = bar_interval_ms(interval);

    let mut bars = rows
        .iter()
        .filter_map(Value::as_array)
        .filter(|row| row.len() >= 9)
        .filter_map(|row| {
            let confirm = row.get(8)?.as_str().unwrap_or("1");
            if confirm != "1" {
                return None;
            }
            Some(RawKline {
                open_time: parse_u64_field(row, 0).ok()?,
                open: parse_f64_field(row, 1).ok()?,
                high: parse_f64_field(row, 2).ok()?,
                low: parse_f64_field(row, 3).ok()?,
                close: parse_f64_field(row, 4).ok()?,
                volume: parse_f64_field(row, 5).ok()?,
                close_time: parse_u64_field(row, 0).ok()?.saturating_add(interval_ms),
            })
        })
        .collect::<Vec<_>>();

    if bars.is_empty() {
        return Err(anyhow!("OKX K 线响应未包含已确认的 K 线数据"));
    }

    bars.sort_by_key(|bar| bar.open_time);
    Ok(bars)
}

pub(super) fn parse_okx_ticker(payload: &Value) -> Result<RawQuote> {
    ensure_okx_success(payload)?;
    let row = payload
        .get("data")
        .and_then(Value::as_array)
        .and_then(|items| items.first())
        .and_then(Value::as_object)
        .ok_or_else(|| anyhow!("OKX ticker 响应缺少 data 条目"))?;

    Ok(RawQuote {
        best_bid: parse_f64_value(
            row.get("bidPx")
                .ok_or_else(|| anyhow!("OKX ticker 缺少 bidPx"))?,
        )?,
        best_ask: parse_f64_value(
            row.get("askPx")
                .ok_or_else(|| anyhow!("OKX ticker 缺少 askPx"))?,
        )?,
        bid_size: parse_f64_value(
            row.get("bidSz")
                .ok_or_else(|| anyhow!("OKX ticker 缺少 bidSz"))?,
        )?,
        ask_size: parse_f64_value(
            row.get("askSz")
                .ok_or_else(|| anyhow!("OKX ticker 缺少 askSz"))?,
        )?,
        ts: parse_u64_value(row.get("ts").ok_or_else(|| anyhow!("OKX ticker 缺少 ts"))?)?,
    })
}

pub(super) fn parse_binance_klines(payload: &Value) -> Result<Vec<RawKline>> {
    let rows = payload
        .as_array()
        .ok_or_else(|| anyhow!("Binance K 线响应必须是数组"))?;
    let mut bars = rows
        .iter()
        .filter_map(Value::as_array)
        .filter(|row| row.len() >= 7)
        .map(|row| {
            Ok(RawKline {
                open_time: row
                    .first()
                    .and_then(Value::as_u64)
                    .ok_or_else(|| anyhow!("Binance K 线缺少开盘时间"))?,
                open: parse_f64_value(
                    row.get(1)
                        .ok_or_else(|| anyhow!("Binance K 线缺少开盘价"))?,
                )?,
                high: parse_f64_value(
                    row.get(2)
                        .ok_or_else(|| anyhow!("Binance K 线缺少最高价"))?,
                )?,
                low: parse_f64_value(
                    row.get(3)
                        .ok_or_else(|| anyhow!("Binance K 线缺少最低价"))?,
                )?,
                close: parse_f64_value(
                    row.get(4)
                        .ok_or_else(|| anyhow!("Binance K 线缺少收盘价"))?,
                )?,
                volume: parse_f64_value(
                    row.get(5)
                        .ok_or_else(|| anyhow!("Binance K 线缺少成交量"))?,
                )?,
                close_time: row
                    .get(6)
                    .and_then(Value::as_u64)
                    .ok_or_else(|| anyhow!("Binance K 线缺少收盘时间"))?,
            })
        })
        .collect::<Result<Vec<_>>>()?;
    if bars.is_empty() {
        return Err(anyhow!("Binance K 线响应为空"));
    }
    bars.sort_by_key(|bar| bar.open_time);
    Ok(bars)
}

fn ensure_okx_success(payload: &Value) -> Result<()> {
    let code = payload
        .get("code")
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow!("OKX 响应缺少状态码"))?;
    if code == "0" {
        return Ok(());
    }
    let msg = payload.get("msg").and_then(Value::as_str).unwrap_or("");
    Err(anyhow!("OKX API 返回代码 {code}: {msg}"))
}

fn parse_u64_field(row: &[Value], index: usize) -> Result<u64> {
    parse_u64_value(
        row.get(index)
            .ok_or_else(|| anyhow!("OKX 数组行缺少索引 {index}"))?,
    )
}

fn parse_f64_field(row: &[Value], index: usize) -> Result<f64> {
    parse_f64_value(
        row.get(index)
            .ok_or_else(|| anyhow!("OKX 数组行缺少索引 {index}"))?,
    )
}

fn parse_u64_value(value: &Value) -> Result<u64> {
    value
        .as_str()
        .ok_or_else(|| anyhow!("预期为字符串类型的 u64 值"))?
        .parse::<u64>()
        .with_context(|| format!("无效的 u64 值: {value}"))
}

fn parse_f64_value(value: &Value) -> Result<f64> {
    let v: f64 = value
        .as_str()
        .ok_or_else(|| anyhow!("预期为字符串类型的 f64 值"))?
        .parse::<f64>()
        .with_context(|| format!("无效的 f64 值: {value}"))?;
    // v1.1.11: 拒绝 NaN/Inf，防止毒数据传播
    if !v.is_finite() {
        bail!("数值必须为有限值 (拒绝 NaN/Inf): {value}");
    }
    Ok(v)
}
