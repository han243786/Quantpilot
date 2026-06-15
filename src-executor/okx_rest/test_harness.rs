use super::{
    signing_request_surface::{build_signed_request_with_timestamp, okx_timestamp},
    transport_response_surface::send_signed_request_raw,
    *,
};
use anyhow::Result;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

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
    let _place: fn(&str, &str, &str, &OkxOrderRequest) -> Result<serde_json::Value> = place_order;
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
        if !api_key.trim().is_empty() && !secret.trim().is_empty() && !passphrase.trim().is_empty()
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
        if !api_key.trim().is_empty() && !secret.trim().is_empty() && !passphrase.trim().is_empty()
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
