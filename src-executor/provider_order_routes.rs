use axum::{extract::State, http::StatusCode, Json};
use std::sync::Arc;

use super::executor_state::ExecutorState;
use super::{
    append_audit, build_okx_demo_order_request, ensure_okx_demo_provider_mode, internal_error,
    load_okx_demo_credentials, okx_demo_lookup_audit_payload, okx_demo_order_audit_payload,
    okx_demo_provider_audit_details, okx_demo_provider_response, okx_provider_error, okx_rest,
    OkxDemoOrderLookupRequest, OkxDemoOrderSubmitRequest,
};

pub(super) async fn submit_okx_demo_order(
    State(state): State<Arc<ExecutorState>>,
    Json(req): Json<OkxDemoOrderSubmitRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    ensure_okx_demo_provider_mode(&state, req.strategy_id.as_deref())?;
    let credentials = load_okx_demo_credentials()?;
    let order = build_okx_demo_order_request(&req)?;
    let audit_request = okx_demo_order_audit_payload(req.strategy_id.as_deref(), &order);
    let credential_source = credentials.source;
    let provider_response = tokio::task::spawn_blocking(move || {
        okx_rest::place_order(
            &credentials.api_key,
            &credentials.secret,
            &credentials.passphrase,
            &order,
        )
    })
    .await
    .map_err(|error| internal_error(format!("OKX 模拟盘下单任务失败: {}", error)))?
    .map_err(|error| okx_provider_error("下单", error))?;

    append_audit(
        &state,
        "okx_demo_provider_order_submit",
        req.strategy_id.clone(),
        okx_demo_provider_audit_details(
            "submit",
            credential_source,
            audit_request,
            &provider_response,
        ),
    );
    Ok(Json(okx_demo_provider_response(
        "submitted",
        req.strategy_id.as_deref(),
        provider_response,
    )))
}

pub(super) async fn query_okx_demo_order(
    State(state): State<Arc<ExecutorState>>,
    Json(req): Json<OkxDemoOrderLookupRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    ensure_okx_demo_provider_mode(&state, req.strategy_id.as_deref())?;
    let credentials = load_okx_demo_credentials()?;
    let audit_request = okx_demo_lookup_audit_payload(req.strategy_id.as_deref(), &req);
    let inst_id = req.inst_id.clone();
    let ord_id = req.ord_id.clone();
    let cl_ord_id = req.cl_ord_id.clone();
    let credential_source = credentials.source;
    let provider_response = tokio::task::spawn_blocking(move || {
        okx_rest::query_order(
            &credentials.api_key,
            &credentials.secret,
            &credentials.passphrase,
            &inst_id,
            ord_id.as_deref(),
            cl_ord_id.as_deref(),
        )
    })
    .await
    .map_err(|error| internal_error(format!("OKX 模拟盘查单任务失败: {}", error)))?
    .map_err(|error| okx_provider_error("查单", error))?;

    append_audit(
        &state,
        "okx_demo_provider_order_query",
        req.strategy_id.clone(),
        okx_demo_provider_audit_details(
            "query",
            credential_source,
            audit_request,
            &provider_response,
        ),
    );
    Ok(Json(okx_demo_provider_response(
        "queried",
        req.strategy_id.as_deref(),
        provider_response,
    )))
}

pub(super) async fn cancel_okx_demo_order(
    State(state): State<Arc<ExecutorState>>,
    Json(req): Json<OkxDemoOrderLookupRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    ensure_okx_demo_provider_mode(&state, req.strategy_id.as_deref())?;
    let credentials = load_okx_demo_credentials()?;
    let audit_request = okx_demo_lookup_audit_payload(req.strategy_id.as_deref(), &req);
    let inst_id = req.inst_id.clone();
    let ord_id = req.ord_id.clone();
    let cl_ord_id = req.cl_ord_id.clone();
    let credential_source = credentials.source;
    let provider_response = tokio::task::spawn_blocking(move || {
        okx_rest::cancel_order(
            &credentials.api_key,
            &credentials.secret,
            &credentials.passphrase,
            &inst_id,
            ord_id.as_deref(),
            cl_ord_id.as_deref(),
        )
    })
    .await
    .map_err(|error| internal_error(format!("OKX 模拟盘撤单任务失败: {}", error)))?
    .map_err(|error| okx_provider_error("撤单", error))?;

    append_audit(
        &state,
        "okx_demo_provider_order_cancel",
        req.strategy_id.clone(),
        okx_demo_provider_audit_details(
            "cancel",
            credential_source,
            audit_request,
            &provider_response,
        ),
    );
    Ok(Json(okx_demo_provider_response(
        "cancelled",
        req.strategy_id.as_deref(),
        provider_response,
    )))
}
