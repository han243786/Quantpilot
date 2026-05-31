use serde::Serialize;

#[derive(Debug, Serialize)]
pub(crate) struct DiscardRuntimeArtifactResponse {
    pub(super) discarded_id: String,
    pub(super) discarded_kind: String,
}

#[derive(Debug, Serialize)]
pub(crate) struct MergeRecordsResponse {
    pub(super) records: Vec<MergeRecordEntry>,
    pub(super) total_conflicts: usize,
    pub(super) total_suppressed: usize,
}

#[derive(Debug, Serialize)]
pub(super) struct MergeRecordEntry {
    pub(super) cycle_name: String,
    pub(super) input_count: usize,
    pub(super) output_count: usize,
    pub(super) conflicts: usize,
    pub(super) suppressed: usize,
    pub(super) merge_policy: String,
}
