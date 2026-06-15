use axum::http::StatusCode;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use super::credential_vault_v2;
use super::executor_state::{ExecutionMode, ExecutorState};
use super::okx_rest::{OkxOrderRequest, OKX_DEMO_AUDIT_ENVIRONMENT, OKX_DEMO_PROVIDER_KEY};

#[derive(Debug, serde::Deserialize)]
pub(super) struct OkxDemoOrderSubmitRequest {
    #[serde(default)]
    pub(super) strategy_id: Option<String>,
    pub(super) inst_id: String,
    pub(super) side: String,
    pub(super) sz: String,
    #[serde(default = "default_okx_td_mode")]
    pub(super) td_mode: String,
    #[serde(default = "default_okx_order_type")]
    pub(super) ord_type: String,
    #[serde(default)]
    pub(super) px: Option<String>,
    #[serde(default)]
    pub(super) cl_ord_id: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
pub(super) struct OkxDemoOrderLookupRequest {
    #[serde(default)]
    pub(super) strategy_id: Option<String>,
    pub(super) inst_id: String,
    #[serde(default)]
    pub(super) ord_id: Option<String>,
    #[serde(default)]
    pub(super) cl_ord_id: Option<String>,
}

#[derive(Debug, Clone)]
pub(super) struct OkxDemoCredentialSet {
    pub(super) api_key: String,
    pub(super) secret: String,
    pub(super) passphrase: String,
    pub(super) source: &'static str,
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

fn default_okx_td_mode() -> String {
    "cash".to_string()
}

fn default_okx_order_type() -> String {
    "limit".to_string()
}

pub(super) fn ensure_okx_demo_provider_mode(
    state: &ExecutorState,
    strategy_id: Option<&str>,
) -> Result<(), (StatusCode, String)> {
    if state.current_mode() != ExecutionMode::PaperActual {
        return Err((
            StatusCode::LOCKED,
            serde_json::json!({
                "error": "paper_actual_required",
                "message": "OKX 模拟盘 provider 回执路径只允许在 paper_actual 模式调用",
                "required_mode": "paper_actual",
                "current_mode": state.current_mode().as_str(),
                "environment": OKX_DEMO_AUDIT_ENVIRONMENT,
            })
            .to_string(),
        ));
    }
    if let Some(strategy_id) = strategy_id {
        let strategies = state
            .strategies
            .read()
            .map_err(|error| (StatusCode::INTERNAL_SERVER_ERROR, format!("锁: {}", error)))?;
        let strategy = strategies.get(strategy_id).ok_or((
            StatusCode::NOT_FOUND,
            format!("策略不存在: {}", strategy_id),
        ))?;
        if strategy.execution_mode != ExecutionMode::PaperActual {
            return Err((
                StatusCode::LOCKED,
                serde_json::json!({
                    "error": "strategy_paper_actual_required",
                    "message": "该策略不是 OKX 模拟盘 / 非真实资金 execution_mode",
                    "strategy_id": strategy_id,
                    "strategy_mode": strategy.execution_mode.as_str(),
                    "required_mode": "paper_actual",
                })
                .to_string(),
            ));
        }
    }
    Ok(())
}

pub(super) fn build_okx_demo_order_request(
    req: &OkxDemoOrderSubmitRequest,
) -> Result<OkxOrderRequest, (StatusCode, String)> {
    let inst_id = clean_required_ascii(&req.inst_id, "inst_id", valid_okx_inst_char)?;
    let side = clean_enum(&req.side, "side", &["buy", "sell"])?;
    let ord_type = clean_enum(&req.ord_type, "ord_type", &["market", "limit"])?;
    let td_mode = clean_enum(&req.td_mode, "td_mode", &["cash"])?;
    let sz = clean_positive_decimal(&req.sz, "sz")?;
    let px = match req.px.as_deref() {
        Some(value) => Some(clean_positive_decimal(value, "px")?),
        None if ord_type == "limit" => {
            return Err((
                StatusCode::BAD_REQUEST,
                "OKX 模拟盘限价单必须提供 px".to_string(),
            ))
        }
        None => None,
    };
    let cl_ord_id = match req.cl_ord_id.as_deref() {
        Some(value) => Some(clean_required_ascii(value, "cl_ord_id", valid_okx_id_char)?),
        None => Some(default_okx_client_order_id()),
    };

    Ok(OkxOrderRequest {
        inst_id,
        td_mode,
        side,
        ord_type,
        sz,
        cl_ord_id,
        px,
    })
}

fn clean_required_ascii(
    value: &str,
    field: &str,
    valid: fn(char) -> bool,
) -> Result<String, (StatusCode, String)> {
    let trimmed = value.trim();
    if trimmed.is_empty() || !trimmed.chars().all(valid) {
        return Err((
            StatusCode::BAD_REQUEST,
            format!("{} 不能为空，且包含不允许的字符", field),
        ));
    }
    Ok(trimmed.to_string())
}

fn clean_enum(value: &str, field: &str, allowed: &[&str]) -> Result<String, (StatusCode, String)> {
    let normalized = value.trim().to_ascii_lowercase();
    if allowed.contains(&normalized.as_str()) {
        Ok(normalized)
    } else {
        Err((
            StatusCode::BAD_REQUEST,
            format!("{} 只允许 {}", field, allowed.join(", ")),
        ))
    }
}

fn clean_positive_decimal(value: &str, field: &str) -> Result<String, (StatusCode, String)> {
    let trimmed = value.trim();
    let parsed = trimmed
        .parse::<f64>()
        .map_err(|_| (StatusCode::BAD_REQUEST, format!("{} 必须是正数", field)))?;
    if !parsed.is_finite() || parsed <= 0.0 {
        return Err((StatusCode::BAD_REQUEST, format!("{} 必须是有限正数", field)));
    }
    Ok(trimmed.to_string())
}

fn valid_okx_inst_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || ch == '-'
}

fn valid_okx_id_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || ch == '-' || ch == '_'
}

fn default_okx_client_order_id() -> String {
    format!("qpw02{}", now_ms())
        .chars()
        .take(32)
        .collect::<String>()
}

pub(super) fn load_okx_demo_credentials() -> Result<OkxDemoCredentialSet, (StatusCode, String)> {
    if let Some(credentials) = credentials_from_env() {
        return Ok(credentials);
    }
    credentials_from_executor_vault().ok_or_else(|| {
        (
            StatusCode::BAD_REQUEST,
            serde_json::json!({
                "error": "missing_okx_demo_credentials",
                "message": "OKX 模拟盘需要配置 QUANTPILOT_OKX_DEMO_API_KEY/SECRET/PASSPHRASE，或执行端凭证保险库服务 okx_demo，或旧主凭证库标签 0:okx_test / 0:regtest_okx",
                "environment": OKX_DEMO_AUDIT_ENVIRONMENT,
            })
            .to_string(),
        )
    })
}

fn credentials_from_env() -> Option<OkxDemoCredentialSet> {
    let api_key = std::env::var("QUANTPILOT_OKX_DEMO_API_KEY").ok()?;
    let secret = std::env::var("QUANTPILOT_OKX_DEMO_SECRET").ok()?;
    let passphrase = std::env::var("QUANTPILOT_OKX_DEMO_PASSPHRASE").ok()?;
    if api_key.trim().is_empty() || secret.trim().is_empty() || passphrase.trim().is_empty() {
        return None;
    }
    Some(OkxDemoCredentialSet {
        api_key,
        secret,
        passphrase,
        source: "env",
    })
}

fn credentials_from_executor_vault() -> Option<OkxDemoCredentialSet> {
    if let Ok(vault) = credential_vault_v2::ExecutorCredentialVault::load(&executor_storage_dir()) {
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
                return Some(OkxDemoCredentialSet {
                    api_key,
                    secret,
                    passphrase,
                    source: "executor_vault",
                });
            }
        }
    }
    credentials_from_legacy_vault()
}

fn credentials_from_legacy_vault() -> Option<OkxDemoCredentialSet> {
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
        if !api_key.trim().is_empty() && !secret.trim().is_empty() && !passphrase.trim().is_empty()
        {
            return Some(OkxDemoCredentialSet {
                api_key,
                secret,
                passphrase,
                source: "legacy_vault",
            });
        }
    }
    None
}

fn executor_storage_dir() -> PathBuf {
    std::env::var_os("QUANTPILOT_STORAGE_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("storage"))
}

pub(super) fn okx_provider_error(action: &str, error: anyhow::Error) -> (StatusCode, String) {
    let message = quantpilot::safe_log::sanitize_secrets(&format!("{:#}", error));
    (
        StatusCode::BAD_GATEWAY,
        serde_json::json!({
            "error": "okx_demo_provider_error",
            "action": action,
            "message": message,
            "provider": OKX_DEMO_PROVIDER_KEY,
            "environment": OKX_DEMO_AUDIT_ENVIRONMENT,
            "simulated_trading": true,
        })
        .to_string(),
    )
}

pub(super) fn okx_demo_order_audit_payload(
    strategy_id: Option<&str>,
    order: &OkxOrderRequest,
) -> serde_json::Value {
    serde_json::json!({
        "strategy_id": strategy_id,
        "inst_id": &order.inst_id,
        "td_mode": &order.td_mode,
        "side": &order.side,
        "ord_type": &order.ord_type,
        "sz": &order.sz,
        "px": &order.px,
        "cl_ord_id": &order.cl_ord_id,
    })
}

pub(super) fn okx_demo_lookup_audit_payload(
    strategy_id: Option<&str>,
    req: &OkxDemoOrderLookupRequest,
) -> serde_json::Value {
    serde_json::json!({
        "strategy_id": strategy_id,
        "inst_id": &req.inst_id,
        "ord_id": &req.ord_id,
        "cl_ord_id": &req.cl_ord_id,
    })
}

pub(super) fn okx_demo_provider_audit_details(
    action: &str,
    credential_source: &str,
    request: serde_json::Value,
    provider_response: &serde_json::Value,
) -> serde_json::Value {
    serde_json::json!({
        "provider": OKX_DEMO_PROVIDER_KEY,
        "environment": OKX_DEMO_AUDIT_ENVIRONMENT,
        "simulated_trading": true,
        "demo_flag": "1",
        "simulated_trading_header": "x-simulated-trading=1",
        "action": action,
        "credential_source": credential_source,
        "request": request,
        "provider_result": {
            "code": provider_response.get("code").and_then(|value| value.as_str()).unwrap_or("unknown"),
            "first_order_id": provider_response
                .get("data")
                .and_then(|value| value.as_array())
                .and_then(|items| items.first())
                .and_then(|item| item.get("ordId"))
                .and_then(|value| value.as_str()),
            "first_state": provider_response
                .get("data")
                .and_then(|value| value.as_array())
                .and_then(|items| items.first())
                .and_then(|item| item.get("state"))
                .and_then(|value| value.as_str()),
        }
    })
}

pub(super) fn okx_demo_provider_response(
    status: &str,
    strategy_id: Option<&str>,
    provider_response: serde_json::Value,
) -> serde_json::Value {
    serde_json::json!({
        "status": status,
        "strategy_id": strategy_id,
        "provider": OKX_DEMO_PROVIDER_KEY,
        "environment": OKX_DEMO_AUDIT_ENVIRONMENT,
        "simulated_trading": true,
        "demo_flag": "1",
        "provider_response": provider_response,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn okx_demo_provider_route_requires_paper_actual_mode() {
        let state = ExecutorState::new();
        let error = ensure_okx_demo_provider_mode(&state, None).unwrap_err();
        assert_eq!(error.0, StatusCode::LOCKED);
        assert!(error.1.contains("paper_actual"));

        state.set_mode(ExecutionMode::PaperActual);
        ensure_okx_demo_provider_mode(&state, None).unwrap();
    }

    #[test]
    fn okx_demo_order_request_validation_keeps_provider_specific_shape() {
        let request = OkxDemoOrderSubmitRequest {
            strategy_id: Some("s1".to_string()),
            inst_id: "BTC-USDT".to_string(),
            side: "BUY".to_string(),
            sz: "0.001".to_string(),
            td_mode: "cash".to_string(),
            ord_type: "limit".to_string(),
            px: Some("70000".to_string()),
            cl_ord_id: Some("qp_w0_2".to_string()),
        };

        let order = build_okx_demo_order_request(&request).unwrap();
        let value = serde_json::to_value(order).unwrap();
        assert_eq!(value["instId"], "BTC-USDT");
        assert_eq!(value["tdMode"], "cash");
        assert_eq!(value["side"], "buy");
        assert_eq!(value["ordType"], "limit");
        assert_eq!(value["clOrdId"], "qp_w0_2");
        assert_eq!(value["px"], "70000");
    }

    #[test]
    fn okx_demo_audit_details_never_include_credentials_or_signatures() {
        let request = serde_json::json!({
            "inst_id": "BTC-USDT",
            "side": "buy",
            "ord_type": "limit",
            "sz": "0.001",
            "px": "70000",
        });
        let provider_response = serde_json::json!({
            "code": "0",
            "data": [{"ordId": "123", "state": "live"}],
        });

        let details = okx_demo_provider_audit_details("submit", "env", request, &provider_response);
        let text = details.to_string();
        assert!(text.contains("x-simulated-trading=1"));
        assert!(text.contains(OKX_DEMO_AUDIT_ENVIRONMENT));
        assert!(!text.contains("OK-ACCESS"));
        assert!(!text.contains("secret"));
        assert!(!text.contains("signature"));
        assert!(!text.contains("passphrase"));
    }
}
