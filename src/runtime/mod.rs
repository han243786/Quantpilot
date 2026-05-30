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
mod experiment_limit;
#[path = "mutation/ai_proposal.rs"]
mod mutation_ai_proposal;
#[path = "mutation/parameter_mutation.rs"]
mod mutation_parameter_mutation;
#[path = "mutation/shared_governance.rs"]
mod mutation_shared_governance;
mod query_support;
mod report_ops;
mod response_support;
mod run_guard;
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
use experiment_limit::MAX_EXPERIMENT_VARIANTS;
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
use mutation_shared_governance::{
    append_parameter_mutation_events_to_run, build_runtime_parameter_mutation_event,
    canonical_runtime_parameter_version, governance_with_parameter_version,
    mutation_event_contract, runtime_parameter_mutation_governance,
    validate_runtime_parameter_mutation_target,
};
use query_support::{
    clean_optional_filter, normalized_replay_options, AuditWeeklyQuery, OpsDailyQuery,
    ResearchMonthlyQuery, RuntimeAiProposalListQuery, RuntimeApprovalListQuery,
    RuntimeParameterMutationListQuery, RuntimeReplayQuery,
};
pub(crate) use report_ops::{
    create_runtime_report, export_runtime_report_artifact, get_audit_weekly_report,
    get_ops_daily_report, get_research_monthly_report, get_runtime_report_detail,
    get_storage_health, list_config_generations, list_merge_records, list_runtime_reports,
};
use response_support::{DiscardRuntimeArtifactResponse, MergeRecordEntry, MergeRecordsResponse};
use run_guard::RunInProgressGuard;
pub(crate) use run_record_store::{discard_run_record, get_run_detail, list_runs, save_run_record};
pub(crate) use run_replay_status::{get_run_replay, get_run_status};
pub(crate) use run_session_start::start_test_run;
pub(crate) use run_v4_handoff::start_v4_runtime_run;
use run_v4_handoff::{runtime_simulated_v4_matrix, runtime_v4_static_bundle};

pub(crate) use backtest_experiment_sweep::{
    discard_experiment_record, get_experiment_detail, list_experiments, save_experiment_record,
    start_backtest_experiment,
};

use super::*;
use axum::extract::Query;
