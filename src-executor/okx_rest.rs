/// v3.5.0: OKX REST API 客户端 (testnet)
/// 文档: https://www.okx.com/docs-v5/
/// Testnet: https://www.okx.com/api/v5 (需在 headers 中设置 x-simulated-trading: 1)
use anyhow::bail;
use anyhow::Result;
use base64::Engine;
use ring::hmac;
use std::time::{SystemTime, UNIX_EPOCH};

const OKX_REST_BASE: &str = "https://www.okx.com";
const OKX_DEMO_SDK_FLAG: &str = "1";
#[cfg(test)]
const OKX_PRODUCTION_SDK_FLAG: &str = "0";
const OKX_SIMULATED_TRADING_HEADER: (&str, &str) = ("x-simulated-trading", "1");

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OkxTradingProfile {
    pub rest_base_url: &'static str,
    pub sdk_flag: &'static str,
    pub simulated_trading_header: Option<(&'static str, &'static str)>,
    pub audit_environment: &'static str,
}

impl OkxTradingProfile {
    pub const fn demo() -> Self {
        Self {
            rest_base_url: OKX_REST_BASE,
            sdk_flag: OKX_DEMO_SDK_FLAG,
            simulated_trading_header: Some(OKX_SIMULATED_TRADING_HEADER),
            audit_environment: "okx_demo_non_real_funds",
        }
    }
}

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
    Ok(base64::engine::general_purpose::STANDARD.encode(signature.as_ref()))
}

fn okx_timestamp() -> String {
    let dur = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    let secs = dur.as_secs();
    let millis = dur.subsec_millis();
    // OKX API v5 要求 ISO 8601 格式: 2025-05-21T00:00:00.000Z
    // 使用 chrono (已是项目依赖) 进行格式化
    let dt = chrono::DateTime::from_timestamp(secs as i64, millis * 1_000_000).unwrap_or_default();
    dt.format("%Y-%m-%dT%H:%M:%S.%3fZ").to_string()
}

/// 下单请求
#[derive(Debug, serde::Serialize)]
pub struct OkxOrderRequest {
    pub inst_id: String,  // BTC-USDT
    pub td_mode: String,  // cash (现货)
    pub side: String,     // buy / sell
    pub ord_type: String, // market / limit
    pub sz: String,       // 数量
    #[serde(skip_serializing_if = "Option::is_none")]
    pub px: Option<String>, // 限价单价格
}

fn validate_credentials(api_key: &str, secret: &str, passphrase: &str) -> Result<()> {
    if api_key.is_empty() || secret.is_empty() || passphrase.is_empty() {
        bail!("OKX API 凭证不能为空: api_key/secret/passphrase 必须全部提供");
    }
    Ok(())
}

/// 提交订单到 OKX 模拟盘。
pub fn place_order(
    api_key: &str,
    secret: &str,
    passphrase: &str,
    order: &OkxOrderRequest,
) -> Result<serde_json::Value> {
    place_order_with_profile(
        OkxTradingProfile::demo(),
        api_key,
        secret,
        passphrase,
        order,
    )
}

pub fn place_order_with_profile(
    profile: OkxTradingProfile,
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

    let url = format!("{}{}", profile.rest_base_url, request_path);
    let mut req = ureq::post(&url)
        .set("OK-ACCESS-KEY", api_key)
        .set("OK-ACCESS-SIGN", &signature)
        .set("OK-ACCESS-TIMESTAMP", &timestamp)
        .set("OK-ACCESS-PASSPHRASE", passphrase)
        .set("Content-Type", "application/json");
    if let Some((name, value)) = profile.simulated_trading_header {
        req = req.set(name, value);
    }
    let res: serde_json::Value = req.send_string(&body)?.into_json()?;

    if res.get("code").and_then(|c| c.as_str()) == Some("0") {
        Ok(res)
    } else {
        let code = res
            .get("code")
            .and_then(|c| c.as_str())
            .unwrap_or("unknown");
        let msg = res
            .get("msg")
            .and_then(|m| m.as_str())
            .unwrap_or("未知错误");
        bail!("OKX 下单失败 [code={}]: {}", code, msg)
    }
}

/// 查询 OKX 模拟盘账户余额。
pub fn fetch_balance(api_key: &str, secret: &str, passphrase: &str) -> Result<serde_json::Value> {
    fetch_balance_with_profile(OkxTradingProfile::demo(), api_key, secret, passphrase)
}

pub fn fetch_balance_with_profile(
    profile: OkxTradingProfile,
    api_key: &str,
    secret: &str,
    passphrase: &str,
) -> Result<serde_json::Value> {
    validate_credentials(api_key, secret, passphrase)?;
    let request_path = "/api/v5/account/balance";
    let body = "";
    let timestamp = okx_timestamp();
    let signature = sign_okx(&timestamp, "GET", request_path, body, secret)?;

    let url = format!("{}{}", profile.rest_base_url, request_path);
    let mut req = ureq::get(&url)
        .set("OK-ACCESS-KEY", api_key)
        .set("OK-ACCESS-SIGN", &signature)
        .set("OK-ACCESS-TIMESTAMP", &timestamp)
        .set("OK-ACCESS-PASSPHRASE", passphrase)
        .set("Content-Type", "application/json");
    if let Some((name, value)) = profile.simulated_trading_header {
        req = req.set(name, value);
    }
    let res: serde_json::Value = req.call()?.into_json()?;

    if res.get("code").and_then(|c| c.as_str()) == Some("0") {
        Ok(res)
    } else {
        let code = res
            .get("code")
            .and_then(|c| c.as_str())
            .unwrap_or("unknown");
        let msg = res
            .get("msg")
            .and_then(|m| m.as_str())
            .unwrap_or("未知错误");
        bail!("OKX 查询余额失败 [code={}]: {}", code, msg)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_okx_sign_string() {
        let sig = sign_okx(
            "2024-01-01T00:00:00.000Z",
            "POST",
            "/api/v5/trade/order",
            "{}",
            "test_secret",
        );
        assert!(sig.is_ok());
        // 签名应为 base64 编码字符串
        let sig_str = sig.unwrap();
        assert!(!sig_str.is_empty());
        // base64 decode 应该成功
        base64::engine::general_purpose::STANDARD
            .decode(&sig_str)
            .expect("签名应为有效 base64");
    }

    #[test]
    fn test_okx_timestamp_is_valid() {
        let ts = okx_timestamp();
        let parsed =
            chrono::DateTime::parse_from_rfc3339(&ts).expect("OKX 时间戳应为 RFC3339/ISO8601 格式");
        assert!(ts.ends_with('Z'));
        assert!(ts.contains('.'));
        // 2024年之后的时间戳
        assert!(parsed.timestamp() > 1_700_000_000);
    }

    #[test]
    fn okx_order_request_serializes_optional_limit_price() {
        let request = OkxOrderRequest {
            inst_id: "BTC-USDT".to_string(),
            td_mode: "cash".to_string(),
            side: "buy".to_string(),
            ord_type: "limit".to_string(),
            sz: "0.01".to_string(),
            px: Some("70000".to_string()),
        };

        let value = serde_json::to_value(request).unwrap();
        assert_eq!(value["inst_id"], "BTC-USDT");
        assert_eq!(value["px"], "70000");
    }

    #[test]
    fn okx_demo_profile_pins_non_real_funds_markers() {
        let profile = OkxTradingProfile::demo();
        assert_eq!(profile.rest_base_url, OKX_REST_BASE);
        assert_eq!(profile.sdk_flag, OKX_DEMO_SDK_FLAG);
        assert_eq!(OKX_PRODUCTION_SDK_FLAG, "0");
        assert_eq!(
            profile.simulated_trading_header,
            Some(OKX_SIMULATED_TRADING_HEADER)
        );
        assert_eq!(profile.audit_environment, "okx_demo_non_real_funds");
    }

    #[test]
    fn okx_demo_rest_function_items_remain_compilable_without_network_calls() {
        let _base = OKX_REST_BASE;
        let _profile = OkxTradingProfile::demo();
        let _place: fn(&str, &str, &str, &OkxOrderRequest) -> Result<serde_json::Value> =
            place_order;
        let _place_with_profile: fn(
            OkxTradingProfile,
            &str,
            &str,
            &str,
            &OkxOrderRequest,
        ) -> Result<serde_json::Value> = place_order_with_profile;
        let _balance: fn(&str, &str, &str) -> Result<serde_json::Value> = fetch_balance;
        let _balance_with_profile: fn(
            OkxTradingProfile,
            &str,
            &str,
            &str,
        ) -> Result<serde_json::Value> = fetch_balance_with_profile;
        assert!(validate_credentials("", "secret", "passphrase").is_err());
        assert!(validate_credentials("key", "secret", "passphrase").is_ok());
    }
}
