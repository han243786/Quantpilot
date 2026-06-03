use crate::*;

pub(super) async fn init_alert_rules(
    state: &AppState,
    default_alert_rules: fn() -> Vec<AlertRule>,
) {
    let mut rules = state.alert_rules.write().await;
    if rules.is_empty() {
        *rules = default_alert_rules();
    }
}
