#[path = "backtest/execution_start.rs"]
mod backtest_execution_start;
#[path = "backtest/experiment_sweep.rs"]
mod backtest_experiment_sweep;
#[path = "backtest/record_store.rs"]
mod backtest_record_store;
#[path = "backtest/replay.rs"]
mod backtest_replay;
mod event_stream;
mod evidence_health;
#[path = "mutation/ai_proposal.rs"]
mod mutation_ai_proposal;
#[path = "mutation/parameter_mutation.rs"]
mod mutation_parameter_mutation;
mod report_ops;
#[path = "run/record_store.rs"]
mod run_record_store;
#[path = "run/replay_status.rs"]
mod run_replay_status;
#[path = "run/session_start.rs"]
mod run_session_start;
#[path = "run/v4_handoff.rs"]
mod run_v4_handoff;
use backtest_execution_start::execute_backtest_request;
pub(crate) use backtest_execution_start::start_backtest_run;
pub(crate) use backtest_record_store::{
    discard_backtest_record, get_backtest_detail, list_backtests, save_backtest_record,
};
pub(crate) use backtest_replay::get_backtest_replay;
pub(crate) use event_stream::stream_run_events;
pub(crate) use evidence_health::{cleanup_runtime_evidence, get_runtime_evidence_health};
pub(crate) use mutation_ai_proposal::{
    approve_ai_proposal, claim_ai_proposal_review, create_runtime_ai_proposal,
    get_runtime_ai_proposal_detail, get_runtime_approval_detail, list_runtime_ai_proposals,
    list_runtime_approvals, reject_ai_proposal,
};
pub(crate) use mutation_parameter_mutation::{
    activate_runtime_parameter_mutation, create_runtime_parameter_mutation,
    get_runtime_parameter_mutation_detail, list_runtime_parameter_mutations,
    rollback_runtime_parameter_mutation,
};
pub(crate) use report_ops::{
    create_runtime_report, export_runtime_report_artifact, get_audit_weekly_report,
    get_ops_daily_report, get_research_monthly_report, get_runtime_report_detail,
    get_storage_health, list_config_generations, list_merge_records, list_runtime_reports,
};
pub(crate) use run_record_store::{discard_run_record, get_run_detail, list_runs, save_run_record};
pub(crate) use run_replay_status::{get_run_replay, get_run_status};
pub(crate) use run_session_start::start_test_run;
pub(crate) use run_v4_handoff::start_v4_runtime_run;
use run_v4_handoff::{runtime_simulated_v4_matrix, runtime_v4_static_bundle};

// Backtest + Experiment handlers
include!("backtest.rs");
pub(crate) use backtest_experiment_sweep::{
    discard_experiment_record, get_experiment_detail, list_experiments, save_experiment_record,
    start_backtest_experiment,
};
// Run + SSE handlers
include!("run.rs");
// Mutation + Proposal + Approval handlers
include!("mutation.rs");

use super::*;
use axum::extract::Query;

const MAX_EXPERIMENT_VARIANTS: usize = 27;
const DEFAULT_REPLAY_PAGE_SIZE: usize = 12;
const MAX_REPLAY_PAGE_SIZE: usize = 50;

#[derive(Debug, Serialize)]
pub(crate) struct DiscardRuntimeArtifactResponse {
    discarded_id: String,
    discarded_kind: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct RuntimeReplayQuery {
    cursor: Option<usize>,
    limit: Option<usize>,
    checkpoint: Option<usize>,
    sequence_cursor: Option<u64>,
    stage: Option<String>,
    severity: Option<String>,
    retention_class: Option<String>,
    module_key: Option<String>,
    #[serde(default)]
    key_only: bool,
}

#[derive(Debug, Deserialize, Default)]
pub(crate) struct RuntimeParameterMutationListQuery {
    source_kind: Option<RuntimeEvidenceSourceKind>,
    source_id: Option<String>,
    limit: Option<u32>,
    offset: Option<u32>,
}

#[derive(Debug, Deserialize, Default)]
pub(crate) struct RuntimeAiProposalListQuery {
    source_kind: Option<RuntimeEvidenceSourceKind>,
    source_id: Option<String>,
    status: Option<RuntimeAiProposalStatus>,
}

fn clean_optional_filter(value: Option<String>) -> Option<String> {
    value
        .map(|item| item.trim().to_string())
        .filter(|item| !item.is_empty())
}

fn normalized_replay_options(query: RuntimeReplayQuery) -> RuntimeReplayOptions {
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

/// v1.3.5: RAII 守卫 — 运行结束后自动复位 run_in_progress
struct RunInProgressGuard<'a>(&'a std::sync::atomic::AtomicBool);
impl Drop for RunInProgressGuard<'_> {
    fn drop(&mut self) {
        self.0.store(false, std::sync::atomic::Ordering::Release);
    }
}
