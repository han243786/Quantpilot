use crate::*;

pub(super) async fn list_alerts(
    user_id: auth::UserId,
    State(state): State<AppState>,
) -> Result<Json<AlertListResponse>, (StatusCode, String)> {
    let prefix = auth::scoped_key(&user_id, "");
    let firings: Vec<AlertFiring> = state
        .alert_firings
        .read()
        .await
        .iter()
        .filter(|(key, _)| key.starts_with(&prefix))
        .map(|(_, value)| value.clone())
        .collect();
    let rules = state.alert_rules.read().await.clone();
    Ok(Json(AlertListResponse { firings, rules }))
}

pub(super) async fn list_alert_rules(
    State(state): State<AppState>,
) -> Result<Json<Vec<AlertRule>>, (StatusCode, String)> {
    let rules = state.alert_rules.read().await.clone();
    Ok(Json(rules))
}
