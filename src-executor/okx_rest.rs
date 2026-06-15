/// v3.5.0/v4.8.0: OKX REST API 客户端 (demo trading only)
/// 文档: https://www.okx.com/docs-v5/
/// Demo trading: https://www.okx.com/api/v5 (需在 headers 中设置 x-simulated-trading: 1)
use anyhow::{bail, Context, Result};
use std::time::Duration;

mod signing_request_surface;
pub use signing_request_surface::build_signed_request;

#[cfg(test)]
use signing_request_surface::{build_signed_request_with_timestamp, okx_timestamp};

const OKX_REST_BASE: &str = "https://www.okx.com";
const OKX_REST_BASE_URL_ENV: &str = "QUANTPILOT_OKX_REST_BASE_URL";
const OKX_REST_PROXY_ENV: &str = "QUANTPILOT_OKX_REST_PROXY";
const OKX_REST_CONNECT_TIMEOUT_SECS: u64 = 10;
const OKX_REST_READ_TIMEOUT_SECS: u64 = 20;
const OKX_REST_WRITE_TIMEOUT_SECS: u64 = 20;
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

fn send_signed_request_raw(request: OkxSignedRequest) -> Result<serde_json::Value> {
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

fn send_signed_request(request: OkxSignedRequest, action: &str) -> Result<serde_json::Value> {
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
    use std::time::{SystemTime, UNIX_EPOCH};

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
    }

    fn probe_credentials_from_env() -> Option<OkxCredentials> {
        let api_key = std::env::var("QUANTPILOT_OKX_DEMO_API_KEY").ok()?;
        let secret = std::env::var("QUANTPILOT_OKX_DEMO_SECRET").ok()?;
        let passphrase = std::env::var("QUANTPILOT_OKX_DEMO_PASSPHRASE").ok()?;
        if api_key.trim().is_empty() || secret.trim().is_empty() || passphrase.trim().is_empty() {
            return None;
        }
        Some(OkxCredentials {
            api_key,
            secret,
            passphrase,
        })
    }

    fn probe_credentials_from_legacy_vault() -> Option<OkxCredentials> {
        let vault = quantpilot::credential_vault::CredentialVault::load().ok()?;
        quantpilot::safe_log::register_credential_patterns(vault.extract_secret_patterns());
        for service in ["okx_demo", "okx", "0:okx_test", "0:regtest_okx"] {
            let Some(fields) = vault.get_service(service) else {
                continue;
            };
            let Some(api_key) = fields
                .get("api_key")
                .or_else(|| fields.get("key"))
                .map(|value| value.as_str().to_string())
            else {
                continue;
            };
            let Some(secret) = fields.get("secret").map(|value| value.as_str().to_string()) else {
                continue;
            };
            let Some(passphrase) = fields
                .get("passphrase")
                .map(|value| value.as_str().to_string())
            else {
                continue;
            };
            if !api_key.trim().is_empty()
                && !secret.trim().is_empty()
                && !passphrase.trim().is_empty()
            {
                return Some(OkxCredentials {
                    api_key,
                    secret,
                    passphrase,
                });
            }
        }
        None
    }

    fn probe_credentials_from_executor_vault() -> Option<OkxCredentials> {
        let storage_dir = std::env::var_os("QUANTPILOT_STORAGE_ROOT")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| std::path::PathBuf::from("storage"));
        let vault = crate::credential_vault_v2::ExecutorCredentialVault::load(&storage_dir).ok()?;
        for service in ["okx_demo", "okx"] {
            let Ok(fields) = vault.get_service(service) else {
                continue;
            };
            let Some(api_key) = fields.get("api_key").or_else(|| fields.get("key")).cloned() else {
                continue;
            };
            let Some(secret) = fields.get("secret").cloned() else {
                continue;
            };
            let Some(passphrase) = fields.get("passphrase").cloned() else {
                continue;
            };
            if !api_key.trim().is_empty()
                && !secret.trim().is_empty()
                && !passphrase.trim().is_empty()
            {
                return Some(OkxCredentials {
                    api_key,
                    secret,
                    passphrase,
                });
            }
        }
        None
    }

    fn probe_credentials() -> Option<OkxCredentials> {
        probe_credentials_from_env()
            .or_else(probe_credentials_from_executor_vault)
            .or_else(probe_credentials_from_legacy_vault)
    }

    fn retry_okx_demo_smoke_action<F>(label: &str, mut action: F) -> serde_json::Value
    where
        F: FnMut() -> Result<serde_json::Value>,
    {
        let mut last_error = None;
        for attempt in 1..=5 {
            match action() {
                Ok(value) => return value,
                Err(error) => {
                    let message = quantpilot::safe_log::sanitize_secrets(&format!("{:#}", error));
                    eprintln!(
                        "{}",
                        serde_json::json!({
                            "probe": "okx_demo_submit_query_cancel",
                            "action": label,
                            "attempt": attempt,
                            "error": message,
                        })
                    );
                    last_error = Some(message);
                    std::thread::sleep(Duration::from_millis(750));
                }
            }
        }
        panic!(
            "OKX demo smoke action failed: {}: {}",
            label,
            last_error.unwrap_or_else(|| "unknown".to_string())
        );
    }

    fn first_order_field<'a>(response: &'a serde_json::Value, field: &str) -> Option<&'a str> {
        response
            .get("data")
            .and_then(|value| value.as_array())
            .and_then(|items| items.first())
            .and_then(|item| item.get(field))
            .and_then(|value| value.as_str())
    }

    #[test]
    #[ignore]
    fn okx_demo_credentials_return_error_on_production_readonly_probe() {
        let Some(credentials) = probe_credentials() else {
            eprintln!("OKX readonly production probe skipped: no demo credentials found");
            return;
        };
        let profile = OkxTradingProfile {
            rest_base_url: OKX_REST_BASE,
            sdk_flag: OKX_PRODUCTION_SDK_FLAG,
            simulated_trading_header: None,
            audit_environment: "okx_production_readonly_probe",
        };
        let request = build_signed_request_with_timestamp(
            profile,
            &credentials,
            "GET",
            "/api/v5/trade/order?instId=BTC-USDT&clOrdId=qp_env_mismatch_probe",
            "",
            &okx_timestamp(),
        )
        .expect("production readonly probe request should sign");
        let mut response = None;
        let mut last_error = None;
        for attempt in 1..=5 {
            match send_signed_request_raw(request.clone()) {
                Ok(value) => {
                    response = Some(value);
                    break;
                }
                Err(error) => {
                    last_error = Some(error.to_string());
                    eprintln!(
                        "{}",
                        serde_json::json!({
                            "probe": "okx_demo_credentials_on_production_readonly_order_query",
                            "attempt": attempt,
                            "network_error": last_error.as_deref().unwrap_or("unknown"),
                        })
                    );
                    std::thread::sleep(Duration::from_millis(500));
                }
            }
        }
        let response = response.unwrap_or_else(|| {
            panic!(
                "production readonly probe did not reach OKX after retries: {}",
                last_error.unwrap_or_else(|| "unknown".to_string())
            )
        });
        let code = response
            .get("code")
            .and_then(|value| value.as_str())
            .unwrap_or("missing");
        let msg = response
            .get("msg")
            .and_then(|value| value.as_str())
            .unwrap_or("");
        let data_len = response
            .get("data")
            .and_then(|value| value.as_array())
            .map_or(0, Vec::len);

        println!(
            "{}",
            serde_json::json!({
                "probe": "okx_demo_credentials_on_production_readonly_order_query",
                "http_path": "/api/v5/trade/order",
                "method": "GET",
                "mutating": false,
                "simulated_trading_header": "absent",
                "sdk_flag": OKX_PRODUCTION_SDK_FLAG,
                "code": code,
                "msg": msg,
                "data_len": data_len,
            })
        );
        assert_ne!(
            code, "0",
            "production readonly probe unexpectedly accepted these credentials"
        );
    }

    #[test]
    #[ignore]
    fn okx_demo_submit_query_cancel_smoke() {
        let Some(credentials) = probe_credentials() else {
            eprintln!("OKX demo submit/query/cancel smoke skipped: no demo credentials found");
            return;
        };

        let cl_ord_id = format!(
            "qpw02{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis()
        )
        .chars()
        .take(32)
        .collect::<String>();
        let order = OkxOrderRequest {
            inst_id: "BTC-USDT".to_string(),
            td_mode: "cash".to_string(),
            side: "buy".to_string(),
            ord_type: "limit".to_string(),
            sz: "0.0001".to_string(),
            cl_ord_id: Some(cl_ord_id.clone()),
            px: Some("50000".to_string()),
        };

        let submit = retry_okx_demo_smoke_action("submit", || {
            place_order(
                &credentials.api_key,
                &credentials.secret,
                &credentials.passphrase,
                &order,
            )
        });
        let ord_id = first_order_field(&submit, "ordId").map(str::to_string);
        println!(
            "{}",
            serde_json::json!({
                "probe": "okx_demo_submit_query_cancel",
                "action": "submit",
                "code": submit.get("code").and_then(|value| value.as_str()).unwrap_or("missing"),
                "ord_id_present": ord_id.is_some(),
                "cl_ord_id": cl_ord_id,
                "environment": OKX_DEMO_AUDIT_ENVIRONMENT,
                "simulated_trading_header": "x-simulated-trading=1",
            })
        );

        let query = retry_okx_demo_smoke_action("query", || {
            query_order(
                &credentials.api_key,
                &credentials.secret,
                &credentials.passphrase,
                "BTC-USDT",
                ord_id.as_deref(),
                Some(&cl_ord_id),
            )
        });
        println!(
            "{}",
            serde_json::json!({
                "probe": "okx_demo_submit_query_cancel",
                "action": "query",
                "code": query.get("code").and_then(|value| value.as_str()).unwrap_or("missing"),
                "state": first_order_field(&query, "state"),
            })
        );

        let cancel = retry_okx_demo_smoke_action("cancel", || {
            cancel_order(
                &credentials.api_key,
                &credentials.secret,
                &credentials.passphrase,
                "BTC-USDT",
                ord_id.as_deref(),
                Some(&cl_ord_id),
            )
        });
        println!(
            "{}",
            serde_json::json!({
                "probe": "okx_demo_submit_query_cancel",
                "action": "cancel",
                "code": cancel.get("code").and_then(|value| value.as_str()).unwrap_or("missing"),
                "ord_id_present": first_order_field(&cancel, "ordId").is_some(),
            })
        );
    }
}
