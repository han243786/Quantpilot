use anyhow::{anyhow, Context, Result};
use qrpc_core::{DataKind, DataSourceConfig, Exchange, NormalizedKline, RawKline, SourceStatus};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{fs, path::PathBuf};

#[cfg(target_os = "windows")]
use super::fetch_json_via_powershell;
use super::{
    block_on_http, exchange_surface, normalize_kline_series, HISTORICAL_CACHE_MAX_BYTES,
    HISTORICAL_CACHE_TTL_MS, HTTP_TIMEOUT_SECS,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct HistoricalBarsCache {
    fetched_at_ms: u64,
    bars: Vec<NormalizedKline>,
}

fn historical_cache_path(source: &DataSourceConfig) -> PathBuf {
    let interval = source.interval.as_deref().unwrap_or("1d");
    let days = source.days.unwrap_or(200);
    let safe_symbol = sanitize_filename_component(
        &exchange_surface::binance_symbol(source.symbol.clone()).to_ascii_lowercase(),
    );
    PathBuf::from("storage")
        .join("cache")
        .join("historical")
        .join(format!(
            "{:?}_{}_{}_{}_{}.json",
            source.exchange,
            safe_symbol,
            interval,
            days,
            match source.kind {
                DataKind::KlineSeries => "kline",
                DataKind::Quote => "quote",
            }
        ))
}

fn sanitize_filename_component(input: &str) -> String {
    input
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '_' || *c == '-')
        .take(64)
        .collect()
}

fn load_historical_cache(source: &DataSourceConfig, now_ms: u64) -> Option<Vec<NormalizedKline>> {
    let path = historical_cache_path(source);
    let body = read_historical_cache_body(&path)?;
    let cache = serde_json::from_str::<HistoricalBarsCache>(&body).ok()?;
    let is_fresh = now_ms.saturating_sub(cache.fetched_at_ms) <= HISTORICAL_CACHE_TTL_MS;
    if is_fresh && !cache.bars.is_empty() {
        Some(cache.bars)
    } else {
        None
    }
}

fn load_stale_historical_cache(source: &DataSourceConfig) -> Option<Vec<NormalizedKline>> {
    let path = historical_cache_path(source);
    let body = read_historical_cache_body(&path)?;
    let cache = serde_json::from_str::<HistoricalBarsCache>(&body).ok()?;
    (!cache.bars.is_empty()).then_some(cache.bars)
}

fn read_historical_cache_body(path: &std::path::Path) -> Option<String> {
    let metadata = fs::metadata(path).ok()?;
    if metadata.len() > HISTORICAL_CACHE_MAX_BYTES {
        return None;
    }
    let body = fs::read_to_string(path).ok()?;
    (body.len() as u64 <= HISTORICAL_CACHE_MAX_BYTES).then_some(body)
}

fn persist_historical_cache(
    source: &DataSourceConfig,
    now_ms: u64,
    bars: &[NormalizedKline],
) -> Result<()> {
    let path = historical_cache_path(source);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("创建历史数据缓存目录失败: {}", parent.display()))?;
        let cache_dir = PathBuf::from("storage").join("cache").join("historical");
        let dir_size: u64 = std::fs::read_dir(&cache_dir)
            .map(|entries| {
                entries
                    .filter_map(|e| e.ok())
                    .filter_map(|e| e.metadata().ok())
                    .filter(|m| m.is_file())
                    .map(|m| m.len())
                    .sum()
            })
            .unwrap_or(0);
        const CACHE_MAX_BYTES: u64 = 200 * 1024 * 1024;
        if dir_size > CACHE_MAX_BYTES {
            return Err(anyhow!(
                "历史缓存目录已满: 当前 {} MB, 上限 200 MB，跳过缓存写入",
                dir_size / (1024 * 1024)
            ));
        }
    } else {
        fs::create_dir_all(path.parent().unwrap_or(&path))?;
    }
    let body = serde_json::to_string_pretty(&HistoricalBarsCache {
        fetched_at_ms: now_ms,
        bars: bars.to_vec(),
    })?;
    let tmp = path.with_extension("tmp");
    fs::write(&tmp, &body).with_context(|| format!("写入历史缓存 {} 失败", path.display()))?;
    if let Ok(f) = std::fs::File::open(&tmp) {
        let _ = f.sync_all();
    }
    fs::rename(&tmp, &path).with_context(|| format!("重命名历史缓存 {} 失败", path.display()))?;
    if let Some(parent) = path.parent() {
        if let Ok(f) = std::fs::File::open(parent) {
            let _ = f.sync_all();
        }
    }
    Ok(())
}

fn shared_http_client() -> &'static Client {
    static CLIENT: std::sync::OnceLock<Client> = std::sync::OnceLock::new();
    CLIENT.get_or_init(|| {
        Client::builder()
            .timeout(std::time::Duration::from_secs(HTTP_TIMEOUT_SECS))
            .user_agent("quantpilot/0.1")
            .build()
            .unwrap_or_else(|_| Client::new())
    })
}

fn fetch_historical_raw_klines(source: &DataSourceConfig) -> Result<Vec<RawKline>> {
    let endpoint = match source.exchange {
        Exchange::Okx => exchange_surface::okx_endpoint_for_source(source),
        Exchange::Binance => exchange_surface::binance_endpoint_for_source(source),
    };
    let client = shared_http_client().clone();
    let endpoint_for_reqwest = endpoint.clone();
    let payload = block_on_http(async move {
        client
            .get(&endpoint_for_reqwest)
            .send()
            .await
            .with_context(|| format!("请求 {endpoint_for_reqwest} 失败"))?
            .error_for_status()
            .with_context(|| format!("从 {endpoint_for_reqwest} 收到非成功响应"))?
            .json::<Value>()
            .await
            .with_context(|| format!("从 {endpoint_for_reqwest} 收到无效 JSON"))
    })
    .or_else(|primary_error| {
        #[cfg(target_os = "windows")]
        {
            fetch_json_via_powershell(endpoint.clone())
                .with_context(|| format!("{primary_error:#}"))
        }
        #[cfg(not(target_os = "windows"))]
        {
            Err(primary_error)
        }
    })?;

    match source.exchange {
        Exchange::Okx => exchange_surface::parse_okx_candles(&payload, source),
        Exchange::Binance => exchange_surface::parse_binance_klines(&payload),
    }
}

pub(crate) fn historical_kline_bars_for_backtest(
    source: &DataSourceConfig,
    now_ms: u64,
) -> Result<Vec<NormalizedKline>> {
    if let Some(cached) = load_historical_cache(source, now_ms) {
        return Ok(cached);
    }

    match fetch_historical_raw_klines(source) {
        Ok(raw) => {
            let bars = normalize_kline_series(source, raw, now_ms, 0, SourceStatus::Healthy).bars;
            persist_historical_cache(source, now_ms, &bars)?;
            Ok(bars)
        }
        Err(error) => {
            if let Some(cached) = load_stale_historical_cache(source) {
                return Ok(cached);
            }
            Err(error).with_context(|| {
                format!(
                    "加载 {} (交易所 {:?}) 的历史重放 K 线数据失败",
                    source.data_id, source.exchange
                )
            })
        }
    }
}
