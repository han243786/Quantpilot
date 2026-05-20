/// v3.5.0: OKX REST API 客户端 (testnet)
/// 文档: https://www.okx.com/docs-v5/
/// Testnet: https://www.okx.com/api/v5 (需在 headers 中设置 x-simulated-trading: 1)

use anyhow::{bail, Result};
use ring::hmac;
use std::time::{SystemTime, UNIX_EPOCH};

const OKX_REST_BASE: &str = "https://www.okx.com";

/// 生成 OKX API 签名 (HMAC-SHA256)
/// 签名消息: timestamp + method + request_path + body
fn sign_okx(
    timestamp: &str,
    method: &str,
    request_path: &str,
    body: &str,
    secret: &str,
) -> Result<String> {
    let sign_str = format!("{}{}{}{}", timestamp, method, request_path, body);
    let key = hmac::Key::new(hmac::HMAC_SHA256, secret.as_bytes());
    let signature = hmac::sign(&key, sign_str.as_bytes());
    Ok(base64::encode(signature.as_ref()))
}

fn okx_timestamp() -> String {
    let dur = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    let secs = dur.as_secs();
    let millis = dur.subsec_millis();
    // OKX API v5 要求 ISO 8601 格式: 2025-05-21T00:00:00.000Z
    // 使用 chrono (已是项目依赖) 进行格式化
    let dt = chrono::DateTime::from_timestamp(secs as i64, millis * 1_000_000)
        .unwrap_or_default();
    dt.format("%Y-%m-%dT%H:%M:%S.%3fZ").to_string()
}

/// 下单请求
#[derive(Debug, serde::Serialize)]
pub struct OkxOrderRequest {
    pub inst_id: String,   // BTC-USDT
    pub td_mode: String,   // cash (现货)
    pub side: String,      // buy / sell
    pub ord_type: String,  // market / limit
    pub sz: String,        // 数量
    #[serde(skip_serializing_if = "Option::is_none")]
    pub px: Option<String>, // 限价单价格
}

fn validate_credentials(api_key: &str, secret: &str, passphrase: &str) -> Result<()> {
    if api_key.is_empty() || secret.is_empty() || passphrase.is_empty() {
        bail!("OKX API 凭证不能为空: api_key/secret/passphrase 必须全部提供");
    }
    Ok(())
}

/// 提交订单到 OKX testnet
pub fn place_order(
    api_key: &str,
    secret: &str,
    passphrase: &str,
    order: &OkxOrderRequest,
) -> Result<serde_json::Value> {
    validate_credentials(api_key, secret, passphrase)?;
    let request_path = "/api/v5/trade/order";
    let body = serde_json::to_string(order)?;
    let timestamp = okx_timestamp();
    let signature = sign_okx(&timestamp, "POST", request_path, &body, secret)?;

    let url = format!("{}{}", OKX_REST_BASE, request_path);
    let res: serde_json::Value = ureq::post(&url)
        .set("OK-ACCESS-KEY", api_key)
        .set("OK-ACCESS-SIGN", &signature)
        .set("OK-ACCESS-TIMESTAMP", &timestamp)
        .set("OK-ACCESS-PASSPHRASE", passphrase)
        .set("x-simulated-trading", "1")
        .set("Content-Type", "application/json")
        .send_string(&body)?
        .into_json()?;

    if res.get("code").and_then(|c| c.as_str()) == Some("0") {
        Ok(res)
    } else {
        let code = res.get("code").and_then(|c| c.as_str()).unwrap_or("unknown");
        let msg = res.get("msg").and_then(|m| m.as_str()).unwrap_or("未知错误");
        bail!("OKX 下单失败 [code={}]: {}", code, msg)
    }
}

/// 查询账户余额
pub fn fetch_balance(
    api_key: &str,
    secret: &str,
    passphrase: &str,
) -> Result<serde_json::Value> {
    validate_credentials(api_key, secret, passphrase)?;
    let request_path = "/api/v5/account/balance";
    let body = "";
    let timestamp = okx_timestamp();
    let signature = sign_okx(&timestamp, "GET", request_path, body, secret)?;

    let url = format!("{}{}", OKX_REST_BASE, request_path);
    let res: serde_json::Value = ureq::get(&url)
        .set("OK-ACCESS-KEY", api_key)
        .set("OK-ACCESS-SIGN", &signature)
        .set("OK-ACCESS-TIMESTAMP", &timestamp)
        .set("OK-ACCESS-PASSPHRASE", passphrase)
        .set("x-simulated-trading", "1")
        .call()?
        .into_json()?;

    if res.get("code").and_then(|c| c.as_str()) == Some("0") {
        Ok(res)
    } else {
        let code = res.get("code").and_then(|c| c.as_str()).unwrap_or("unknown");
        let msg = res.get("msg").and_then(|m| m.as_str()).unwrap_or("未知错误");
        bail!("OKX 查询余额失败 [code={}]: {}", code, msg)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_okx_sign_string() {
        let sig = sign_okx("2024-01-01T00:00:00.000Z", "POST", "/api/v5/trade/order", "{}", "test_secret");
        assert!(sig.is_ok());
        // 签名应为 base64 编码字符串
        let sig_str = sig.unwrap();
        assert!(!sig_str.is_empty());
        // base64 decode 应该成功
        base64::decode(&sig_str).expect("签名应为有效 base64");
    }

    #[test]
    fn test_okx_timestamp_is_valid() {
        let ts = okx_timestamp();
        let parsed: u64 = ts.parse().unwrap();
        // 2024年之后的时间戳
        assert!(parsed > 1700000000);
    }
}
