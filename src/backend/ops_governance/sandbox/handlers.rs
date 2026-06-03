use crate::*;

// ── 沙箱验证服务 ──
// Block 5 核心技术闸门：AI 提案必须经过独立沙箱回放验证方可提交审批

pub(super) async fn load_or_fetch_ai_proposal(
    state: &AppState,
    proposal_id: &str,
) -> Result<RuntimeAiProposalRecord, (StatusCode, String)> {
    if let Some(record) = state.ai_proposals.read().await.get(proposal_id).cloned() {
        return Ok(record);
    }
    load_runtime_ai_proposal_record(state.ai_proposal_store_dir.as_ref(), proposal_id).await
}

pub(crate) async fn load_sandbox_report_from_disk(
    store_dir: &FsPath,
    proposal_id: &str,
) -> Result<SandboxVerificationReport, (StatusCode, String)> {
    if proposal_id.contains("..")
        || proposal_id.contains('/')
        || proposal_id.contains('\\')
        || proposal_id.is_empty()
        || proposal_id.len() > 128
    {
        return Err((StatusCode::BAD_REQUEST, "proposal_id 无效".to_string()));
    }
    let file_path = store_dir.join(format!("{}.json", proposal_id));
    let json = fs::read(&file_path).await.map_err(|_| {
        json_bad_request(
            "not_found",
            format!("提案 '{}' 的沙箱报告不存在", proposal_id),
        )
    })?;
    serde_json::from_slice(&json).map_err(|error| internal_error(anyhow::anyhow!("{}", error)))
}
