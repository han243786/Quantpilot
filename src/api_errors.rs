use super::*;

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
