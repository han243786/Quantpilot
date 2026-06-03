use super::persist_alert_firing;
use crate::*;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct AcknowledgeAlertRequest {
    actor_id: String,
}

pub(super) async fn acknowledge_alert(
    user_id: auth::UserId,
    State(state): State<AppState>,
    Path(firing_id): Path<String>,
    Json(request): Json<AcknowledgeAlertRequest>,
) -> Result<Json<AlertFiring>, (StatusCode, String)> {
    let now_ms = current_time_ms();
    let scoped = auth::scoped_key(&user_id, &firing_id);
    let updated = {
        let mut firings = state.alert_firings.write().await;
        let Some(firing) = firings.get_mut(&scoped) else {
            return Err(json_not_found(
                "not_found",
                crate::error_codes::ERR_ALERT_NOT_FOUND,
                format!("告警触发记录 '{}' 不存在", firing_id),
            ));
        };
        // v1.2.1: 已确认的告警再次调用时标记为已解决
        if firing.state == AlertFiringState::Acknowledged {
            firing.state = AlertFiringState::Resolved;
            firing.resolved_at_ms = Some(now_ms);
        } else {
            firing.state = AlertFiringState::Acknowledged;
            firing.acknowledged_at_ms = Some(now_ms);
            firing.acknowledged_by = Some(request.actor_id.clone());
        }
        firing.clone()
    };
    // v4.2.0: 文件 I/O 放在 alert_firings 写锁外，避免锁内 await。
    let _ = persist_alert_firing(state.alert_store_dir.as_ref(), &updated).await;
    Ok(Json(updated))
}
