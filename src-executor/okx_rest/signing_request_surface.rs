use anyhow::{bail, Result};
use base64::Engine;
use ring::hmac;
use std::time::{SystemTime, UNIX_EPOCH};

use super::{OkxCredentials, OkxSignedRequest, OkxTradingProfile, OKX_REST_BASE_URL_ENV};

/// Generate OKX API HMAC-SHA256 signature.
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

pub(super) fn okx_timestamp() -> String {
    let dur = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    let secs = dur.as_secs();
    let millis = dur.subsec_millis();
    let dt = chrono::DateTime::from_timestamp(secs as i64, millis * 1_000_000).unwrap_or_default();
    dt.format("%Y-%m-%dT%H:%M:%S.%3fZ").to_string()
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

pub(super) fn build_signed_request_with_timestamp(
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
        url: format!(
            "{}{}",
            resolved_rest_base_url(profile.rest_base_url)?,
            request_path
        ),
        body: body.to_string(),
        headers,
        audit_environment: profile.audit_environment.to_string(),
        sdk_flag: profile.sdk_flag.to_string(),
    })
}

fn resolved_rest_base_url(default_base_url: &str) -> Result<String> {
    let raw = std::env::var(OKX_REST_BASE_URL_ENV).unwrap_or_else(|_| default_base_url.to_string());
    let base_url = raw.trim().trim_end_matches('/').to_string();
    if !base_url.starts_with("https://") {
        bail!("{OKX_REST_BASE_URL_ENV} 必须是 https:// 开头的 OKX REST base URL");
    }
    if base_url.contains("/api/v5") {
        bail!("{OKX_REST_BASE_URL_ENV} 只允许配置 base URL，不包含 /api/v5 路径");
    }
    Ok(base_url)
}

#[cfg(test)]
mod tests {
    use super::super::{
        OkxCredentials, OkxTradingProfile, OKX_DEMO_AUDIT_ENVIRONMENT, OKX_ORDER_PATH,
        OKX_PRODUCTION_SDK_FLAG,
    };
    use super::*;
    use base64::Engine;

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
        let sig_str = sig.unwrap();
        assert!(!sig_str.is_empty());
        base64::engine::general_purpose::STANDARD
            .decode(&sig_str)
            .expect("signature should be valid base64");
    }

    #[test]
    fn test_okx_timestamp_is_valid() {
        let ts = okx_timestamp();
        let parsed =
            chrono::DateTime::parse_from_rfc3339(&ts).expect("OKX timestamp should be RFC3339");
        assert!(ts.ends_with('Z'));
        assert!(ts.contains('.'));
        assert!(parsed.timestamp() > 1_700_000_000);
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
    fn production_readonly_profile_rejects_empty_credentials_before_network() {
        let profile = OkxTradingProfile {
            rest_base_url: "https://www.okx.com",
            sdk_flag: OKX_PRODUCTION_SDK_FLAG,
            simulated_trading_header: None,
            audit_environment: "okx_production_readonly_probe",
        };
        let credentials = OkxCredentials {
            api_key: String::new(),
            secret: "secret".to_string(),
            passphrase: "pass".to_string(),
        };
        let error = build_signed_request(
            profile,
            &credentials,
            "GET",
            "/api/v5/trade/order?instId=BTC-USDT&clOrdId=qp_env_mismatch_probe",
            "",
        )
        .unwrap_err();
        assert!(error.to_string().contains("api_key"));
    }
}
