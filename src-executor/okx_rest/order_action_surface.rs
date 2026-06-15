use anyhow::Result;

use super::{
    build_signed_request, clean_optional_token, okx_order_lookup_path,
    transport_response_surface::send_signed_request, validate_order_lookup, OkxCancelOrderRequest,
    OkxCredentials, OkxOrderRequest, OkxTradingProfile, OKX_BALANCE_PATH, OKX_CANCEL_ORDER_PATH,
    OKX_ORDER_PATH,
};

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
