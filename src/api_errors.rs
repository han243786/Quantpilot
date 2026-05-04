use super::*;

/// @deprecated 遗留错误格式。新代码请使用 `problem_bad_request()` (RFC 9457)。
pub(super) fn json_bad_request(
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
            "{\"error\":\"serialization_failure\",\"message\":\"failed to serialize error response\",\"details\":[]}".to_string()
        }),
    )
}

pub(super) fn internal_error(error: anyhow::Error) -> (StatusCode, String) {
    json_bad_request("bad_request", error.to_string())
}

pub(super) fn io_error(error: std::io::Error) -> (StatusCode, String) {
    (StatusCode::INTERNAL_SERVER_ERROR, error.to_string())
}

pub(super) fn not_found_io_error(error: std::io::Error) -> (StatusCode, String) {
    if error.kind() == std::io::ErrorKind::NotFound {
        (StatusCode::NOT_FOUND, error.to_string())
    } else {
        (StatusCode::INTERNAL_SERVER_ERROR, error.to_string())
    }
}

// ── RFC 9457 统一错误响应构造函数 ──

pub(super) fn problem_not_found(
    error_code: &str,
    detail: impl Into<String>,
) -> (StatusCode, String) {
    problem_response(StatusCode::NOT_FOUND, error_code, detail.into(), None)
}

pub(super) fn problem_conflict(
    error_code: &str,
    detail: impl Into<String>,
) -> (StatusCode, String) {
    problem_response(StatusCode::CONFLICT, error_code, detail.into(), None)
}

pub(super) fn problem_bad_request(
    error_code: &str,
    detail: impl Into<String>,
) -> (StatusCode, String) {
    problem_response(StatusCode::BAD_REQUEST, error_code, detail.into(), None)
}

pub(super) fn problem_internal(
    error_code: &str,
    detail: impl Into<String>,
) -> (StatusCode, String) {
    problem_response(
        StatusCode::INTERNAL_SERVER_ERROR,
        error_code,
        detail.into(),
        None,
    )
}

pub(super) fn problem_response(
    status: StatusCode,
    error_code: &str,
    detail: String,
    instance: Option<String>,
) -> (StatusCode, String) {
    let problem_type = format!("https://quantpilot.dev/problems/{}", error_code.to_lowercase());
    let title = error_title_for_status(status);
    let body = serde_json::json!({
        "type": problem_type,
        "title": title,
        "status": status.as_u16(),
        "detail": detail,
        "instance": instance.unwrap_or_default(),
        "error_code": error_code,
    });
    (
        status,
        serde_json::to_string(&body).unwrap_or_else(|_| {
            format!(
                "{{\"type\":\"https://quantpilot.dev/problems/internal-error\",\"title\":\"Internal Server Error\",\"status\":500,\"detail\":\"failed to serialize error response\",\"error_code\":\"INTERNAL_ERROR\"}}"
            )
        }),
    )
}

fn error_title_for_status(status: StatusCode) -> &'static str {
    match status.as_u16() {
        400 => "Bad Request",
        404 => "Not Found",
        409 => "Conflict",
        422 => "Unprocessable Entity",
        429 => "Too Many Requests",
        500 => "Internal Server Error",
        _ => "Error",
    }
}
