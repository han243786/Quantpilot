use super::*;

pub(super) async fn json_rejection_middleware(
    req: axum::extract::Request,
    next: axum::middleware::Next,
) -> axum::response::Response {
    let response = next.run(req).await;
    // 覆盖 Axum 默认的 JSON 解析错误 (422/400/415), 统一返回中文 JSON
    let status = response.status();
    if status == StatusCode::UNPROCESSABLE_ENTITY
        || status == StatusCode::BAD_REQUEST
        || status == StatusCode::UNSUPPORTED_MEDIA_TYPE
    {
        let body = axum::Json(serde_json::json!({
            "error": "bad_request",
            "message": "请求格式错误: 请使用 Content-Type: application/json 并确保请求体为有效 JSON"
        }));
        return (StatusCode::BAD_REQUEST, body).into_response();
    }
    response
}
