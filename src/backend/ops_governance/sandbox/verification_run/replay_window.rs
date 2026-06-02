use crate::*;

pub(super) fn build_replay_window() -> (u64, String, ReplayWindow) {
    let now_ms = current_time_ms();
    let sandbox_run_id = format!("sbx-run-{}", now_ms);

    let replay_days: u64 = std::env::var("QUANTPILOT_SANDBOX_REPLAY_WINDOW_DAYS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(30);
    let replay_window = ReplayWindow {
        from_ts: epoch_ms_to_iso8601(now_ms.saturating_sub(replay_days * 24 * 3600 * 1000)),
        to_ts: epoch_ms_to_iso8601(now_ms),
    };

    (now_ms, sandbox_run_id, replay_window)
}
