// ── Block 5: 审批流引擎 ──

#[derive(Debug, Deserialize)]
pub(crate) struct RuntimeApprovalListQuery {
    #[serde(default)]
    review_state: Option<String>,
}

// ── Block 5: 合并记录 API ──

#[derive(Debug, Serialize)]
pub(crate) struct MergeRecordsResponse {
    records: Vec<MergeRecordEntry>,
    total_conflicts: usize,
    total_suppressed: usize,
}

#[derive(Debug, Serialize)]
struct MergeRecordEntry {
    cycle_name: String,
    input_count: usize,
    output_count: usize,
    conflicts: usize,
    suppressed: usize,
    merge_policy: String,
}
