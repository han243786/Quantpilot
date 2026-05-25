use super::*;

/// QuantPilot 标准错误格式。所有 API handler 使用此格式返回错误。
/// 格式: `{"error": "<type>", "error_code": "<code>", "message": "<message>", "details": [...]}`
/// v2.3.0: 新增 json_bad_request_with_code 支持语言中立 error_code, 旧函数保持兼容
/// v4.1.0: 默认结构化错误也必须携带稳定 error_code, 避免前端只能解析中文 message。
pub(crate) fn json_bad_request(
    error: &'static str,
    message: impl Into<String>,
) -> (StatusCode, String) {
    json_bad_request_with_details_and_partial(error, message, Vec::new(), None)
}

/// v2.3.0: 带语言中立错误码的错误响应
#[allow(dead_code)]
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
    (
        StatusCode::BAD_REQUEST,
        serde_json::to_string(&payload).unwrap_or_else(|_| {
            "{\"error\":\"bad_request\",\"message\":\"序列化错误响应失败\"}".to_string()
        }),
    )
}

pub(crate) fn json_not_found(
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
    (
        StatusCode::NOT_FOUND,
        serde_json::to_string(&payload).unwrap_or_else(|_| {
            "{\"error\":\"not_found\",\"error_code\":\"NOT_FOUND\",\"message\":\"序列化错误响应失败\",\"details\":[]}".to_string()
        }),
    )
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
        error_code: Some(crate::error_codes::ERR_BAD_REQUEST),
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
    let payload = ApiErrorResponse {
        error: "internal_error",
        error_code: Some(crate::error_codes::ERR_INTERNAL),
        message,
        details: Vec::new(),
        partial_artifacts: None,
    };
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        serde_json::to_string(&payload).unwrap_or_else(|_| {
            "{\"error\":\"internal_error\",\"error_code\":\"INTERNAL_ERROR\",\"message\":\"序列化错误响应失败\",\"details\":[]}".to_string()
        }),
    )
}

pub(super) fn io_error(error: std::io::Error) -> (StatusCode, String) {
    // v1.2.0: 不向用户泄露 OS 路径详情，仅记录日志
    safe_eprintln!("[io_error] {}", error);
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        "内部服务器错误，请重试".to_string(),
    )
}

pub(super) fn not_found_io_error(error: std::io::Error) -> (StatusCode, String) {
    if error.kind() == std::io::ErrorKind::NotFound {
        (StatusCode::NOT_FOUND, "请求的资源不存在".to_string())
    } else {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "内部服务器错误".to_string(),
        )
    }
}

// v0.4.2 D3: RFC 9457 problem_* 系列已移除。json_bad_request* 为当前标准错误格式。
// v1.2.0: 统一注释，消除 deprecated 声明与实际使用的矛盾。

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn json_bad_request_returns_400_and_json() {
        let (status, body) = json_bad_request("test_error", "测试错误消息");
        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert!(body.contains("test_error"));
        assert!(body.contains("BAD_REQUEST"));
        assert!(body.contains("测试错误消息"));
    }

    #[test]
    fn json_bad_request_with_code_includes_error_code() {
        let (status, body) = json_bad_request_with_code("test", "ERR_TEST", "带码错误");
        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert!(body.contains("ERR_TEST"));
        assert!(body.contains("error_code"));
    }

    #[test]
    fn json_bad_request_with_details_serializes_details_array() {
        let detail = ApiErrorDetail {
            code: "QS0001".to_string(),
            target: Some("graph_id".to_string()),
            message: "graph_id 不能为空".to_string(),
            span_label: None,
            reason: Some("不能为空".to_string()),
        };
        let (status, body) =
            json_bad_request_with_details("validation_error", "校验失败", vec![detail]);
        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert!(body.contains("validation_error"));
        assert!(body.contains("BAD_REQUEST"));
        assert!(body.contains("graph_id"));
        assert!(body.contains("QS0001"));
    }

    #[test]
    fn json_not_found_returns_404_with_code() {
        let (status, body) = json_not_found("not_found", "ALERT_NOT_FOUND", "告警不存在");
        assert_eq!(status, StatusCode::NOT_FOUND);
        assert!(body.contains("ALERT_NOT_FOUND"));
        assert!(body.contains("告警不存在"));
    }

    #[test]
    fn internal_error_returns_500_with_chinese_message() {
        let error = anyhow::anyhow!("内部测试错误");
        let (status, body) = internal_error(error);
        assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
        assert!(body.contains("内部服务器错误"));
        assert!(body.contains("INTERNAL_ERROR"));
        // 响应应包含 sanitize 后的简短摘要
        assert!(!body.contains("RUST_BACKTRACE"));
    }

    #[test]
    fn io_error_returns_uniform_message() {
        let error = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "denied");
        let (status, body) = io_error(error);
        assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
        assert!(body.contains("内部服务器错误"));
    }

    #[test]
    fn not_found_io_error_detects_not_found_kind() {
        let error = std::io::Error::new(std::io::ErrorKind::NotFound, "missing");
        let (status, body) = not_found_io_error(error);
        assert_eq!(status, StatusCode::NOT_FOUND);
        assert!(body.contains("不存在"));
    }

    #[test]
    fn not_found_io_error_treats_other_errors_as_internal() {
        let error = std::io::Error::new(std::io::ErrorKind::BrokenPipe, "pipe");
        let (status, body) = not_found_io_error(error);
        assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
        assert!(body.contains("内部服务器错误"));
    }
}
