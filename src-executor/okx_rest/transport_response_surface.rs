use anyhow::{bail, Context, Result};
use std::time::Duration;

use super::{
    OkxSignedRequest, OKX_REST_CONNECT_TIMEOUT_SECS, OKX_REST_PROXY_ENV,
    OKX_REST_READ_TIMEOUT_SECS, OKX_REST_WRITE_TIMEOUT_SECS,
};

fn okx_rest_proxy_url() -> Option<String> {
    [
        OKX_REST_PROXY_ENV,
        "ALL_PROXY",
        "all_proxy",
        "HTTPS_PROXY",
        "https_proxy",
        "HTTP_PROXY",
        "http_proxy",
    ]
    .into_iter()
    .find_map(|name| {
        std::env::var(name)
            .ok()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
    })
}

fn okx_rest_agent() -> Result<ureq::Agent> {
    let mut builder = ureq::AgentBuilder::new()
        .timeout_connect(Duration::from_secs(OKX_REST_CONNECT_TIMEOUT_SECS))
        .timeout_read(Duration::from_secs(OKX_REST_READ_TIMEOUT_SECS))
        .timeout_write(Duration::from_secs(OKX_REST_WRITE_TIMEOUT_SECS));
    if let Some(proxy_url) = okx_rest_proxy_url() {
        let proxy = ureq::Proxy::new(&proxy_url)
            .with_context(|| format!("OKX REST proxy 配置无效: {}", proxy_url))?;
        builder = builder.proxy(proxy);
    }
    Ok(builder.build())
}

pub(super) fn send_signed_request_raw(request: OkxSignedRequest) -> Result<serde_json::Value> {
    let agent = okx_rest_agent()?;
    let mut req = agent.request(&request.method, &request.url);
    for (name, value) in &request.headers {
        req = req.set(name, value);
    }
    let response_result = if request.body.is_empty() {
        req.call()
    } else {
        req.send_string(&request.body)
    };
    let response = response_result.or_else(|error| match error {
        ureq::Error::Status(_, response) => Ok(response),
        other => Err(other),
    })?;
    let res: serde_json::Value = response.into_json()?;
    Ok(res)
}

pub(super) fn send_signed_request(
    request: OkxSignedRequest,
    action: &str,
) -> Result<serde_json::Value> {
    let res = send_signed_request_raw(request)?;
    ensure_okx_success(action, &res)?;
    Ok(res)
}

fn ensure_okx_success(action: &str, response: &serde_json::Value) -> Result<()> {
    if response.get("code").and_then(|c| c.as_str()) != Some("0") {
        let code = response
            .get("code")
            .and_then(|c| c.as_str())
            .unwrap_or("unknown");
        let msg = response
            .get("msg")
            .and_then(|m| m.as_str())
            .unwrap_or("未知错误");
        bail!("OKX {}失败 [code={}]: {}", action, code, msg);
    }

    if let Some(first) = response
        .get("data")
        .and_then(|data| data.as_array())
        .and_then(|items| items.first())
    {
        if let Some(s_code) = first.get("sCode").and_then(|value| value.as_str()) {
            if s_code != "0" {
                let s_msg = first
                    .get("sMsg")
                    .and_then(|value| value.as_str())
                    .unwrap_or("未知错误");
                bail!("OKX {}失败 [sCode={}]: {}", action, s_code, s_msg);
            }
        }
    }

    Ok(())
}
