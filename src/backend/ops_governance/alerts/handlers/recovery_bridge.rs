use crate::*;

pub(super) async fn is_condition_resolved(
    state: &AppState,
    user_id: &auth::UserId,
    rule: &AlertRule,
) -> bool {
    // 鍛婅鎭㈠ = 瑙﹀彂鏉′欢涓嶅啀鎴愮珛
    !super::should_fire_alert(state, user_id, rule).await
}
