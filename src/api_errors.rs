use super::*;

/// @deprecated 遗留错误格式。新代码请使用 `problem_bad_request()` (RFC 9457)。
pub(crate) fn json_bad_request(
    error: &'static str,
    message: impl Into<String>,
) -> (StatusCode, String) {
    json_bad_request_with_details_and_partial(error, message, Vec::new(), None)
}

pub(super) fn json_bad_request_with_details(
    error: &'static str,
    message: impl Into<String>,
    details: Vec<ApiErrorDetail>,
) -> (StatusCode, String) {
    json_bad_request_with_details_and_partial(error, message, details, None)
}

#[allow(dead_code)]
pub(super) fn json_bad_request_with_partial(
    error: &'static str,
    message: impl Into<String>,
    quantscript_authoring_view: Option<QuantScriptAuthoringView>,
) -> (StatusCode, String) {
    json_bad_request_with_details_and_partial(
        error,
        message,
        Vec::new(),
        quantscript_authoring_view,
    )
}

pub(super) fn json_bad_request_with_details_and_partial(
    error: &'static str,
    message: impl Into<String>,
    details: Vec<ApiErrorDetail>,
    quantscript_authoring_view: Option<QuantScriptAuthoringView>,
) -> (StatusCode, String) {
    let payload = ApiErrorResponse {
        error,
        message: message.into(),
        details,
        partial_artifacts: if quantscript_authoring_view.is_some() {
            Some(ApiPartialArtifacts {
                quantscript_authoring_view,
            })
        } else {
            None
        },
    };
    (
        StatusCode::BAD_REQUEST,
        serde_json::to_string(&payload).unwrap_or_else(|_| {
            "{\"error\":\"serialization_failure\",\"message\":\"序列化错误响应失败\",\"details\":[]}".to_string()
        }),
    )
}

pub(super) fn internal_error(error: anyhow::Error) -> (StatusCode, String) {
    let dev_mode = std::env::var("QUANTPILOT_DEV")
        .map(|v| v == "true" || v == "1")
        .unwrap_or(false);
    let detail = crate::safe_log::sanitize_secrets(&format!("{:#}", error));
    let message = if dev_mode {
        format!("内部服务器错误。详情: {}", detail)
    } else {
        "内部服务器错误。请重试或联系支持。".to_string()
    };
    let payload = serde_json::json!({
        "error": "internal_error",
        "message": message,
    });
    (StatusCode::INTERNAL_SERVER_ERROR, payload.to_string())
}

pub(super) fn io_error(error: std::io::Error) -> (StatusCode, String) {
    (StatusCode::INTERNAL_SERVER_ERROR, error.to_string())
}

pub(super) fn not_found_io_error(error: std::io::Error) -> (StatusCode, String) {
    if error.kind() == std::io::ErrorKind::NotFound {
        (StatusCode::NOT_FOUND, "请求的资源不存在".to_string())
    } else {
        (StatusCode::INTERNAL_SERVER_ERROR, "内部服务器错误".to_string())
    }
}

// ── 废弃的 problem_* 系列已随 v0.4.2 D3 迁移全部移除 ──
