// v3.5.0/v4.8.0: OKX REST API 客户端 (demo trading only)
// 文档: https://www.okx.com/docs-v5/
// Demo trading: https://www.okx.com/api/v5 (需在 headers 中设置 x-simulated-trading: 1)
mod lookup_validation_surface;
mod order_action_surface;
mod signing_request_surface;
mod transport_response_surface;
pub use lookup_validation_surface::okx_order_lookup_path;
pub use order_action_surface::{
    cancel_order, cancel_order_with_profile, fetch_balance, fetch_balance_with_profile,
    place_order, place_order_with_profile, query_order, query_order_with_profile,
};
pub use signing_request_surface::build_signed_request;

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

#[cfg(test)]
mod test_harness;
