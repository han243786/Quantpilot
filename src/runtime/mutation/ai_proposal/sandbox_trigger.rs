use super::approval_persistence::persist_approval;
use crate::{
    current_time_ms, sandbox_verification, AppState, RequestSandboxVerificationRequest,
    RuntimeAiProposalRecord, RuntimeAiProposalStatus, RuntimeApprovalLifecycleEntry,
    SandboxVerdict, SandboxVerificationReport,
};
use axum::http::StatusCode;
use futures_util::FutureExt;
use serde_json::json;

async fn load_sandbox_report_for_proposal(
    state: &AppState,
    proposal_id: &str,
) -> Result<SandboxVerificationReport, (StatusCode, String)> {
    if let Some(report) = state.sandbox_reports.read().await.get(proposal_id).cloned() {
        return Ok(report);
    }
    sandbox_verification::load_sandbox_report_from_disk(
        state.sandbox_report_store_dir.as_ref(),
        proposal_id,
    )
    .await
}

pub(super) async fn ensure_ai_proposal_can_be_approved(
    state: &AppState,
    proposal: &RuntimeAiProposalRecord,
) -> Result<(), (StatusCode, String)> {
    if proposal.config_domain_binding.is_none() {
        return Err((
            StatusCode::LOCKED,
            json!({
                "error": "strategy_config_ai_binding_required",
                "message": "AI proposal 缺少策略配置域绑定，不能通过审批。",
            })
            .to_string(),
        ));
    }
    if proposal.status != RuntimeAiProposalStatus::StaticCheckPassed {
        return Err((
            StatusCode::LOCKED,
            json!({
                "error": "ai_proposal_static_check_required",
                "message": "AI proposal 必须先通过静态检查，才能进入审批通过路径。",
            })
            .to_string(),
        ));
    }
    let sandbox_report = load_sandbox_report_for_proposal(state, &proposal.ai_proposal_id)
        .await
        .map_err(|_| {
            (
                StatusCode::LOCKED,
                json!({
                    "error": "ai_proposal_sandbox_required",
                    "message": "AI proposal 必须先生成沙箱验证报告，才能通过审批。",
                })
                .to_string(),
            )
        })?;
    if sandbox_report.verdict == SandboxVerdict::CandidateUnderperforms {
        return Err((
            StatusCode::LOCKED,
            json!({
                "error": "ai_proposal_sandbox_failed",
                "message": "AI proposal 沙箱验证未通过，不能通过审批。",
            })
            .to_string(),
        ));
    }
    Ok(())
}

pub(super) fn spawn_ai_proposal_sandbox_verification(state: AppState, proposal_id: String) {
    // v1.1.2: 异步触发沙箱验证，JoinHandle 存入 state 防止 panic 静默丢失
    // v2.4.0 P1-B2: 添加 catch_unwind + 3次退避重试
    let state_clone = state.clone();
    let pid = proposal_id.clone();
    let handle = tokio::spawn(async move {
        let sandbox_request = RequestSandboxVerificationRequest {
            backtest_id: None,
            proposal_id: pid.clone(),
        };
        let mut success = false;
        for attempt in 0u32..3 {
            let result = std::panic::AssertUnwindSafe(
                sandbox_verification::run_sandbox_verification(&state_clone, &sandbox_request),
            )
            .catch_unwind()
            .await;
            match result {
                Ok(Ok(_report)) => {
                    success = true;
                    break;
                }
                Ok(Err(e)) => {
                    safe_eprintln!("[sandbox] 验证尝试 {}/3 失败: {}", attempt + 1, e.1);
                }
                Err(panic_err) => {
                    let msg = panic_err
                        .downcast_ref::<String>()
                        .map(|s| s.as_str())
                        .or_else(|| panic_err.downcast_ref::<&str>().copied())
                        .unwrap_or("未知 panic");
                    safe_eprintln!("[sandbox] 验证尝试 {}/3 panic: {}", attempt + 1, msg);
                }
            }
            if attempt < 2 {
                tokio::time::sleep(std::time::Duration::from_millis(500 * (attempt + 1) as u64))
                    .await;
            }
        }
        if success {
            // 更新审批单的沙箱报告 URL
            let approval_to_persist = {
                let mut approvals = state_clone.approval_records.write().await;
                let mut updated = None;
                for approval in approvals.values_mut() {
                    if approval.proposal_id == pid {
                        approval.sandbox_report_url =
                            Some(format!("/api/v1/ai/proposals/{}/sandbox-report", pid));
                        updated = Some(approval.clone());
                        break;
                    }
                }
                updated
            };
            if let Some(approval) = approval_to_persist {
                let _ = persist_approval(&state_clone.approval_store_dir, &approval).await;
            }
        } else {
            let failed_at_ms = current_time_ms();
            let approval_to_persist = {
                let mut approvals = state_clone.approval_records.write().await;
                let mut updated = None;
                for approval in approvals.values_mut() {
                    if approval.proposal_id == pid {
                        approval.lifecycle.push(RuntimeApprovalLifecycleEntry {
                            review_state: approval.review_state,
                            event_id: format!("event_sandbox_failed_{}", failed_at_ms),
                            sequence_no: approval.lifecycle.len() as u64 + 1,
                            occurred_at_ms: failed_at_ms,
                            reason_code: "SANDBOX_VERIFICATION_FAILED".to_string(),
                            message: "沙箱验证 3 次尝试全部失败，审批通过路径保持阻断。"
                                .to_string(),
                            actor_id: None,
                        });
                        updated = Some(approval.clone());
                        break;
                    }
                }
                updated
            };
            if let Some(approval) = approval_to_persist {
                let _ = persist_approval(&state_clone.approval_store_dir, &approval).await;
            }
            safe_eprintln!("[sandbox] 沙箱验证 3 次尝试全部失败, proposal={}", pid);
        }
    });
    // v1.1.2: 监视 JoinHandle 防止 panic 静默丢失
    tokio::spawn(async move {
        if let Err(e) = handle.await {
            safe_eprintln!("[sandbox] 沙箱验证任务异常: {}", e);
        }
    });
}
