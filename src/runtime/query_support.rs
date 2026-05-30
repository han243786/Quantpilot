use super::*;

const DEFAULT_REPLAY_PAGE_SIZE: usize = 12;
const MAX_REPLAY_PAGE_SIZE: usize = 50;

#[derive(Debug, Deserialize)]
pub(crate) struct RuntimeReplayQuery {
    pub(super) cursor: Option<usize>,
    pub(super) limit: Option<usize>,
    pub(super) checkpoint: Option<usize>,
    pub(super) sequence_cursor: Option<u64>,
    pub(super) stage: Option<String>,
    pub(super) severity: Option<String>,
    pub(super) retention_class: Option<String>,
    pub(super) module_key: Option<String>,
    #[serde(default)]
    pub(super) key_only: bool,
}

#[derive(Debug, Deserialize, Default)]
pub(crate) struct RuntimeParameterMutationListQuery {
    pub(super) source_kind: Option<RuntimeEvidenceSourceKind>,
    pub(super) source_id: Option<String>,
    pub(super) limit: Option<u32>,
    pub(super) offset: Option<u32>,
}

#[derive(Debug, Deserialize, Default)]
pub(crate) struct RuntimeAiProposalListQuery {
    pub(super) source_kind: Option<RuntimeEvidenceSourceKind>,
    pub(super) source_id: Option<String>,
    pub(super) status: Option<RuntimeAiProposalStatus>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct RuntimeApprovalListQuery {
    #[serde(default)]
    pub(super) review_state: Option<String>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct OpsDailyQuery {
    pub(super) date: Option<String>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct AuditWeeklyQuery {
    pub(super) week_start: Option<String>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ResearchMonthlyQuery {
    pub(super) month: Option<String>,
}

pub(super) fn clean_optional_filter(value: Option<String>) -> Option<String> {
    value
        .map(|item| item.trim().to_string())
        .filter(|item| !item.is_empty())
}

pub(super) fn normalized_replay_options(query: RuntimeReplayQuery) -> RuntimeReplayOptions {
    let cursor = query.checkpoint.or(query.cursor).unwrap_or(0);
    let limit = query
        .limit
        .unwrap_or(DEFAULT_REPLAY_PAGE_SIZE)
        .clamp(1, MAX_REPLAY_PAGE_SIZE);
    RuntimeReplayOptions {
        cursor,
        limit,
        sequence_cursor: query.sequence_cursor,
        filters: RuntimeReplayFilters {
            stage: clean_optional_filter(query.stage),
            severity: clean_optional_filter(query.severity),
            retention_class: clean_optional_filter(query.retention_class),
            module_key: clean_optional_filter(query.module_key),
            key_only: query.key_only,
        },
    }
}
