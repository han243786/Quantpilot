use axum::http::StatusCode;

pub(super) fn validate_service_path(service: String) -> Result<String, (StatusCode, String)> {
    if service.is_empty()
        || service.len() > 64
        || service.contains('/')
        || service.contains('\\')
        || service.contains("..")
        || service.contains('\0')
    {
        return Err((StatusCode::BAD_REQUEST, "凭证标签无效".to_string()));
    }

    Ok(service)
}
