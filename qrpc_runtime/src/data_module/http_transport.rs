use anyhow::{anyhow, Context, Result};
use qrpc_core::{DataKind, DataSourceConfig, NormalizedMarketData, SourceStatus};
use reqwest::Client;
use serde_json::Value;
#[cfg(target_os = "windows")]
use std::process::Command;
use std::time::Instant;
use tokio::{runtime::Handle, task};

use super::{
    exchange_surface, normalize_kline_series, normalize_quote, FetchDiagnostics, HTTP_TIMEOUT_SECS,
};

#[derive(Debug, Clone, Default)]
pub(super) struct PingProbe {
    pub(super) latency_ms: Option<u64>,
    pub(super) endpoint: Option<String>,
    pub(super) error: Option<String>,
}

impl PingProbe {
    pub(super) fn disabled() -> Self {
        Self::default()
    }
}

pub(super) fn probe_ping(client: &Client, source: &DataSourceConfig) -> PingProbe {
    if !source.ping_enabled {
        return PingProbe::disabled();
    }

    let endpoint = exchange_surface::ping_endpoint_for_source(source);
    let started = Instant::now();
    match fetch_json(client, &endpoint) {
        Ok(_) => PingProbe {
            latency_ms: Some(started.elapsed().as_millis() as u64),
            endpoint: Some(endpoint),
            error: None,
        },
        Err(error) => PingProbe {
            latency_ms: None,
            endpoint: Some(endpoint),
            error: Some(format!("{error:#}")),
        },
    }
}

pub(super) fn fetch_okx(
    client: &Client,
    source: &DataSourceConfig,
    now_ms: u64,
) -> Result<(NormalizedMarketData, FetchDiagnostics)> {
    match source.kind {
        DataKind::KlineSeries => {
            let endpoint = exchange_surface::okx_endpoint_for_source(source);
            let started = Instant::now();
            let payload = fetch_json(client, &endpoint)
                .with_context(|| format!("GET 请求 {endpoint} 失败"))?;
            let raw = exchange_surface::parse_okx_candles(&payload, source)?;
            let elapsed = started.elapsed().as_millis() as u64;
            Ok((
                NormalizedMarketData::KlineSeries(normalize_kline_series(
                    source,
                    raw,
                    now_ms,
                    elapsed,
                    SourceStatus::Healthy,
                )),
                FetchDiagnostics {
                    provider_key: "builtin.data.okx_v5_http",
                    source_status: SourceStatus::Healthy,
                    source_latency_ms: elapsed,
                    endpoint: Some(endpoint),
                    ping_latency_ms: None,
                    ping_endpoint: None,
                    ping_error: None,
                    fallback: None,
                    error: None,
                },
            ))
        }
        DataKind::Quote => {
            let endpoint = exchange_surface::okx_endpoint_for_source(source);
            let started = Instant::now();
            let payload = fetch_json(client, &endpoint)
                .with_context(|| format!("GET 请求 {endpoint} 失败"))?;
            let raw = exchange_surface::parse_okx_ticker(&payload)?;
            let elapsed = started.elapsed().as_millis() as u64;
            Ok((
                NormalizedMarketData::Quote(normalize_quote(
                    source,
                    raw,
                    elapsed,
                    SourceStatus::Healthy,
                )),
                FetchDiagnostics {
                    provider_key: "builtin.data.okx_v5_http",
                    source_status: SourceStatus::Healthy,
                    source_latency_ms: elapsed,
                    endpoint: Some(endpoint),
                    ping_latency_ms: None,
                    ping_endpoint: None,
                    ping_error: None,
                    fallback: None,
                    error: None,
                },
            ))
        }
    }
}

pub(super) fn fetch_json(client: &Client, endpoint: &str) -> Result<Value> {
    let client = client.clone();
    let endpoint = endpoint.to_string();
    let endpoint_for_reqwest = endpoint.clone();
    let primary_result = block_on_http(async move {
        #[allow(unused_assignments)]
        let mut last_status = 0u16;
        for attempt in 0..4 {
            let resp = client
                .get(&endpoint_for_reqwest)
                .send()
                .await
                .with_context(|| format!("请求 {endpoint_for_reqwest} 失败"))?;
            last_status = resp.status().as_u16();
            if last_status == 429 && attempt < 3 {
                let delay = std::time::Duration::from_secs(1u64 << attempt);
                tokio::time::sleep(delay).await;
                continue;
            }
            let status = resp.status().as_u16();
            let category = if status >= 500 {
                "服务端临时错误"
            } else {
                "客户端请求错误"
            };
            return resp
                .error_for_status()
                .with_context(|| {
                    format!("从 {endpoint_for_reqwest} 收到非成功响应 (HTTP {status}, {category})")
                })?
                .json::<Value>()
                .await
                .with_context(|| format!("从 {endpoint_for_reqwest} 收到无效 JSON"));
        }
        Err(anyhow!(
            "请求 {endpoint_for_reqwest} 被限流 (429), 重试 3 次后仍失败"
        ))
    });

    match primary_result {
        Ok(payload) => Ok(payload),
        Err(primary_error) => {
            #[cfg(target_os = "windows")]
            {
                fetch_json_via_powershell(endpoint).with_context(|| format!("{primary_error:#}"))
            }

            #[cfg(not(target_os = "windows"))]
            {
                Err(primary_error)
            }
        }
    }
}

pub(crate) fn block_on_http<F, T>(future: F) -> Result<T>
where
    F: std::future::Future<Output = Result<T>>,
{
    match Handle::try_current() {
        Ok(handle) => task::block_in_place(|| handle.block_on(future)),
        // v2.4.0 P1-B3: 使用 LazyLock 全局单例 Runtime, 避免每次创建新线程池
        Err(_) => FALLBACK_RT.block_on(future),
    }
}

static FALLBACK_RT: std::sync::LazyLock<tokio::runtime::Runtime> = std::sync::LazyLock::new(|| {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("创建 Fallback tokio 运行时失败")
});

#[cfg(target_os = "windows")]
pub(crate) fn fetch_json_via_powershell(endpoint: String) -> Result<Value> {
    let escaped = endpoint.replace('\'', "''");
    let script = format!(
        "[Console]::OutputEncoding = [System.Text.Encoding]::UTF8; \
         $resp = Invoke-RestMethod -Uri '{escaped}' -TimeoutSec {HTTP_TIMEOUT_SECS}; \
         $resp | ConvertTo-Json -Depth 64 -Compress"
    );
    let output = Command::new("powershell")
        .args(["-NoProfile", "-Command", &script])
        .output()
        .with_context(|| format!("调用 powershell 获取 {endpoint} 失败"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        return Err(anyhow!(
            "powershell HTTP 回退获取 {endpoint} 失败: {}",
            if stderr.is_empty() { stdout } else { stderr }
        ));
    }

    serde_json::from_slice::<Value>(&output.stdout)
        .with_context(|| format!("powershell 为 {endpoint} 返回了无效 JSON"))
}
