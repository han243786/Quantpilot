use crate::{backend::ops_governance::alerts, AppState};

pub(super) async fn init_alert_rules(state: &AppState) {
    alerts::init_alert_rules(state).await;
}
