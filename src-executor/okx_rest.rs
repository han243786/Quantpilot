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
pub const OKX_DEMO_PROVIDER_KEY: &str = "okx";
pub const OKX_DEMO_AUDIT_ENVIRONMENT: &str = "okx_demo_non_real_funds";
pub const OKX_ORDER_PATH: &str = "/api/v5/trade/order";
pub const OKX_CANCEL_ORDER_PATH: &str = "/api/v5/trade/cancel-order";
pub const OKX_BALANCE_PATH: &str = "/api/v5/account/balance";

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
            audit_environment: OKX_DEMO_AUDIT_ENVIRONMENT,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OkxCredentials {
    pub api_key: String,
    pub secret: String,
    pub passphrase: String,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct OkxSignedRequest {
    pub method: String,
    pub path: String,
    pub url: String,
    pub body: String,
    pub headers: Vec<(String, String)>,
    pub audit_environment: String,
    pub sdk_flag: String,
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
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OkxOrderRequest {
    pub inst_id: String,  // BTC-USDT
    pub td_mode: String,  // cash (现货)
    pub side: String,     // buy / sell
    pub ord_type: String, // market / limit
    pub sz: String,       // 数量
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cl_ord_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub px: Option<String>, // 限价单价格
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OkxCancelOrderRequest {
    pub inst_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ord_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cl_ord_id: Option<String>,
}

fn validate_credentials(api_key: &str, secret: &str, passphrase: &str) -> Result<()> {
    if api_key.is_empty() || secret.is_empty() || passphrase.is_empty() {
        bail!("OKX API 凭证不能为空: api_key/secret/passphrase 必须全部提供");
    }
    Ok(())
}

pub fn build_signed_request(
    profile: OkxTradingProfile,
    credentials: &OkxCredentials,
    method: &str,
    request_path: &str,
    body: &str,
) -> Result<OkxSignedRequest> {
    validate_credentials(
        &credentials.api_key,
        &credentials.secret,
        &credentials.passphrase,
    )?;
    let method = method.trim().to_ascii_uppercase();
    if method.is_empty() {
        bail!("OKX 请求 method 不能为空");
    }
    if !request_path.starts_with("/api/v5/") {
        bail!("OKX 模拟盘请求路径必须固定在 /api/v5/ 下");
    }
    let timestamp = okx_timestamp();
    build_signed_request_with_timestamp(
        profile,
        credentials,
        &method,
        request_path,
        body,
        &timestamp,
    )
}

fn build_signed_request_with_timestamp(
    profile: OkxTradingProfile,
    credentials: &OkxCredentials,
    method: &str,
    request_path: &str,
    body: &str,
    timestamp: &str,
) -> Result<OkxSignedRequest> {
    let signature = sign_okx(timestamp, method, request_path, body, &credentials.secret)?;
    let mut headers = vec![
        ("OK-ACCESS-KEY".to_string(), credentials.api_key.clone()),
        ("OK-ACCESS-SIGN".to_string(), signature),
        ("OK-ACCESS-TIMESTAMP".to_string(), timestamp.to_string()),
        (
            "OK-ACCESS-PASSPHRASE".to_string(),
            credentials.passphrase.clone(),
        ),
        ("Content-Type".to_string(), "application/json".to_string()),
    ];
    if let Some((name, value)) = profile.simulated_trading_header {
        headers.push((name.to_string(), value.to_string()));
    }

    Ok(OkxSignedRequest {
        method: method.to_string(),
        path: request_path.to_string(),
        url: format!("{}{}", profile.rest_base_url, request_path),
        body: body.to_string(),
        headers,
        audit_environment: profile.audit_environment.to_string(),
        sdk_flag: profile.sdk_flag.to_string(),
    })
}

fn send_signed_request(request: OkxSignedRequest, action: &str) -> Result<serde_json::Value> {
    let mut req = ureq::request(&request.method, &request.url);
    for (name, value) in &request.headers {
        req = req.set(name, value);
    }
    let res: serde_json::Value = if request.body.is_empty() {
        req.call()?.into_json()?
    } else {
        req.send_string(&request.body)?.into_json()?
    };
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
    let credentials = OkxCredentials {
        api_key: api_key.to_string(),
        secret: secret.to_string(),
        passphrase: passphrase.to_string(),
    };
    let request_path = OKX_ORDER_PATH;
    let body = serde_json::to_string(order)?;
    let request = build_signed_request(profile, &credentials, "POST", request_path, &body)?;
    send_signed_request(request, "下单")
}

pub fn query_order(
    api_key: &str,
    secret: &str,
    passphrase: &str,
    inst_id: &str,
    ord_id: Option<&str>,
    cl_ord_id: Option<&str>,
) -> Result<serde_json::Value> {
    query_order_with_profile(
        OkxTradingProfile::demo(),
        api_key,
        secret,
        passphrase,
        inst_id,
        ord_id,
        cl_ord_id,
    )
}

pub fn query_order_with_profile(
    profile: OkxTradingProfile,
    api_key: &str,
    secret: &str,
    passphrase: &str,
    inst_id: &str,
    ord_id: Option<&str>,
    cl_ord_id: Option<&str>,
) -> Result<serde_json::Value> {
    let credentials = OkxCredentials {
        api_key: api_key.to_string(),
        secret: secret.to_string(),
        passphrase: passphrase.to_string(),
    };
    let path = okx_order_lookup_path(inst_id, ord_id, cl_ord_id)?;
    let request = build_signed_request(profile, &credentials, "GET", &path, "")?;
    send_signed_request(request, "查单")
}

pub fn cancel_order(
    api_key: &str,
    secret: &str,
    passphrase: &str,
    inst_id: &str,
    ord_id: Option<&str>,
    cl_ord_id: Option<&str>,
) -> Result<serde_json::Value> {
    cancel_order_with_profile(
        OkxTradingProfile::demo(),
        api_key,
        secret,
        passphrase,
        inst_id,
        ord_id,
        cl_ord_id,
    )
}

pub fn cancel_order_with_profile(
    profile: OkxTradingProfile,
    api_key: &str,
    secret: &str,
    passphrase: &str,
    inst_id: &str,
    ord_id: Option<&str>,
    cl_ord_id: Option<&str>,
) -> Result<serde_json::Value> {
    validate_order_lookup(inst_id, ord_id, cl_ord_id)?;
    let credentials = OkxCredentials {
        api_key: api_key.to_string(),
        secret: secret.to_string(),
        passphrase: passphrase.to_string(),
    };
    let body = serde_json::to_string(&OkxCancelOrderRequest {
        inst_id: inst_id.to_string(),
        ord_id: clean_optional_token(ord_id),
        cl_ord_id: clean_optional_token(cl_ord_id),
    })?;
    let request =
        build_signed_request(profile, &credentials, "POST", OKX_CANCEL_ORDER_PATH, &body)?;
    send_signed_request(request, "撤单")
}

pub fn okx_order_lookup_path(
    inst_id: &str,
    ord_id: Option<&str>,
    cl_ord_id: Option<&str>,
) -> Result<String> {
    validate_order_lookup(inst_id, ord_id, cl_ord_id)?;
    let mut path = format!("{}?instId={}", OKX_ORDER_PATH, inst_id.trim());
    if let Some(ord_id) = clean_optional_token(ord_id) {
        path.push_str("&ordId=");
        path.push_str(&ord_id);
    }
    if let Some(cl_ord_id) = clean_optional_token(cl_ord_id) {
        path.push_str("&clOrdId=");
        path.push_str(&cl_ord_id);
    }
    Ok(path)
}

fn validate_order_lookup(
    inst_id: &str,
    ord_id: Option<&str>,
    cl_ord_id: Option<&str>,
) -> Result<()> {
    let inst_id = inst_id.trim();
    if inst_id.is_empty() || !inst_id.chars().all(valid_okx_inst_char) {
        bail!("OKX instId 不能为空，且只能包含 ASCII 字母、数字和连字符");
    }
    if clean_optional_token(ord_id).is_none() && clean_optional_token(cl_ord_id).is_none() {
        bail!("OKX 查单/撤单必须提供 ordId 或 clOrdId");
    }
    for token in [ord_id, cl_ord_id].into_iter().flatten() {
        let token = token.trim();
        if token.is_empty() || !token.chars().all(valid_okx_id_char) {
            bail!("OKX ordId/clOrdId 只能包含 ASCII 字母、数字、连字符和下划线");
        }
    }
    Ok(())
}

fn clean_optional_token(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

fn valid_okx_inst_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || ch == '-'
}

fn valid_okx_id_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || ch == '-' || ch == '_'
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
    let credentials = OkxCredentials {
        api_key: api_key.to_string(),
        secret: secret.to_string(),
        passphrase: passphrase.to_string(),
    };
    let request = build_signed_request(profile, &credentials, "GET", OKX_BALANCE_PATH, "")?;
    send_signed_request(request, "查询余额")
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
    fn okx_order_request_serializes_okx_v5_field_names() {
        let request = OkxOrderRequest {
            inst_id: "BTC-USDT".to_string(),
            td_mode: "cash".to_string(),
            side: "buy".to_string(),
            ord_type: "limit".to_string(),
            sz: "0.01".to_string(),
            cl_ord_id: Some("qp-w0-2".to_string()),
            px: Some("70000".to_string()),
        };

        let value = serde_json::to_value(request).unwrap();
        assert_eq!(value["instId"], "BTC-USDT");
        assert_eq!(value["tdMode"], "cash");
        assert_eq!(value["ordType"], "limit");
        assert_eq!(value["clOrdId"], "qp-w0-2");
        assert!(value.get("inst_id").is_none());
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
        assert_eq!(profile.audit_environment, OKX_DEMO_AUDIT_ENVIRONMENT);
    }

    #[test]
    fn okx_demo_signed_request_pins_simulated_header_and_flag() {
        let credentials = OkxCredentials {
            api_key: "key".to_string(),
            secret: "secret".to_string(),
            passphrase: "pass".to_string(),
        };
        let signed = build_signed_request_with_timestamp(
            OkxTradingProfile::demo(),
            &credentials,
            "POST",
            OKX_ORDER_PATH,
            r#"{"instId":"BTC-USDT"}"#,
            "2026-05-25T00:00:00.000Z",
        )
        .unwrap();

        assert_eq!(signed.method, "POST");
        assert_eq!(signed.path, OKX_ORDER_PATH);
        assert_eq!(signed.sdk_flag, "1");
        assert_eq!(signed.audit_environment, OKX_DEMO_AUDIT_ENVIRONMENT);
        assert!(signed
            .headers
            .iter()
            .any(|(name, value)| name == "x-simulated-trading" && value == "1"));
        assert!(!signed
            .headers
            .iter()
            .any(|(name, value)| name == "x-simulated-trading" && value == "0"));
    }

    #[test]
    fn okx_order_query_and_cancel_paths_are_acknowledgement_paths() {
        let path = okx_order_lookup_path("BTC-USDT", Some("123"), Some("qp-w0-2")).unwrap();
        assert_eq!(
            path,
            "/api/v5/trade/order?instId=BTC-USDT&ordId=123&clOrdId=qp-w0-2"
        );

        let cancel = OkxCancelOrderRequest {
            inst_id: "BTC-USDT".to_string(),
            ord_id: Some("123".to_string()),
            cl_ord_id: None,
        };
        let value = serde_json::to_value(cancel).unwrap();
        assert_eq!(value["instId"], "BTC-USDT");
        assert_eq!(value["ordId"], "123");
        assert!(value.get("clOrdId").is_none());
        assert!(okx_order_lookup_path("BTC-USDT", None, None).is_err());
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
        let _query: fn(
            &str,
            &str,
            &str,
            &str,
            Option<&str>,
            Option<&str>,
        ) -> Result<serde_json::Value> = query_order;
        let _cancel: fn(
            &str,
            &str,
            &str,
            &str,
            Option<&str>,
            Option<&str>,
        ) -> Result<serde_json::Value> = cancel_order;
        assert!(validate_credentials("", "secret", "passphrase").is_err());
        assert!(validate_credentials("key", "secret", "passphrase").is_ok());
    }
}
