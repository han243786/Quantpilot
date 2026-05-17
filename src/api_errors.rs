use super::*;

/// QuantPilot 标准错误格式。所有 API handler 使用此格式返回错误。
/// 格式: `{"error": "<type>", "error_code": "<code>", "message": "<message>", "details": [...]}`
/// v2.3.0: 新增 json_bad_request_with_code 支持语言中立 error_code, 旧函数保持兼容
pub(crate) fn json_bad_request(
    error: &'static str,
    message: impl Into<String>,
) -> (StatusCode, String) {
    json_bad_request_with_details_and_partial(error, message, Vec::new(), None)
}

/// v2.3.0: 带语言中立错误码的错误响应
pub(crate) fn json_bad_request_with_code(
    error: &'static str,
    code: &'static str,
    message: impl Into<String>,
) -> (StatusCode, String) {
    let payload = ApiErrorResponse {
        error,
        error_code: Some(code),
        message: message.into(),
        details: Vec::new(),
        partial_artifacts: None,
    };
    (StatusCode::BAD_REQUEST, serde_json::to_string(&payload).unwrap_or_else(|_| {
        "{\"error\":\"bad_request\",\"message\":\"序列化错误响应失败\"}".to_string()
    }))
}

pub(super) fn json_bad_request_with_details(
    error: &'static str,
    message: impl Into<String>,
    details: Vec<ApiErrorDetail>,
) -> (StatusCode, String) {
    json_bad_request_with_details_and_partial(error, message, details, None)
}

pub(super) fn json_bad_request_with_details_and_partial(
    error: &'static str,
    message: impl Into<String>,
    details: Vec<ApiErrorDetail>,
    quantscript_authoring_view: Option<QuantScriptAuthoringView>,
) -> (StatusCode, String) {
    let payload = ApiErrorResponse {
        error,
        error_code: None,
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
    let short = crate::safe_log::sanitize_secrets(&format!("{}", error.root_cause()));
    // v2.1.2: DEV模式下将完整错误写入日志，API响应仅返回简短摘要
    let dev_mode = std::env::var("QUANTPILOT_DEV")
        .map(|v| v == "true" || v == "1")
        .unwrap_or(false);
    if dev_mode {
        let full = crate::safe_log::sanitize_secrets(&format!("{:#}", error));
        safe_eprintln!("[internal_error] {}", full);
    }
    let message = format!("内部服务器错误: {}。如问题持续请重试或联系支持。", short);
    let payload = serde_json::json!({
        "error": "internal_error",
        "message": message,
    });
    (StatusCode::INTERNAL_SERVER_ERROR, payload.to_string())
}

pub(super) fn io_error(error: std::io::Error) -> (StatusCode, String) {
    // v1.2.0: 不向用户泄露 OS 路径详情，仅记录日志
    safe_eprintln!("[io_error] {}", error);
    (StatusCode::INTERNAL_SERVER_ERROR, "内部服务器错误，请重试".to_string())
}

pub(super) fn not_found_io_error(error: std::io::Error) -> (StatusCode, String) {
    if error.kind() == std::io::ErrorKind::NotFound {
        (StatusCode::NOT_FOUND, "请求的资源不存在".to_string())
    } else {
        (StatusCode::INTERNAL_SERVER_ERROR, "内部服务器错误".to_string())
    }
}

// v0.4.2 D3: RFC 9457 problem_* 系列已移除。json_bad_request* 为当前标准错误格式。
// v1.2.0: 统一注释，消除 deprecated 声明与实际使用的矛盾。
