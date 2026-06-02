use axum::http::StatusCode;
use std::collections::BTreeMap;

pub(super) fn validate_set_request(
    body: &serde_json::Value,
) -> Result<(String, BTreeMap<String, String>), (StatusCode, String)> {
    let service = body["service"]
        .as_str()
        .filter(|s| {
            !s.trim().is_empty()
                && s.len() <= 64
                && !s.contains('/')
                && !s.contains('\\')
                && !s.contains("..")
        })
        .ok_or_else(|| (StatusCode::BAD_REQUEST, "缺少 'service' 字段".to_string()))?;

    let fields_obj = body["fields"]
        .as_object()
        .ok_or_else(|| (StatusCode::BAD_REQUEST, "缺少 'fields' 对象".to_string()))?;

    let mut fields: BTreeMap<String, String> = BTreeMap::new();
    for (k, v) in fields_obj {
        let val = v.as_str().unwrap_or_default().to_string();
        if val.is_empty() {
            return Err((
                StatusCode::BAD_REQUEST,
                format!("字段 '{}' 的值不能为空", k),
            ));
        }
        fields.insert(k.clone(), val);
    }

    Ok((service.to_string(), fields))
}
