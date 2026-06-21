use crate::runtime_persistence::sanitize_storage_path_segment;
use crate::{
    auth, canonical_json_sha256_digest, internal_error, json_bad_request, json_not_found, AppState,
    ContractRepairApprovalApproveExecutionAdmissionGate,
    ContractRepairApprovalApproveExecutionAtomicSideEffectsEnablementDryRun,
    ContractRepairApprovalApproveExecutionAtomicSideEffectsGate,
    ContractRepairApprovalApproveExecutionAtomicSideEffectsReadinessDryRun,
    ContractRepairApprovalApproveExecutionContractMutationReadinessDryRun,
    ContractRepairApprovalApproveExecutionDecisionLockSummaryDryRun,
    ContractRepairApprovalApproveExecutionDurableDiskApplicationExecution,
    ContractRepairApprovalApproveExecutionFinalAtomicAdmissionGateDryRun,
    ContractRepairApprovalApproveExecutionFinalAtomicExecutionPlanDryRun,
    ContractRepairApprovalApproveExecutionFinalAtomicReadinessDryRun,
    ContractRepairApprovalApproveExecutionFinalExecutionDurableWritebackBundleDryRun,
    ContractRepairApprovalApproveExecutionFinalExecutionEntryDryRun,
    ContractRepairApprovalApproveExecutionFinalExecutionOrderedHandlerExecutionConfirmationDryRun,
    ContractRepairApprovalApproveExecutionFinalExecutionReplayExecutorDryRun,
    ContractRepairApprovalApproveExecutionFinalExecutionReplayExecutorRoutingDryRun,
    ContractRepairApprovalApproveExecutionFinalExecutionReplayPlanDryRun,
    ContractRepairApprovalApproveExecutionFinalExecutionRollbackReadinessDryRun,
    ContractRepairApprovalApproveExecutionFinalExecutionRoutedExecutionAttemptDryRun,
    ContractRepairApprovalApproveExecutionFinalExecutionRoutedHandlerPlanDryRun,
    ContractRepairApprovalApproveExecutionFinalExecutionRoutedRouteSuccessReleaseDryRun,
    ContractRepairApprovalApproveExecutionFinalExecutionRoutedWriteHandoffDryRun,
    ContractRepairApprovalApproveExecutionFinalExecutionSwitchReadinessDryRun,
    ContractRepairApprovalApproveExecutionFormalReviewExecutionReadinessDryRun,
    ContractRepairApprovalApproveExecutionGate,
    ContractRepairApprovalApproveExecutionLifecycleEffectsReadinessDryRun,
    ContractRepairApprovalApproveExecutionLifecycleEntryAppendEnablementDryRun,
    ContractRepairApprovalApproveExecutionLifecycleEventEmissionEnablementDryRun,
    ContractRepairApprovalApproveExecutionRecoveryMarkerIdempotencyPrecheck,
    ContractRepairApprovalApproveExecutionRecoveryMarkerPersistenceGate,
    ContractRepairApprovalApproveExecutionRecoveryMarkerPersistenceReadinessDryRun,
    ContractRepairApprovalApproveExecutionRecoveryMarkerWriteDryRun,
    ContractRepairApprovalApproveExecutionRunnerActivationAdmissionHandoffDryRun,
    ContractRepairApprovalApproveExecutionRunnerActivationAdmissionHandoffReadinessDryRun,
    ContractRepairApprovalApproveExecutionRunnerActivationEnablementDryRun,
    ContractRepairApprovalApproveExecutionRunnerActivationEnablementReadinessDryRun,
    ContractRepairApprovalApproveExecutionRunnerActivationExecutionPlanDryRun,
    ContractRepairApprovalApproveExecutionRunnerActivationExecutionPlanReadinessDryRun,
    ContractRepairApprovalApproveExecutionRunnerActivationHandoffAttemptDryRun,
    ContractRepairApprovalApproveExecutionRunnerActivationHandoffAttemptReadinessDryRun,
    ContractRepairApprovalApproveExecutionRunnerActivationHandoffEnablementDryRun,
    ContractRepairApprovalApproveExecutionRunnerActivationHandoffEnablementReadinessDryRun,
    ContractRepairApprovalApproveExecutionRunnerActivationPathDryRun,
    ContractRepairApprovalApproveExecutionRunnerActivationPathReadinessDryRun,
    ContractRepairApprovalApproveExecutionRunnerActivationPostHandoffAttemptDryRun,
    ContractRepairApprovalApproveExecutionRunnerActivationPostHandoffAttemptReadinessDryRun,
    ContractRepairApprovalApproveExecutionRunnerActivationSuccessAdmissionDryRun,
    ContractRepairApprovalApproveExecutionRunnerActivationSuccessAdmissionReadinessDryRun,
    ContractRepairApprovalApproveExecutionRunnerActivationSuccessReturnDryRun,
    ContractRepairApprovalApproveExecutionRunnerActivationSuccessReturnReadinessDryRun,
    ContractRepairApprovalApproveExecutionRunnerActivationSwitchTransactionProofDryRun,
    ContractRepairApprovalApproveExecutionRunnerActivationSwitchTransactionProofReadinessDryRun,
    ContractRepairApprovalApproveExecutionRunnerActivationSwitchWriteTransactionDryRun,
    ContractRepairApprovalApproveExecutionRunnerActivationSwitchWriteTransactionEnablementDryRun,
    ContractRepairApprovalApproveExecutionRunnerActivationSwitchWriteTransactionEnablementReadinessDryRun,
    ContractRepairApprovalApproveExecutionRunnerActivationSwitchWriteTransactionReadinessDryRun,
    ContractRepairApprovalApproveExecutionRunnerActivationTransactionAdmissionGateDryRun,
    ContractRepairApprovalApproveExecutionRunnerActivationTransactionAdmissionReadinessDryRun,
    ContractRepairApprovalApproveExecutionRunnerAttempt,
    ContractRepairApprovalApproveExecutionRunnerAttemptEnablementDryRun,
    ContractRepairApprovalApproveExecutionRunnerAttemptReadinessDryRun,
    ContractRepairApprovalApproveExecutionRunnerBodyEnablementDryRun,
    ContractRepairApprovalApproveExecutionRunnerBodyPhaseSequenceDryRun,
    ContractRepairApprovalApproveExecutionRunnerBodyReadinessDryRun,
    ContractRepairApprovalApproveExecutionRunnerCallBodyDryRun,
    ContractRepairApprovalApproveExecutionRunnerCallDryRun,
    ContractRepairApprovalApproveExecutionRunnerCallEnablementDryRun,
    ContractRepairApprovalApproveExecutionRunnerCallReadinessDryRun,
    ContractRepairApprovalApproveExecutionRunnerControlReadinessDryRun,
    ContractRepairApprovalApproveExecutionRunnerDispatchGate,
    ContractRepairApprovalApproveExecutionRunnerDispatchReadinessDryRun,
    ContractRepairApprovalApproveExecutionRunnerEnablementPlanDryRun,
    ContractRepairApprovalApproveExecutionRunnerExecutionEnablementDryRun,
    ContractRepairApprovalApproveExecutionRunnerExecutionReadinessDryRun,
    ContractRepairApprovalApproveExecutionRunnerHandoff,
    ContractRepairApprovalApproveExecutionRunnerHandoffReadinessDryRun,
    ContractRepairApprovalApproveExecutionRunnerLifecyclePhaseDryRun,
    ContractRepairApprovalApproveExecutionRunnerLifecyclePhaseEnablementDryRun,
    ContractRepairApprovalApproveExecutionRunnerLifecyclePhaseReadinessDryRun,
    ContractRepairApprovalApproveExecutionRunnerOutcome,
    ContractRepairApprovalApproveExecutionRunnerPhaseExecutionEnablementDryRun,
    ContractRepairApprovalApproveExecutionRunnerPhasesReadinessDryRun,
    ContractRepairApprovalApproveExecutionRunnerRecoveryMarkerCleanupPhaseDryRun,
    ContractRepairApprovalApproveExecutionRunnerRecoveryMarkerCleanupPhaseEnablementDryRun,
    ContractRepairApprovalApproveExecutionRunnerRecoveryMarkerCleanupPhaseReadinessDryRun,
    ContractRepairApprovalApproveExecutionRunnerRollbackExecutionPhaseDryRun,
    ContractRepairApprovalApproveExecutionRunnerRollbackExecutionPhaseEnablementDryRun,
    ContractRepairApprovalApproveExecutionRunnerRollbackExecutionPhaseReadinessDryRun,
    ContractRepairApprovalApproveExecutionRunnerRouteDispatchEnablementDryRun,
    ContractRepairApprovalApproveExecutionRunnerRouteStatusReadinessDryRun,
    ContractRepairApprovalApproveExecutionRunnerRouteSuccessReadinessDryRun,
    ContractRepairApprovalApproveExecutionRunnerSourceMutationPhaseDryRun,
    ContractRepairApprovalApproveExecutionRunnerSourceMutationPhaseEnablementDryRun,
    ContractRepairApprovalApproveExecutionRunnerSourceMutationPhaseReadinessDryRun,
    ContractRepairApprovalApproveExecutionRunnerTransactionCommitPhaseDryRun,
    ContractRepairApprovalApproveExecutionRunnerTransactionCommitPhaseEnablementDryRun,
    ContractRepairApprovalApproveExecutionRunnerTransactionCommitPhaseReadinessDryRun,
    ContractRepairApprovalApproveExecutionTransactionCommitGate,
    ContractRepairApprovalApproveExecutionTransactionCommitReadinessDryRun,
    ContractRepairApprovalApproveExecutionTransactionDryRun,
    ContractRepairApprovalApproveExecutionTransactionRunnerDryRun,
    ContractRepairApprovalApproveExecutionTransactionRunnerEnablementDryRun,
    ContractRepairApprovalBlockedResponse, ContractRepairApprovalContractMutationEnablementGate,
    ContractRepairApprovalContractPatchApplyDryRun,
    ContractRepairApprovalContractPatchOperationPreview,
    ContractRepairApprovalContractPatchPlanPreview, ContractRepairApprovalContractSourceRef,
    ContractRepairApprovalContractSourceResolutionDryRun,
    ContractRepairApprovalContractSourceWriteDryRun, ContractRepairApprovalContractWritebackDryRun,
    ContractRepairApprovalDetailReadModelResponse, ContractRepairApprovalIdempotencyPrecheck,
    ContractRepairApprovalLifecycleEmissionEnablementGate,
    ContractRepairApprovalLifecycleEntryAppendDryRun, ContractRepairApprovalLifecycleEntryPreview,
    ContractRepairApprovalLifecycleEventDryRun, ContractRepairApprovalPersistencePathPreview,
    ContractRepairApprovalPersistencePlanPreview,
    ContractRepairApprovalReadModelDecisionLockSummary, ContractRepairApprovalReadModelResponse,
    ContractRepairApprovalRecordPreview, ContractRepairApprovalRecordSnapshotPreview,
    ContractRepairApprovalRecordWriteDryRun, ContractRepairApprovalReviewBlockedResponse,
    ContractRepairApprovalReviewExecutionGate, ContractRepairApprovalReviewExecutionPlanPreview,
    ContractRepairApprovalReviewIntentRequest, ContractRepairApprovalReviewTransitionDryRun,
    ContractRepairApprovalReviewerAuthorizationPrecheck,
    ContractRepairApprovalStorageDryRunPreview, ContractRepairApprovalStorageReadinessGate,
    CreateContractRepairApprovalRequest, RuntimeApprovalReviewState,
};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;
use serde_json::json;
use std::{
    collections::BTreeMap,
    path::{Path as FsPath, PathBuf},
};
use tokio::fs;

const CONTRACT_REPAIR_PAYLOAD_KIND: &str = "v4_contract_repair_approval_request";
const CONTRACT_REPAIR_APPROVAL_STORE_KIND: &str = "contract_repair_approval_records";
const CONTRACT_REPAIR_REVIEWER_POLICY_VERSION: &str =
    "quantpilot/contract-repair-reviewer-role-policy/v1";
const CONTRACT_REPAIR_REVIEWER_REQUIRED_ROLE: &str = "contract_repair_reviewer";
const CONTRACT_REPAIR_APPROVAL_APPROVE_LIVE_ROUTE_POLICY_NAME: &str =
    "quantpilot/contract-repair-approve-live-route-policy/v1";
const CONTRACT_REPAIR_APPROVAL_APPROVE_LIVE_ROUTE_ENV: &str =
    "QUANTPILOT_CONTRACT_REPAIR_APPROVE_LIVE_ROUTE";
const CONTRACT_REPAIR_APPROVAL_APPROVE_LIVE_ROUTE_ENABLED_VALUE: &str =
    "verified-live-approve-route";
const CONTRACT_REPAIR_APPROVAL_APPROVE_LIVE_ROUTE_POLICY_GATE: &str =
    "approve_live_route_activation_policy_verified";

#[derive(Debug, Clone, PartialEq, Eq)]
struct ContractRepairApprovalApproveLiveRouteActivationPolicy {
    policy_name: &'static str,
    env_name: &'static str,
    required_value: &'static str,
    configured_value: Option<String>,
    live_route_enabled: bool,
    blocked_gates: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ContractRepairApprovalApproveLiveRouteGates {
    review_transition_enabled: bool,
    lifecycle_effects_enabled: bool,
    contract_source_write_enabled: bool,
    transaction_runner_enabled: bool,
    runner_attempt_enabled: bool,
    runner_execution_enabled: bool,
    route_dispatch_enabled: bool,
    runner_call_enabled: bool,
    runner_body_enabled: bool,
    phase_execution_enabled: bool,
    lifecycle_phase_enabled: bool,
    source_mutation_phase_enabled: bool,
    rollback_execution_enabled: bool,
    runner_activation_enabled: bool,
    activation_switch_write_transaction_enabled: bool,
    recovery_marker_persistence_enabled: bool,
    recovery_marker_cleanup_phase_enabled: bool,
    transaction_commit_enabled: bool,
    transaction_commit_phase_enabled: bool,
    atomic_side_effects_enabled: bool,
    route_success_enabled: bool,
    formal_approve_review_execution_enabled: bool,
}

fn contract_repair_approval_approve_live_route_activation_policy_from_value(
    configured_value: Option<&str>,
) -> ContractRepairApprovalApproveLiveRouteActivationPolicy {
    let configured_value = configured_value
        .map(str::trim)
        .filter(|value| !value.is_empty());
    let live_route_enabled = matches!(
        configured_value,
        Some(value) if value == CONTRACT_REPAIR_APPROVAL_APPROVE_LIVE_ROUTE_ENABLED_VALUE
    );
    let blocked_gates = if live_route_enabled {
        Vec::new()
    } else {
        vec![CONTRACT_REPAIR_APPROVAL_APPROVE_LIVE_ROUTE_POLICY_GATE.to_string()]
    };

    ContractRepairApprovalApproveLiveRouteActivationPolicy {
        policy_name: CONTRACT_REPAIR_APPROVAL_APPROVE_LIVE_ROUTE_POLICY_NAME,
        env_name: CONTRACT_REPAIR_APPROVAL_APPROVE_LIVE_ROUTE_ENV,
        required_value: CONTRACT_REPAIR_APPROVAL_APPROVE_LIVE_ROUTE_ENABLED_VALUE,
        configured_value: configured_value.map(str::to_string),
        live_route_enabled,
        blocked_gates,
    }
}

fn contract_repair_approval_approve_live_route_activation_policy_from_env(
) -> ContractRepairApprovalApproveLiveRouteActivationPolicy {
    let configured_value = std::env::var(CONTRACT_REPAIR_APPROVAL_APPROVE_LIVE_ROUTE_ENV).ok();
    contract_repair_approval_approve_live_route_activation_policy_from_value(
        configured_value.as_deref(),
    )
}

fn contract_repair_approval_approve_live_route_gates_for_policy(
    policy: &ContractRepairApprovalApproveLiveRouteActivationPolicy,
) -> ContractRepairApprovalApproveLiveRouteGates {
    let enabled = policy.live_route_enabled;

    ContractRepairApprovalApproveLiveRouteGates {
        review_transition_enabled: enabled,
        lifecycle_effects_enabled: enabled,
        contract_source_write_enabled: enabled,
        transaction_runner_enabled: enabled,
        runner_attempt_enabled: enabled,
        runner_execution_enabled: enabled,
        route_dispatch_enabled: enabled,
        runner_call_enabled: enabled,
        runner_body_enabled: enabled,
        phase_execution_enabled: enabled,
        lifecycle_phase_enabled: enabled,
        source_mutation_phase_enabled: enabled,
        rollback_execution_enabled: enabled,
        runner_activation_enabled: enabled,
        activation_switch_write_transaction_enabled: enabled,
        recovery_marker_persistence_enabled: enabled,
        recovery_marker_cleanup_phase_enabled: enabled,
        transaction_commit_enabled: enabled,
        transaction_commit_phase_enabled: enabled,
        atomic_side_effects_enabled: enabled,
        route_success_enabled: enabled,
        formal_approve_review_execution_enabled: enabled,
    }
}

fn contract_repair_approval_approve_live_route_gates() -> ContractRepairApprovalApproveLiveRouteGates
{
    let policy = contract_repair_approval_approve_live_route_activation_policy_from_env();
    contract_repair_approval_approve_live_route_gates_for_policy(&policy)
}

fn contract_repair_approval_read_model_decision_lock_summary(
    status: &str,
    route_status: &str,
    persisted_record_count: usize,
    preview_record_count: usize,
    persistence_enabled: bool,
    mutation_enabled: bool,
    blocked_reasons: &[String],
) -> ContractRepairApprovalReadModelDecisionLockSummary {
    let primary_blocked_reason = blocked_reasons
        .first()
        .cloned()
        .unwrap_or_else(|| "decision_execution_locked".to_string());

    ContractRepairApprovalReadModelDecisionLockSummary {
        status: "read_model_decision_lock_summary".to_string(),
        target_action: "approve".to_string(),
        response_status: status.to_string(),
        route_status: route_status.to_string(),
        target_response_status: "review_approve_executed".to_string(),
        target_route_status: "review_approve_executed".to_string(),
        expected_http_status: 423,
        persisted_record_count,
        preview_record_count,
        persistence_enabled,
        mutation_enabled,
        final_execution_locked: true,
        primary_blocked_reason,
        blocked_reason_count: blocked_reasons.len(),
        would_execute_decision: false,
        would_mutate_contract: false,
        would_return_http_ok: false,
        would_touch_disk: false,
        inherited_blocked_reasons: blocked_reasons.to_vec(),
    }
}

pub(crate) async fn list_contract_repair_approval_requests(
    user_id: auth::UserId,
    State(state): State<AppState>,
) -> Result<Json<ContractRepairApprovalReadModelResponse>, (StatusCode, String)> {
    let records = visible_contract_repair_approval_records(&state, &user_id).await?;
    let persisted_record_count = records
        .iter()
        .filter(|record| record.persistence_enabled)
        .count();
    let preview_record_count = records.len().saturating_sub(persisted_record_count);
    let has_records = !records.is_empty();
    let status = if persisted_record_count > 0 {
        "persisted_read_model".to_string()
    } else if has_records {
        "preview_read_model".to_string()
    } else {
        "read_model_disabled".to_string()
    };
    let route_status = if persisted_record_count > 0 {
        "approval_persistence_enabled".to_string()
    } else {
        "approval_persistence_not_enabled".to_string()
    };
    let blocked_reasons = if persisted_record_count > 0 {
        vec!["contract_mutation_api_disabled".to_string()]
    } else {
        vec!["approval_persistence_not_enabled".to_string()]
    };
    let decision_lock_summary = contract_repair_approval_read_model_decision_lock_summary(
        &status,
        &route_status,
        persisted_record_count,
        preview_record_count,
        persisted_record_count > 0,
        false,
        &blocked_reasons,
    );

    Ok(Json(ContractRepairApprovalReadModelResponse {
        status,
        route_status,
        record_source_status: if persisted_record_count > 0 {
            "durable_records_ready".to_string()
        } else if has_records {
            "transient_preview_ready".to_string()
        } else {
            "record_source_disabled".to_string()
        },
        record_source_kind: if persisted_record_count > 0 {
            CONTRACT_REPAIR_APPROVAL_STORE_KIND.to_string()
        } else if has_records {
            "memory_preview_cache".to_string()
        } else {
            "none".to_string()
        },
        persisted_record_count,
        preview_record_count,
        persistence_enabled: persisted_record_count > 0,
        mutation_enabled: false,
        decision_lock_summary,
        records,
        blocked_reasons,
    }))
}

pub(crate) async fn get_contract_repair_approval_request_detail(
    user_id: auth::UserId,
    State(state): State<AppState>,
    Path(approval_id): Path<String>,
) -> Result<Json<ContractRepairApprovalDetailReadModelResponse>, (StatusCode, String)> {
    let scoped = auth::scoped_key(&user_id, &approval_id);
    let memory_record = state
        .contract_repair_approval_previews
        .read()
        .await
        .get(&scoped)
        .cloned();
    let disk_record = if memory_record.is_none() {
        load_contract_repair_approval_record_from_disk(
            state.contract_repair_approval_store_dir.as_ref(),
            &approval_id,
        )
        .await?
    } else {
        None
    };
    let record = memory_record.or(disk_record);
    let has_preview_record = record.is_some();
    let has_persisted_record = record
        .as_ref()
        .map(|record| record.persistence_enabled)
        .unwrap_or(false);
    let status = if has_persisted_record {
        "detail_persisted_record".to_string()
    } else if has_preview_record {
        "detail_preview_record".to_string()
    } else {
        "detail_read_model_disabled".to_string()
    };
    let route_status = if has_persisted_record {
        "approval_persistence_enabled".to_string()
    } else {
        "approval_persistence_not_enabled".to_string()
    };
    let persisted_record_count = usize::from(has_persisted_record);
    let preview_record_count = usize::from(has_preview_record && !has_persisted_record);
    let blocked_reasons = if has_persisted_record {
        vec!["contract_mutation_api_disabled".to_string()]
    } else {
        vec!["approval_persistence_not_enabled".to_string()]
    };
    let decision_lock_summary = contract_repair_approval_read_model_decision_lock_summary(
        &status,
        &route_status,
        persisted_record_count,
        preview_record_count,
        has_persisted_record,
        false,
        &blocked_reasons,
    );

    Ok(Json(ContractRepairApprovalDetailReadModelResponse {
        status,
        approval_id,
        route_status,
        record_source_status: if has_persisted_record {
            "durable_records_ready".to_string()
        } else if has_preview_record {
            "transient_preview_ready".to_string()
        } else {
            "record_source_disabled".to_string()
        },
        record_source_kind: if has_persisted_record {
            CONTRACT_REPAIR_APPROVAL_STORE_KIND.to_string()
        } else if has_preview_record {
            "memory_preview_cache".to_string()
        } else {
            "none".to_string()
        },
        persisted_record_count,
        preview_record_count,
        persistence_enabled: has_persisted_record,
        mutation_enabled: false,
        decision_lock_summary,
        record,
        blocked_reasons,
    }))
}

pub(crate) async fn create_contract_repair_approval_request(
    user_id: auth::UserId,
    State(state): State<AppState>,
    Json(request): Json<CreateContractRepairApprovalRequest>,
) -> Result<Json<ContractRepairApprovalBlockedResponse>, (StatusCode, String)> {
    validate_contract_repair_approval_request(&request)?;
    let mut approval_record_preview = contract_repair_approval_record_preview(&request)?;
    mark_contract_repair_approval_record_persisted(&mut approval_record_preview);
    persist_contract_repair_approval_record(
        state.contract_repair_approval_store_dir.as_ref(),
        &approval_record_preview,
    )
    .await
    .map_err(|error| internal_error(anyhow::Error::new(error)))?;
    let scoped = auth::scoped_key(&user_id, &approval_record_preview.approval_id);
    state
        .contract_repair_approval_previews
        .write()
        .await
        .insert(scoped, approval_record_preview.clone());

    Err((
        StatusCode::LOCKED,
        serde_json::to_string(&ContractRepairApprovalBlockedResponse {
            status: "blocked".to_string(),
            request_id: request.request_id,
            route_status: "contract_mutation_api_disabled".to_string(),
            payload_kind: request.payload_kind,
            target_path: request.target_path,
            target_kind: request.target_kind,
            changed_fields: request.changed_fields,
            patch_payload: request.patch_payload,
            contract_source_ref: request.contract_source_ref,
            mutation_enabled: false,
            review_required: true,
            blocked_reasons: vec![
                "contract_mutation_api_disabled".to_string(),
                "review_workflow_disabled".to_string(),
            ],
            approval_record_preview,
        })
        .unwrap_or_else(|_| {
            r#"{"status":"blocked","route_status":"contract_mutation_api_disabled"}"#.to_string()
        }),
    ))
}

pub(crate) async fn review_contract_repair_approval_request_preview(
    user_id: auth::UserId,
    State(state): State<AppState>,
    Path(approval_id): Path<String>,
    Json(request): Json<ContractRepairApprovalReviewIntentRequest>,
) -> Result<Json<ContractRepairApprovalReviewBlockedResponse>, (StatusCode, String)> {
    validate_contract_repair_approval_review_intent(&request)?;
    let scoped = auth::scoped_key(&user_id, &approval_id);
    if !state
        .contract_repair_approval_previews
        .read()
        .await
        .contains_key(&scoped)
    {
        if let Some(record) = load_contract_repair_approval_record_from_disk(
            state.contract_repair_approval_store_dir.as_ref(),
            &approval_id,
        )
        .await?
        {
            state
                .contract_repair_approval_previews
                .write()
                .await
                .insert(scoped.clone(), record);
        }
    }
    let action = request.action.trim();
    let approval_record_preview = {
        let mut previews = state.contract_repair_approval_previews.write().await;
        let preview = previews.get_mut(&scoped).ok_or_else(|| {
            json_not_found(
                "contract_repair_approval_preview_not_found",
                "CONTRACT_REPAIR_APPROVAL_PREVIEW_NOT_FOUND",
                "Contract repair approval preview does not exist in the transient review cache.",
            )
        })?;
        let transient_review_status = transient_review_status_for_action(action);
        preview.transient_review_status = transient_review_status.to_string();
        preview.transient_review_action = Some(action.to_string());
        preview.transient_reviewer_id = Some(request.reviewer_id.clone());
        preview.transient_review_reason = Some(request.reason.clone());
        preview.clone()
    };
    if approval_record_preview.persistence_enabled {
        persist_contract_repair_approval_record(
            state.contract_repair_approval_store_dir.as_ref(),
            &approval_record_preview,
        )
        .await
        .map_err(|error| internal_error(anyhow::Error::new(error)))?;
    }
    let persistence_plan_preview =
        contract_repair_approval_persistence_plan_preview(&approval_record_preview);
    let persistence_path_preview =
        contract_repair_approval_persistence_path_preview(&persistence_plan_preview);
    let record_snapshot_preview = contract_repair_approval_record_snapshot_preview(
        &approval_record_preview,
        action,
        &request.reviewer_id,
        &request.reason,
    );
    let storage_readiness_gate = contract_repair_approval_storage_readiness_gate(
        state.contract_repair_approval_store_dir.as_ref(),
        &persistence_plan_preview,
        &persistence_path_preview,
        &record_snapshot_preview,
    )
    .await;
    let storage_dry_run_preview = contract_repair_approval_storage_dry_run_preview(
        &persistence_plan_preview,
        &storage_readiness_gate,
    );
    let idempotency_precheck = contract_repair_approval_idempotency_precheck(
        state.contract_repair_approval_store_dir.as_ref(),
        &persistence_plan_preview,
    )
    .await?;
    let reviewer_authorization_precheck = contract_repair_approval_reviewer_authorization_precheck(
        state.contract_repair_reviewer_grants_path.as_ref(),
        &user_id,
        &request.reviewer_id,
    )
    .await;
    let approve_live_route_gates = contract_repair_approval_approve_live_route_gates();
    let execution_gate = contract_repair_review_execution_gate(
        &user_id,
        action,
        request.review_enabled,
        approval_record_preview.persistence_enabled,
        &request.reviewer_id,
        &storage_readiness_gate,
        &idempotency_precheck,
        &reviewer_authorization_precheck,
        approve_live_route_gates.lifecycle_effects_enabled,
        approve_live_route_gates.contract_source_write_enabled,
    );
    let review_execution_enabled = request.review_enabled
        && matches!(action, "claim" | "reject")
        && execution_gate.blocked_gates.is_empty();
    let approve_final_execution_enabled = true;
    let approve_final_execution_routed_write_handoff_enabled = true;
    let approve_final_execution_ordered_handler_execution_connected = true;
    let approve_final_execution_routed_route_success_application_enabled = true;
    let approve_final_execution_durable_writeback_bundle_enabled = true;
    let approve_final_execution_durable_writeback_bundle_execution_enabled = true;
    let approve_final_execution_durable_writeback_bundle_disk_application_enabled = true;
    let approve_final_execution_durable_disk_application_helper_execution_connected = true;
    let approve_final_execution_legacy_inline_writes_enabled = false;
    let review_record_execution_enabled =
        review_execution_enabled || approve_final_execution_legacy_inline_writes_enabled;
    let approve_live_route_transition_execution_enabled =
        action == "approve" && approve_live_route_gates.review_transition_enabled;
    let review_transition_execution_enabled =
        review_execution_enabled || approve_live_route_transition_execution_enabled;
    let claim_execution_enabled = review_execution_enabled && action == "claim";
    let reject_execution_enabled = review_execution_enabled && action == "reject";
    let decision_execution_preflight_requested = request.review_enabled
        && matches!(action, "approve" | "reject")
        && !reject_execution_enabled;
    let execution_plan_preview = contract_repair_review_execution_plan_preview(
        action,
        &execution_gate,
        review_transition_execution_enabled,
        review_transition_execution_enabled,
    );
    let review_transition_dry_run = contract_repair_approval_review_transition_dry_run(
        &approval_record_preview,
        &record_snapshot_preview,
        &execution_gate,
        review_transition_execution_enabled,
    );
    let record_write_dry_run = contract_repair_approval_record_write_dry_run(
        &persistence_plan_preview,
        &persistence_path_preview,
        &storage_readiness_gate,
        &idempotency_precheck,
        &review_transition_dry_run,
        review_transition_execution_enabled,
    );
    let lifecycle_event_dry_run = contract_repair_approval_lifecycle_event_dry_run(
        &record_snapshot_preview,
        action,
        &review_transition_dry_run,
        review_transition_execution_enabled,
    );
    let lifecycle_event_dry_run = contract_repair_approval_lifecycle_event_with_gate(
        &lifecycle_event_dry_run,
        approve_live_route_gates.lifecycle_effects_enabled,
    );
    let lifecycle_entry_append_dry_run = contract_repair_approval_lifecycle_entry_append_dry_run(
        &lifecycle_event_dry_run,
        review_transition_execution_enabled,
    );
    let lifecycle_entry_append_dry_run = contract_repair_approval_lifecycle_entry_append_with_gate(
        &lifecycle_event_dry_run,
        &lifecycle_entry_append_dry_run,
        approve_live_route_gates.lifecycle_effects_enabled,
    );
    let lifecycle_emission_enablement_gate =
        contract_repair_approval_lifecycle_emission_enablement_gate(
            &lifecycle_event_dry_run,
            &lifecycle_entry_append_dry_run,
        );
    let approve_execution_lifecycle_effects_readiness_dry_run =
        contract_repair_approval_approve_execution_lifecycle_effects_readiness_dry_run(
            action,
            &approval_id,
            &lifecycle_emission_enablement_gate,
        );
    let approve_execution_lifecycle_event_emission_enablement_dry_run =
        contract_repair_approval_approve_execution_lifecycle_event_emission_enablement_dry_run(
            action,
            &approval_id,
            &lifecycle_emission_enablement_gate,
        );
    let approve_execution_lifecycle_entry_append_enablement_dry_run =
        contract_repair_approval_approve_execution_lifecycle_entry_append_enablement_dry_run(
            action,
            &approval_id,
            &lifecycle_emission_enablement_gate,
        );
    let contract_source_resolution_dry_run =
        contract_repair_approval_contract_source_resolution_dry_run(
            state.graph_store_dir.as_ref(),
            &record_snapshot_preview.contract_source_ref,
        )
        .await;
    let contract_patch_plan_preview =
        contract_repair_approval_contract_patch_plan_preview(&record_snapshot_preview);
    let contract_patch_apply_dry_run = contract_repair_approval_contract_patch_apply_dry_run(
        state.graph_store_dir.as_ref(),
        &record_snapshot_preview.contract_source_ref,
        &contract_source_resolution_dry_run,
        &contract_patch_plan_preview,
    )
    .await;
    let contract_source_write_execution_enabled = approve_live_route_gates
        .contract_source_write_enabled
        && approve_final_execution_legacy_inline_writes_enabled;
    let contract_source_write_dry_run = contract_repair_approval_contract_source_write_with_gate(
        state.graph_store_dir.as_ref(),
        &record_snapshot_preview.contract_source_ref,
        &contract_source_resolution_dry_run,
        &contract_patch_plan_preview,
        &contract_patch_apply_dry_run,
        contract_source_write_execution_enabled,
    )
    .await;
    let contract_writeback_dry_run = contract_repair_approval_contract_writeback_dry_run(
        &record_snapshot_preview,
        action,
        &review_transition_dry_run,
        &lifecycle_entry_append_dry_run,
        &contract_source_resolution_dry_run,
        contract_patch_plan_preview.clone(),
        &contract_patch_apply_dry_run,
        &contract_source_write_dry_run,
    );
    let contract_mutation_enablement_gate =
        contract_repair_approval_contract_mutation_enablement_gate(
            &lifecycle_emission_enablement_gate,
            &contract_writeback_dry_run,
            approve_live_route_gates.contract_source_write_enabled,
        );
    let approve_execution_contract_mutation_readiness_dry_run =
        contract_repair_approval_approve_execution_contract_mutation_readiness_dry_run(
            action,
            &approval_id,
            &contract_mutation_enablement_gate,
        );
    let approve_execution_gate = contract_repair_approval_approve_execution_gate(
        action,
        approval_record_preview.persistence_enabled,
        &review_transition_dry_run,
        &lifecycle_event_dry_run,
        &lifecycle_entry_append_dry_run,
        &contract_writeback_dry_run,
        &contract_mutation_enablement_gate,
    );
    let approve_execution_transaction_dry_run =
        contract_repair_approval_approve_execution_transaction_dry_run(
            action,
            &approval_id,
            &approve_execution_gate,
            &review_transition_dry_run,
            &record_write_dry_run,
            &lifecycle_event_dry_run,
            &lifecycle_entry_append_dry_run,
            &contract_writeback_dry_run,
        );
    let approve_execution_admission_gate =
        contract_repair_approval_approve_execution_admission_gate(
            action,
            &approval_id,
            &approve_execution_transaction_dry_run,
        );
    let approve_execution_admission_gate =
        contract_repair_approval_approve_execution_admission_gate_with_gate(
            &approve_execution_admission_gate,
            approve_live_route_gates.transaction_runner_enabled,
        );
    let approve_execution_transaction_runner_enablement_dry_run =
        contract_repair_approval_approve_execution_transaction_runner_enablement_dry_run(
            action,
            &approval_id,
            &approve_execution_transaction_dry_run,
            &approve_execution_admission_gate,
        );
    let approve_execution_transaction_runner_dry_run =
        contract_repair_approval_approve_execution_transaction_runner_dry_run(
            action,
            &approval_id,
            &approve_execution_transaction_dry_run,
            &approve_execution_admission_gate,
        );
    let approve_execution_recovery_marker_write_dry_run =
        contract_repair_approval_approve_execution_recovery_marker_write_dry_run(
            &approval_id,
            &storage_readiness_gate,
            &approve_execution_transaction_runner_dry_run,
        );
    let approve_execution_recovery_marker_idempotency_precheck =
        contract_repair_approval_approve_execution_recovery_marker_idempotency_precheck(
            state.contract_repair_approval_store_dir.as_ref(),
            &approve_execution_recovery_marker_write_dry_run,
        )
        .await;
    let recovery_marker_persistence_execution_enabled = approve_live_route_gates
        .recovery_marker_persistence_enabled
        && approve_final_execution_legacy_inline_writes_enabled;
    let approve_execution_recovery_marker_write_dry_run =
        contract_repair_approval_approve_execution_recovery_marker_write_with_gate(
            state.contract_repair_approval_store_dir.as_ref(),
            &approve_execution_recovery_marker_write_dry_run,
            &approve_execution_recovery_marker_idempotency_precheck,
            &approve_execution_transaction_runner_dry_run,
            recovery_marker_persistence_execution_enabled,
        )
        .await;
    let approve_execution_recovery_marker_persistence_gate =
        contract_repair_approval_approve_execution_recovery_marker_persistence_gate(
            &approve_execution_recovery_marker_write_dry_run,
            &approve_execution_recovery_marker_idempotency_precheck,
            approve_live_route_gates.recovery_marker_persistence_enabled,
        );
    let approve_execution_recovery_marker_persistence_readiness_dry_run =
        contract_repair_approval_approve_execution_recovery_marker_persistence_readiness_dry_run(
            action,
            &approval_id,
            &approve_execution_recovery_marker_persistence_gate,
        );
    let approve_execution_transaction_commit_gate =
        contract_repair_approval_approve_execution_transaction_commit_gate(
            &approve_execution_transaction_runner_dry_run,
            &approve_execution_recovery_marker_persistence_gate,
        );
    let approve_execution_transaction_commit_gate =
        contract_repair_approval_approve_execution_transaction_commit_gate_with_gate(
            &approve_execution_transaction_commit_gate,
            approve_live_route_gates.transaction_commit_enabled,
        );
    let approve_execution_transaction_commit_readiness_dry_run =
        contract_repair_approval_approve_execution_transaction_commit_readiness_dry_run(
            &approve_execution_transaction_commit_gate,
        );
    let approve_execution_atomic_side_effects_gate =
        contract_repair_approval_approve_execution_atomic_side_effects_gate(
            &lifecycle_emission_enablement_gate,
            &contract_mutation_enablement_gate,
            &approve_execution_recovery_marker_persistence_gate,
            &approve_execution_transaction_commit_gate,
            approve_live_route_gates.atomic_side_effects_enabled,
        );
    let approve_execution_atomic_side_effects_enablement_dry_run =
        contract_repair_approval_approve_execution_atomic_side_effects_enablement_dry_run(
            &approve_execution_atomic_side_effects_gate,
        );
    let approve_execution_atomic_side_effects_readiness_dry_run =
        contract_repair_approval_approve_execution_atomic_side_effects_readiness_dry_run(
            &approve_execution_atomic_side_effects_gate,
        );
    let approve_execution_runner_attempt_enablement_dry_run =
        contract_repair_approval_approve_execution_runner_attempt_enablement_dry_run(
            action,
            &approval_id,
            request.review_enabled,
            &approve_execution_atomic_side_effects_gate,
            &approve_execution_transaction_runner_enablement_dry_run,
            approve_live_route_gates.runner_attempt_enabled,
        );
    let approve_execution_runner_attempt =
        contract_repair_approval_approve_execution_runner_attempt(
            action,
            &approval_id,
            request.review_enabled,
            &approve_execution_atomic_side_effects_gate,
            approve_live_route_gates.runner_attempt_enabled,
        );
    let approve_execution_runner_attempt_readiness_dry_run =
        contract_repair_approval_approve_execution_runner_attempt_readiness_dry_run(
            &approve_execution_runner_attempt,
            &approve_execution_runner_attempt_enablement_dry_run,
        );
    let approve_execution_runner_execution_enablement_dry_run =
        contract_repair_approval_approve_execution_runner_execution_enablement_dry_run(
            &approve_execution_runner_attempt,
            &approve_execution_runner_attempt_enablement_dry_run,
            approve_live_route_gates.runner_execution_enabled,
        );
    let approve_execution_runner_outcome =
        contract_repair_approval_approve_execution_runner_outcome(
            &approve_execution_runner_attempt,
            approve_live_route_gates.runner_execution_enabled,
        );
    let approve_execution_runner_execution_readiness_dry_run =
        contract_repair_approval_approve_execution_runner_execution_readiness_dry_run(
            &approve_execution_runner_outcome,
            &approve_execution_runner_execution_enablement_dry_run,
        );
    let approve_execution_runner_route_dispatch_enablement_dry_run =
        contract_repair_approval_approve_execution_runner_route_dispatch_enablement_dry_run(
            &approve_execution_runner_outcome,
            &approve_execution_runner_execution_enablement_dry_run,
            approve_live_route_gates.route_dispatch_enabled,
        );
    let approve_execution_runner_dispatch_gate =
        contract_repair_approval_approve_execution_runner_dispatch_gate(
            &approve_execution_runner_outcome,
            approve_live_route_gates.route_dispatch_enabled,
        );
    let approve_execution_runner_dispatch_readiness_dry_run =
        contract_repair_approval_approve_execution_runner_dispatch_readiness_dry_run(
            &approve_execution_runner_dispatch_gate,
            &approve_execution_runner_route_dispatch_enablement_dry_run,
        );
    let approve_execution_runner_handoff =
        contract_repair_approval_approve_execution_runner_handoff(
            &approve_execution_runner_dispatch_gate,
        );
    let approve_execution_runner_handoff_readiness_dry_run =
        contract_repair_approval_approve_execution_runner_handoff_readiness_dry_run(
            &approve_execution_runner_handoff,
            &approve_execution_runner_dispatch_readiness_dry_run,
        );
    let approve_execution_runner_call_enablement_dry_run =
        contract_repair_approval_approve_execution_runner_call_enablement_dry_run(
            &approve_execution_runner_handoff,
            &approve_execution_runner_route_dispatch_enablement_dry_run,
            approve_live_route_gates.runner_call_enabled,
        );
    let approve_execution_runner_call_dry_run =
        contract_repair_approval_approve_execution_runner_call_dry_run(
            &approve_execution_runner_handoff,
            approve_live_route_gates.runner_call_enabled,
        );
    let approve_execution_runner_call_readiness_dry_run =
        contract_repair_approval_approve_execution_runner_call_readiness_dry_run(
            &approve_execution_runner_call_dry_run,
            &approve_execution_runner_call_enablement_dry_run,
        );
    let approve_execution_runner_body_enablement_dry_run =
        contract_repair_approval_approve_execution_runner_body_enablement_dry_run(
            &approve_execution_runner_call_dry_run,
            &approve_execution_atomic_side_effects_gate,
            approve_live_route_gates.runner_body_enabled,
        );
    let approve_execution_runner_call_body_dry_run =
        contract_repair_approval_approve_execution_runner_call_body_dry_run(
            &approve_execution_runner_call_dry_run,
            &approve_execution_atomic_side_effects_gate,
            approve_live_route_gates.runner_body_enabled,
        );
    let approve_execution_runner_body_readiness_dry_run =
        contract_repair_approval_approve_execution_runner_body_readiness_dry_run(
            &approve_execution_runner_call_body_dry_run,
            &approve_execution_runner_body_enablement_dry_run,
        );
    let approve_execution_runner_phase_execution_enablement_dry_run =
        contract_repair_approval_approve_execution_runner_phase_execution_enablement_dry_run(
            &approve_execution_runner_call_body_dry_run,
            &approve_execution_transaction_runner_dry_run,
            approve_live_route_gates.phase_execution_enabled,
        );
    let approve_execution_runner_body_phase_sequence_dry_run =
        contract_repair_approval_approve_execution_runner_body_phase_sequence_dry_run(
            &approve_execution_runner_call_body_dry_run,
            &approve_execution_transaction_runner_dry_run,
            approve_live_route_gates.phase_execution_enabled,
        );
    let approve_execution_runner_phases_readiness_dry_run =
        contract_repair_approval_approve_execution_runner_phases_readiness_dry_run(
            &approve_execution_runner_body_phase_sequence_dry_run,
            &approve_execution_runner_phase_execution_enablement_dry_run,
        );
    let approve_execution_runner_lifecycle_phase_enablement_dry_run =
        contract_repair_approval_approve_execution_runner_lifecycle_phase_enablement_dry_run(
            &approve_execution_runner_body_phase_sequence_dry_run,
            &approve_execution_runner_phase_execution_enablement_dry_run,
            &lifecycle_emission_enablement_gate,
            approve_live_route_gates.lifecycle_phase_enabled,
        );
    let approve_execution_runner_lifecycle_phase_dry_run =
        contract_repair_approval_approve_execution_runner_lifecycle_phase_dry_run(
            &approve_execution_runner_body_phase_sequence_dry_run,
            &lifecycle_emission_enablement_gate,
            approve_live_route_gates.lifecycle_phase_enabled,
        );
    let approve_execution_runner_lifecycle_phase_readiness_dry_run =
        contract_repair_approval_approve_execution_runner_lifecycle_phase_readiness_dry_run(
            &approve_execution_runner_lifecycle_phase_dry_run,
            &approve_execution_runner_lifecycle_phase_enablement_dry_run,
        );
    let approve_execution_runner_source_mutation_phase_enablement_dry_run =
        contract_repair_approval_approve_execution_runner_source_mutation_phase_enablement_dry_run(
            &approve_execution_runner_body_phase_sequence_dry_run,
            &approve_execution_runner_lifecycle_phase_enablement_dry_run,
            &approve_execution_runner_lifecycle_phase_dry_run,
            &contract_mutation_enablement_gate,
            approve_live_route_gates.source_mutation_phase_enabled,
        );
    let approve_execution_runner_source_mutation_phase_dry_run =
        contract_repair_approval_approve_execution_runner_source_mutation_phase_dry_run(
            &approve_execution_runner_body_phase_sequence_dry_run,
            &approve_execution_runner_lifecycle_phase_dry_run,
            &contract_mutation_enablement_gate,
            approve_live_route_gates.source_mutation_phase_enabled,
        );
    let approve_execution_runner_source_mutation_phase_readiness_dry_run =
        contract_repair_approval_approve_execution_runner_source_mutation_phase_readiness_dry_run(
            &approve_execution_runner_source_mutation_phase_dry_run,
            &approve_execution_runner_source_mutation_phase_enablement_dry_run,
        );
    let approve_execution_runner_recovery_marker_cleanup_phase_enablement_dry_run =
        contract_repair_approval_approve_execution_runner_recovery_marker_cleanup_phase_enablement_dry_run(
            &approve_execution_runner_body_phase_sequence_dry_run,
            &approve_execution_runner_source_mutation_phase_enablement_dry_run,
            &approve_execution_runner_source_mutation_phase_dry_run,
            &approve_execution_recovery_marker_persistence_gate,
            approve_live_route_gates.recovery_marker_cleanup_phase_enabled,
        );
    let approve_execution_runner_recovery_marker_cleanup_phase_dry_run =
        contract_repair_approval_approve_execution_runner_recovery_marker_cleanup_phase_dry_run(
            &approve_execution_runner_body_phase_sequence_dry_run,
            &approve_execution_runner_source_mutation_phase_dry_run,
            &approve_execution_recovery_marker_persistence_gate,
            approve_live_route_gates.recovery_marker_cleanup_phase_enabled,
        );
    let approve_execution_runner_recovery_marker_cleanup_phase_dry_run =
        contract_repair_approval_approve_execution_runner_recovery_marker_cleanup_phase_with_gate(
            state.contract_repair_approval_store_dir.as_ref(),
            &approve_execution_runner_recovery_marker_cleanup_phase_dry_run,
            &approve_execution_runner_recovery_marker_cleanup_phase_enablement_dry_run,
            approve_live_route_gates.recovery_marker_cleanup_phase_enabled
                && approve_final_execution_legacy_inline_writes_enabled,
        )
        .await;
    let approve_execution_runner_recovery_marker_cleanup_phase_readiness_dry_run =
        contract_repair_approval_approve_execution_runner_recovery_marker_cleanup_phase_readiness_dry_run(
            &approve_execution_runner_recovery_marker_cleanup_phase_dry_run,
            &approve_execution_runner_recovery_marker_cleanup_phase_enablement_dry_run,
        );
    let approve_execution_runner_transaction_commit_phase_enablement_dry_run =
        contract_repair_approval_approve_execution_runner_transaction_commit_phase_enablement_dry_run(
            &approve_execution_runner_body_phase_sequence_dry_run,
            &approve_execution_runner_recovery_marker_cleanup_phase_enablement_dry_run,
            &approve_execution_runner_recovery_marker_cleanup_phase_dry_run,
            &approve_execution_transaction_commit_gate,
            approve_live_route_gates.transaction_commit_phase_enabled,
        );
    let approve_execution_runner_transaction_commit_phase_dry_run =
        contract_repair_approval_approve_execution_runner_transaction_commit_phase_dry_run(
            &approve_execution_runner_body_phase_sequence_dry_run,
            &approve_execution_runner_recovery_marker_cleanup_phase_dry_run,
            &approve_execution_transaction_commit_gate,
            approve_live_route_gates.transaction_commit_phase_enabled,
        );
    let approve_execution_runner_transaction_commit_phase_dry_run =
        contract_repair_approval_approve_execution_runner_transaction_commit_phase_with_gate(
            &approve_execution_runner_transaction_commit_phase_dry_run,
            &approve_execution_runner_transaction_commit_phase_enablement_dry_run,
            approve_live_route_gates.transaction_commit_phase_enabled
                && approve_final_execution_legacy_inline_writes_enabled,
        );
    let approve_execution_runner_transaction_commit_phase_readiness_dry_run =
        contract_repair_approval_approve_execution_runner_transaction_commit_phase_readiness_dry_run(
            &approve_execution_runner_transaction_commit_phase_dry_run,
            &approve_execution_runner_transaction_commit_phase_enablement_dry_run,
        );
    let approve_execution_runner_rollback_execution_phase_enablement_dry_run =
        contract_repair_approval_approve_execution_runner_rollback_execution_phase_enablement_dry_run(
            &approve_execution_runner_body_phase_sequence_dry_run,
            &approve_execution_runner_transaction_commit_phase_enablement_dry_run,
            &approve_execution_runner_transaction_commit_phase_dry_run,
            approve_live_route_gates.rollback_execution_enabled,
        );
    let approve_execution_runner_rollback_execution_phase_dry_run =
        contract_repair_approval_approve_execution_runner_rollback_execution_phase_dry_run(
            &approve_execution_runner_body_phase_sequence_dry_run,
            &approve_execution_runner_transaction_commit_phase_dry_run,
            approve_live_route_gates.rollback_execution_enabled,
        );
    let approve_execution_runner_rollback_execution_phase_readiness_dry_run =
        contract_repair_approval_approve_execution_runner_rollback_execution_phase_readiness_dry_run(
            &approve_execution_runner_rollback_execution_phase_dry_run,
            &approve_execution_runner_rollback_execution_phase_enablement_dry_run,
        );
    let approve_execution_runner_enablement_plan_dry_run =
        contract_repair_approval_approve_execution_runner_enablement_plan_dry_run(
            &approve_execution_runner_attempt,
            &approve_execution_runner_outcome,
            &approve_execution_runner_dispatch_gate,
            &approve_execution_runner_call_dry_run,
            &approve_execution_runner_call_body_dry_run,
            &approve_execution_runner_body_phase_sequence_dry_run,
            &approve_execution_runner_lifecycle_phase_dry_run,
            &approve_execution_runner_source_mutation_phase_dry_run,
            &approve_execution_runner_recovery_marker_cleanup_phase_dry_run,
            &approve_execution_runner_transaction_commit_phase_dry_run,
            &approve_execution_runner_rollback_execution_phase_dry_run,
            approve_live_route_gates.runner_activation_enabled,
        );
    let approve_execution_runner_activation_enablement_dry_run =
        contract_repair_approval_approve_execution_runner_activation_enablement_dry_run(
            &approve_execution_runner_enablement_plan_dry_run,
            &approve_execution_runner_rollback_execution_phase_enablement_dry_run,
        );
    let approve_execution_runner_activation_enablement_readiness_dry_run =
        contract_repair_approval_approve_execution_runner_activation_enablement_readiness_dry_run(
            &approve_execution_runner_activation_enablement_dry_run,
        );
    let approve_execution_runner_activation_path_dry_run =
        contract_repair_approval_approve_execution_runner_activation_path_dry_run(
            &approve_execution_runner_activation_enablement_dry_run,
        );
    let approve_execution_runner_activation_path_readiness_dry_run =
        contract_repair_approval_approve_execution_runner_activation_path_readiness_dry_run(
            &approve_execution_runner_activation_path_dry_run,
            &approve_execution_runner_activation_enablement_readiness_dry_run,
        );
    let approve_execution_runner_activation_execution_plan_dry_run =
        contract_repair_approval_approve_execution_runner_activation_execution_plan_dry_run(
            &approve_execution_runner_activation_path_dry_run,
        );
    let approve_execution_runner_activation_execution_plan_readiness_dry_run =
        contract_repair_approval_approve_execution_runner_activation_execution_plan_readiness_dry_run(
            &approve_execution_runner_activation_execution_plan_dry_run,
            &approve_execution_runner_activation_path_readiness_dry_run,
        );
    let approve_execution_runner_activation_switch_transaction_proof_dry_run =
        contract_repair_approval_approve_execution_runner_activation_switch_transaction_proof_dry_run(
            &approve_execution_runner_activation_execution_plan_dry_run,
        );
    let approve_execution_runner_activation_switch_transaction_proof_readiness_dry_run =
        contract_repair_approval_approve_execution_runner_activation_switch_transaction_proof_readiness_dry_run(
            &approve_execution_runner_activation_switch_transaction_proof_dry_run,
            &approve_execution_runner_activation_execution_plan_readiness_dry_run,
        );
    let approve_execution_runner_activation_switch_write_transaction_enablement_dry_run =
        contract_repair_approval_approve_execution_runner_activation_switch_write_transaction_enablement_dry_run(
            &approve_execution_runner_activation_switch_transaction_proof_dry_run,
            approve_live_route_gates.activation_switch_write_transaction_enabled,
        );
    let approve_execution_runner_activation_switch_write_transaction_enablement_readiness_dry_run =
        contract_repair_approval_approve_execution_runner_activation_switch_write_transaction_enablement_readiness_dry_run(
            &approve_execution_runner_activation_switch_write_transaction_enablement_dry_run,
            &approve_execution_runner_activation_switch_transaction_proof_readiness_dry_run,
        );
    let approve_execution_runner_activation_switch_write_transaction_dry_run =
        contract_repair_approval_approve_execution_runner_activation_switch_write_transaction_dry_run(
            &approve_execution_runner_activation_switch_write_transaction_enablement_dry_run,
        );
    let approve_execution_runner_activation_switch_write_transaction_readiness_dry_run =
        contract_repair_approval_approve_execution_runner_activation_switch_write_transaction_readiness_dry_run(
            &approve_execution_runner_activation_switch_write_transaction_dry_run,
            &approve_execution_runner_activation_switch_write_transaction_enablement_readiness_dry_run,
        );
    let approve_execution_runner_activation_transaction_admission_gate_dry_run =
        contract_repair_approval_approve_execution_runner_activation_transaction_admission_gate_dry_run(
            &approve_execution_runner_activation_path_dry_run,
            &approve_execution_runner_activation_execution_plan_dry_run,
            &approve_execution_runner_activation_switch_transaction_proof_dry_run,
            &approve_execution_runner_activation_switch_write_transaction_enablement_dry_run,
            &approve_execution_runner_activation_switch_write_transaction_dry_run,
        );
    let approve_execution_runner_activation_transaction_admission_readiness_dry_run =
        contract_repair_approval_approve_execution_runner_activation_transaction_admission_readiness_dry_run(
            &approve_execution_runner_activation_transaction_admission_gate_dry_run,
            &approve_execution_runner_activation_switch_write_transaction_readiness_dry_run,
        );
    let approve_execution_runner_activation_admission_handoff_dry_run =
        contract_repair_approval_approve_execution_runner_activation_admission_handoff_dry_run(
            &approve_execution_runner_enablement_plan_dry_run,
            &approve_execution_runner_activation_transaction_admission_gate_dry_run,
        );
    let approve_execution_runner_activation_admission_handoff_readiness_dry_run =
        contract_repair_approval_approve_execution_runner_activation_admission_handoff_readiness_dry_run(
            &approve_execution_runner_activation_admission_handoff_dry_run,
            &approve_execution_runner_activation_transaction_admission_readiness_dry_run,
        );
    let approve_execution_runner_activation_handoff_enablement_dry_run =
        contract_repair_approval_approve_execution_runner_activation_handoff_enablement_dry_run(
            &approve_execution_runner_activation_admission_handoff_dry_run,
        );
    let approve_execution_runner_activation_handoff_enablement_readiness_dry_run =
        contract_repair_approval_approve_execution_runner_activation_handoff_enablement_readiness_dry_run(
            &approve_execution_runner_activation_handoff_enablement_dry_run,
            &approve_execution_runner_activation_admission_handoff_readiness_dry_run,
        );
    let approve_execution_runner_activation_handoff_attempt_dry_run =
        contract_repair_approval_approve_execution_runner_activation_handoff_attempt_dry_run(
            &approve_execution_runner_activation_handoff_enablement_dry_run,
        );
    let approve_execution_runner_activation_handoff_attempt_readiness_dry_run =
        contract_repair_approval_approve_execution_runner_activation_handoff_attempt_readiness_dry_run(
            &approve_execution_runner_activation_handoff_attempt_dry_run,
            &approve_execution_runner_activation_handoff_enablement_readiness_dry_run,
        );
    let approve_execution_runner_activation_post_handoff_attempt_dry_run =
        contract_repair_approval_approve_execution_runner_activation_post_handoff_attempt_dry_run(
            &approve_execution_runner_activation_handoff_attempt_dry_run,
        );
    let approve_execution_runner_activation_post_handoff_attempt_readiness_dry_run =
        contract_repair_approval_approve_execution_runner_activation_post_handoff_attempt_readiness_dry_run(
            &approve_execution_runner_activation_post_handoff_attempt_dry_run,
            &approve_execution_runner_activation_handoff_attempt_readiness_dry_run,
        );
    let approve_execution_runner_activation_success_admission_dry_run =
        contract_repair_approval_approve_execution_runner_activation_success_admission_dry_run(
            &approve_execution_runner_activation_post_handoff_attempt_dry_run,
        );
    let approve_execution_runner_activation_success_admission_readiness_dry_run =
        contract_repair_approval_approve_execution_runner_activation_success_admission_readiness_dry_run(
            &approve_execution_runner_activation_success_admission_dry_run,
            &approve_execution_runner_activation_post_handoff_attempt_readiness_dry_run,
        );
    let approve_execution_runner_activation_success_return_dry_run =
        contract_repair_approval_approve_execution_runner_activation_success_return_dry_run(
            &approve_execution_runner_activation_success_admission_dry_run,
        );
    let approve_execution_runner_activation_success_return_readiness_dry_run =
        contract_repair_approval_approve_execution_runner_activation_success_return_readiness_dry_run(
            &approve_execution_runner_activation_success_return_dry_run,
            &approve_execution_runner_activation_success_admission_readiness_dry_run,
        );
    let approve_execution_runner_route_success_readiness_dry_run =
        contract_repair_approval_approve_execution_runner_route_success_readiness_dry_run(
            &approve_execution_runner_activation_success_return_readiness_dry_run,
            &approve_execution_runner_enablement_plan_dry_run,
        );
    let approve_execution_runner_route_success_readiness_dry_run =
        contract_repair_approval_approve_execution_runner_route_success_with_gate(
            &approve_execution_runner_route_success_readiness_dry_run,
            approve_live_route_gates.route_success_enabled
                && approve_final_execution_legacy_inline_writes_enabled,
        );
    let approve_execution_runner_control_readiness_dry_run =
        contract_repair_approval_approve_execution_runner_control_readiness_dry_run(
            &approve_execution_runner_attempt,
            &approve_execution_runner_outcome,
            &approve_execution_runner_dispatch_gate,
            &approve_execution_runner_call_dry_run,
            &approve_execution_runner_call_body_dry_run,
            &approve_execution_runner_body_phase_sequence_dry_run,
        );
    let approve_runner_success = contract_repair_approval_approve_runner_success_from_route_success(
        &approve_execution_runner_route_success_readiness_dry_run,
    );
    let pre_release_response_status = if claim_execution_enabled {
        "review_claim_executed"
    } else if reject_execution_enabled {
        "review_reject_executed"
    } else if approve_runner_success {
        "review_approve_executed"
    } else if decision_execution_preflight_requested {
        "review_decision_execution_blocked"
    } else {
        "blocked"
    };
    let pre_release_route_status = if claim_execution_enabled {
        "review_claim_executed"
    } else if reject_execution_enabled {
        "review_reject_executed"
    } else if approve_runner_success {
        "review_approve_executed"
    } else if decision_execution_preflight_requested {
        "review_decision_execution_blocked"
    } else {
        "review_workflow_disabled"
    };
    let pre_release_blocked_reasons =
        if claim_execution_enabled || reject_execution_enabled || approve_runner_success {
            Vec::new()
        } else if decision_execution_preflight_requested {
            let decision_execution_blocker = if action == "approve" {
                "approve_execution_not_enabled"
            } else {
                "reject_execution_not_enabled"
            };
            vec![
                decision_execution_blocker.to_string(),
                "lifecycle_event_emission_disabled".to_string(),
                "contract_mutation_api_disabled".to_string(),
            ]
        } else {
            vec![
                "review_workflow_disabled".to_string(),
                "approval_persistence_not_enabled".to_string(),
                "contract_mutation_api_disabled".to_string(),
            ]
        };
    let approve_execution_runner_route_status_readiness_dry_run =
        contract_repair_approval_approve_execution_runner_route_status_readiness_dry_run(
            &request.action,
            &approval_id,
            &pre_release_response_status,
            &pre_release_route_status,
            &pre_release_blocked_reasons,
            &approve_execution_runner_route_success_readiness_dry_run,
            decision_execution_preflight_requested,
            review_execution_enabled,
            approve_runner_success,
        );
    let approve_execution_formal_review_execution_readiness_dry_run =
        contract_repair_approval_approve_execution_formal_review_execution_readiness_dry_run(
            &request.action,
            &approval_id,
            request.review_enabled,
            &execution_gate,
            &approve_execution_gate,
            &approve_execution_runner_route_success_readiness_dry_run,
            &approve_execution_runner_route_status_readiness_dry_run,
            &pre_release_blocked_reasons,
            decision_execution_preflight_requested,
            approve_live_route_gates.formal_approve_review_execution_enabled,
            review_execution_enabled,
        );
    let approve_execution_final_atomic_readiness_dry_run =
        contract_repair_approval_approve_execution_final_atomic_readiness_dry_run(
            &request.action,
            &approval_id,
            &record_write_dry_run,
            &lifecycle_emission_enablement_gate,
            &contract_mutation_enablement_gate,
            &approve_execution_recovery_marker_persistence_gate,
            &approve_execution_transaction_commit_gate,
            &approve_execution_runner_route_success_readiness_dry_run,
            &approve_execution_formal_review_execution_readiness_dry_run,
            decision_execution_preflight_requested,
            review_execution_enabled,
        );
    let approve_execution_final_atomic_execution_plan_dry_run =
        contract_repair_approval_approve_execution_final_atomic_execution_plan_dry_run(
            &request.action,
            &approval_id,
            &approve_execution_transaction_runner_dry_run,
            &approve_execution_runner_recovery_marker_cleanup_phase_readiness_dry_run,
            &approve_execution_runner_transaction_commit_phase_readiness_dry_run,
            &approve_execution_runner_rollback_execution_phase_readiness_dry_run,
            &approve_execution_final_atomic_readiness_dry_run,
            review_execution_enabled,
        );
    let approve_execution_final_atomic_admission_gate_dry_run =
        contract_repair_approval_approve_execution_final_atomic_admission_gate_dry_run(
            &request.action,
            &approval_id,
            &approve_execution_final_atomic_execution_plan_dry_run,
        );
    let approve_execution_final_execution_entry_dry_run =
        contract_repair_approval_approve_execution_final_execution_entry_dry_run(
            &request.action,
            &approval_id,
            &approve_execution_final_atomic_admission_gate_dry_run,
            &approve_execution_runner_route_status_readiness_dry_run,
            &pre_release_blocked_reasons,
            approve_final_execution_enabled,
        );
    let approve_execution_final_execution_switch_readiness_dry_run =
        contract_repair_approval_approve_execution_final_execution_switch_readiness_dry_run(
            &request.action,
            &approval_id,
            &approve_execution_final_execution_entry_dry_run,
            &record_write_dry_run,
            &contract_mutation_enablement_gate,
            &approve_execution_recovery_marker_persistence_gate,
            &approve_execution_runner_recovery_marker_cleanup_phase_readiness_dry_run,
            &approve_execution_runner_transaction_commit_phase_readiness_dry_run,
            approve_final_execution_enabled,
        );
    let approve_execution_final_execution_rollback_readiness_dry_run =
        contract_repair_approval_approve_execution_final_execution_rollback_readiness_dry_run(
            &request.action,
            &approval_id,
            &approve_execution_final_execution_switch_readiness_dry_run,
            &approve_execution_runner_rollback_execution_phase_readiness_dry_run,
            approve_final_execution_enabled,
        );
    let approve_execution_final_execution_replay_plan_dry_run =
        contract_repair_approval_approve_execution_final_execution_replay_plan_dry_run(
            &request.action,
            &approval_id,
            &approve_execution_final_execution_switch_readiness_dry_run,
            &approve_execution_final_execution_rollback_readiness_dry_run,
            approve_final_execution_enabled,
        );
    let approve_execution_final_execution_replay_executor_dry_run =
        contract_repair_approval_approve_execution_final_execution_replay_executor_dry_run(
            &request.action,
            &approval_id,
            &approve_execution_final_execution_replay_plan_dry_run,
            approve_final_execution_enabled,
        );
    let approve_execution_final_execution_replay_executor_routing_dry_run =
        contract_repair_approval_approve_execution_final_execution_replay_executor_routing_dry_run(
            &request.action,
            &approval_id,
            &approve_execution_final_execution_replay_executor_dry_run,
            approve_final_execution_enabled,
        );
    let approve_execution_final_execution_routed_write_handoff_dry_run =
        contract_repair_approval_approve_execution_final_execution_routed_write_handoff_dry_run(
            &request.action,
            &approval_id,
            &approve_execution_final_execution_replay_executor_routing_dry_run,
            approve_final_execution_enabled,
            approve_final_execution_routed_write_handoff_enabled,
            approve_final_execution_legacy_inline_writes_enabled,
        );
    let approve_execution_final_execution_routed_handler_plan_dry_run =
        contract_repair_approval_approve_execution_final_execution_routed_handler_plan_dry_run(
            &request.action,
            &approval_id,
            &approve_execution_final_execution_routed_write_handoff_dry_run,
            &approve_execution_recovery_marker_write_dry_run,
            &review_transition_dry_run,
            &record_write_dry_run,
            &lifecycle_event_dry_run,
            &lifecycle_entry_append_dry_run,
            &contract_source_write_dry_run,
            &approve_execution_runner_recovery_marker_cleanup_phase_readiness_dry_run,
            &approve_execution_runner_transaction_commit_phase_readiness_dry_run,
            &approve_execution_runner_route_success_readiness_dry_run,
        );
    let approve_execution_final_execution_routed_execution_attempt_dry_run =
        contract_repair_approval_approve_execution_final_execution_routed_execution_attempt_dry_run(
            &request.action,
            &approval_id,
            &approve_execution_final_execution_routed_handler_plan_dry_run,
        );
    let approve_execution_final_execution_ordered_handler_execution_confirmation_dry_run =
        contract_repair_approval_approve_execution_final_execution_ordered_handler_execution_confirmation_dry_run(
            &request.action,
            &approval_id,
            &approve_execution_final_execution_entry_dry_run,
            &approve_execution_final_execution_rollback_readiness_dry_run,
            &approve_execution_final_execution_routed_execution_attempt_dry_run,
            approve_final_execution_ordered_handler_execution_connected,
        );
    let approve_execution_final_execution_routed_route_success_release_dry_run =
        contract_repair_approval_approve_execution_final_execution_routed_route_success_release_dry_run(
            &request.action,
            &approval_id,
            pre_release_response_status,
            pre_release_route_status,
            &approve_execution_final_execution_routed_execution_attempt_dry_run,
            &approve_execution_final_execution_ordered_handler_execution_confirmation_dry_run,
            approve_final_execution_legacy_inline_writes_enabled,
            approve_final_execution_routed_route_success_application_enabled,
        );
    let routed_route_success_release_applied =
        approve_execution_final_execution_routed_route_success_release_dry_run
            .routed_route_success_release_applied;
    let preliminary_response_application_success =
        approve_runner_success || routed_route_success_release_applied;
    let approve_execution_final_execution_durable_writeback_bundle_dry_run =
        contract_repair_approval_approve_execution_final_execution_durable_writeback_bundle_dry_run(
            &request.action,
            &approval_id,
            preliminary_response_application_success,
            &record_write_dry_run,
            review_record_execution_enabled,
            &approve_execution_recovery_marker_write_dry_run,
            &contract_source_write_dry_run,
            contract_source_write_execution_enabled,
            &approve_execution_recovery_marker_persistence_gate,
            &approve_execution_transaction_commit_gate,
            &approve_execution_runner_recovery_marker_cleanup_phase_readiness_dry_run,
            &approve_execution_final_execution_rollback_readiness_dry_run,
            &approve_execution_final_execution_entry_dry_run,
            &approve_execution_final_execution_routed_route_success_release_dry_run,
            approve_final_execution_durable_writeback_bundle_enabled,
            approve_final_execution_durable_writeback_bundle_execution_enabled,
            approve_final_execution_durable_writeback_bundle_disk_application_enabled,
            approve_final_execution_durable_disk_application_helper_execution_connected,
        );
    let mut approval_record_for_response_application = approval_record_preview.clone();
    if review_record_execution_enabled || routed_route_success_release_applied {
        approval_record_for_response_application.review_state =
            target_review_state_for_action(action);
        approval_record_for_response_application.transient_review_status =
            executed_review_status_for_action(action).to_string();
        approval_record_for_response_application.transient_review_action = Some(action.to_string());
        approval_record_for_response_application.transient_reviewer_id =
            Some(request.reviewer_id.clone());
        approval_record_for_response_application.transient_review_reason =
            Some(request.reason.clone());
        if lifecycle_entry_append_dry_run.would_append {
            approval_record_for_response_application.lifecycle.push(
                contract_repair_approval_lifecycle_entry_preview(&lifecycle_entry_append_dry_run),
            );
        }
    }
    let approve_execution_durable_disk_application_execution =
        if approve_execution_final_execution_durable_writeback_bundle_dry_run
            .disk_application_endpoint_helper_would_execute
        {
            let execution = contract_repair_approval_execute_durable_disk_application_handlers(
                state.contract_repair_approval_store_dir.as_ref(),
                state.graph_store_dir.as_ref(),
                &approval_record_for_response_application,
                &approve_execution_recovery_marker_write_dry_run,
                &approve_execution_recovery_marker_idempotency_precheck,
                &approve_execution_transaction_runner_dry_run,
                &record_snapshot_preview.contract_source_ref,
                &contract_source_resolution_dry_run,
                &contract_patch_plan_preview,
                &contract_patch_apply_dry_run,
                &approve_execution_runner_recovery_marker_cleanup_phase_dry_run,
                &approve_execution_runner_recovery_marker_cleanup_phase_enablement_dry_run,
                &approve_execution_runner_transaction_commit_phase_dry_run,
                &approve_execution_runner_transaction_commit_phase_enablement_dry_run,
                approve_execution_final_execution_durable_writeback_bundle_dry_run
                    .disk_application_executor_admitted,
            )
            .await;
            contract_repair_approval_durable_disk_application_execution_response(
                &execution,
                approve_execution_final_execution_durable_writeback_bundle_dry_run
                    .disk_application_endpoint_helper_execution_connected,
                approve_execution_final_execution_durable_writeback_bundle_dry_run
                    .disk_application_endpoint_helper_would_execute,
            )
        } else {
            contract_repair_approval_durable_disk_application_execution_locked_response(
                &approve_execution_final_execution_durable_writeback_bundle_dry_run,
            )
        };
    let durable_disk_application_committed =
        if approve_execution_final_execution_durable_writeback_bundle_dry_run
            .disk_application_endpoint_helper_would_execute
        {
            approve_execution_durable_disk_application_execution.transaction_committed
                && approve_execution_durable_disk_application_execution.status
                    == "approve_execution_durable_disk_application_executor_committed"
        } else {
            true
        };
    let final_response_application_success =
        preliminary_response_application_success && durable_disk_application_committed;
    let response_status =
        if routed_route_success_release_applied && durable_disk_application_committed {
            "review_approve_executed"
        } else if routed_route_success_release_applied && !durable_disk_application_committed {
            "review_approve_durable_disk_application_blocked"
        } else {
            pre_release_response_status
        };
    let route_status = if routed_route_success_release_applied && durable_disk_application_committed
    {
        "review_approve_executed"
    } else if routed_route_success_release_applied && !durable_disk_application_committed {
        "review_approve_durable_disk_application_blocked"
    } else {
        pre_release_route_status
    };
    let blocked_reasons =
        if routed_route_success_release_applied && durable_disk_application_committed {
            Vec::new()
        } else if routed_route_success_release_applied {
            let mut reasons = vec!["durable_disk_application_committed".to_string()];
            reasons.push(
                approve_execution_durable_disk_application_execution
                    .status
                    .clone(),
            );
            for blocker in &approve_execution_durable_disk_application_execution.blocked_by {
                push_unique_blocker(&mut reasons, blocker);
            }
            reasons
        } else {
            pre_release_blocked_reasons.clone()
        };
    let approve_execution_decision_lock_summary_dry_run =
        contract_repair_approval_approve_execution_decision_lock_summary_dry_run(
            &request.action,
            &approval_id,
            response_status,
            route_status,
            &blocked_reasons,
            &approve_execution_runner_route_status_readiness_dry_run,
            &approve_execution_final_execution_entry_dry_run,
            decision_execution_preflight_requested,
            review_execution_enabled,
            approve_final_execution_enabled,
            approve_runner_success,
            routed_route_success_release_applied,
            approve_execution_final_execution_durable_writeback_bundle_dry_run.would_touch_disk,
        );
    let review_record_response_application_enabled =
        review_record_execution_enabled || final_response_application_success;
    let approval_record_preview = if review_record_response_application_enabled {
        let mut previews = state.contract_repair_approval_previews.write().await;
        let preview = previews.get_mut(&scoped).ok_or_else(|| {
            json_not_found(
                "contract_repair_approval_preview_not_found",
                "CONTRACT_REPAIR_APPROVAL_PREVIEW_NOT_FOUND",
                "Contract repair approval preview does not exist in the transient review cache.",
            )
        })?;
        *preview = approval_record_for_response_application.clone();
        preview.clone()
    } else {
        approval_record_preview
    };
    if review_record_execution_enabled {
        persist_contract_repair_approval_record(
            state.contract_repair_approval_store_dir.as_ref(),
            &approval_record_preview,
        )
        .await
        .map_err(|error| internal_error(anyhow::Error::new(error)))?;
    }
    let response = ContractRepairApprovalReviewBlockedResponse {
        status: response_status.to_string(),
        approval_id,
        route_status: route_status.to_string(),
        action: request.action,
        reviewer_id: request.reviewer_id,
        review_enabled: review_record_response_application_enabled,
        persistence_enabled: review_record_execution_enabled,
        mutation_enabled: false,
        execution_gate,
        execution_plan_preview,
        approve_execution_gate,
        approve_execution_transaction_dry_run,
        approve_execution_admission_gate,
        approve_execution_transaction_runner_enablement_dry_run,
        approve_execution_transaction_runner_dry_run,
        approve_execution_recovery_marker_write_dry_run,
        approve_execution_recovery_marker_idempotency_precheck,
        approve_execution_recovery_marker_persistence_gate,
        approve_execution_recovery_marker_persistence_readiness_dry_run,
        approve_execution_transaction_commit_gate,
        approve_execution_transaction_commit_readiness_dry_run,
        approve_execution_atomic_side_effects_gate,
        approve_execution_atomic_side_effects_enablement_dry_run,
        approve_execution_atomic_side_effects_readiness_dry_run,
        approve_execution_runner_attempt_enablement_dry_run,
        approve_execution_runner_attempt,
        approve_execution_runner_attempt_readiness_dry_run,
        approve_execution_runner_execution_enablement_dry_run,
        approve_execution_runner_execution_readiness_dry_run,
        approve_execution_runner_outcome,
        approve_execution_runner_route_dispatch_enablement_dry_run,
        approve_execution_runner_dispatch_gate,
        approve_execution_runner_dispatch_readiness_dry_run,
        approve_execution_runner_handoff,
        approve_execution_runner_handoff_readiness_dry_run,
        approve_execution_runner_call_enablement_dry_run,
        approve_execution_runner_call_dry_run,
        approve_execution_runner_call_readiness_dry_run,
        approve_execution_runner_body_enablement_dry_run,
        approve_execution_runner_call_body_dry_run,
        approve_execution_runner_body_readiness_dry_run,
        approve_execution_runner_phase_execution_enablement_dry_run,
        approve_execution_runner_body_phase_sequence_dry_run,
        approve_execution_runner_phases_readiness_dry_run,
        approve_execution_runner_lifecycle_phase_enablement_dry_run,
        approve_execution_runner_lifecycle_phase_dry_run,
        approve_execution_runner_lifecycle_phase_readiness_dry_run,
        approve_execution_runner_source_mutation_phase_enablement_dry_run,
        approve_execution_runner_source_mutation_phase_dry_run,
        approve_execution_runner_source_mutation_phase_readiness_dry_run,
        approve_execution_runner_recovery_marker_cleanup_phase_enablement_dry_run,
        approve_execution_runner_recovery_marker_cleanup_phase_dry_run,
        approve_execution_runner_recovery_marker_cleanup_phase_readiness_dry_run,
        approve_execution_runner_transaction_commit_phase_enablement_dry_run,
        approve_execution_runner_transaction_commit_phase_dry_run,
        approve_execution_runner_transaction_commit_phase_readiness_dry_run,
        approve_execution_runner_rollback_execution_phase_enablement_dry_run,
        approve_execution_runner_rollback_execution_phase_dry_run,
        approve_execution_runner_rollback_execution_phase_readiness_dry_run,
        approve_execution_runner_activation_enablement_dry_run,
        approve_execution_runner_activation_enablement_readiness_dry_run,
        approve_execution_runner_activation_path_dry_run,
        approve_execution_runner_activation_path_readiness_dry_run,
        approve_execution_runner_activation_execution_plan_dry_run,
        approve_execution_runner_activation_execution_plan_readiness_dry_run,
        approve_execution_runner_activation_switch_transaction_proof_dry_run,
        approve_execution_runner_activation_switch_transaction_proof_readiness_dry_run,
        approve_execution_runner_activation_switch_write_transaction_enablement_dry_run,
        approve_execution_runner_activation_switch_write_transaction_enablement_readiness_dry_run,
        approve_execution_runner_activation_switch_write_transaction_dry_run,
        approve_execution_runner_activation_switch_write_transaction_readiness_dry_run,
        approve_execution_runner_activation_transaction_admission_gate_dry_run,
        approve_execution_runner_activation_transaction_admission_readiness_dry_run,
        approve_execution_runner_activation_admission_handoff_dry_run,
        approve_execution_runner_activation_admission_handoff_readiness_dry_run,
        approve_execution_runner_activation_handoff_enablement_dry_run,
        approve_execution_runner_activation_handoff_enablement_readiness_dry_run,
        approve_execution_runner_activation_handoff_attempt_dry_run,
        approve_execution_runner_activation_handoff_attempt_readiness_dry_run,
        approve_execution_runner_activation_post_handoff_attempt_dry_run,
        approve_execution_runner_activation_post_handoff_attempt_readiness_dry_run,
        approve_execution_runner_activation_success_admission_dry_run,
        approve_execution_runner_activation_success_admission_readiness_dry_run,
        approve_execution_runner_activation_success_return_dry_run,
        approve_execution_runner_activation_success_return_readiness_dry_run,
        approve_execution_runner_route_success_readiness_dry_run,
        approve_execution_runner_route_status_readiness_dry_run,
        approve_execution_formal_review_execution_readiness_dry_run,
        approve_execution_final_atomic_readiness_dry_run,
        approve_execution_final_atomic_execution_plan_dry_run,
        approve_execution_final_atomic_admission_gate_dry_run,
        approve_execution_final_execution_entry_dry_run,
        approve_execution_final_execution_switch_readiness_dry_run,
        approve_execution_final_execution_rollback_readiness_dry_run,
        approve_execution_final_execution_replay_plan_dry_run,
        approve_execution_final_execution_replay_executor_dry_run,
        approve_execution_final_execution_replay_executor_routing_dry_run,
        approve_execution_final_execution_routed_write_handoff_dry_run,
        approve_execution_final_execution_routed_handler_plan_dry_run,
        approve_execution_final_execution_routed_execution_attempt_dry_run,
        approve_execution_final_execution_ordered_handler_execution_confirmation_dry_run,
        approve_execution_final_execution_routed_route_success_release_dry_run,
        approve_execution_final_execution_durable_writeback_bundle_dry_run,
        approve_execution_durable_disk_application_execution,
        approve_execution_decision_lock_summary_dry_run,
        approve_execution_runner_control_readiness_dry_run,
        approve_execution_runner_enablement_plan_dry_run,
        persistence_plan_preview,
        persistence_path_preview,
        record_snapshot_preview,
        storage_readiness_gate,
        storage_dry_run_preview,
        idempotency_precheck,
        reviewer_authorization_precheck,
        review_transition_dry_run,
        record_write_dry_run,
        lifecycle_event_dry_run,
        lifecycle_entry_append_dry_run,
        lifecycle_emission_enablement_gate,
        approve_execution_lifecycle_effects_readiness_dry_run,
        approve_execution_lifecycle_event_emission_enablement_dry_run,
        approve_execution_lifecycle_entry_append_enablement_dry_run,
        contract_writeback_dry_run,
        contract_mutation_enablement_gate,
        approve_execution_contract_mutation_readiness_dry_run,
        blocked_reasons,
        approval_record_preview,
    };
    if review_record_execution_enabled || final_response_application_success {
        Ok(Json(response))
    } else {
        Err((
            StatusCode::LOCKED,
            serde_json::to_string(&response).unwrap_or_else(|_| {
                r#"{"status":"blocked","route_status":"review_workflow_disabled"}"#.to_string()
            }),
        ))
    }
}

fn validate_contract_repair_approval_request(
    request: &CreateContractRepairApprovalRequest,
) -> Result<(), (StatusCode, String)> {
    if request.status.trim() != "body_preview_only" {
        return Err(json_bad_request(
            "contract_repair_request_not_preview",
            "Contract repair approval requests must start from a body_preview_only envelope.",
        ));
    }
    if request.payload_kind.trim() != CONTRACT_REPAIR_PAYLOAD_KIND {
        return Err(json_bad_request(
            "contract_repair_payload_kind_mismatch",
            "Contract repair approval requests require v4_contract_repair_approval_request payload kind.",
        ));
    }
    if request.request_id.trim().is_empty() {
        return Err(json_bad_request(
            "contract_repair_request_id_missing",
            "Contract repair approval requests require a request id.",
        ));
    }
    if request.target_path.trim().is_empty() || request.target_kind.trim().is_empty() {
        return Err(json_bad_request(
            "contract_repair_target_missing",
            "Contract repair approval requests require target path and kind.",
        ));
    }
    if request.changed_fields.is_empty() {
        return Err(json_bad_request(
            "contract_repair_changed_fields_missing",
            "Contract repair approval requests require at least one changed field.",
        ));
    }
    if request.mutation_enabled {
        return Err(json_bad_request(
            "contract_repair_mutation_must_be_disabled",
            "Contract repair approval request previews must keep mutation disabled.",
        ));
    }
    if !request.review_required {
        return Err(json_bad_request(
            "contract_repair_review_required",
            "Contract repair approval requests require human review.",
        ));
    }
    Ok(())
}

fn validate_contract_repair_approval_review_intent(
    request: &ContractRepairApprovalReviewIntentRequest,
) -> Result<(), (StatusCode, String)> {
    let action = request.action.trim();
    if !matches!(action, "claim" | "approve" | "reject") {
        return Err(json_bad_request(
            "contract_repair_review_action_invalid",
            "Contract repair approval review intent requires claim, approve, or reject action.",
        ));
    }
    if request.reviewer_id.trim().is_empty() {
        return Err(json_bad_request(
            "contract_repair_reviewer_missing",
            "Contract repair approval review intent requires a reviewer id.",
        ));
    }
    if request.reason.trim().is_empty() {
        return Err(json_bad_request(
            "contract_repair_review_reason_missing",
            "Contract repair approval review intent requires a reason.",
        ));
    }
    Ok(())
}

fn transient_review_status_for_action(action: &str) -> &'static str {
    match action {
        "claim" => "claim_intent_recorded",
        "approve" => "approve_intent_recorded",
        "reject" => "reject_intent_recorded",
        _ => "unknown_intent_recorded",
    }
}

fn executed_review_status_for_action(action: &str) -> &'static str {
    match action {
        "claim" => "claim_executed",
        "approve" => "approve_executed",
        "reject" => "reject_executed",
        _ => "unknown_execution",
    }
}

fn reviewer_identity_format_valid(reviewer_id: &str) -> bool {
    reviewer_id
        .trim()
        .strip_prefix("user:")
        .and_then(|value| value.parse::<i64>().ok())
        .is_some()
}

fn contract_repair_review_execution_gate(
    user_id: &auth::UserId,
    action: &str,
    review_enabled: bool,
    approval_record_persistence_enabled: bool,
    reviewer_id: &str,
    storage_readiness_gate: &ContractRepairApprovalStorageReadinessGate,
    idempotency_precheck: &ContractRepairApprovalIdempotencyPrecheck,
    authorization_precheck: &ContractRepairApprovalReviewerAuthorizationPrecheck,
    lifecycle_event_emission_enabled: bool,
    contract_mutation_api_enabled: bool,
) -> ContractRepairApprovalReviewExecutionGate {
    let idempotency_precheck_passed =
        idempotency_precheck.store_lookup_enabled && !idempotency_precheck.conflict_detected;
    let reviewer_identity_present = !reviewer_id.trim().is_empty();
    let reviewer_identity_format_valid = reviewer_identity_format_valid(reviewer_id);
    let reviewer_identity_matches_auth_subject =
        reviewer_identity_format_valid && reviewer_id.trim() == format!("user:{}", user_id.0);
    let mut passed_gates = vec![
        "transient_preview_exists".to_string(),
        "review_intent_valid".to_string(),
    ];
    if reviewer_identity_present {
        passed_gates.push("reviewer_identity_present".to_string());
    }
    if reviewer_identity_format_valid {
        passed_gates.push("reviewer_identity_format_valid".to_string());
    }
    if reviewer_identity_matches_auth_subject {
        passed_gates.push("reviewer_identity_matches_auth_subject".to_string());
    }
    if storage_readiness_gate.store_ready {
        passed_gates.push("contract_repair_approval_store_ready".to_string());
    }
    if idempotency_precheck_passed {
        passed_gates.push("idempotency_precheck_passed".to_string());
    }
    if authorization_precheck.authorized {
        passed_gates.push("formal_reviewer_authorized".to_string());
    }
    let review_action_execution_requested = review_enabled && matches!(action, "claim" | "reject");
    let approve_execution_preflight_requested = review_enabled && action == "approve";
    let formal_review_flow_requested =
        review_action_execution_requested || approve_execution_preflight_requested;
    if formal_review_flow_requested {
        passed_gates.push("review_workflow_enabled".to_string());
    }
    if formal_review_flow_requested && approval_record_persistence_enabled {
        passed_gates.push("approval_persistence_enabled".to_string());
    }
    let contract_mutation_required = action == "approve";
    if contract_mutation_required && lifecycle_event_emission_enabled {
        passed_gates.push("lifecycle_event_emission_enabled".to_string());
    }
    if contract_mutation_required && contract_mutation_api_enabled {
        passed_gates.push("contract_mutation_api_enabled".to_string());
    }
    let mut blocked_gates = Vec::new();
    if !formal_review_flow_requested {
        blocked_gates.push("review_workflow_enabled".to_string());
    }
    if !formal_review_flow_requested || !approval_record_persistence_enabled {
        blocked_gates.push("approval_persistence_enabled".to_string());
    }
    if contract_mutation_required && !lifecycle_event_emission_enabled {
        blocked_gates.push("lifecycle_event_emission_enabled".to_string());
    }
    if contract_mutation_required && !contract_mutation_api_enabled {
        blocked_gates.push("contract_mutation_api_enabled".to_string());
    }
    if !authorization_precheck.authorized {
        blocked_gates.push("formal_reviewer_authorized".to_string());
    }
    if !storage_readiness_gate.store_ready {
        blocked_gates.push("contract_repair_approval_store_ready".to_string());
    }
    if !idempotency_precheck_passed {
        blocked_gates.push("idempotency_precheck_passed".to_string());
    }
    if !reviewer_identity_present {
        blocked_gates.push("reviewer_identity_present".to_string());
    }
    if !reviewer_identity_format_valid {
        blocked_gates.push("reviewer_identity_format_valid".to_string());
    }
    if !reviewer_identity_matches_auth_subject {
        blocked_gates.push("reviewer_identity_matches_auth_subject".to_string());
    }
    ContractRepairApprovalReviewExecutionGate {
        status: if blocked_gates.is_empty() {
            "ready".to_string()
        } else {
            "blocked".to_string()
        },
        required_gates: vec![
            "transient_preview_exists".to_string(),
            "review_intent_valid".to_string(),
            "reviewer_identity_present".to_string(),
            "reviewer_identity_format_valid".to_string(),
            "reviewer_identity_matches_auth_subject".to_string(),
            "contract_repair_approval_store_ready".to_string(),
            "idempotency_precheck_passed".to_string(),
            "review_workflow_enabled".to_string(),
            "approval_persistence_enabled".to_string(),
            "lifecycle_event_emission_enabled".to_string(),
            "contract_mutation_api_enabled".to_string(),
            "formal_reviewer_authorized".to_string(),
        ],
        passed_gates,
        blocked_gates,
    }
}

fn contract_repair_review_execution_plan_preview(
    action: &str,
    gate: &ContractRepairApprovalReviewExecutionGate,
    execution_enabled: bool,
    lifecycle_event_emission_enabled: bool,
) -> ContractRepairApprovalReviewExecutionPlanPreview {
    ContractRepairApprovalReviewExecutionPlanPreview {
        status: if execution_enabled {
            "execution_plan_ready".to_string()
        } else {
            "execution_plan_preview_only".to_string()
        },
        execution_enabled,
        action: action.to_string(),
        target_review_state: target_review_state_for_action(action),
        would_persist_approval_record: execution_enabled,
        would_mutate_contract: false,
        would_emit_lifecycle_event: lifecycle_event_emission_enabled,
        side_effects: vec![
            "transition_review_state".to_string(),
            "persist_approval_record".to_string(),
            "emit_review_lifecycle_event".to_string(),
            "apply_contract_repair_if_approved".to_string(),
        ],
        blocked_by: gate.blocked_gates.clone(),
    }
}

fn contract_repair_approval_approve_execution_gate(
    action: &str,
    approval_persistence_ready: bool,
    transition_dry_run: &ContractRepairApprovalReviewTransitionDryRun,
    lifecycle_event_dry_run: &ContractRepairApprovalLifecycleEventDryRun,
    lifecycle_entry_append_dry_run: &ContractRepairApprovalLifecycleEntryAppendDryRun,
    contract_writeback_dry_run: &ContractRepairApprovalContractWritebackDryRun,
    contract_mutation_gate: &ContractRepairApprovalContractMutationEnablementGate,
) -> ContractRepairApprovalApproveExecutionGate {
    let approve_action = action == "approve";
    let transition_ready = transition_dry_run.transition_ready
        && transition_dry_run.target_review_state == RuntimeApprovalReviewState::Approved;
    let lifecycle_emission_ready = lifecycle_event_dry_run.emission_ready;
    let lifecycle_append_ready = lifecycle_entry_append_dry_run.append_ready;
    let contract_writeback_ready = contract_writeback_dry_run.writeback_ready;
    let mutation_api_enabled = contract_mutation_gate.mutation_ready;
    let approve_preconditions_ready = approve_action
        && approval_persistence_ready
        && transition_ready
        && lifecycle_emission_ready
        && lifecycle_append_ready
        && contract_writeback_ready;
    let approve_execution_ready = approve_preconditions_ready && mutation_api_enabled;
    let mut blocked_by = Vec::new();
    if !approve_action {
        blocked_by.push("approve_action_required".to_string());
    }
    if !approval_persistence_ready {
        blocked_by.push("approval_persistence_enabled".to_string());
    }
    if !transition_ready {
        blocked_by.push("approved_review_transition_ready".to_string());
    }
    if !lifecycle_emission_ready {
        blocked_by.push("lifecycle_event_emission_ready".to_string());
    }
    if !lifecycle_event_dry_run.would_emit {
        blocked_by.push("lifecycle_event_emission_enabled".to_string());
    }
    if !lifecycle_append_ready {
        blocked_by.push("lifecycle_entry_append_ready".to_string());
    }
    if !contract_writeback_ready {
        blocked_by.push("contract_writeback_ready".to_string());
    }
    if !mutation_api_enabled {
        blocked_by.push("contract_mutation_api_enabled".to_string());
    }

    ContractRepairApprovalApproveExecutionGate {
        status: if approve_execution_ready {
            "approve_execution_ready".to_string()
        } else if approve_preconditions_ready {
            "approve_execution_ready_blocked".to_string()
        } else {
            "approve_execution_blocked".to_string()
        },
        action: action.to_string(),
        target_review_state: RuntimeApprovalReviewState::Approved,
        approval_persistence_ready,
        transition_ready,
        lifecycle_emission_ready,
        lifecycle_append_ready,
        contract_writeback_ready,
        mutation_api_enabled,
        approve_execution_ready,
        would_execute: false,
        blocked_by,
    }
}

fn contract_repair_approval_approve_execution_transaction_dry_run(
    action: &str,
    approval_id: &str,
    approve_execution_gate: &ContractRepairApprovalApproveExecutionGate,
    transition_dry_run: &ContractRepairApprovalReviewTransitionDryRun,
    record_write_dry_run: &ContractRepairApprovalRecordWriteDryRun,
    lifecycle_event_dry_run: &ContractRepairApprovalLifecycleEventDryRun,
    lifecycle_entry_append_dry_run: &ContractRepairApprovalLifecycleEntryAppendDryRun,
    contract_writeback_dry_run: &ContractRepairApprovalContractWritebackDryRun,
) -> ContractRepairApprovalApproveExecutionTransactionDryRun {
    let approve_action = action == "approve";
    let approved_transition_ready = transition_dry_run.transition_ready
        && transition_dry_run.target_review_state == RuntimeApprovalReviewState::Approved;
    let record_write_ready = record_write_dry_run.write_ready;
    let lifecycle_emission_ready = lifecycle_event_dry_run.emission_ready;
    let lifecycle_append_ready = lifecycle_entry_append_dry_run.append_ready;
    let contract_writeback_ready = contract_writeback_dry_run.writeback_ready;
    let lifecycle_emission_enabled = lifecycle_event_dry_run.would_emit;
    let mutation_api_enabled = approve_execution_gate.mutation_api_enabled;
    let transaction_plan_ready = approve_action
        && approved_transition_ready
        && record_write_ready
        && lifecycle_emission_ready
        && lifecycle_append_ready
        && contract_writeback_ready;
    let execution_ready =
        transaction_plan_ready && lifecycle_emission_enabled && mutation_api_enabled;
    let mut blocked_by = Vec::new();
    if !approve_action {
        blocked_by.push("approve_action_required".to_string());
    }
    if !approved_transition_ready {
        blocked_by.push("approved_review_transition_ready".to_string());
    }
    if !record_write_ready {
        blocked_by.push("approval_record_write_ready".to_string());
    }
    if !lifecycle_emission_ready {
        blocked_by.push("lifecycle_event_emission_ready".to_string());
    }
    if !lifecycle_append_ready {
        blocked_by.push("lifecycle_entry_append_ready".to_string());
    }
    if !contract_writeback_ready {
        blocked_by.push("contract_writeback_ready".to_string());
    }
    if !lifecycle_emission_enabled {
        blocked_by.push("lifecycle_event_emission_enabled".to_string());
    }
    if !mutation_api_enabled {
        blocked_by.push("contract_mutation_api_enabled".to_string());
    }

    ContractRepairApprovalApproveExecutionTransactionDryRun {
        status: if execution_ready {
            "approve_execution_transaction_ready".to_string()
        } else if transaction_plan_ready {
            "approve_execution_transaction_ready_blocked".to_string()
        } else {
            "approve_execution_transaction_blocked".to_string()
        },
        action: action.to_string(),
        approval_id: approval_id.to_string(),
        transaction_plan_ready,
        execution_ready,
        approved_transition_ready,
        record_write_ready,
        lifecycle_emission_ready,
        lifecycle_append_ready,
        contract_writeback_ready,
        lifecycle_emission_enabled,
        mutation_api_enabled,
        would_transition_review: transition_dry_run.would_transition,
        would_write_approval_record: record_write_dry_run.would_write,
        would_emit_lifecycle_event: lifecycle_event_dry_run.would_emit,
        would_append_lifecycle_entry: lifecycle_entry_append_dry_run.would_append,
        would_write_contract_source: contract_writeback_dry_run
            .source_write_dry_run
            .would_write_source,
        would_execute_transaction: false,
        step_order: vec![
            "transition_review_state".to_string(),
            "persist_approval_record".to_string(),
            "emit_lifecycle_event".to_string(),
            "append_lifecycle_entry".to_string(),
            "write_contract_source".to_string(),
        ],
        atomicity_scope: vec![
            "approval_record".to_string(),
            "lifecycle_entry".to_string(),
            "contract_source".to_string(),
        ],
        blocked_by,
    }
}

fn contract_repair_approval_approve_execution_admission_gate(
    action: &str,
    approval_id: &str,
    transaction_dry_run: &ContractRepairApprovalApproveExecutionTransactionDryRun,
) -> ContractRepairApprovalApproveExecutionAdmissionGate {
    let approve_action = action == "approve";
    let required_atomicity_scope = ["approval_record", "lifecycle_entry", "contract_source"];
    let atomicity_scope_ready = required_atomicity_scope.iter().all(|required| {
        transaction_dry_run
            .atomicity_scope
            .iter()
            .any(|scope| scope == required)
    });
    let transaction_runner_enabled = false;
    let transaction_execution_ready = transaction_dry_run.execution_ready;
    let admission_ready = approve_action
        && transaction_execution_ready
        && atomicity_scope_ready
        && transaction_runner_enabled;
    let mut passed_gates = Vec::new();
    if approve_action {
        passed_gates.push("approve_action_required".to_string());
    }
    if transaction_dry_run.transaction_plan_ready {
        passed_gates.push("transaction_plan_ready".to_string());
    }
    if atomicity_scope_ready {
        passed_gates.push("atomicity_scope_ready".to_string());
    }
    if transaction_dry_run.lifecycle_emission_enabled {
        passed_gates.push("lifecycle_event_emission_enabled".to_string());
    }
    if transaction_dry_run.mutation_api_enabled {
        passed_gates.push("contract_mutation_api_enabled".to_string());
    }
    if transaction_runner_enabled {
        passed_gates.push("approve_execution_transaction_runner_enabled".to_string());
    }
    let mut blocked_gates = Vec::new();
    if !approve_action {
        blocked_gates.push("approve_action_required".to_string());
    }
    if !transaction_dry_run.transaction_plan_ready {
        blocked_gates.push("transaction_plan_ready".to_string());
    }
    if !atomicity_scope_ready {
        blocked_gates.push("atomicity_scope_ready".to_string());
    }
    if !transaction_dry_run.lifecycle_emission_enabled {
        blocked_gates.push("lifecycle_event_emission_enabled".to_string());
    }
    if !transaction_dry_run.mutation_api_enabled {
        blocked_gates.push("contract_mutation_api_enabled".to_string());
    }
    if !transaction_runner_enabled {
        blocked_gates.push("approve_execution_transaction_runner_enabled".to_string());
    }

    ContractRepairApprovalApproveExecutionAdmissionGate {
        status: if admission_ready {
            "approve_execution_admission_ready".to_string()
        } else if approve_action
            && transaction_dry_run.transaction_plan_ready
            && atomicity_scope_ready
        {
            "approve_execution_admission_ready_blocked".to_string()
        } else {
            "approve_execution_admission_blocked".to_string()
        },
        action: action.to_string(),
        approval_id: approval_id.to_string(),
        transaction_plan_ready: transaction_dry_run.transaction_plan_ready,
        transaction_execution_ready,
        atomicity_scope_ready,
        transaction_runner_enabled,
        admission_ready,
        partial_execution_allowed: false,
        would_start_transaction: false,
        would_persist_any_side_effect: false,
        required_gates: vec![
            "approve_action_required".to_string(),
            "transaction_plan_ready".to_string(),
            "atomicity_scope_ready".to_string(),
            "lifecycle_event_emission_enabled".to_string(),
            "contract_mutation_api_enabled".to_string(),
            "approve_execution_transaction_runner_enabled".to_string(),
        ],
        passed_gates,
        blocked_gates,
    }
}

fn contract_repair_approval_approve_execution_admission_gate_with_gate(
    admission_gate: &ContractRepairApprovalApproveExecutionAdmissionGate,
    transaction_runner_enabled: bool,
) -> ContractRepairApprovalApproveExecutionAdmissionGate {
    let mut result = admission_gate.clone();
    if !transaction_runner_enabled {
        return result;
    }

    result.transaction_runner_enabled = true;
    result
        .blocked_gates
        .retain(|gate| gate != "approve_execution_transaction_runner_enabled");
    push_unique_blocker(
        &mut result.passed_gates,
        "approve_execution_transaction_runner_enabled",
    );

    let gate_states = [
        (
            "approve_action_required",
            admission_gate.action == "approve",
        ),
        (
            "transaction_plan_ready",
            admission_gate.transaction_plan_ready,
        ),
        (
            "transaction_execution_ready",
            admission_gate.transaction_execution_ready,
        ),
        (
            "atomicity_scope_ready",
            admission_gate.atomicity_scope_ready,
        ),
    ];
    for (gate, passed) in gate_states {
        if !passed {
            push_unique_blocker(&mut result.blocked_gates, gate);
        }
    }
    if !result.blocked_gates.is_empty() {
        result.admission_ready = false;
        result.status = "approve_execution_admission_blocked".to_string();
        return result;
    }

    result.status = "approve_execution_admission_ready".to_string();
    result.admission_ready = true;
    result.partial_execution_allowed = false;
    result.would_start_transaction = true;
    result.would_persist_any_side_effect = false;
    result
}

fn contract_repair_approval_approve_execution_transaction_runner_enablement_dry_run(
    action: &str,
    approval_id: &str,
    transaction_dry_run: &ContractRepairApprovalApproveExecutionTransactionDryRun,
    admission_gate: &ContractRepairApprovalApproveExecutionAdmissionGate,
) -> ContractRepairApprovalApproveExecutionTransactionRunnerEnablementDryRun {
    let approve_action = action == "approve";
    let transaction_plan_ready = transaction_dry_run.transaction_plan_ready;
    let transaction_execution_ready = transaction_dry_run.execution_ready;
    let atomicity_scope_ready = admission_gate.atomicity_scope_ready;
    let lifecycle_emission_enabled = transaction_dry_run.lifecycle_emission_enabled;
    let contract_mutation_api_enabled = transaction_dry_run.mutation_api_enabled;
    let transaction_runner_enabled = admission_gate.transaction_runner_enabled;
    let enablement_prerequisites_ready = approve_action
        && transaction_plan_ready
        && transaction_execution_ready
        && atomicity_scope_ready
        && lifecycle_emission_enabled
        && contract_mutation_api_enabled;
    let runner_enablement_ready = enablement_prerequisites_ready && transaction_runner_enabled;
    let required_gates = vec![
        "approve_action_required".to_string(),
        "transaction_plan_ready".to_string(),
        "transaction_execution_ready".to_string(),
        "atomicity_scope_ready".to_string(),
        "lifecycle_event_emission_enabled".to_string(),
        "contract_mutation_api_enabled".to_string(),
        "approve_execution_transaction_runner_enabled".to_string(),
    ];
    let gate_states = [
        ("approve_action_required", approve_action),
        ("transaction_plan_ready", transaction_plan_ready),
        ("transaction_execution_ready", transaction_execution_ready),
        ("atomicity_scope_ready", atomicity_scope_ready),
        (
            "lifecycle_event_emission_enabled",
            lifecycle_emission_enabled,
        ),
        (
            "contract_mutation_api_enabled",
            contract_mutation_api_enabled,
        ),
        (
            "approve_execution_transaction_runner_enabled",
            transaction_runner_enabled,
        ),
    ];
    let mut passed_gates = Vec::new();
    let mut blocked_gates = Vec::new();
    for (gate, passed) in gate_states {
        if passed {
            passed_gates.push(gate.to_string());
        } else {
            blocked_gates.push(gate.to_string());
        }
    }

    ContractRepairApprovalApproveExecutionTransactionRunnerEnablementDryRun {
        status: if runner_enablement_ready {
            "approve_execution_transaction_runner_enablement_ready".to_string()
        } else if approve_action && transaction_plan_ready && atomicity_scope_ready {
            "approve_execution_transaction_runner_enablement_ready_blocked".to_string()
        } else {
            "approve_execution_transaction_runner_enablement_blocked".to_string()
        },
        action: action.to_string(),
        approval_id: approval_id.to_string(),
        switch_name: "approve_execution_transaction_runner_enabled".to_string(),
        approve_action,
        transaction_plan_ready,
        transaction_execution_ready,
        atomicity_scope_ready,
        lifecycle_emission_enabled,
        contract_mutation_api_enabled,
        transaction_runner_enabled,
        enablement_prerequisites_ready,
        runner_enablement_ready,
        would_enable_runner: false,
        would_start_runner: false,
        would_touch_disk: false,
        required_gates,
        passed_gates,
        blocked_gates,
    }
}

fn contract_repair_approval_approve_execution_transaction_runner_dry_run(
    action: &str,
    approval_id: &str,
    transaction_dry_run: &ContractRepairApprovalApproveExecutionTransactionDryRun,
    admission_gate: &ContractRepairApprovalApproveExecutionAdmissionGate,
) -> ContractRepairApprovalApproveExecutionTransactionRunnerDryRun {
    let phase_order = vec![
        "write_recovery_marker".to_string(),
        "transition_review_state".to_string(),
        "persist_approval_record".to_string(),
        "emit_lifecycle_event".to_string(),
        "append_lifecycle_entry".to_string(),
        "write_contract_source".to_string(),
        "clear_recovery_marker".to_string(),
    ];
    let rollback_order = vec![
        "restore_contract_source".to_string(),
        "restore_approval_record".to_string(),
        "mark_recovery_marker_rolled_back".to_string(),
    ];
    let runner_enabled = admission_gate.transaction_runner_enabled;
    let commit_barrier_ready = transaction_dry_run.transaction_plan_ready
        && admission_gate.atomicity_scope_ready
        && !transaction_dry_run.step_order.is_empty();
    let recovery_marker_ready =
        !approval_id.trim().is_empty() && admission_gate.atomicity_scope_ready;
    let rollback_plan_ready = !rollback_order.is_empty()
        && rollback_order
            .iter()
            .any(|phase| phase == "restore_contract_source")
        && rollback_order
            .iter()
            .any(|phase| phase == "restore_approval_record");
    let commit_ready = admission_gate.admission_ready
        && runner_enabled
        && commit_barrier_ready
        && recovery_marker_ready
        && rollback_plan_ready;
    let mut blocked_by = Vec::new();
    if action != "approve" {
        blocked_by.push("approve_action_required".to_string());
    }
    if !transaction_dry_run.transaction_plan_ready {
        blocked_by.push("transaction_plan_ready".to_string());
    }
    if !admission_gate.admission_ready {
        blocked_by.push("approve_execution_admission_ready".to_string());
    }
    if !commit_barrier_ready {
        blocked_by.push("commit_barrier_ready".to_string());
    }
    if !recovery_marker_ready {
        blocked_by.push("recovery_marker_ready".to_string());
    }
    if !rollback_plan_ready {
        blocked_by.push("rollback_plan_ready".to_string());
    }
    for gate in &admission_gate.blocked_gates {
        if !blocked_by.contains(gate) {
            blocked_by.push(gate.clone());
        }
    }
    if !runner_enabled
        && !blocked_by
            .iter()
            .any(|gate| gate == "approve_execution_transaction_runner_enabled")
    {
        blocked_by.push("approve_execution_transaction_runner_enabled".to_string());
    }

    ContractRepairApprovalApproveExecutionTransactionRunnerDryRun {
        status: if commit_ready {
            "approve_execution_transaction_runner_ready".to_string()
        } else if transaction_dry_run.transaction_plan_ready
            && commit_barrier_ready
            && recovery_marker_ready
            && rollback_plan_ready
        {
            "approve_execution_transaction_runner_ready_blocked".to_string()
        } else {
            "approve_execution_transaction_runner_blocked".to_string()
        },
        action: action.to_string(),
        approval_id: approval_id.to_string(),
        runner_enabled,
        admission_ready: admission_gate.admission_ready,
        transaction_plan_ready: transaction_dry_run.transaction_plan_ready,
        commit_barrier_ready,
        recovery_marker_ready,
        rollback_plan_ready,
        commit_ready,
        would_start_runner: false,
        would_write_recovery_marker: false,
        would_commit_transaction: false,
        would_rollback_on_error: false,
        phase_order,
        rollback_order,
        blocked_by,
    }
}

fn contract_repair_approval_approve_execution_recovery_marker_write_dry_run(
    approval_id: &str,
    storage_readiness_gate: &ContractRepairApprovalStorageReadinessGate,
    runner_dry_run: &ContractRepairApprovalApproveExecutionTransactionRunnerDryRun,
) -> ContractRepairApprovalApproveExecutionRecoveryMarkerWriteDryRun {
    let marker_kind = "approve_execution_recovery_marker".to_string();
    let store_kind = CONTRACT_REPAIR_APPROVAL_STORE_KIND.to_string();
    let marker_key = format!("{approval_id}:approve-execution-recovery-marker");
    let path_segment = sanitize_storage_path_segment(&marker_key);
    let file_name = format!("{path_segment}.json");
    let payload_fields = vec![
        "approval_id".to_string(),
        "marker_kind".to_string(),
        "phase_order".to_string(),
        "rollback_order".to_string(),
        "commit_barrier_ready".to_string(),
        "recovery_marker_ready".to_string(),
    ];
    let payload_ready = !approval_id.trim().is_empty()
        && !runner_dry_run.phase_order.is_empty()
        && !runner_dry_run.rollback_order.is_empty()
        && runner_dry_run.commit_barrier_ready
        && runner_dry_run.recovery_marker_ready;
    let path_ready = !path_segment.trim().is_empty() && !file_name.trim().is_empty();
    let storage_ready = storage_readiness_gate.store_ready;
    let runner_ready = runner_dry_run.runner_enabled && runner_dry_run.admission_ready;
    let write_ready = runner_dry_run.action == "approve"
        && runner_dry_run.transaction_plan_ready
        && payload_ready
        && path_ready
        && storage_ready;
    let mut blocked_by = Vec::new();
    if runner_dry_run.action != "approve" {
        blocked_by.push("approve_action_required".to_string());
    }
    if !runner_dry_run.transaction_plan_ready {
        blocked_by.push("transaction_plan_ready".to_string());
    }
    if !payload_ready {
        blocked_by.push("recovery_marker_payload_ready".to_string());
    }
    if !path_ready {
        blocked_by.push("recovery_marker_path_ready".to_string());
    }
    if !storage_ready {
        blocked_by.push("contract_repair_approval_store_ready".to_string());
    }
    if !runner_ready {
        blocked_by.push("approve_execution_transaction_runner_ready".to_string());
    }
    for gate in &runner_dry_run.blocked_by {
        if !blocked_by.contains(gate) {
            blocked_by.push(gate.clone());
        }
    }

    ContractRepairApprovalApproveExecutionRecoveryMarkerWriteDryRun {
        status: if write_ready && runner_ready {
            "approve_execution_recovery_marker_write_ready".to_string()
        } else if write_ready {
            "approve_execution_recovery_marker_write_ready_blocked".to_string()
        } else {
            "approve_execution_recovery_marker_write_blocked".to_string()
        },
        approval_id: approval_id.to_string(),
        marker_kind,
        store_kind,
        marker_key,
        path_segment,
        file_name,
        payload_ready,
        path_ready,
        storage_ready,
        runner_ready,
        write_ready,
        atomic_write_required: true,
        would_write_marker: false,
        would_touch_disk: false,
        payload_fields,
        blocked_by,
    }
}

async fn contract_repair_approval_approve_execution_recovery_marker_idempotency_precheck(
    store_dir: &FsPath,
    marker_write_dry_run: &ContractRepairApprovalApproveExecutionRecoveryMarkerWriteDryRun,
) -> ContractRepairApprovalApproveExecutionRecoveryMarkerIdempotencyPrecheck {
    let store_lookup_enabled =
        marker_write_dry_run.storage_ready && !marker_write_dry_run.file_name.trim().is_empty();
    let marker_path = store_dir.join(&marker_write_dry_run.file_name);
    let existing_marker_found = if store_lookup_enabled {
        fs::metadata(&marker_path).await.is_ok()
    } else {
        false
    };
    let conflict_detected = existing_marker_found;
    let marker_write_ready = marker_write_dry_run.write_ready;
    let safe_to_write_marker = marker_write_ready
        && store_lookup_enabled
        && !conflict_detected
        && marker_write_dry_run.runner_ready;
    let mut blocked_by = Vec::new();
    if !marker_write_ready {
        blocked_by.push("recovery_marker_write_ready".to_string());
    }
    if !store_lookup_enabled {
        blocked_by.push("contract_repair_approval_store_ready".to_string());
    }
    if conflict_detected {
        blocked_by.push("recovery_marker_idempotency_conflict".to_string());
    }
    if !marker_write_dry_run.runner_ready {
        blocked_by.push("approve_execution_transaction_runner_ready".to_string());
    }

    ContractRepairApprovalApproveExecutionRecoveryMarkerIdempotencyPrecheck {
        status: if safe_to_write_marker {
            "approve_execution_recovery_marker_idempotency_safe".to_string()
        } else if store_lookup_enabled && marker_write_ready && !conflict_detected {
            "approve_execution_recovery_marker_idempotency_checked_blocked".to_string()
        } else {
            "approve_execution_recovery_marker_idempotency_blocked".to_string()
        },
        marker_key: marker_write_dry_run.marker_key.clone(),
        file_name: marker_write_dry_run.file_name.clone(),
        store_lookup_enabled,
        existing_marker_found,
        conflict_detected,
        marker_write_ready,
        safe_to_write_marker,
        would_write_marker: false,
        blocked_by,
    }
}

async fn contract_repair_approval_approve_execution_recovery_marker_write_with_gate(
    store_dir: &FsPath,
    marker_write_dry_run: &ContractRepairApprovalApproveExecutionRecoveryMarkerWriteDryRun,
    idempotency_precheck: &ContractRepairApprovalApproveExecutionRecoveryMarkerIdempotencyPrecheck,
    runner_dry_run: &ContractRepairApprovalApproveExecutionTransactionRunnerDryRun,
    marker_persistence_enabled: bool,
) -> ContractRepairApprovalApproveExecutionRecoveryMarkerWriteDryRun {
    let mut result = marker_write_dry_run.clone();
    if !marker_persistence_enabled {
        return result;
    }
    if !idempotency_precheck.safe_to_write_marker {
        push_unique_blocker(&mut result.blocked_by, "recovery_marker_safe_to_write");
        return result;
    }
    if !runner_dry_run.runner_enabled || !runner_dry_run.admission_ready {
        push_unique_blocker(
            &mut result.blocked_by,
            "approve_execution_transaction_runner_ready",
        );
        return result;
    }
    if !marker_write_dry_run.write_ready || !marker_write_dry_run.runner_ready {
        push_unique_blocker(&mut result.blocked_by, "recovery_marker_write_ready");
        return result;
    }

    let marker_path = store_dir.join(&marker_write_dry_run.file_name);
    let marker_payload = json!({
        "schema_version": "quantpilot/contract-repair-approve-execution-recovery-marker/v1",
        "approval_id": marker_write_dry_run.approval_id,
        "marker_kind": marker_write_dry_run.marker_kind,
        "marker_key": marker_write_dry_run.marker_key,
        "phase_order": runner_dry_run.phase_order,
        "rollback_order": runner_dry_run.rollback_order,
        "commit_barrier_ready": runner_dry_run.commit_barrier_ready,
        "recovery_marker_ready": runner_dry_run.recovery_marker_ready
    });
    match crate::runtime_persistence::atomic_write_json(&marker_path, &marker_payload).await {
        Ok(()) => {
            result.status = "approve_execution_recovery_marker_written".to_string();
            result.would_write_marker = true;
            result.would_touch_disk = true;
            result.blocked_by.clear();
        }
        Err(_) => push_unique_blocker(&mut result.blocked_by, "recovery_marker_atomic_write"),
    }
    result
}

fn contract_repair_approval_approve_execution_recovery_marker_persistence_gate(
    marker_write_dry_run: &ContractRepairApprovalApproveExecutionRecoveryMarkerWriteDryRun,
    idempotency_precheck: &ContractRepairApprovalApproveExecutionRecoveryMarkerIdempotencyPrecheck,
    marker_persistence_enabled: bool,
) -> ContractRepairApprovalApproveExecutionRecoveryMarkerPersistenceGate {
    let marker_write_ready = marker_write_dry_run.write_ready;
    let idempotency_checked =
        idempotency_precheck.store_lookup_enabled && idempotency_precheck.marker_write_ready;
    let no_existing_marker_conflict = !idempotency_precheck.conflict_detected;
    let runner_ready = marker_write_dry_run.runner_ready;
    let marker_persistence_plan_ready =
        marker_write_ready && idempotency_checked && no_existing_marker_conflict;
    let persistence_ready =
        marker_persistence_plan_ready && runner_ready && marker_persistence_enabled;
    let mut passed_gates = Vec::new();
    if marker_write_ready {
        passed_gates.push("recovery_marker_write_ready".to_string());
    }
    if idempotency_checked {
        passed_gates.push("recovery_marker_idempotency_checked".to_string());
    }
    if no_existing_marker_conflict {
        passed_gates.push("recovery_marker_no_existing_conflict".to_string());
    }
    if runner_ready {
        passed_gates.push("approve_execution_transaction_runner_ready".to_string());
    }
    if marker_persistence_enabled {
        passed_gates.push("approve_recovery_marker_persistence_enabled".to_string());
    }
    let mut blocked_gates = Vec::new();
    if !marker_write_ready {
        blocked_gates.push("recovery_marker_write_ready".to_string());
    }
    if !idempotency_checked {
        blocked_gates.push("recovery_marker_idempotency_checked".to_string());
    }
    if !no_existing_marker_conflict {
        blocked_gates.push("recovery_marker_no_existing_conflict".to_string());
    }
    if !runner_ready {
        blocked_gates.push("approve_execution_transaction_runner_ready".to_string());
    }
    if !marker_persistence_enabled {
        blocked_gates.push("approve_recovery_marker_persistence_enabled".to_string());
    }

    ContractRepairApprovalApproveExecutionRecoveryMarkerPersistenceGate {
        status: if persistence_ready {
            "approve_execution_recovery_marker_persistence_ready".to_string()
        } else if marker_persistence_plan_ready {
            "approve_execution_recovery_marker_persistence_ready_blocked".to_string()
        } else {
            "approve_execution_recovery_marker_persistence_blocked".to_string()
        },
        marker_key: marker_write_dry_run.marker_key.clone(),
        file_name: marker_write_dry_run.file_name.clone(),
        marker_persistence_plan_ready,
        marker_write_ready,
        idempotency_checked,
        no_existing_marker_conflict,
        runner_ready,
        marker_persistence_enabled,
        persistence_ready,
        would_persist_marker: false,
        would_touch_disk: false,
        required_gates: vec![
            "recovery_marker_write_ready".to_string(),
            "recovery_marker_idempotency_checked".to_string(),
            "recovery_marker_no_existing_conflict".to_string(),
            "approve_execution_transaction_runner_ready".to_string(),
            "approve_recovery_marker_persistence_enabled".to_string(),
        ],
        passed_gates,
        blocked_gates,
    }
}

fn contract_repair_approval_approve_execution_recovery_marker_persistence_readiness_dry_run(
    action: &str,
    approval_id: &str,
    marker_persistence_gate: &ContractRepairApprovalApproveExecutionRecoveryMarkerPersistenceGate,
) -> ContractRepairApprovalApproveExecutionRecoveryMarkerPersistenceReadinessDryRun {
    let marker_persistence_plan_ready = marker_persistence_gate.marker_persistence_plan_ready;
    let marker_write_ready = marker_persistence_gate.marker_write_ready;
    let idempotency_checked = marker_persistence_gate.idempotency_checked;
    let no_existing_marker_conflict = marker_persistence_gate.no_existing_marker_conflict;
    let runner_ready = marker_persistence_gate.runner_ready;
    let marker_persistence_enabled = marker_persistence_gate.marker_persistence_enabled;
    let recovery_marker_persistence_ready = marker_persistence_gate.persistence_ready;
    let required_gates = vec![
        "recovery_marker_write_ready".to_string(),
        "recovery_marker_idempotency_checked".to_string(),
        "recovery_marker_no_existing_conflict".to_string(),
        "approve_execution_transaction_runner_ready".to_string(),
        "approve_recovery_marker_persistence_enabled".to_string(),
    ];
    let gate_states = [
        ("recovery_marker_write_ready", marker_write_ready),
        ("recovery_marker_idempotency_checked", idempotency_checked),
        (
            "recovery_marker_no_existing_conflict",
            no_existing_marker_conflict,
        ),
        ("approve_execution_transaction_runner_ready", runner_ready),
        (
            "approve_recovery_marker_persistence_enabled",
            marker_persistence_enabled,
        ),
    ];
    let mut passed_gates = Vec::new();
    let mut blocked_gates = Vec::new();
    for (gate, passed) in gate_states {
        if passed {
            passed_gates.push(gate.to_string());
        } else {
            blocked_gates.push(gate.to_string());
        }
    }

    ContractRepairApprovalApproveExecutionRecoveryMarkerPersistenceReadinessDryRun {
        status: if recovery_marker_persistence_ready {
            "approve_execution_recovery_marker_persistence_readiness_ready".to_string()
        } else if action == "approve" && marker_persistence_plan_ready {
            "approve_execution_recovery_marker_persistence_readiness_ready_blocked".to_string()
        } else {
            "approve_execution_recovery_marker_persistence_readiness_blocked".to_string()
        },
        action: action.to_string(),
        approval_id: approval_id.to_string(),
        gate_name: "recovery_marker_persistence_ready".to_string(),
        marker_key: marker_persistence_gate.marker_key.clone(),
        file_name: marker_persistence_gate.file_name.clone(),
        marker_persistence_plan_ready,
        marker_write_ready,
        idempotency_checked,
        no_existing_marker_conflict,
        runner_ready,
        marker_persistence_enabled,
        recovery_marker_persistence_ready,
        would_persist_marker: false,
        would_touch_disk: false,
        would_unblock_transaction_commit: false,
        would_unblock_atomic_side_effects: false,
        inherited_recovery_marker_persistence_blocked_gates: marker_persistence_gate
            .blocked_gates
            .clone(),
        required_gates,
        passed_gates,
        blocked_gates,
    }
}

fn contract_repair_approval_approve_execution_transaction_commit_gate(
    runner_dry_run: &ContractRepairApprovalApproveExecutionTransactionRunnerDryRun,
    marker_persistence_gate: &ContractRepairApprovalApproveExecutionRecoveryMarkerPersistenceGate,
) -> ContractRepairApprovalApproveExecutionTransactionCommitGate {
    let runner_plan_ready = runner_dry_run.transaction_plan_ready
        && runner_dry_run.commit_barrier_ready
        && runner_dry_run.rollback_plan_ready
        && marker_persistence_gate.marker_persistence_plan_ready;
    let commit_gate_enabled = false;
    let commit_ready = runner_plan_ready
        && runner_dry_run.runner_enabled
        && runner_dry_run.admission_ready
        && marker_persistence_gate.persistence_ready
        && commit_gate_enabled;
    let mut passed_gates = Vec::new();
    if runner_dry_run.runner_enabled {
        passed_gates.push("approve_execution_transaction_runner_enabled".to_string());
    }
    if runner_dry_run.admission_ready {
        passed_gates.push("approve_execution_admission_ready".to_string());
    }
    if runner_dry_run.commit_barrier_ready {
        passed_gates.push("commit_barrier_ready".to_string());
    }
    if runner_dry_run.rollback_plan_ready {
        passed_gates.push("rollback_plan_ready".to_string());
    }
    if marker_persistence_gate.marker_persistence_plan_ready {
        passed_gates.push("recovery_marker_persistence_plan_ready".to_string());
    }
    if marker_persistence_gate.persistence_ready {
        passed_gates.push("recovery_marker_persistence_ready".to_string());
    }
    if commit_gate_enabled {
        passed_gates.push("approve_execution_transaction_commit_enabled".to_string());
    }
    let mut blocked_gates = Vec::new();
    if !runner_dry_run.runner_enabled {
        blocked_gates.push("approve_execution_transaction_runner_enabled".to_string());
    }
    if !runner_dry_run.admission_ready {
        blocked_gates.push("approve_execution_admission_ready".to_string());
    }
    if !runner_dry_run.commit_barrier_ready {
        blocked_gates.push("commit_barrier_ready".to_string());
    }
    if !runner_dry_run.rollback_plan_ready {
        blocked_gates.push("rollback_plan_ready".to_string());
    }
    if !marker_persistence_gate.marker_persistence_plan_ready {
        blocked_gates.push("recovery_marker_persistence_plan_ready".to_string());
    }
    if !marker_persistence_gate.persistence_ready {
        blocked_gates.push("recovery_marker_persistence_ready".to_string());
    }
    if !commit_gate_enabled {
        blocked_gates.push("approve_execution_transaction_commit_enabled".to_string());
    }

    ContractRepairApprovalApproveExecutionTransactionCommitGate {
        status: if commit_ready {
            "approve_execution_transaction_commit_ready".to_string()
        } else if runner_plan_ready {
            "approve_execution_transaction_commit_ready_blocked".to_string()
        } else {
            "approve_execution_transaction_commit_blocked".to_string()
        },
        action: runner_dry_run.action.clone(),
        approval_id: runner_dry_run.approval_id.clone(),
        runner_plan_ready,
        runner_enabled: runner_dry_run.runner_enabled,
        admission_ready: runner_dry_run.admission_ready,
        commit_barrier_ready: runner_dry_run.commit_barrier_ready,
        rollback_plan_ready: runner_dry_run.rollback_plan_ready,
        recovery_marker_persistence_plan_ready: marker_persistence_gate
            .marker_persistence_plan_ready,
        recovery_marker_persistence_ready: marker_persistence_gate.persistence_ready,
        commit_gate_enabled,
        commit_ready,
        would_start_runner: false,
        would_persist_recovery_marker: false,
        would_commit_transaction: false,
        would_touch_disk: false,
        required_gates: vec![
            "approve_execution_transaction_runner_enabled".to_string(),
            "approve_execution_admission_ready".to_string(),
            "commit_barrier_ready".to_string(),
            "rollback_plan_ready".to_string(),
            "recovery_marker_persistence_plan_ready".to_string(),
            "recovery_marker_persistence_ready".to_string(),
            "approve_execution_transaction_commit_enabled".to_string(),
        ],
        passed_gates,
        blocked_gates,
    }
}

fn contract_repair_approval_approve_execution_transaction_commit_gate_with_gate(
    commit_gate: &ContractRepairApprovalApproveExecutionTransactionCommitGate,
    transaction_commit_enabled: bool,
) -> ContractRepairApprovalApproveExecutionTransactionCommitGate {
    let mut result = commit_gate.clone();
    if !transaction_commit_enabled {
        return result;
    }

    result.commit_gate_enabled = true;
    result
        .blocked_gates
        .retain(|gate| gate != "approve_execution_transaction_commit_enabled");
    push_unique_blocker(
        &mut result.passed_gates,
        "approve_execution_transaction_commit_enabled",
    );

    let gate_states = [
        (
            "approve_execution_transaction_commit_plan_ready",
            commit_gate.runner_plan_ready,
        ),
        (
            "approve_execution_transaction_runner_enabled",
            commit_gate.runner_enabled,
        ),
        (
            "approve_execution_admission_ready",
            commit_gate.admission_ready,
        ),
        ("commit_barrier_ready", commit_gate.commit_barrier_ready),
        ("rollback_plan_ready", commit_gate.rollback_plan_ready),
        (
            "recovery_marker_persistence_plan_ready",
            commit_gate.recovery_marker_persistence_plan_ready,
        ),
        (
            "recovery_marker_persistence_ready",
            commit_gate.recovery_marker_persistence_ready,
        ),
    ];
    for (gate, passed) in gate_states {
        if !passed {
            push_unique_blocker(&mut result.blocked_gates, gate);
        }
    }
    if !result.blocked_gates.is_empty() {
        result.commit_ready = false;
        result.status = "approve_execution_transaction_commit_blocked".to_string();
        return result;
    }

    result.status = "approve_execution_transaction_commit_ready".to_string();
    result.commit_ready = true;
    result.would_commit_transaction = true;
    result.would_touch_disk = false;
    result
}

fn contract_repair_approval_approve_execution_transaction_commit_readiness_dry_run(
    commit_gate: &ContractRepairApprovalApproveExecutionTransactionCommitGate,
) -> ContractRepairApprovalApproveExecutionTransactionCommitReadinessDryRun {
    let runner_plan_ready = commit_gate.runner_plan_ready;
    let runner_enabled = commit_gate.runner_enabled;
    let admission_ready = commit_gate.admission_ready;
    let commit_barrier_ready = commit_gate.commit_barrier_ready;
    let rollback_plan_ready = commit_gate.rollback_plan_ready;
    let recovery_marker_persistence_plan_ready = commit_gate.recovery_marker_persistence_plan_ready;
    let recovery_marker_persistence_ready = commit_gate.recovery_marker_persistence_ready;
    let commit_gate_enabled = commit_gate.commit_gate_enabled;
    let transaction_commit_ready = commit_gate.commit_ready;
    let required_gates = vec![
        "approve_execution_transaction_runner_enabled".to_string(),
        "approve_execution_admission_ready".to_string(),
        "commit_barrier_ready".to_string(),
        "rollback_plan_ready".to_string(),
        "recovery_marker_persistence_plan_ready".to_string(),
        "recovery_marker_persistence_ready".to_string(),
        "approve_execution_transaction_commit_enabled".to_string(),
    ];
    let gate_states = [
        (
            "approve_execution_transaction_runner_enabled",
            runner_enabled,
        ),
        ("approve_execution_admission_ready", admission_ready),
        ("commit_barrier_ready", commit_barrier_ready),
        ("rollback_plan_ready", rollback_plan_ready),
        (
            "recovery_marker_persistence_plan_ready",
            recovery_marker_persistence_plan_ready,
        ),
        (
            "recovery_marker_persistence_ready",
            recovery_marker_persistence_ready,
        ),
        (
            "approve_execution_transaction_commit_enabled",
            commit_gate_enabled,
        ),
    ];
    let mut passed_gates = Vec::new();
    let mut blocked_gates = Vec::new();
    for (gate, passed) in gate_states {
        if passed {
            passed_gates.push(gate.to_string());
        } else {
            blocked_gates.push(gate.to_string());
        }
    }

    ContractRepairApprovalApproveExecutionTransactionCommitReadinessDryRun {
        status: if transaction_commit_ready {
            "approve_execution_transaction_commit_readiness_ready".to_string()
        } else if commit_gate.action == "approve" && runner_plan_ready {
            "approve_execution_transaction_commit_readiness_ready_blocked".to_string()
        } else {
            "approve_execution_transaction_commit_readiness_blocked".to_string()
        },
        action: commit_gate.action.clone(),
        approval_id: commit_gate.approval_id.clone(),
        gate_name: "approve_execution_transaction_commit_ready".to_string(),
        runner_plan_ready,
        runner_enabled,
        admission_ready,
        commit_barrier_ready,
        rollback_plan_ready,
        recovery_marker_persistence_plan_ready,
        recovery_marker_persistence_ready,
        commit_gate_enabled,
        transaction_commit_ready,
        would_start_runner: false,
        would_persist_recovery_marker: false,
        would_commit_transaction: false,
        would_touch_disk: false,
        would_unblock_atomic_side_effects: false,
        inherited_transaction_commit_blocked_gates: commit_gate.blocked_gates.clone(),
        required_gates,
        passed_gates,
        blocked_gates,
    }
}

fn contract_repair_approval_approve_execution_atomic_side_effects_gate(
    lifecycle_enablement_gate: &ContractRepairApprovalLifecycleEmissionEnablementGate,
    contract_mutation_gate: &ContractRepairApprovalContractMutationEnablementGate,
    marker_persistence_gate: &ContractRepairApprovalApproveExecutionRecoveryMarkerPersistenceGate,
    transaction_commit_gate: &ContractRepairApprovalApproveExecutionTransactionCommitGate,
    atomic_side_effects_enabled: bool,
) -> ContractRepairApprovalApproveExecutionAtomicSideEffectsGate {
    let lifecycle_plan_ready = lifecycle_enablement_gate.lifecycle_emission_plan_ready;
    let contract_mutation_plan_ready =
        contract_mutation_gate.writeback_plan_ready && contract_mutation_gate.source_write_ready;
    let recovery_marker_persistence_plan_ready =
        marker_persistence_gate.marker_persistence_plan_ready;
    let transaction_commit_plan_ready = transaction_commit_gate.runner_plan_ready;
    let atomic_side_effects_plan_ready = lifecycle_plan_ready
        && contract_mutation_plan_ready
        && recovery_marker_persistence_plan_ready
        && transaction_commit_plan_ready;
    let lifecycle_effects_ready = lifecycle_enablement_gate.lifecycle_effects_ready;
    let contract_mutation_ready = contract_mutation_gate.mutation_ready;
    let recovery_marker_persistence_ready = marker_persistence_gate.persistence_ready;
    let transaction_commit_ready = transaction_commit_gate.commit_ready;
    let atomic_side_effects_ready = atomic_side_effects_plan_ready
        && lifecycle_effects_ready
        && contract_mutation_ready
        && recovery_marker_persistence_ready
        && transaction_commit_ready
        && atomic_side_effects_enabled;
    let mut passed_gates = Vec::new();
    if lifecycle_plan_ready {
        passed_gates.push("lifecycle_emission_plan_ready".to_string());
    }
    if contract_mutation_plan_ready {
        passed_gates.push("contract_mutation_plan_ready".to_string());
    }
    if recovery_marker_persistence_plan_ready {
        passed_gates.push("recovery_marker_persistence_plan_ready".to_string());
    }
    if transaction_commit_plan_ready {
        passed_gates.push("approve_execution_transaction_commit_plan_ready".to_string());
    }
    if lifecycle_effects_ready {
        passed_gates.push("lifecycle_effects_ready".to_string());
    }
    if contract_mutation_ready {
        passed_gates.push("contract_mutation_ready".to_string());
    }
    if recovery_marker_persistence_ready {
        passed_gates.push("recovery_marker_persistence_ready".to_string());
    }
    if transaction_commit_ready {
        passed_gates.push("approve_execution_transaction_commit_ready".to_string());
    }
    if atomic_side_effects_enabled {
        passed_gates.push("approve_execution_atomic_side_effects_enabled".to_string());
    }
    let mut blocked_gates = Vec::new();
    if !lifecycle_plan_ready {
        blocked_gates.push("lifecycle_emission_plan_ready".to_string());
    }
    if !contract_mutation_plan_ready {
        blocked_gates.push("contract_mutation_plan_ready".to_string());
    }
    if !recovery_marker_persistence_plan_ready {
        blocked_gates.push("recovery_marker_persistence_plan_ready".to_string());
    }
    if !transaction_commit_plan_ready {
        blocked_gates.push("approve_execution_transaction_commit_plan_ready".to_string());
    }
    if !lifecycle_effects_ready {
        blocked_gates.push("lifecycle_effects_ready".to_string());
    }
    if !contract_mutation_ready {
        blocked_gates.push("contract_mutation_ready".to_string());
    }
    if !recovery_marker_persistence_ready {
        blocked_gates.push("recovery_marker_persistence_ready".to_string());
    }
    if !transaction_commit_ready {
        blocked_gates.push("approve_execution_transaction_commit_ready".to_string());
    }
    if !atomic_side_effects_enabled {
        blocked_gates.push("approve_execution_atomic_side_effects_enabled".to_string());
    }

    ContractRepairApprovalApproveExecutionAtomicSideEffectsGate {
        status: if atomic_side_effects_ready {
            "approve_execution_atomic_side_effects_ready".to_string()
        } else if atomic_side_effects_plan_ready {
            "approve_execution_atomic_side_effects_ready_blocked".to_string()
        } else {
            "approve_execution_atomic_side_effects_blocked".to_string()
        },
        action: transaction_commit_gate.action.clone(),
        approval_id: transaction_commit_gate.approval_id.clone(),
        lifecycle_plan_ready,
        contract_mutation_plan_ready,
        recovery_marker_persistence_plan_ready,
        transaction_commit_plan_ready,
        atomic_side_effects_plan_ready,
        lifecycle_effects_ready,
        contract_mutation_ready,
        recovery_marker_persistence_ready,
        transaction_commit_ready,
        atomic_side_effects_enabled,
        atomic_side_effects_ready,
        would_emit_lifecycle: false,
        would_mutate_contract: false,
        would_persist_recovery_marker: false,
        would_commit_transaction: false,
        would_touch_disk: false,
        required_gates: vec![
            "lifecycle_emission_plan_ready".to_string(),
            "contract_mutation_plan_ready".to_string(),
            "recovery_marker_persistence_plan_ready".to_string(),
            "approve_execution_transaction_commit_plan_ready".to_string(),
            "lifecycle_effects_ready".to_string(),
            "contract_mutation_ready".to_string(),
            "recovery_marker_persistence_ready".to_string(),
            "approve_execution_transaction_commit_ready".to_string(),
            "approve_execution_atomic_side_effects_enabled".to_string(),
        ],
        passed_gates,
        blocked_gates,
    }
}

fn contract_repair_approval_approve_execution_atomic_side_effects_enablement_dry_run(
    atomic_side_effects_gate: &ContractRepairApprovalApproveExecutionAtomicSideEffectsGate,
) -> ContractRepairApprovalApproveExecutionAtomicSideEffectsEnablementDryRun {
    let lifecycle_plan_ready = atomic_side_effects_gate.lifecycle_plan_ready;
    let contract_mutation_plan_ready = atomic_side_effects_gate.contract_mutation_plan_ready;
    let recovery_marker_persistence_plan_ready =
        atomic_side_effects_gate.recovery_marker_persistence_plan_ready;
    let transaction_commit_plan_ready = atomic_side_effects_gate.transaction_commit_plan_ready;
    let atomic_side_effects_plan_ready = atomic_side_effects_gate.atomic_side_effects_plan_ready;
    let lifecycle_effects_ready = atomic_side_effects_gate.lifecycle_effects_ready;
    let contract_mutation_ready = atomic_side_effects_gate.contract_mutation_ready;
    let recovery_marker_persistence_ready =
        atomic_side_effects_gate.recovery_marker_persistence_ready;
    let transaction_commit_ready = atomic_side_effects_gate.transaction_commit_ready;
    let atomic_side_effects_enabled = atomic_side_effects_gate.atomic_side_effects_enabled;
    let enablement_prerequisites_ready = atomic_side_effects_plan_ready
        && lifecycle_effects_ready
        && contract_mutation_ready
        && recovery_marker_persistence_ready
        && transaction_commit_ready;
    let atomic_side_effects_enablement_ready =
        enablement_prerequisites_ready && atomic_side_effects_enabled;
    let required_gates = vec![
        "approve_execution_atomic_side_effects_plan_ready".to_string(),
        "lifecycle_effects_ready".to_string(),
        "contract_mutation_ready".to_string(),
        "recovery_marker_persistence_ready".to_string(),
        "approve_execution_transaction_commit_ready".to_string(),
        "approve_execution_atomic_side_effects_enabled".to_string(),
    ];
    let gate_states = [
        (
            "approve_execution_atomic_side_effects_plan_ready",
            atomic_side_effects_plan_ready,
        ),
        ("lifecycle_effects_ready", lifecycle_effects_ready),
        ("contract_mutation_ready", contract_mutation_ready),
        (
            "recovery_marker_persistence_ready",
            recovery_marker_persistence_ready,
        ),
        (
            "approve_execution_transaction_commit_ready",
            transaction_commit_ready,
        ),
        (
            "approve_execution_atomic_side_effects_enabled",
            atomic_side_effects_enabled,
        ),
    ];
    let mut passed_gates = Vec::new();
    let mut blocked_gates = Vec::new();
    for (gate, passed) in gate_states {
        if passed {
            passed_gates.push(gate.to_string());
        } else {
            blocked_gates.push(gate.to_string());
        }
    }

    ContractRepairApprovalApproveExecutionAtomicSideEffectsEnablementDryRun {
        status: if atomic_side_effects_enablement_ready {
            "approve_execution_atomic_side_effects_enablement_ready".to_string()
        } else if atomic_side_effects_gate.action == "approve" && atomic_side_effects_plan_ready {
            "approve_execution_atomic_side_effects_enablement_ready_blocked".to_string()
        } else {
            "approve_execution_atomic_side_effects_enablement_blocked".to_string()
        },
        action: atomic_side_effects_gate.action.clone(),
        approval_id: atomic_side_effects_gate.approval_id.clone(),
        switch_name: "approve_execution_atomic_side_effects_enabled".to_string(),
        lifecycle_plan_ready,
        contract_mutation_plan_ready,
        recovery_marker_persistence_plan_ready,
        transaction_commit_plan_ready,
        atomic_side_effects_plan_ready,
        lifecycle_effects_ready,
        contract_mutation_ready,
        recovery_marker_persistence_ready,
        transaction_commit_ready,
        atomic_side_effects_enabled,
        enablement_prerequisites_ready,
        atomic_side_effects_enablement_ready,
        would_enable_atomic_side_effects: false,
        would_emit_lifecycle: false,
        would_mutate_contract: false,
        would_persist_recovery_marker: false,
        would_commit_transaction: false,
        would_touch_disk: false,
        would_unblock_atomic_side_effects_readiness: false,
        would_unblock_runner_attempt_readiness: false,
        inherited_atomic_side_effects_blocked_gates: atomic_side_effects_gate.blocked_gates.clone(),
        required_gates,
        passed_gates,
        blocked_gates,
    }
}

fn contract_repair_approval_approve_execution_atomic_side_effects_readiness_dry_run(
    atomic_side_effects_gate: &ContractRepairApprovalApproveExecutionAtomicSideEffectsGate,
) -> ContractRepairApprovalApproveExecutionAtomicSideEffectsReadinessDryRun {
    let atomic_side_effects_plan_ready = atomic_side_effects_gate.atomic_side_effects_plan_ready;
    let lifecycle_effects_ready = atomic_side_effects_gate.lifecycle_effects_ready;
    let contract_mutation_ready = atomic_side_effects_gate.contract_mutation_ready;
    let recovery_marker_persistence_ready =
        atomic_side_effects_gate.recovery_marker_persistence_ready;
    let transaction_commit_ready = atomic_side_effects_gate.transaction_commit_ready;
    let atomic_side_effects_enabled = atomic_side_effects_gate.atomic_side_effects_enabled;
    let atomic_side_effects_ready = atomic_side_effects_gate.atomic_side_effects_ready;
    let required_gates = vec![
        "approve_execution_atomic_side_effects_plan_ready".to_string(),
        "lifecycle_effects_ready".to_string(),
        "contract_mutation_ready".to_string(),
        "recovery_marker_persistence_ready".to_string(),
        "approve_execution_transaction_commit_ready".to_string(),
        "approve_execution_atomic_side_effects_enabled".to_string(),
    ];
    let gate_states = [
        (
            "approve_execution_atomic_side_effects_plan_ready",
            atomic_side_effects_plan_ready,
        ),
        ("lifecycle_effects_ready", lifecycle_effects_ready),
        ("contract_mutation_ready", contract_mutation_ready),
        (
            "recovery_marker_persistence_ready",
            recovery_marker_persistence_ready,
        ),
        (
            "approve_execution_transaction_commit_ready",
            transaction_commit_ready,
        ),
        (
            "approve_execution_atomic_side_effects_enabled",
            atomic_side_effects_enabled,
        ),
    ];
    let mut passed_gates = Vec::new();
    let mut blocked_gates = Vec::new();
    for (gate, passed) in gate_states {
        if passed {
            passed_gates.push(gate.to_string());
        } else {
            blocked_gates.push(gate.to_string());
        }
    }

    ContractRepairApprovalApproveExecutionAtomicSideEffectsReadinessDryRun {
        status: if atomic_side_effects_ready {
            "approve_execution_atomic_side_effects_readiness_ready".to_string()
        } else if atomic_side_effects_gate.action == "approve" && atomic_side_effects_plan_ready {
            "approve_execution_atomic_side_effects_readiness_ready_blocked".to_string()
        } else {
            "approve_execution_atomic_side_effects_readiness_blocked".to_string()
        },
        action: atomic_side_effects_gate.action.clone(),
        approval_id: atomic_side_effects_gate.approval_id.clone(),
        gate_name: "approve_execution_atomic_side_effects_ready".to_string(),
        lifecycle_plan_ready: atomic_side_effects_gate.lifecycle_plan_ready,
        contract_mutation_plan_ready: atomic_side_effects_gate.contract_mutation_plan_ready,
        recovery_marker_persistence_plan_ready: atomic_side_effects_gate
            .recovery_marker_persistence_plan_ready,
        transaction_commit_plan_ready: atomic_side_effects_gate.transaction_commit_plan_ready,
        atomic_side_effects_plan_ready,
        lifecycle_effects_ready,
        contract_mutation_ready,
        recovery_marker_persistence_ready,
        transaction_commit_ready,
        atomic_side_effects_enabled,
        atomic_side_effects_ready,
        would_emit_lifecycle: false,
        would_mutate_contract: false,
        would_persist_recovery_marker: false,
        would_commit_transaction: false,
        would_touch_disk: false,
        would_unblock_runner_attempt_readiness: false,
        inherited_atomic_side_effects_blocked_gates: atomic_side_effects_gate.blocked_gates.clone(),
        required_gates,
        passed_gates,
        blocked_gates,
    }
}

fn contract_repair_approval_approve_execution_runner_attempt_enablement_dry_run(
    action: &str,
    approval_id: &str,
    review_enabled: bool,
    atomic_side_effects_gate: &ContractRepairApprovalApproveExecutionAtomicSideEffectsGate,
    transaction_runner_enablement_dry_run: &ContractRepairApprovalApproveExecutionTransactionRunnerEnablementDryRun,
    runner_attempt_enabled: bool,
) -> ContractRepairApprovalApproveExecutionRunnerAttemptEnablementDryRun {
    let attempt_requested = review_enabled && action == "approve";
    let atomic_side_effects_plan_ready = atomic_side_effects_gate.atomic_side_effects_plan_ready;
    let atomic_side_effects_ready = atomic_side_effects_gate.atomic_side_effects_ready;
    let transaction_runner_enablement_ready =
        transaction_runner_enablement_dry_run.runner_enablement_ready;
    let enablement_prerequisites_ready = attempt_requested
        && atomic_side_effects_plan_ready
        && atomic_side_effects_ready
        && transaction_runner_enablement_ready;
    let runner_attempt_enablement_ready = enablement_prerequisites_ready && runner_attempt_enabled;
    let required_gates = vec![
        "approve_execution_attempt_requested".to_string(),
        "approve_execution_atomic_side_effects_plan_ready".to_string(),
        "approve_execution_atomic_side_effects_ready".to_string(),
        "approve_execution_transaction_runner_enablement_ready".to_string(),
        "approve_execution_runner_attempt_enabled".to_string(),
    ];
    let gate_states = [
        ("approve_execution_attempt_requested", attempt_requested),
        (
            "approve_execution_atomic_side_effects_plan_ready",
            atomic_side_effects_plan_ready,
        ),
        (
            "approve_execution_atomic_side_effects_ready",
            atomic_side_effects_ready,
        ),
        (
            "approve_execution_transaction_runner_enablement_ready",
            transaction_runner_enablement_ready,
        ),
        (
            "approve_execution_runner_attempt_enabled",
            runner_attempt_enabled,
        ),
    ];
    let mut passed_gates = Vec::new();
    let mut blocked_gates = Vec::new();
    for (gate, passed) in gate_states {
        if passed {
            passed_gates.push(gate.to_string());
        } else {
            blocked_gates.push(gate.to_string());
        }
    }

    ContractRepairApprovalApproveExecutionRunnerAttemptEnablementDryRun {
        status: if runner_attempt_enablement_ready {
            "approve_execution_runner_attempt_enablement_ready".to_string()
        } else if attempt_requested && atomic_side_effects_plan_ready {
            "approve_execution_runner_attempt_enablement_ready_blocked".to_string()
        } else {
            "approve_execution_runner_attempt_enablement_blocked".to_string()
        },
        action: action.to_string(),
        approval_id: approval_id.to_string(),
        switch_name: "approve_execution_runner_attempt_enabled".to_string(),
        attempt_requested,
        atomic_side_effects_plan_ready,
        atomic_side_effects_ready,
        transaction_runner_enablement_ready,
        runner_attempt_enabled,
        enablement_prerequisites_ready,
        runner_attempt_enablement_ready,
        would_enable_runner_attempt: false,
        would_start_runner: false,
        would_persist_any_side_effect: false,
        would_rollback_on_error: false,
        required_gates,
        passed_gates,
        blocked_gates,
    }
}

fn contract_repair_approval_approve_execution_runner_attempt(
    action: &str,
    approval_id: &str,
    review_enabled: bool,
    atomic_side_effects_gate: &ContractRepairApprovalApproveExecutionAtomicSideEffectsGate,
    runner_attempt_enabled: bool,
) -> ContractRepairApprovalApproveExecutionRunnerAttempt {
    let attempt_requested = review_enabled && action == "approve";
    let atomic_side_effects_plan_ready = atomic_side_effects_gate.atomic_side_effects_plan_ready;
    let atomic_side_effects_ready = atomic_side_effects_gate.atomic_side_effects_ready;
    let runner_attempt_ready =
        attempt_requested && atomic_side_effects_ready && runner_attempt_enabled;
    let mut blocked_by = Vec::new();
    if !attempt_requested {
        blocked_by.push("approve_execution_attempt_requested".to_string());
    }
    if !atomic_side_effects_plan_ready {
        blocked_by.push("approve_execution_atomic_side_effects_plan_ready".to_string());
    }
    if !atomic_side_effects_ready {
        blocked_by.push("approve_execution_atomic_side_effects_ready".to_string());
    }
    if !runner_attempt_enabled {
        blocked_by.push("approve_execution_runner_attempt_enabled".to_string());
    }

    ContractRepairApprovalApproveExecutionRunnerAttempt {
        status: if runner_attempt_ready {
            "approve_execution_runner_attempt_ready".to_string()
        } else if attempt_requested && atomic_side_effects_plan_ready {
            "approve_execution_runner_attempt_ready_blocked".to_string()
        } else {
            "approve_execution_runner_attempt_blocked".to_string()
        },
        action: action.to_string(),
        approval_id: approval_id.to_string(),
        attempt_requested,
        atomic_side_effects_plan_ready,
        atomic_side_effects_ready,
        runner_attempt_enabled,
        runner_attempt_ready,
        would_start_runner: false,
        would_persist_any_side_effect: false,
        would_rollback_on_error: false,
        blocked_by,
    }
}

fn contract_repair_approval_approve_execution_runner_attempt_readiness_dry_run(
    runner_attempt: &ContractRepairApprovalApproveExecutionRunnerAttempt,
    attempt_enablement_dry_run: &ContractRepairApprovalApproveExecutionRunnerAttemptEnablementDryRun,
) -> ContractRepairApprovalApproveExecutionRunnerAttemptReadinessDryRun {
    let attempt_requested = runner_attempt.attempt_requested;
    let atomic_side_effects_plan_ready = runner_attempt.atomic_side_effects_plan_ready;
    let atomic_side_effects_ready = runner_attempt.atomic_side_effects_ready;
    let transaction_runner_enablement_ready =
        attempt_enablement_dry_run.transaction_runner_enablement_ready;
    let runner_attempt_enabled = runner_attempt.runner_attempt_enabled;
    let runner_attempt_enablement_ready =
        attempt_enablement_dry_run.runner_attempt_enablement_ready;
    let runner_attempt_ready = runner_attempt.runner_attempt_ready;
    let required_gates = vec![
        "approve_execution_attempt_requested".to_string(),
        "approve_execution_atomic_side_effects_ready".to_string(),
        "approve_execution_runner_attempt_enabled".to_string(),
    ];
    let gate_states = [
        ("approve_execution_attempt_requested", attempt_requested),
        (
            "approve_execution_atomic_side_effects_ready",
            atomic_side_effects_ready,
        ),
        (
            "approve_execution_runner_attempt_enabled",
            runner_attempt_enabled,
        ),
    ];
    let mut passed_gates = Vec::new();
    let mut blocked_gates = Vec::new();
    for (gate, passed) in gate_states {
        if passed {
            passed_gates.push(gate.to_string());
        } else {
            blocked_gates.push(gate.to_string());
        }
    }

    ContractRepairApprovalApproveExecutionRunnerAttemptReadinessDryRun {
        status: if runner_attempt_ready {
            "approve_execution_runner_attempt_readiness_ready".to_string()
        } else if runner_attempt.action == "approve"
            && attempt_requested
            && atomic_side_effects_plan_ready
        {
            "approve_execution_runner_attempt_readiness_ready_blocked".to_string()
        } else {
            "approve_execution_runner_attempt_readiness_blocked".to_string()
        },
        action: runner_attempt.action.clone(),
        approval_id: runner_attempt.approval_id.clone(),
        gate_name: "approve_execution_runner_attempt_ready".to_string(),
        attempt_requested,
        atomic_side_effects_plan_ready,
        atomic_side_effects_ready,
        transaction_runner_enablement_ready,
        runner_attempt_enabled,
        runner_attempt_enablement_ready,
        runner_attempt_ready,
        would_enable_runner_attempt: false,
        would_start_runner: false,
        would_persist_any_side_effect: false,
        would_rollback_on_error: false,
        would_unblock_control_readiness: false,
        inherited_runner_attempt_blockers: runner_attempt.blocked_by.clone(),
        inherited_attempt_enablement_blocked_gates: attempt_enablement_dry_run
            .blocked_gates
            .clone(),
        required_gates,
        passed_gates,
        blocked_gates,
    }
}

fn contract_repair_approval_approve_execution_runner_execution_enablement_dry_run(
    runner_attempt: &ContractRepairApprovalApproveExecutionRunnerAttempt,
    runner_attempt_enablement_dry_run: &ContractRepairApprovalApproveExecutionRunnerAttemptEnablementDryRun,
    runner_execution_enabled: bool,
) -> ContractRepairApprovalApproveExecutionRunnerExecutionEnablementDryRun {
    let branch_selected = runner_attempt.attempt_requested;
    let runner_attempt_ready = runner_attempt.runner_attempt_ready;
    let atomic_side_effects_ready = runner_attempt.atomic_side_effects_ready;
    let runner_attempt_enablement_ready =
        runner_attempt_enablement_dry_run.runner_attempt_enablement_ready;
    let enablement_prerequisites_ready = branch_selected
        && runner_attempt_ready
        && atomic_side_effects_ready
        && runner_attempt_enablement_ready;
    let runner_execution_enablement_ready =
        enablement_prerequisites_ready && runner_execution_enabled;
    let required_gates = vec![
        "approve_execution_runner_branch_selected".to_string(),
        "approve_execution_runner_attempt_ready".to_string(),
        "approve_execution_atomic_side_effects_ready".to_string(),
        "approve_execution_runner_attempt_enablement_ready".to_string(),
        "approve_execution_runner_execution_enabled".to_string(),
    ];
    let gate_states = [
        ("approve_execution_runner_branch_selected", branch_selected),
        (
            "approve_execution_runner_attempt_ready",
            runner_attempt_ready,
        ),
        (
            "approve_execution_atomic_side_effects_ready",
            atomic_side_effects_ready,
        ),
        (
            "approve_execution_runner_attempt_enablement_ready",
            runner_attempt_enablement_ready,
        ),
        (
            "approve_execution_runner_execution_enabled",
            runner_execution_enabled,
        ),
    ];
    let mut passed_gates = Vec::new();
    let mut blocked_gates = Vec::new();
    for (gate, passed) in gate_states {
        if passed {
            passed_gates.push(gate.to_string());
        } else {
            blocked_gates.push(gate.to_string());
        }
    }

    ContractRepairApprovalApproveExecutionRunnerExecutionEnablementDryRun {
        status: if runner_execution_enablement_ready {
            "approve_execution_runner_execution_enablement_ready".to_string()
        } else if branch_selected {
            "approve_execution_runner_execution_enablement_ready_blocked".to_string()
        } else {
            "approve_execution_runner_execution_enablement_blocked".to_string()
        },
        action: runner_attempt.action.clone(),
        approval_id: runner_attempt.approval_id.clone(),
        switch_name: "approve_execution_runner_execution_enabled".to_string(),
        branch_selected,
        runner_attempt_ready,
        atomic_side_effects_ready,
        runner_attempt_enablement_ready,
        runner_execution_enabled,
        enablement_prerequisites_ready,
        runner_execution_enablement_ready,
        would_enable_runner_execution: false,
        would_start_runner: false,
        would_persist_any_side_effect: false,
        would_commit_transaction: false,
        would_rollback_on_error: false,
        required_gates,
        passed_gates,
        blocked_gates,
    }
}

fn contract_repair_approval_approve_execution_runner_outcome(
    runner_attempt: &ContractRepairApprovalApproveExecutionRunnerAttempt,
    runner_execution_enabled: bool,
) -> ContractRepairApprovalApproveExecutionRunnerOutcome {
    let branch_selected = runner_attempt.attempt_requested;
    let runner_attempt_ready = runner_attempt.runner_attempt_ready;
    let atomic_side_effects_ready = runner_attempt.atomic_side_effects_ready;
    let runner_execution_ready =
        branch_selected && runner_attempt_ready && runner_execution_enabled;
    let mut blocked_by = Vec::new();
    if !branch_selected {
        blocked_by.push("approve_execution_runner_branch_selected".to_string());
    }
    if !runner_attempt_ready {
        blocked_by.push("approve_execution_runner_attempt_ready".to_string());
    }
    if !atomic_side_effects_ready {
        blocked_by.push("approve_execution_atomic_side_effects_ready".to_string());
    }
    if !runner_execution_enabled {
        blocked_by.push("approve_execution_runner_execution_enabled".to_string());
    }

    ContractRepairApprovalApproveExecutionRunnerOutcome {
        status: if runner_execution_ready {
            "approve_execution_runner_outcome_ready".to_string()
        } else if branch_selected {
            "approve_execution_runner_outcome_ready_blocked".to_string()
        } else {
            "approve_execution_runner_outcome_blocked".to_string()
        },
        action: runner_attempt.action.clone(),
        approval_id: runner_attempt.approval_id.clone(),
        branch_selected,
        runner_attempt_ready,
        atomic_side_effects_ready,
        runner_execution_enabled,
        runner_execution_ready,
        would_start_runner: false,
        would_persist_any_side_effect: false,
        would_commit_transaction: false,
        would_rollback_on_error: false,
        blocked_by,
    }
}

fn contract_repair_approval_approve_execution_runner_execution_readiness_dry_run(
    runner_outcome: &ContractRepairApprovalApproveExecutionRunnerOutcome,
    runner_execution_enablement_dry_run: &ContractRepairApprovalApproveExecutionRunnerExecutionEnablementDryRun,
) -> ContractRepairApprovalApproveExecutionRunnerExecutionReadinessDryRun {
    let branch_selected = runner_outcome.branch_selected;
    let runner_attempt_ready = runner_outcome.runner_attempt_ready;
    let atomic_side_effects_ready = runner_outcome.atomic_side_effects_ready;
    let runner_execution_enabled = runner_outcome.runner_execution_enabled;
    let runner_execution_enablement_ready =
        runner_execution_enablement_dry_run.runner_execution_enablement_ready;
    let runner_execution_ready = runner_outcome.runner_execution_ready;
    let required_gates = vec![
        "approve_execution_runner_branch_selected".to_string(),
        "approve_execution_runner_attempt_ready".to_string(),
        "approve_execution_atomic_side_effects_ready".to_string(),
        "approve_execution_runner_execution_enabled".to_string(),
    ];
    let gate_states = [
        ("approve_execution_runner_branch_selected", branch_selected),
        (
            "approve_execution_runner_attempt_ready",
            runner_attempt_ready,
        ),
        (
            "approve_execution_atomic_side_effects_ready",
            atomic_side_effects_ready,
        ),
        (
            "approve_execution_runner_execution_enabled",
            runner_execution_enabled,
        ),
    ];
    let mut passed_gates = Vec::new();
    let mut blocked_gates = Vec::new();
    for (gate, passed) in gate_states {
        if passed {
            passed_gates.push(gate.to_string());
        } else {
            blocked_gates.push(gate.to_string());
        }
    }

    ContractRepairApprovalApproveExecutionRunnerExecutionReadinessDryRun {
        status: if runner_execution_ready {
            "approve_execution_runner_execution_readiness_ready".to_string()
        } else if branch_selected {
            "approve_execution_runner_execution_readiness_ready_blocked".to_string()
        } else {
            "approve_execution_runner_execution_readiness_blocked".to_string()
        },
        action: runner_outcome.action.clone(),
        approval_id: runner_outcome.approval_id.clone(),
        gate_name: "approve_execution_runner_execution_ready".to_string(),
        branch_selected,
        runner_attempt_ready,
        atomic_side_effects_ready,
        runner_execution_enabled,
        runner_execution_enablement_ready,
        runner_execution_ready,
        would_enable_runner_execution: false,
        would_start_runner: false,
        would_persist_any_side_effect: false,
        would_commit_transaction: false,
        would_rollback_on_error: false,
        would_unblock_route_dispatch: false,
        would_unblock_control_readiness: false,
        inherited_runner_outcome_blockers: runner_outcome.blocked_by.clone(),
        inherited_execution_enablement_blocked_gates: runner_execution_enablement_dry_run
            .blocked_gates
            .clone(),
        required_gates,
        passed_gates,
        blocked_gates,
    }
}

fn contract_repair_approval_approve_execution_runner_route_dispatch_enablement_dry_run(
    runner_outcome: &ContractRepairApprovalApproveExecutionRunnerOutcome,
    runner_execution_enablement_dry_run: &ContractRepairApprovalApproveExecutionRunnerExecutionEnablementDryRun,
    route_dispatch_enabled: bool,
) -> ContractRepairApprovalApproveExecutionRunnerRouteDispatchEnablementDryRun {
    let branch_selected = runner_outcome.branch_selected;
    let runner_execution_ready = runner_outcome.runner_execution_ready;
    let runner_execution_enablement_ready =
        runner_execution_enablement_dry_run.runner_execution_enablement_ready;
    let enablement_prerequisites_ready =
        branch_selected && runner_execution_ready && runner_execution_enablement_ready;
    let route_dispatch_enablement_ready = enablement_prerequisites_ready && route_dispatch_enabled;
    let required_gates = vec![
        "approve_execution_runner_branch_selected".to_string(),
        "approve_execution_runner_execution_ready".to_string(),
        "approve_execution_runner_execution_enablement_ready".to_string(),
        "approve_execution_runner_route_dispatch_enabled".to_string(),
    ];
    let gate_states = [
        ("approve_execution_runner_branch_selected", branch_selected),
        (
            "approve_execution_runner_execution_ready",
            runner_execution_ready,
        ),
        (
            "approve_execution_runner_execution_enablement_ready",
            runner_execution_enablement_ready,
        ),
        (
            "approve_execution_runner_route_dispatch_enabled",
            route_dispatch_enabled,
        ),
    ];
    let mut passed_gates = Vec::new();
    let mut blocked_gates = Vec::new();
    for (gate, passed) in gate_states {
        if passed {
            passed_gates.push(gate.to_string());
        } else {
            blocked_gates.push(gate.to_string());
        }
    }

    ContractRepairApprovalApproveExecutionRunnerRouteDispatchEnablementDryRun {
        status: if route_dispatch_enablement_ready {
            "approve_execution_runner_route_dispatch_enablement_ready".to_string()
        } else if branch_selected {
            "approve_execution_runner_route_dispatch_enablement_ready_blocked".to_string()
        } else {
            "approve_execution_runner_route_dispatch_enablement_blocked".to_string()
        },
        action: runner_outcome.action.clone(),
        approval_id: runner_outcome.approval_id.clone(),
        switch_name: "approve_execution_runner_route_dispatch_enabled".to_string(),
        branch_selected,
        runner_execution_ready,
        runner_execution_enablement_ready,
        route_dispatch_enabled,
        enablement_prerequisites_ready,
        route_dispatch_enablement_ready,
        would_enable_route_dispatch: false,
        would_enter_runner_branch: false,
        would_return_success: false,
        would_persist_any_side_effect: false,
        would_touch_disk: false,
        required_gates,
        passed_gates,
        blocked_gates,
    }
}

fn contract_repair_approval_approve_execution_runner_dispatch_gate(
    runner_outcome: &ContractRepairApprovalApproveExecutionRunnerOutcome,
    route_dispatch_enabled: bool,
) -> ContractRepairApprovalApproveExecutionRunnerDispatchGate {
    let branch_selected = runner_outcome.branch_selected;
    let runner_execution_ready = runner_outcome.runner_execution_ready;
    let dispatch_ready = branch_selected && runner_execution_ready && route_dispatch_enabled;
    let mut passed_gates = Vec::new();
    if branch_selected {
        passed_gates.push("approve_execution_runner_branch_selected".to_string());
    }
    if runner_execution_ready {
        passed_gates.push("approve_execution_runner_execution_ready".to_string());
    }
    if route_dispatch_enabled {
        passed_gates.push("approve_execution_runner_route_dispatch_enabled".to_string());
    }
    let mut blocked_gates = Vec::new();
    if !branch_selected {
        blocked_gates.push("approve_execution_runner_branch_selected".to_string());
    }
    if !runner_execution_ready {
        blocked_gates.push("approve_execution_runner_execution_ready".to_string());
    }
    if !route_dispatch_enabled {
        blocked_gates.push("approve_execution_runner_route_dispatch_enabled".to_string());
    }

    ContractRepairApprovalApproveExecutionRunnerDispatchGate {
        status: if dispatch_ready {
            "approve_execution_runner_dispatch_ready".to_string()
        } else if branch_selected {
            "approve_execution_runner_dispatch_ready_blocked".to_string()
        } else {
            "approve_execution_runner_dispatch_blocked".to_string()
        },
        action: runner_outcome.action.clone(),
        approval_id: runner_outcome.approval_id.clone(),
        branch_selected,
        runner_execution_ready,
        route_dispatch_enabled,
        dispatch_ready,
        would_enter_runner_branch: false,
        would_return_success: false,
        would_persist_any_side_effect: false,
        would_touch_disk: false,
        required_gates: vec![
            "approve_execution_runner_branch_selected".to_string(),
            "approve_execution_runner_execution_ready".to_string(),
            "approve_execution_runner_route_dispatch_enabled".to_string(),
        ],
        passed_gates,
        blocked_gates,
    }
}

fn contract_repair_approval_approve_execution_runner_dispatch_readiness_dry_run(
    dispatch_gate: &ContractRepairApprovalApproveExecutionRunnerDispatchGate,
    route_dispatch_enablement_dry_run: &ContractRepairApprovalApproveExecutionRunnerRouteDispatchEnablementDryRun,
) -> ContractRepairApprovalApproveExecutionRunnerDispatchReadinessDryRun {
    let branch_selected = dispatch_gate.branch_selected;
    let runner_execution_ready = dispatch_gate.runner_execution_ready;
    let route_dispatch_enabled = dispatch_gate.route_dispatch_enabled;
    let route_dispatch_enablement_ready =
        route_dispatch_enablement_dry_run.route_dispatch_enablement_ready;
    let dispatch_ready = dispatch_gate.dispatch_ready;
    let required_gates = vec![
        "approve_execution_runner_branch_selected".to_string(),
        "approve_execution_runner_execution_ready".to_string(),
        "approve_execution_runner_route_dispatch_enabled".to_string(),
    ];
    let gate_states = [
        ("approve_execution_runner_branch_selected", branch_selected),
        (
            "approve_execution_runner_execution_ready",
            runner_execution_ready,
        ),
        (
            "approve_execution_runner_route_dispatch_enabled",
            route_dispatch_enabled,
        ),
    ];
    let mut passed_gates = Vec::new();
    let mut blocked_gates = Vec::new();
    for (gate, passed) in gate_states {
        if passed {
            passed_gates.push(gate.to_string());
        } else {
            blocked_gates.push(gate.to_string());
        }
    }

    ContractRepairApprovalApproveExecutionRunnerDispatchReadinessDryRun {
        status: if dispatch_ready {
            "approve_execution_runner_dispatch_readiness_ready".to_string()
        } else if branch_selected {
            "approve_execution_runner_dispatch_readiness_ready_blocked".to_string()
        } else {
            "approve_execution_runner_dispatch_readiness_blocked".to_string()
        },
        action: dispatch_gate.action.clone(),
        approval_id: dispatch_gate.approval_id.clone(),
        gate_name: "approve_execution_runner_dispatch_ready".to_string(),
        branch_selected,
        runner_execution_ready,
        route_dispatch_enabled,
        route_dispatch_enablement_ready,
        dispatch_ready,
        would_enter_runner_branch: false,
        would_return_success: false,
        would_persist_any_side_effect: false,
        would_touch_disk: false,
        would_unblock_handoff: false,
        would_unblock_control_readiness: false,
        inherited_dispatch_blocked_gates: dispatch_gate.blocked_gates.clone(),
        inherited_route_dispatch_enablement_blocked_gates: route_dispatch_enablement_dry_run
            .blocked_gates
            .clone(),
        required_gates,
        passed_gates,
        blocked_gates,
    }
}

fn contract_repair_approval_approve_execution_runner_handoff(
    dispatch_gate: &ContractRepairApprovalApproveExecutionRunnerDispatchGate,
) -> ContractRepairApprovalApproveExecutionRunnerHandoff {
    let dispatch_ready = dispatch_gate.dispatch_ready;
    let route_dispatch_enabled = dispatch_gate.route_dispatch_enabled;
    let handoff_ready = dispatch_ready && route_dispatch_enabled;
    let mut blocked_by = Vec::new();
    if !dispatch_ready {
        blocked_by.push("approve_execution_runner_dispatch_ready".to_string());
    }
    if !route_dispatch_enabled {
        blocked_by.push("approve_execution_runner_route_dispatch_enabled".to_string());
    }
    for gate in &dispatch_gate.blocked_gates {
        if !blocked_by.contains(gate) {
            blocked_by.push(gate.clone());
        }
    }

    ContractRepairApprovalApproveExecutionRunnerHandoff {
        status: if handoff_ready {
            "approve_execution_runner_handoff_ready".to_string()
        } else if dispatch_gate.branch_selected {
            "approve_execution_runner_handoff_ready_blocked".to_string()
        } else {
            "approve_execution_runner_handoff_blocked".to_string()
        },
        action: dispatch_gate.action.clone(),
        approval_id: dispatch_gate.approval_id.clone(),
        dispatch_ready,
        route_dispatch_enabled,
        handoff_ready,
        expected_http_status: if handoff_ready { 200 } else { 423 },
        expected_route_status: if handoff_ready {
            "review_approve_executed".to_string()
        } else if dispatch_gate.branch_selected {
            "review_decision_execution_blocked".to_string()
        } else {
            "review_workflow_disabled".to_string()
        },
        would_call_runner: false,
        would_return_success: false,
        would_persist_any_side_effect: false,
        would_touch_disk: false,
        blocked_by,
    }
}

fn contract_repair_approval_approve_execution_runner_handoff_readiness_dry_run(
    handoff: &ContractRepairApprovalApproveExecutionRunnerHandoff,
    dispatch_readiness_dry_run: &ContractRepairApprovalApproveExecutionRunnerDispatchReadinessDryRun,
) -> ContractRepairApprovalApproveExecutionRunnerHandoffReadinessDryRun {
    let dispatch_ready = handoff.dispatch_ready;
    let route_dispatch_enabled = handoff.route_dispatch_enabled;
    let dispatch_readiness_ready = dispatch_readiness_dry_run.dispatch_ready;
    let handoff_ready = handoff.handoff_ready;
    let required_gates = vec![
        "approve_execution_runner_dispatch_ready".to_string(),
        "approve_execution_runner_route_dispatch_enabled".to_string(),
    ];
    let gate_states = [
        ("approve_execution_runner_dispatch_ready", dispatch_ready),
        (
            "approve_execution_runner_route_dispatch_enabled",
            route_dispatch_enabled,
        ),
    ];
    let mut passed_gates = Vec::new();
    let mut blocked_gates = Vec::new();
    for (gate, passed) in gate_states {
        if passed {
            passed_gates.push(gate.to_string());
        } else {
            blocked_gates.push(gate.to_string());
        }
    }

    ContractRepairApprovalApproveExecutionRunnerHandoffReadinessDryRun {
        status: if handoff_ready {
            "approve_execution_runner_handoff_readiness_ready".to_string()
        } else if handoff.action == "approve" {
            "approve_execution_runner_handoff_readiness_ready_blocked".to_string()
        } else {
            "approve_execution_runner_handoff_readiness_blocked".to_string()
        },
        action: handoff.action.clone(),
        approval_id: handoff.approval_id.clone(),
        gate_name: "approve_execution_runner_handoff_ready".to_string(),
        dispatch_ready,
        route_dispatch_enabled,
        dispatch_readiness_ready,
        handoff_ready,
        expected_http_status: handoff.expected_http_status,
        expected_route_status: handoff.expected_route_status.clone(),
        would_call_runner: false,
        would_return_success: false,
        would_persist_any_side_effect: false,
        would_touch_disk: false,
        would_unblock_call: false,
        would_unblock_control_readiness: false,
        inherited_handoff_blockers: handoff.blocked_by.clone(),
        inherited_dispatch_readiness_blocked_gates: dispatch_readiness_dry_run
            .blocked_gates
            .clone(),
        required_gates,
        passed_gates,
        blocked_gates,
    }
}

fn contract_repair_approval_approve_execution_runner_call_enablement_dry_run(
    handoff: &ContractRepairApprovalApproveExecutionRunnerHandoff,
    route_dispatch_enablement_dry_run: &ContractRepairApprovalApproveExecutionRunnerRouteDispatchEnablementDryRun,
    runner_call_enabled: bool,
) -> ContractRepairApprovalApproveExecutionRunnerCallEnablementDryRun {
    let handoff_ready = handoff.handoff_ready;
    let route_dispatch_enablement_ready =
        route_dispatch_enablement_dry_run.route_dispatch_enablement_ready;
    let enablement_prerequisites_ready = handoff_ready && route_dispatch_enablement_ready;
    let runner_call_enablement_ready = enablement_prerequisites_ready && runner_call_enabled;
    let required_gates = vec![
        "approve_execution_runner_handoff_ready".to_string(),
        "approve_execution_runner_route_dispatch_enablement_ready".to_string(),
        "approve_execution_runner_call_enabled".to_string(),
    ];
    let gate_states = [
        ("approve_execution_runner_handoff_ready", handoff_ready),
        (
            "approve_execution_runner_route_dispatch_enablement_ready",
            route_dispatch_enablement_ready,
        ),
        ("approve_execution_runner_call_enabled", runner_call_enabled),
    ];
    let mut passed_gates = Vec::new();
    let mut blocked_gates = Vec::new();
    for (gate, passed) in gate_states {
        if passed {
            passed_gates.push(gate.to_string());
        } else {
            blocked_gates.push(gate.to_string());
        }
    }

    ContractRepairApprovalApproveExecutionRunnerCallEnablementDryRun {
        status: if runner_call_enablement_ready {
            "approve_execution_runner_call_enablement_ready".to_string()
        } else if handoff.action == "approve" {
            "approve_execution_runner_call_enablement_ready_blocked".to_string()
        } else {
            "approve_execution_runner_call_enablement_blocked".to_string()
        },
        action: handoff.action.clone(),
        approval_id: handoff.approval_id.clone(),
        switch_name: "approve_execution_runner_call_enabled".to_string(),
        handoff_ready,
        route_dispatch_enablement_ready,
        runner_call_enabled,
        enablement_prerequisites_ready,
        runner_call_enablement_ready,
        would_enable_runner_call: false,
        would_call_runner: false,
        would_return_success: false,
        would_persist_any_side_effect: false,
        would_commit_transaction: false,
        would_rollback_on_error: false,
        would_touch_disk: false,
        required_gates,
        passed_gates,
        blocked_gates,
    }
}

fn contract_repair_approval_approve_execution_runner_call_dry_run(
    handoff: &ContractRepairApprovalApproveExecutionRunnerHandoff,
    runner_call_enabled: bool,
) -> ContractRepairApprovalApproveExecutionRunnerCallDryRun {
    let handoff_ready = handoff.handoff_ready;
    let call_ready = handoff_ready && runner_call_enabled;
    let mut blocked_by = Vec::new();
    if !handoff_ready {
        blocked_by.push("approve_execution_runner_handoff_ready".to_string());
    }
    if !runner_call_enabled {
        blocked_by.push("approve_execution_runner_call_enabled".to_string());
    }
    for gate in &handoff.blocked_by {
        if !blocked_by.contains(gate) {
            blocked_by.push(gate.clone());
        }
    }

    ContractRepairApprovalApproveExecutionRunnerCallDryRun {
        status: if call_ready {
            "approve_execution_runner_call_ready".to_string()
        } else if handoff.action == "approve" {
            "approve_execution_runner_call_ready_blocked".to_string()
        } else {
            "approve_execution_runner_call_blocked".to_string()
        },
        action: handoff.action.clone(),
        approval_id: handoff.approval_id.clone(),
        handoff_ready,
        runner_call_enabled,
        call_ready,
        expected_runner_result: if call_ready {
            "approve_execution_committed".to_string()
        } else {
            "not_invoked".to_string()
        },
        would_call_runner: false,
        would_return_success: false,
        would_persist_any_side_effect: false,
        would_commit_transaction: false,
        would_rollback_on_error: false,
        would_touch_disk: false,
        blocked_by,
    }
}

fn contract_repair_approval_approve_execution_runner_call_readiness_dry_run(
    call_dry_run: &ContractRepairApprovalApproveExecutionRunnerCallDryRun,
    call_enablement_dry_run: &ContractRepairApprovalApproveExecutionRunnerCallEnablementDryRun,
) -> ContractRepairApprovalApproveExecutionRunnerCallReadinessDryRun {
    let handoff_ready = call_dry_run.handoff_ready;
    let runner_call_enabled = call_dry_run.runner_call_enabled;
    let runner_call_enablement_ready = call_enablement_dry_run.runner_call_enablement_ready;
    let call_ready = call_dry_run.call_ready;
    let required_gates = vec![
        "approve_execution_runner_handoff_ready".to_string(),
        "approve_execution_runner_call_enabled".to_string(),
    ];
    let gate_states = [
        ("approve_execution_runner_handoff_ready", handoff_ready),
        ("approve_execution_runner_call_enabled", runner_call_enabled),
    ];
    let mut passed_gates = Vec::new();
    let mut blocked_gates = Vec::new();
    for (gate, passed) in gate_states {
        if passed {
            passed_gates.push(gate.to_string());
        } else {
            blocked_gates.push(gate.to_string());
        }
    }

    ContractRepairApprovalApproveExecutionRunnerCallReadinessDryRun {
        status: if call_ready {
            "approve_execution_runner_call_readiness_ready".to_string()
        } else if call_dry_run.action == "approve" {
            "approve_execution_runner_call_readiness_ready_blocked".to_string()
        } else {
            "approve_execution_runner_call_readiness_blocked".to_string()
        },
        action: call_dry_run.action.clone(),
        approval_id: call_dry_run.approval_id.clone(),
        gate_name: "approve_execution_runner_call_ready".to_string(),
        handoff_ready,
        runner_call_enabled,
        runner_call_enablement_ready,
        call_ready,
        expected_runner_result: call_dry_run.expected_runner_result.clone(),
        would_call_runner: false,
        would_return_success: false,
        would_persist_any_side_effect: false,
        would_commit_transaction: false,
        would_rollback_on_error: false,
        would_touch_disk: false,
        would_unblock_body: false,
        would_unblock_control_readiness: false,
        inherited_call_blockers: call_dry_run.blocked_by.clone(),
        inherited_call_enablement_blocked_gates: call_enablement_dry_run.blocked_gates.clone(),
        required_gates,
        passed_gates,
        blocked_gates,
    }
}

fn contract_repair_approval_approve_execution_runner_body_enablement_dry_run(
    call_dry_run: &ContractRepairApprovalApproveExecutionRunnerCallDryRun,
    atomic_side_effects_gate: &ContractRepairApprovalApproveExecutionAtomicSideEffectsGate,
    runner_body_enabled: bool,
) -> ContractRepairApprovalApproveExecutionRunnerBodyEnablementDryRun {
    let call_ready = call_dry_run.call_ready;
    let side_effect_bundle_ready = atomic_side_effects_gate.atomic_side_effects_plan_ready;
    let atomic_side_effects_ready = atomic_side_effects_gate.atomic_side_effects_ready;
    let enablement_prerequisites_ready =
        call_ready && side_effect_bundle_ready && atomic_side_effects_ready;
    let runner_body_enablement_ready = enablement_prerequisites_ready && runner_body_enabled;
    let required_gates = vec![
        "approve_execution_runner_call_ready".to_string(),
        "approve_execution_side_effect_bundle_ready".to_string(),
        "approve_execution_atomic_side_effects_ready".to_string(),
        "approve_execution_runner_body_enabled".to_string(),
    ];
    let gate_states = [
        ("approve_execution_runner_call_ready", call_ready),
        (
            "approve_execution_side_effect_bundle_ready",
            side_effect_bundle_ready,
        ),
        (
            "approve_execution_atomic_side_effects_ready",
            atomic_side_effects_ready,
        ),
        ("approve_execution_runner_body_enabled", runner_body_enabled),
    ];
    let mut passed_gates = Vec::new();
    let mut blocked_gates = Vec::new();
    for (gate, passed) in gate_states {
        if passed {
            passed_gates.push(gate.to_string());
        } else {
            blocked_gates.push(gate.to_string());
        }
    }

    ContractRepairApprovalApproveExecutionRunnerBodyEnablementDryRun {
        status: if runner_body_enablement_ready {
            "approve_execution_runner_body_enablement_ready".to_string()
        } else if call_dry_run.action == "approve" {
            "approve_execution_runner_body_enablement_ready_blocked".to_string()
        } else {
            "approve_execution_runner_body_enablement_blocked".to_string()
        },
        action: call_dry_run.action.clone(),
        approval_id: call_dry_run.approval_id.clone(),
        switch_name: "approve_execution_runner_body_enabled".to_string(),
        call_ready,
        side_effect_bundle_ready,
        atomic_side_effects_ready,
        runner_body_enabled,
        enablement_prerequisites_ready,
        runner_body_enablement_ready,
        would_enable_runner_body: false,
        would_enter_body: false,
        would_emit_lifecycle: false,
        would_mutate_contract: false,
        would_persist_recovery_marker: false,
        would_commit_transaction: false,
        would_rollback_on_error: false,
        would_return_success: false,
        would_touch_disk: false,
        required_gates,
        passed_gates,
        blocked_gates,
    }
}

fn contract_repair_approval_approve_execution_runner_call_body_dry_run(
    call_dry_run: &ContractRepairApprovalApproveExecutionRunnerCallDryRun,
    atomic_side_effects_gate: &ContractRepairApprovalApproveExecutionAtomicSideEffectsGate,
    runner_body_enabled: bool,
) -> ContractRepairApprovalApproveExecutionRunnerCallBodyDryRun {
    let call_ready = call_dry_run.call_ready;
    let side_effect_bundle_ready = atomic_side_effects_gate.atomic_side_effects_plan_ready;
    let atomic_side_effects_ready = atomic_side_effects_gate.atomic_side_effects_ready;
    let body_ready =
        call_ready && side_effect_bundle_ready && atomic_side_effects_ready && runner_body_enabled;
    let mut blocked_by = Vec::new();
    if !call_ready {
        blocked_by.push("approve_execution_runner_call_ready".to_string());
    }
    if !side_effect_bundle_ready {
        blocked_by.push("approve_execution_side_effect_bundle_ready".to_string());
    }
    if !atomic_side_effects_ready {
        blocked_by.push("approve_execution_atomic_side_effects_ready".to_string());
    }
    if !runner_body_enabled {
        blocked_by.push("approve_execution_runner_body_enabled".to_string());
    }
    for gate in &call_dry_run.blocked_by {
        if !blocked_by.contains(gate) {
            blocked_by.push(gate.clone());
        }
    }

    ContractRepairApprovalApproveExecutionRunnerCallBodyDryRun {
        status: if body_ready {
            "approve_execution_runner_call_body_ready".to_string()
        } else if call_dry_run.action == "approve" {
            "approve_execution_runner_call_body_ready_blocked".to_string()
        } else {
            "approve_execution_runner_call_body_blocked".to_string()
        },
        action: call_dry_run.action.clone(),
        approval_id: call_dry_run.approval_id.clone(),
        runner_entrypoint: "contract_repair_approval_approve_execution_runner".to_string(),
        call_ready,
        side_effect_bundle_ready,
        atomic_side_effects_ready,
        runner_body_enabled,
        body_ready,
        would_enter_body: false,
        would_emit_lifecycle: false,
        would_mutate_contract: false,
        would_persist_recovery_marker: false,
        would_commit_transaction: false,
        would_rollback_on_error: false,
        would_return_success: false,
        would_touch_disk: false,
        blocked_by,
    }
}

fn contract_repair_approval_approve_execution_runner_body_readiness_dry_run(
    body_dry_run: &ContractRepairApprovalApproveExecutionRunnerCallBodyDryRun,
    body_enablement_dry_run: &ContractRepairApprovalApproveExecutionRunnerBodyEnablementDryRun,
) -> ContractRepairApprovalApproveExecutionRunnerBodyReadinessDryRun {
    let call_ready = body_dry_run.call_ready;
    let side_effect_bundle_ready = body_dry_run.side_effect_bundle_ready;
    let atomic_side_effects_ready = body_dry_run.atomic_side_effects_ready;
    let runner_body_enabled = body_dry_run.runner_body_enabled;
    let runner_body_enablement_ready = body_enablement_dry_run.runner_body_enablement_ready;
    let body_ready = body_dry_run.body_ready;
    let required_gates = vec![
        "approve_execution_runner_call_ready".to_string(),
        "approve_execution_side_effect_bundle_ready".to_string(),
        "approve_execution_atomic_side_effects_ready".to_string(),
        "approve_execution_runner_body_enabled".to_string(),
    ];
    let gate_states = [
        ("approve_execution_runner_call_ready", call_ready),
        (
            "approve_execution_side_effect_bundle_ready",
            side_effect_bundle_ready,
        ),
        (
            "approve_execution_atomic_side_effects_ready",
            atomic_side_effects_ready,
        ),
        ("approve_execution_runner_body_enabled", runner_body_enabled),
    ];
    let mut passed_gates = Vec::new();
    let mut blocked_gates = Vec::new();
    for (gate, passed) in gate_states {
        if passed {
            passed_gates.push(gate.to_string());
        } else {
            blocked_gates.push(gate.to_string());
        }
    }

    ContractRepairApprovalApproveExecutionRunnerBodyReadinessDryRun {
        status: if body_ready {
            "approve_execution_runner_body_readiness_ready".to_string()
        } else if body_dry_run.action == "approve" {
            "approve_execution_runner_body_readiness_ready_blocked".to_string()
        } else {
            "approve_execution_runner_body_readiness_blocked".to_string()
        },
        action: body_dry_run.action.clone(),
        approval_id: body_dry_run.approval_id.clone(),
        gate_name: "approve_execution_runner_body_ready".to_string(),
        runner_entrypoint: body_dry_run.runner_entrypoint.clone(),
        call_ready,
        side_effect_bundle_ready,
        atomic_side_effects_ready,
        runner_body_enabled,
        runner_body_enablement_ready,
        body_ready,
        would_enter_body: false,
        would_emit_lifecycle: false,
        would_mutate_contract: false,
        would_persist_recovery_marker: false,
        would_commit_transaction: false,
        would_rollback_on_error: false,
        would_return_success: false,
        would_touch_disk: false,
        would_unblock_phase_sequence: false,
        would_unblock_control_readiness: false,
        inherited_body_blockers: body_dry_run.blocked_by.clone(),
        inherited_body_enablement_blocked_gates: body_enablement_dry_run.blocked_gates.clone(),
        required_gates,
        passed_gates,
        blocked_gates,
    }
}

fn contract_repair_approval_approve_execution_runner_phase_execution_enablement_dry_run(
    body_dry_run: &ContractRepairApprovalApproveExecutionRunnerCallBodyDryRun,
    runner_dry_run: &ContractRepairApprovalApproveExecutionTransactionRunnerDryRun,
    phase_execution_enabled: bool,
) -> ContractRepairApprovalApproveExecutionRunnerPhaseExecutionEnablementDryRun {
    let phase_order = runner_dry_run.phase_order.clone();
    let rollback_order = runner_dry_run.rollback_order.clone();
    let body_ready = body_dry_run.body_ready;
    let phase_sequence_ready = body_dry_run.side_effect_bundle_ready
        && phase_order
            .iter()
            .any(|phase| phase == "write_recovery_marker")
        && phase_order
            .iter()
            .any(|phase| phase == "emit_lifecycle_event")
        && phase_order
            .iter()
            .any(|phase| phase == "write_contract_source")
        && phase_order
            .iter()
            .any(|phase| phase == "clear_recovery_marker");
    let rollback_sequence_ready = rollback_order
        .iter()
        .any(|phase| phase == "restore_contract_source")
        && rollback_order
            .iter()
            .any(|phase| phase == "restore_approval_record")
        && rollback_order
            .iter()
            .any(|phase| phase == "mark_recovery_marker_rolled_back");
    let enablement_prerequisites_ready =
        body_ready && phase_sequence_ready && rollback_sequence_ready;
    let phase_execution_enablement_ready =
        enablement_prerequisites_ready && phase_execution_enabled;
    let required_gates = vec![
        "approve_execution_runner_body_ready".to_string(),
        "approve_execution_runner_phase_sequence_ready".to_string(),
        "approve_execution_runner_rollback_sequence_ready".to_string(),
        "approve_execution_runner_phase_execution_enabled".to_string(),
    ];
    let gate_states = [
        ("approve_execution_runner_body_ready", body_ready),
        (
            "approve_execution_runner_phase_sequence_ready",
            phase_sequence_ready,
        ),
        (
            "approve_execution_runner_rollback_sequence_ready",
            rollback_sequence_ready,
        ),
        (
            "approve_execution_runner_phase_execution_enabled",
            phase_execution_enabled,
        ),
    ];
    let mut passed_gates = Vec::new();
    let mut blocked_gates = Vec::new();
    for (gate, passed) in gate_states {
        if passed {
            passed_gates.push(gate.to_string());
        } else {
            blocked_gates.push(gate.to_string());
        }
    }

    ContractRepairApprovalApproveExecutionRunnerPhaseExecutionEnablementDryRun {
        status: if phase_execution_enablement_ready {
            "approve_execution_runner_phase_execution_enablement_ready".to_string()
        } else if body_dry_run.action == "approve" {
            "approve_execution_runner_phase_execution_enablement_ready_blocked".to_string()
        } else {
            "approve_execution_runner_phase_execution_enablement_blocked".to_string()
        },
        action: body_dry_run.action.clone(),
        approval_id: body_dry_run.approval_id.clone(),
        switch_name: "approve_execution_runner_phase_execution_enabled".to_string(),
        body_ready,
        phase_sequence_ready,
        rollback_sequence_ready,
        phase_execution_enabled,
        enablement_prerequisites_ready,
        phase_execution_enablement_ready,
        phase_order,
        rollback_order,
        would_enable_phase_execution: false,
        would_execute_phase_sequence: false,
        would_execute_rollback_sequence: false,
        would_return_success: false,
        would_touch_disk: false,
        required_gates,
        passed_gates,
        blocked_gates,
    }
}

fn contract_repair_approval_approve_execution_runner_body_phase_sequence_dry_run(
    body_dry_run: &ContractRepairApprovalApproveExecutionRunnerCallBodyDryRun,
    runner_dry_run: &ContractRepairApprovalApproveExecutionTransactionRunnerDryRun,
    phase_execution_enabled: bool,
) -> ContractRepairApprovalApproveExecutionRunnerBodyPhaseSequenceDryRun {
    let phase_order = runner_dry_run.phase_order.clone();
    let rollback_order = runner_dry_run.rollback_order.clone();
    let body_ready = body_dry_run.body_ready;
    let phase_sequence_ready = body_dry_run.side_effect_bundle_ready
        && phase_order
            .iter()
            .any(|phase| phase == "write_recovery_marker")
        && phase_order
            .iter()
            .any(|phase| phase == "emit_lifecycle_event")
        && phase_order
            .iter()
            .any(|phase| phase == "write_contract_source")
        && phase_order
            .iter()
            .any(|phase| phase == "clear_recovery_marker");
    let rollback_sequence_ready = rollback_order
        .iter()
        .any(|phase| phase == "restore_contract_source")
        && rollback_order
            .iter()
            .any(|phase| phase == "restore_approval_record")
        && rollback_order
            .iter()
            .any(|phase| phase == "mark_recovery_marker_rolled_back");
    let phases_ready =
        body_ready && phase_sequence_ready && rollback_sequence_ready && phase_execution_enabled;
    let mut blocked_by = Vec::new();
    if !body_ready {
        blocked_by.push("approve_execution_runner_body_ready".to_string());
    }
    if !phase_sequence_ready {
        blocked_by.push("approve_execution_runner_phase_sequence_ready".to_string());
    }
    if !rollback_sequence_ready {
        blocked_by.push("approve_execution_runner_rollback_sequence_ready".to_string());
    }
    if !phase_execution_enabled {
        blocked_by.push("approve_execution_runner_phase_execution_enabled".to_string());
    }
    for gate in &body_dry_run.blocked_by {
        if !blocked_by.contains(gate) {
            blocked_by.push(gate.clone());
        }
    }

    ContractRepairApprovalApproveExecutionRunnerBodyPhaseSequenceDryRun {
        status: if phases_ready {
            "approve_execution_runner_body_phase_sequence_ready".to_string()
        } else if body_dry_run.action == "approve"
            && phase_sequence_ready
            && rollback_sequence_ready
        {
            "approve_execution_runner_body_phase_sequence_ready_blocked".to_string()
        } else {
            "approve_execution_runner_body_phase_sequence_blocked".to_string()
        },
        action: body_dry_run.action.clone(),
        approval_id: body_dry_run.approval_id.clone(),
        body_ready,
        phase_sequence_ready,
        rollback_sequence_ready,
        phase_execution_enabled,
        phases_ready,
        phase_order,
        rollback_order,
        would_execute_phase_sequence: false,
        would_execute_rollback_sequence: false,
        would_return_success: false,
        would_touch_disk: false,
        blocked_by,
    }
}

fn contract_repair_approval_approve_execution_runner_phases_readiness_dry_run(
    phase_sequence_dry_run: &ContractRepairApprovalApproveExecutionRunnerBodyPhaseSequenceDryRun,
    phase_execution_enablement_dry_run: &ContractRepairApprovalApproveExecutionRunnerPhaseExecutionEnablementDryRun,
) -> ContractRepairApprovalApproveExecutionRunnerPhasesReadinessDryRun {
    let body_ready = phase_sequence_dry_run.body_ready;
    let phase_sequence_ready = phase_sequence_dry_run.phase_sequence_ready;
    let rollback_sequence_ready = phase_sequence_dry_run.rollback_sequence_ready;
    let phase_execution_enabled = phase_sequence_dry_run.phase_execution_enabled;
    let phase_execution_enablement_ready =
        phase_execution_enablement_dry_run.phase_execution_enablement_ready;
    let phases_ready = phase_sequence_dry_run.phases_ready;
    let required_gates = vec![
        "approve_execution_runner_body_ready".to_string(),
        "approve_execution_runner_phase_sequence_ready".to_string(),
        "approve_execution_runner_rollback_sequence_ready".to_string(),
        "approve_execution_runner_phase_execution_enabled".to_string(),
    ];
    let gate_states = [
        ("approve_execution_runner_body_ready", body_ready),
        (
            "approve_execution_runner_phase_sequence_ready",
            phase_sequence_ready,
        ),
        (
            "approve_execution_runner_rollback_sequence_ready",
            rollback_sequence_ready,
        ),
        (
            "approve_execution_runner_phase_execution_enabled",
            phase_execution_enabled,
        ),
    ];
    let mut passed_gates = Vec::new();
    let mut blocked_gates = Vec::new();
    for (gate, passed) in gate_states {
        if passed {
            passed_gates.push(gate.to_string());
        } else {
            blocked_gates.push(gate.to_string());
        }
    }

    ContractRepairApprovalApproveExecutionRunnerPhasesReadinessDryRun {
        status: if phases_ready {
            "approve_execution_runner_phases_readiness_ready".to_string()
        } else if phase_sequence_dry_run.action == "approve"
            && phase_sequence_ready
            && rollback_sequence_ready
        {
            "approve_execution_runner_phases_readiness_ready_blocked".to_string()
        } else {
            "approve_execution_runner_phases_readiness_blocked".to_string()
        },
        action: phase_sequence_dry_run.action.clone(),
        approval_id: phase_sequence_dry_run.approval_id.clone(),
        gate_name: "approve_execution_runner_phases_ready".to_string(),
        body_ready,
        phase_sequence_ready,
        rollback_sequence_ready,
        phase_execution_enabled,
        phase_execution_enablement_ready,
        phases_ready,
        phase_order: phase_sequence_dry_run.phase_order.clone(),
        rollback_order: phase_sequence_dry_run.rollback_order.clone(),
        would_execute_phase_sequence: false,
        would_execute_rollback_sequence: false,
        would_return_success: false,
        would_touch_disk: false,
        would_unblock_lifecycle_phase: false,
        would_unblock_control_readiness: false,
        inherited_phase_sequence_blockers: phase_sequence_dry_run.blocked_by.clone(),
        inherited_phase_execution_enablement_blocked_gates: phase_execution_enablement_dry_run
            .blocked_gates
            .clone(),
        required_gates,
        passed_gates,
        blocked_gates,
    }
}

fn contract_repair_approval_approve_execution_runner_lifecycle_phase_enablement_dry_run(
    phase_sequence_dry_run: &ContractRepairApprovalApproveExecutionRunnerBodyPhaseSequenceDryRun,
    phase_execution_enablement_dry_run: &ContractRepairApprovalApproveExecutionRunnerPhaseExecutionEnablementDryRun,
    lifecycle_enablement_gate: &ContractRepairApprovalLifecycleEmissionEnablementGate,
    lifecycle_phase_enabled: bool,
) -> ContractRepairApprovalApproveExecutionRunnerLifecyclePhaseEnablementDryRun {
    let phase_sequence_ready = phase_sequence_dry_run.phase_sequence_ready;
    let phase_execution_enablement_ready =
        phase_execution_enablement_dry_run.phase_execution_enablement_ready;
    let lifecycle_phase_present = phase_sequence_dry_run
        .phase_order
        .iter()
        .any(|phase| phase == "emit_lifecycle_event")
        && phase_sequence_dry_run
            .phase_order
            .iter()
            .any(|phase| phase == "append_lifecycle_entry");
    let lifecycle_plan_ready = lifecycle_enablement_gate.lifecycle_emission_plan_ready;
    let lifecycle_effects_ready = lifecycle_enablement_gate.lifecycle_effects_ready;
    let enablement_prerequisites_ready = phase_sequence_ready
        && phase_execution_enablement_ready
        && lifecycle_phase_present
        && lifecycle_plan_ready
        && lifecycle_effects_ready;
    let lifecycle_phase_enablement_ready =
        enablement_prerequisites_ready && lifecycle_phase_enabled;
    let required_gates = vec![
        "approve_execution_runner_phase_sequence_ready".to_string(),
        "approve_execution_runner_phase_execution_enablement_ready".to_string(),
        "approve_execution_runner_lifecycle_phase_present".to_string(),
        "lifecycle_emission_plan_ready".to_string(),
        "lifecycle_effects_ready".to_string(),
        "approve_execution_runner_lifecycle_phase_enabled".to_string(),
    ];
    let gate_states = [
        (
            "approve_execution_runner_phase_sequence_ready",
            phase_sequence_ready,
        ),
        (
            "approve_execution_runner_phase_execution_enablement_ready",
            phase_execution_enablement_ready,
        ),
        (
            "approve_execution_runner_lifecycle_phase_present",
            lifecycle_phase_present,
        ),
        ("lifecycle_emission_plan_ready", lifecycle_plan_ready),
        ("lifecycle_effects_ready", lifecycle_effects_ready),
        (
            "approve_execution_runner_lifecycle_phase_enabled",
            lifecycle_phase_enabled,
        ),
    ];
    let mut passed_gates = Vec::new();
    let mut blocked_gates = Vec::new();
    for (gate, passed) in gate_states {
        if passed {
            passed_gates.push(gate.to_string());
        } else {
            blocked_gates.push(gate.to_string());
        }
    }

    ContractRepairApprovalApproveExecutionRunnerLifecyclePhaseEnablementDryRun {
        status: if lifecycle_phase_enablement_ready {
            "approve_execution_runner_lifecycle_phase_enablement_ready".to_string()
        } else if phase_sequence_dry_run.action == "approve" {
            "approve_execution_runner_lifecycle_phase_enablement_ready_blocked".to_string()
        } else {
            "approve_execution_runner_lifecycle_phase_enablement_blocked".to_string()
        },
        action: phase_sequence_dry_run.action.clone(),
        approval_id: phase_sequence_dry_run.approval_id.clone(),
        event_id: lifecycle_enablement_gate.event_id.clone(),
        switch_name: "approve_execution_runner_lifecycle_phase_enabled".to_string(),
        phase_sequence_ready,
        phase_execution_enablement_ready,
        lifecycle_phase_present,
        lifecycle_plan_ready,
        lifecycle_effects_ready,
        lifecycle_phase_enabled,
        enablement_prerequisites_ready,
        lifecycle_phase_enablement_ready,
        would_enable_lifecycle_phase: false,
        would_emit_lifecycle: false,
        would_append_lifecycle: false,
        would_touch_lifecycle_log: false,
        would_continue_to_next_phase: false,
        would_return_success: false,
        would_touch_disk: false,
        required_gates,
        passed_gates,
        blocked_gates,
    }
}

fn contract_repair_approval_approve_execution_runner_lifecycle_phase_dry_run(
    phase_sequence_dry_run: &ContractRepairApprovalApproveExecutionRunnerBodyPhaseSequenceDryRun,
    lifecycle_enablement_gate: &ContractRepairApprovalLifecycleEmissionEnablementGate,
    lifecycle_phase_enabled: bool,
) -> ContractRepairApprovalApproveExecutionRunnerLifecyclePhaseDryRun {
    let phase_sequence_ready = phase_sequence_dry_run.phase_sequence_ready;
    let lifecycle_phase_present = phase_sequence_dry_run
        .phase_order
        .iter()
        .any(|phase| phase == "emit_lifecycle_event")
        && phase_sequence_dry_run
            .phase_order
            .iter()
            .any(|phase| phase == "append_lifecycle_entry");
    let lifecycle_plan_ready = lifecycle_enablement_gate.lifecycle_emission_plan_ready;
    let lifecycle_effects_ready = lifecycle_enablement_gate.lifecycle_effects_ready;
    let lifecycle_phase_ready = phase_sequence_ready
        && lifecycle_phase_present
        && lifecycle_plan_ready
        && lifecycle_effects_ready
        && lifecycle_phase_enabled;
    let mut blocked_by = Vec::new();
    if !phase_sequence_ready {
        blocked_by.push("approve_execution_runner_phase_sequence_ready".to_string());
    }
    if !lifecycle_phase_present {
        blocked_by.push("approve_execution_runner_lifecycle_phase_present".to_string());
    }
    if !lifecycle_plan_ready {
        blocked_by.push("lifecycle_emission_plan_ready".to_string());
    }
    if !lifecycle_effects_ready {
        blocked_by.push("lifecycle_effects_ready".to_string());
    }
    if !lifecycle_phase_enabled {
        blocked_by.push("approve_execution_runner_lifecycle_phase_enabled".to_string());
    }
    for gate in &phase_sequence_dry_run.blocked_by {
        if !blocked_by.contains(gate) {
            blocked_by.push(gate.clone());
        }
    }

    ContractRepairApprovalApproveExecutionRunnerLifecyclePhaseDryRun {
        status: if lifecycle_phase_ready {
            "approve_execution_runner_lifecycle_phase_ready".to_string()
        } else if phase_sequence_dry_run.action == "approve"
            && lifecycle_phase_present
            && lifecycle_plan_ready
        {
            "approve_execution_runner_lifecycle_phase_ready_blocked".to_string()
        } else {
            "approve_execution_runner_lifecycle_phase_blocked".to_string()
        },
        action: phase_sequence_dry_run.action.clone(),
        approval_id: phase_sequence_dry_run.approval_id.clone(),
        event_id: lifecycle_enablement_gate.event_id.clone(),
        phase_sequence_ready,
        lifecycle_phase_present,
        lifecycle_plan_ready,
        lifecycle_effects_ready,
        lifecycle_phase_enabled,
        lifecycle_phase_ready,
        would_emit_lifecycle: false,
        would_append_lifecycle: false,
        would_touch_lifecycle_log: false,
        would_continue_to_next_phase: false,
        would_return_success: false,
        would_touch_disk: false,
        blocked_by,
    }
}

fn contract_repair_approval_approve_execution_runner_lifecycle_phase_readiness_dry_run(
    lifecycle_phase_dry_run: &ContractRepairApprovalApproveExecutionRunnerLifecyclePhaseDryRun,
    lifecycle_phase_enablement_dry_run: &ContractRepairApprovalApproveExecutionRunnerLifecyclePhaseEnablementDryRun,
) -> ContractRepairApprovalApproveExecutionRunnerLifecyclePhaseReadinessDryRun {
    let phase_sequence_ready = lifecycle_phase_dry_run.phase_sequence_ready;
    let phase_execution_enablement_ready =
        lifecycle_phase_enablement_dry_run.phase_execution_enablement_ready;
    let lifecycle_phase_present = lifecycle_phase_dry_run.lifecycle_phase_present;
    let lifecycle_plan_ready = lifecycle_phase_dry_run.lifecycle_plan_ready;
    let lifecycle_effects_ready = lifecycle_phase_dry_run.lifecycle_effects_ready;
    let lifecycle_phase_enabled = lifecycle_phase_dry_run.lifecycle_phase_enabled;
    let lifecycle_phase_enablement_ready =
        lifecycle_phase_enablement_dry_run.lifecycle_phase_enablement_ready;
    let lifecycle_phase_ready = lifecycle_phase_dry_run.lifecycle_phase_ready;
    let required_gates = vec![
        "approve_execution_runner_phase_sequence_ready".to_string(),
        "approve_execution_runner_lifecycle_phase_present".to_string(),
        "lifecycle_emission_plan_ready".to_string(),
        "lifecycle_effects_ready".to_string(),
        "approve_execution_runner_lifecycle_phase_enabled".to_string(),
    ];
    let gate_states = [
        (
            "approve_execution_runner_phase_sequence_ready",
            phase_sequence_ready,
        ),
        (
            "approve_execution_runner_lifecycle_phase_present",
            lifecycle_phase_present,
        ),
        ("lifecycle_emission_plan_ready", lifecycle_plan_ready),
        ("lifecycle_effects_ready", lifecycle_effects_ready),
        (
            "approve_execution_runner_lifecycle_phase_enabled",
            lifecycle_phase_enabled,
        ),
    ];
    let mut passed_gates = Vec::new();
    let mut blocked_gates = Vec::new();
    for (gate, passed) in gate_states {
        if passed {
            passed_gates.push(gate.to_string());
        } else {
            blocked_gates.push(gate.to_string());
        }
    }

    ContractRepairApprovalApproveExecutionRunnerLifecyclePhaseReadinessDryRun {
        status: if lifecycle_phase_ready {
            "approve_execution_runner_lifecycle_phase_readiness_ready".to_string()
        } else if lifecycle_phase_dry_run.action == "approve"
            && lifecycle_phase_present
            && lifecycle_plan_ready
        {
            "approve_execution_runner_lifecycle_phase_readiness_ready_blocked".to_string()
        } else {
            "approve_execution_runner_lifecycle_phase_readiness_blocked".to_string()
        },
        action: lifecycle_phase_dry_run.action.clone(),
        approval_id: lifecycle_phase_dry_run.approval_id.clone(),
        event_id: lifecycle_phase_dry_run.event_id.clone(),
        gate_name: "approve_execution_runner_lifecycle_phase_ready".to_string(),
        phase_sequence_ready,
        phase_execution_enablement_ready,
        lifecycle_phase_present,
        lifecycle_plan_ready,
        lifecycle_effects_ready,
        lifecycle_phase_enabled,
        lifecycle_phase_enablement_ready,
        lifecycle_phase_ready,
        would_emit_lifecycle: false,
        would_append_lifecycle: false,
        would_touch_lifecycle_log: false,
        would_continue_to_next_phase: false,
        would_return_success: false,
        would_touch_disk: false,
        would_unblock_source_mutation_phase: false,
        would_unblock_control_readiness: false,
        inherited_lifecycle_phase_blockers: lifecycle_phase_dry_run.blocked_by.clone(),
        inherited_lifecycle_phase_enablement_blocked_gates: lifecycle_phase_enablement_dry_run
            .blocked_gates
            .clone(),
        required_gates,
        passed_gates,
        blocked_gates,
    }
}

fn contract_repair_approval_approve_execution_runner_source_mutation_phase_enablement_dry_run(
    phase_sequence_dry_run: &ContractRepairApprovalApproveExecutionRunnerBodyPhaseSequenceDryRun,
    lifecycle_phase_enablement_dry_run: &ContractRepairApprovalApproveExecutionRunnerLifecyclePhaseEnablementDryRun,
    lifecycle_phase_dry_run: &ContractRepairApprovalApproveExecutionRunnerLifecyclePhaseDryRun,
    contract_mutation_gate: &ContractRepairApprovalContractMutationEnablementGate,
    source_mutation_phase_enabled: bool,
) -> ContractRepairApprovalApproveExecutionRunnerSourceMutationPhaseEnablementDryRun {
    let phase_sequence_ready = phase_sequence_dry_run.phase_sequence_ready;
    let lifecycle_phase_enablement_ready =
        lifecycle_phase_enablement_dry_run.lifecycle_phase_enablement_ready;
    let lifecycle_phase_ready = lifecycle_phase_dry_run.lifecycle_phase_ready;
    let source_mutation_phase_present = phase_sequence_dry_run
        .phase_order
        .iter()
        .any(|phase| phase == "write_contract_source");
    let writeback_plan_ready = contract_mutation_gate.writeback_plan_ready;
    let source_write_ready = contract_mutation_gate.source_write_ready;
    let lifecycle_effects_ready = contract_mutation_gate.lifecycle_effects_ready;
    let contract_mutation_api_enabled = contract_mutation_gate.contract_mutation_api_enabled;
    let contract_mutation_ready = contract_mutation_gate.mutation_ready;
    let enablement_prerequisites_ready = phase_sequence_ready
        && lifecycle_phase_enablement_ready
        && lifecycle_phase_ready
        && source_mutation_phase_present
        && writeback_plan_ready
        && source_write_ready
        && lifecycle_effects_ready
        && contract_mutation_api_enabled
        && contract_mutation_ready;
    let source_mutation_phase_enablement_ready =
        enablement_prerequisites_ready && source_mutation_phase_enabled;
    let required_gates = vec![
        "approve_execution_runner_phase_sequence_ready".to_string(),
        "approve_execution_runner_lifecycle_phase_enablement_ready".to_string(),
        "approve_execution_runner_lifecycle_phase_ready".to_string(),
        "approve_execution_runner_source_mutation_phase_present".to_string(),
        "contract_writeback_ready".to_string(),
        "contract_source_write_ready".to_string(),
        "lifecycle_effects_ready".to_string(),
        "contract_mutation_api_enabled".to_string(),
        "contract_mutation_ready".to_string(),
        "approve_execution_runner_source_mutation_phase_enabled".to_string(),
    ];
    let gate_states = [
        (
            "approve_execution_runner_phase_sequence_ready",
            phase_sequence_ready,
        ),
        (
            "approve_execution_runner_lifecycle_phase_enablement_ready",
            lifecycle_phase_enablement_ready,
        ),
        (
            "approve_execution_runner_lifecycle_phase_ready",
            lifecycle_phase_ready,
        ),
        (
            "approve_execution_runner_source_mutation_phase_present",
            source_mutation_phase_present,
        ),
        ("contract_writeback_ready", writeback_plan_ready),
        ("contract_source_write_ready", source_write_ready),
        ("lifecycle_effects_ready", lifecycle_effects_ready),
        (
            "contract_mutation_api_enabled",
            contract_mutation_api_enabled,
        ),
        ("contract_mutation_ready", contract_mutation_ready),
        (
            "approve_execution_runner_source_mutation_phase_enabled",
            source_mutation_phase_enabled,
        ),
    ];
    let mut passed_gates = Vec::new();
    let mut blocked_gates = Vec::new();
    for (gate, passed) in gate_states {
        if passed {
            passed_gates.push(gate.to_string());
        } else {
            blocked_gates.push(gate.to_string());
        }
    }

    ContractRepairApprovalApproveExecutionRunnerSourceMutationPhaseEnablementDryRun {
        status: if source_mutation_phase_enablement_ready {
            "approve_execution_runner_source_mutation_phase_enablement_ready".to_string()
        } else if phase_sequence_dry_run.action == "approve" {
            "approve_execution_runner_source_mutation_phase_enablement_ready_blocked".to_string()
        } else {
            "approve_execution_runner_source_mutation_phase_enablement_blocked".to_string()
        },
        action: phase_sequence_dry_run.action.clone(),
        approval_id: phase_sequence_dry_run.approval_id.clone(),
        target_path: contract_mutation_gate.target_path.clone(),
        target_kind: contract_mutation_gate.target_kind.clone(),
        source_path: contract_mutation_gate.source_path.clone(),
        switch_name: "approve_execution_runner_source_mutation_phase_enabled".to_string(),
        phase_sequence_ready,
        lifecycle_phase_enablement_ready,
        lifecycle_phase_ready,
        source_mutation_phase_present,
        writeback_plan_ready,
        source_write_ready,
        lifecycle_effects_ready,
        contract_mutation_api_enabled,
        contract_mutation_ready,
        source_mutation_phase_enabled,
        enablement_prerequisites_ready,
        source_mutation_phase_enablement_ready,
        would_enable_source_mutation_phase: false,
        would_mutate_contract: false,
        would_write_contract_source: false,
        would_continue_to_next_phase: false,
        would_return_success: false,
        would_touch_disk: false,
        required_gates,
        passed_gates,
        blocked_gates,
    }
}

fn contract_repair_approval_approve_execution_runner_source_mutation_phase_dry_run(
    phase_sequence_dry_run: &ContractRepairApprovalApproveExecutionRunnerBodyPhaseSequenceDryRun,
    lifecycle_phase_dry_run: &ContractRepairApprovalApproveExecutionRunnerLifecyclePhaseDryRun,
    contract_mutation_gate: &ContractRepairApprovalContractMutationEnablementGate,
    source_mutation_phase_enabled: bool,
) -> ContractRepairApprovalApproveExecutionRunnerSourceMutationPhaseDryRun {
    let phase_sequence_ready = phase_sequence_dry_run.phase_sequence_ready;
    let lifecycle_phase_ready = lifecycle_phase_dry_run.lifecycle_phase_ready;
    let source_mutation_phase_present = phase_sequence_dry_run
        .phase_order
        .iter()
        .any(|phase| phase == "write_contract_source");
    let writeback_plan_ready = contract_mutation_gate.writeback_plan_ready;
    let source_write_ready = contract_mutation_gate.source_write_ready;
    let lifecycle_effects_ready = contract_mutation_gate.lifecycle_effects_ready;
    let contract_mutation_api_enabled = contract_mutation_gate.contract_mutation_api_enabled;
    let contract_mutation_ready = contract_mutation_gate.mutation_ready;
    let source_mutation_phase_ready = phase_sequence_ready
        && lifecycle_phase_ready
        && source_mutation_phase_present
        && writeback_plan_ready
        && source_write_ready
        && lifecycle_effects_ready
        && contract_mutation_api_enabled
        && source_mutation_phase_enabled;
    let mut blocked_by = Vec::new();
    if !phase_sequence_ready {
        blocked_by.push("approve_execution_runner_phase_sequence_ready".to_string());
    }
    if !lifecycle_phase_ready {
        blocked_by.push("approve_execution_runner_lifecycle_phase_ready".to_string());
    }
    if !source_mutation_phase_present {
        blocked_by.push("approve_execution_runner_source_mutation_phase_present".to_string());
    }
    if !writeback_plan_ready {
        blocked_by.push("contract_writeback_ready".to_string());
    }
    if !source_write_ready {
        blocked_by.push("contract_source_write_ready".to_string());
    }
    if !lifecycle_effects_ready {
        blocked_by.push("lifecycle_effects_ready".to_string());
    }
    if !contract_mutation_api_enabled {
        blocked_by.push("contract_mutation_api_enabled".to_string());
    }
    if !source_mutation_phase_enabled {
        blocked_by.push("approve_execution_runner_source_mutation_phase_enabled".to_string());
    }
    for gate in &lifecycle_phase_dry_run.blocked_by {
        if !blocked_by.contains(gate) {
            blocked_by.push(gate.clone());
        }
    }

    ContractRepairApprovalApproveExecutionRunnerSourceMutationPhaseDryRun {
        status: if source_mutation_phase_ready {
            "approve_execution_runner_source_mutation_phase_ready".to_string()
        } else if phase_sequence_dry_run.action == "approve"
            && source_mutation_phase_present
            && writeback_plan_ready
            && source_write_ready
        {
            "approve_execution_runner_source_mutation_phase_ready_blocked".to_string()
        } else {
            "approve_execution_runner_source_mutation_phase_blocked".to_string()
        },
        action: phase_sequence_dry_run.action.clone(),
        approval_id: phase_sequence_dry_run.approval_id.clone(),
        target_path: contract_mutation_gate.target_path.clone(),
        target_kind: contract_mutation_gate.target_kind.clone(),
        source_path: contract_mutation_gate.source_path.clone(),
        phase_sequence_ready,
        lifecycle_phase_ready,
        source_mutation_phase_present,
        writeback_plan_ready,
        source_write_ready,
        lifecycle_effects_ready,
        contract_mutation_api_enabled,
        contract_mutation_ready,
        source_mutation_phase_enabled,
        source_mutation_phase_ready,
        would_mutate_contract: false,
        would_write_contract_source: false,
        would_continue_to_next_phase: false,
        would_return_success: false,
        would_touch_disk: false,
        blocked_by,
    }
}

fn contract_repair_approval_approve_execution_runner_source_mutation_phase_readiness_dry_run(
    source_mutation_phase_dry_run: &ContractRepairApprovalApproveExecutionRunnerSourceMutationPhaseDryRun,
    source_mutation_phase_enablement_dry_run: &ContractRepairApprovalApproveExecutionRunnerSourceMutationPhaseEnablementDryRun,
) -> ContractRepairApprovalApproveExecutionRunnerSourceMutationPhaseReadinessDryRun {
    let phase_sequence_ready = source_mutation_phase_dry_run.phase_sequence_ready;
    let lifecycle_phase_enablement_ready =
        source_mutation_phase_enablement_dry_run.lifecycle_phase_enablement_ready;
    let lifecycle_phase_ready = source_mutation_phase_dry_run.lifecycle_phase_ready;
    let source_mutation_phase_present = source_mutation_phase_dry_run.source_mutation_phase_present;
    let writeback_plan_ready = source_mutation_phase_dry_run.writeback_plan_ready;
    let source_write_ready = source_mutation_phase_dry_run.source_write_ready;
    let lifecycle_effects_ready = source_mutation_phase_dry_run.lifecycle_effects_ready;
    let contract_mutation_api_enabled = source_mutation_phase_dry_run.contract_mutation_api_enabled;
    let contract_mutation_ready = source_mutation_phase_dry_run.contract_mutation_ready;
    let source_mutation_phase_enabled = source_mutation_phase_dry_run.source_mutation_phase_enabled;
    let source_mutation_phase_enablement_ready =
        source_mutation_phase_enablement_dry_run.source_mutation_phase_enablement_ready;
    let source_mutation_phase_ready = source_mutation_phase_dry_run.source_mutation_phase_ready;
    let required_gates = vec![
        "approve_execution_runner_phase_sequence_ready".to_string(),
        "approve_execution_runner_lifecycle_phase_ready".to_string(),
        "approve_execution_runner_source_mutation_phase_present".to_string(),
        "contract_writeback_ready".to_string(),
        "contract_source_write_ready".to_string(),
        "lifecycle_effects_ready".to_string(),
        "contract_mutation_api_enabled".to_string(),
        "approve_execution_runner_source_mutation_phase_enabled".to_string(),
    ];
    let gate_states = [
        (
            "approve_execution_runner_phase_sequence_ready",
            phase_sequence_ready,
        ),
        (
            "approve_execution_runner_lifecycle_phase_ready",
            lifecycle_phase_ready,
        ),
        (
            "approve_execution_runner_source_mutation_phase_present",
            source_mutation_phase_present,
        ),
        ("contract_writeback_ready", writeback_plan_ready),
        ("contract_source_write_ready", source_write_ready),
        ("lifecycle_effects_ready", lifecycle_effects_ready),
        (
            "contract_mutation_api_enabled",
            contract_mutation_api_enabled,
        ),
        (
            "approve_execution_runner_source_mutation_phase_enabled",
            source_mutation_phase_enabled,
        ),
    ];
    let mut passed_gates = Vec::new();
    let mut blocked_gates = Vec::new();
    for (gate, passed) in gate_states {
        if passed {
            passed_gates.push(gate.to_string());
        } else {
            blocked_gates.push(gate.to_string());
        }
    }

    ContractRepairApprovalApproveExecutionRunnerSourceMutationPhaseReadinessDryRun {
        status: if source_mutation_phase_ready {
            "approve_execution_runner_source_mutation_phase_readiness_ready".to_string()
        } else if source_mutation_phase_dry_run.action == "approve"
            && source_mutation_phase_present
            && writeback_plan_ready
            && source_write_ready
        {
            "approve_execution_runner_source_mutation_phase_readiness_ready_blocked".to_string()
        } else {
            "approve_execution_runner_source_mutation_phase_readiness_blocked".to_string()
        },
        action: source_mutation_phase_dry_run.action.clone(),
        approval_id: source_mutation_phase_dry_run.approval_id.clone(),
        target_path: source_mutation_phase_dry_run.target_path.clone(),
        target_kind: source_mutation_phase_dry_run.target_kind.clone(),
        source_path: source_mutation_phase_dry_run.source_path.clone(),
        gate_name: "approve_execution_runner_source_mutation_phase_ready".to_string(),
        phase_sequence_ready,
        lifecycle_phase_enablement_ready,
        lifecycle_phase_ready,
        source_mutation_phase_present,
        writeback_plan_ready,
        source_write_ready,
        lifecycle_effects_ready,
        contract_mutation_api_enabled,
        contract_mutation_ready,
        source_mutation_phase_enabled,
        source_mutation_phase_enablement_ready,
        source_mutation_phase_ready,
        would_mutate_contract: false,
        would_write_contract_source: false,
        would_continue_to_next_phase: false,
        would_return_success: false,
        would_touch_disk: false,
        would_unblock_recovery_marker_cleanup: false,
        would_unblock_control_readiness: false,
        inherited_source_mutation_phase_blockers: source_mutation_phase_dry_run.blocked_by.clone(),
        inherited_source_mutation_phase_enablement_blocked_gates:
            source_mutation_phase_enablement_dry_run
                .blocked_gates
                .clone(),
        required_gates,
        passed_gates,
        blocked_gates,
    }
}

fn contract_repair_approval_approve_execution_runner_recovery_marker_cleanup_phase_dry_run(
    phase_sequence_dry_run: &ContractRepairApprovalApproveExecutionRunnerBodyPhaseSequenceDryRun,
    source_mutation_phase_dry_run: &ContractRepairApprovalApproveExecutionRunnerSourceMutationPhaseDryRun,
    marker_persistence_gate: &ContractRepairApprovalApproveExecutionRecoveryMarkerPersistenceGate,
    cleanup_phase_enabled: bool,
) -> ContractRepairApprovalApproveExecutionRunnerRecoveryMarkerCleanupPhaseDryRun {
    let phase_sequence_ready = phase_sequence_dry_run.phase_sequence_ready;
    let source_mutation_phase_ready = source_mutation_phase_dry_run.source_mutation_phase_ready;
    let cleanup_phase_present = phase_sequence_dry_run
        .phase_order
        .iter()
        .any(|phase| phase == "clear_recovery_marker");
    let marker_persistence_plan_ready = marker_persistence_gate.marker_persistence_plan_ready;
    let marker_persistence_ready = marker_persistence_gate.persistence_ready;
    let cleanup_phase_ready = phase_sequence_ready
        && source_mutation_phase_ready
        && cleanup_phase_present
        && marker_persistence_plan_ready
        && marker_persistence_ready
        && cleanup_phase_enabled;
    let mut blocked_by = Vec::new();
    if !phase_sequence_ready {
        blocked_by.push("approve_execution_runner_phase_sequence_ready".to_string());
    }
    if !source_mutation_phase_ready {
        blocked_by.push("approve_execution_runner_source_mutation_phase_ready".to_string());
    }
    if !cleanup_phase_present {
        blocked_by
            .push("approve_execution_runner_recovery_marker_cleanup_phase_present".to_string());
    }
    if !marker_persistence_plan_ready {
        blocked_by.push("recovery_marker_persistence_plan_ready".to_string());
    }
    if !marker_persistence_ready {
        blocked_by.push("recovery_marker_persistence_ready".to_string());
    }
    if !cleanup_phase_enabled {
        blocked_by
            .push("approve_execution_runner_recovery_marker_cleanup_phase_enabled".to_string());
    }
    for gate in &source_mutation_phase_dry_run.blocked_by {
        if !blocked_by.contains(gate) {
            blocked_by.push(gate.clone());
        }
    }

    ContractRepairApprovalApproveExecutionRunnerRecoveryMarkerCleanupPhaseDryRun {
        status: if cleanup_phase_ready {
            "approve_execution_runner_recovery_marker_cleanup_phase_ready".to_string()
        } else if phase_sequence_dry_run.action == "approve"
            && cleanup_phase_present
            && marker_persistence_plan_ready
        {
            "approve_execution_runner_recovery_marker_cleanup_phase_ready_blocked".to_string()
        } else {
            "approve_execution_runner_recovery_marker_cleanup_phase_blocked".to_string()
        },
        action: phase_sequence_dry_run.action.clone(),
        approval_id: phase_sequence_dry_run.approval_id.clone(),
        marker_key: marker_persistence_gate.marker_key.clone(),
        file_name: marker_persistence_gate.file_name.clone(),
        phase_sequence_ready,
        source_mutation_phase_ready,
        cleanup_phase_present,
        marker_persistence_plan_ready,
        marker_persistence_ready,
        cleanup_phase_enabled,
        cleanup_phase_ready,
        would_clear_recovery_marker: false,
        would_continue_to_commit: false,
        would_return_success: false,
        would_touch_disk: false,
        blocked_by,
    }
}

fn contract_repair_approval_approve_execution_runner_recovery_marker_cleanup_phase_enablement_dry_run(
    phase_sequence_dry_run: &ContractRepairApprovalApproveExecutionRunnerBodyPhaseSequenceDryRun,
    source_mutation_phase_enablement_dry_run: &ContractRepairApprovalApproveExecutionRunnerSourceMutationPhaseEnablementDryRun,
    source_mutation_phase_dry_run: &ContractRepairApprovalApproveExecutionRunnerSourceMutationPhaseDryRun,
    marker_persistence_gate: &ContractRepairApprovalApproveExecutionRecoveryMarkerPersistenceGate,
    cleanup_phase_enabled: bool,
) -> ContractRepairApprovalApproveExecutionRunnerRecoveryMarkerCleanupPhaseEnablementDryRun {
    let phase_sequence_ready = phase_sequence_dry_run.phase_sequence_ready;
    let source_mutation_phase_enablement_ready =
        source_mutation_phase_enablement_dry_run.source_mutation_phase_enablement_ready;
    let source_mutation_phase_ready = source_mutation_phase_dry_run.source_mutation_phase_ready;
    let cleanup_phase_present = phase_sequence_dry_run
        .phase_order
        .iter()
        .any(|phase| phase == "clear_recovery_marker");
    let marker_persistence_plan_ready = marker_persistence_gate.marker_persistence_plan_ready;
    let marker_persistence_ready = marker_persistence_gate.persistence_ready;
    let enablement_prerequisites_ready = phase_sequence_ready
        && source_mutation_phase_enablement_ready
        && source_mutation_phase_ready
        && cleanup_phase_present
        && marker_persistence_plan_ready
        && marker_persistence_ready;
    let cleanup_phase_enablement_ready = enablement_prerequisites_ready && cleanup_phase_enabled;
    let required_gates = vec![
        "approve_execution_runner_phase_sequence_ready".to_string(),
        "approve_execution_runner_source_mutation_phase_enablement_ready".to_string(),
        "approve_execution_runner_source_mutation_phase_ready".to_string(),
        "approve_execution_runner_recovery_marker_cleanup_phase_present".to_string(),
        "recovery_marker_persistence_plan_ready".to_string(),
        "recovery_marker_persistence_ready".to_string(),
        "approve_execution_runner_recovery_marker_cleanup_phase_enabled".to_string(),
    ];
    let gate_states = [
        (
            "approve_execution_runner_phase_sequence_ready",
            phase_sequence_ready,
        ),
        (
            "approve_execution_runner_source_mutation_phase_enablement_ready",
            source_mutation_phase_enablement_ready,
        ),
        (
            "approve_execution_runner_source_mutation_phase_ready",
            source_mutation_phase_ready,
        ),
        (
            "approve_execution_runner_recovery_marker_cleanup_phase_present",
            cleanup_phase_present,
        ),
        (
            "recovery_marker_persistence_plan_ready",
            marker_persistence_plan_ready,
        ),
        (
            "recovery_marker_persistence_ready",
            marker_persistence_ready,
        ),
        (
            "approve_execution_runner_recovery_marker_cleanup_phase_enabled",
            cleanup_phase_enabled,
        ),
    ];
    let mut passed_gates = Vec::new();
    let mut blocked_gates = Vec::new();
    for (gate, passed) in gate_states {
        if passed {
            passed_gates.push(gate.to_string());
        } else {
            blocked_gates.push(gate.to_string());
        }
    }

    ContractRepairApprovalApproveExecutionRunnerRecoveryMarkerCleanupPhaseEnablementDryRun {
        status: if cleanup_phase_enablement_ready {
            "approve_execution_runner_recovery_marker_cleanup_phase_enablement_ready".to_string()
        } else if phase_sequence_dry_run.action == "approve" {
            "approve_execution_runner_recovery_marker_cleanup_phase_enablement_ready_blocked"
                .to_string()
        } else {
            "approve_execution_runner_recovery_marker_cleanup_phase_enablement_blocked".to_string()
        },
        action: phase_sequence_dry_run.action.clone(),
        approval_id: phase_sequence_dry_run.approval_id.clone(),
        marker_key: marker_persistence_gate.marker_key.clone(),
        file_name: marker_persistence_gate.file_name.clone(),
        switch_name: "approve_execution_runner_recovery_marker_cleanup_phase_enabled".to_string(),
        phase_sequence_ready,
        source_mutation_phase_enablement_ready,
        source_mutation_phase_ready,
        cleanup_phase_present,
        marker_persistence_plan_ready,
        marker_persistence_ready,
        cleanup_phase_enabled,
        enablement_prerequisites_ready,
        cleanup_phase_enablement_ready,
        would_enable_cleanup_phase: false,
        would_clear_recovery_marker: false,
        would_continue_to_commit: false,
        would_return_success: false,
        would_touch_disk: false,
        required_gates,
        passed_gates,
        blocked_gates,
    }
}

async fn contract_repair_approval_approve_execution_runner_recovery_marker_cleanup_phase_with_gate(
    store_dir: &FsPath,
    cleanup_phase_dry_run: &ContractRepairApprovalApproveExecutionRunnerRecoveryMarkerCleanupPhaseDryRun,
    cleanup_phase_enablement_dry_run: &ContractRepairApprovalApproveExecutionRunnerRecoveryMarkerCleanupPhaseEnablementDryRun,
    cleanup_phase_enabled: bool,
) -> ContractRepairApprovalApproveExecutionRunnerRecoveryMarkerCleanupPhaseDryRun {
    let mut result = cleanup_phase_dry_run.clone();
    if !cleanup_phase_enabled {
        return result;
    }

    result.cleanup_phase_enabled = true;
    result.blocked_by.retain(|blocker| {
        blocker != "approve_execution_runner_recovery_marker_cleanup_phase_enabled"
    });

    if !cleanup_phase_enablement_dry_run.enablement_prerequisites_ready {
        push_unique_blocker(
            &mut result.blocked_by,
            "approve_execution_runner_recovery_marker_cleanup_phase_enablement_ready",
        );
    }
    if !cleanup_phase_dry_run.phase_sequence_ready {
        push_unique_blocker(
            &mut result.blocked_by,
            "approve_execution_runner_phase_sequence_ready",
        );
    }
    if !cleanup_phase_dry_run.source_mutation_phase_ready {
        push_unique_blocker(
            &mut result.blocked_by,
            "approve_execution_runner_source_mutation_phase_ready",
        );
    }
    if !cleanup_phase_dry_run.cleanup_phase_present {
        push_unique_blocker(
            &mut result.blocked_by,
            "approve_execution_runner_recovery_marker_cleanup_phase_present",
        );
    }
    if !cleanup_phase_dry_run.marker_persistence_plan_ready {
        push_unique_blocker(
            &mut result.blocked_by,
            "recovery_marker_persistence_plan_ready",
        );
    }
    if !cleanup_phase_dry_run.marker_persistence_ready {
        push_unique_blocker(&mut result.blocked_by, "recovery_marker_persistence_ready");
    }
    if !result.blocked_by.is_empty() {
        return result;
    }

    let marker_path = store_dir.join(&cleanup_phase_dry_run.file_name);
    match fs::remove_file(&marker_path).await {
        Ok(()) => {
            result.status =
                "approve_execution_runner_recovery_marker_cleanup_phase_cleared".to_string();
            result.cleanup_phase_ready = true;
            result.would_clear_recovery_marker = true;
            result.would_continue_to_commit = true;
            result.would_touch_disk = true;
            result.blocked_by.clear();
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            push_unique_blocker(&mut result.blocked_by, "recovery_marker_file_exists");
        }
        Err(_) => {
            push_unique_blocker(
                &mut result.blocked_by,
                "recovery_marker_cleanup_remove_file",
            );
        }
    }
    result
}

fn contract_repair_approval_approve_execution_runner_recovery_marker_cleanup_phase_readiness_dry_run(
    cleanup_phase_dry_run: &ContractRepairApprovalApproveExecutionRunnerRecoveryMarkerCleanupPhaseDryRun,
    cleanup_phase_enablement_dry_run: &ContractRepairApprovalApproveExecutionRunnerRecoveryMarkerCleanupPhaseEnablementDryRun,
) -> ContractRepairApprovalApproveExecutionRunnerRecoveryMarkerCleanupPhaseReadinessDryRun {
    let phase_sequence_ready = cleanup_phase_dry_run.phase_sequence_ready;
    let source_mutation_phase_enablement_ready =
        cleanup_phase_enablement_dry_run.source_mutation_phase_enablement_ready;
    let source_mutation_phase_ready = cleanup_phase_dry_run.source_mutation_phase_ready;
    let cleanup_phase_present = cleanup_phase_dry_run.cleanup_phase_present;
    let marker_persistence_plan_ready = cleanup_phase_dry_run.marker_persistence_plan_ready;
    let marker_persistence_ready = cleanup_phase_dry_run.marker_persistence_ready;
    let cleanup_phase_enabled = cleanup_phase_dry_run.cleanup_phase_enabled;
    let cleanup_phase_enablement_ready =
        cleanup_phase_enablement_dry_run.cleanup_phase_enablement_ready;
    let cleanup_phase_ready = cleanup_phase_dry_run.cleanup_phase_ready;
    let required_gates = vec![
        "approve_execution_runner_phase_sequence_ready".to_string(),
        "approve_execution_runner_source_mutation_phase_ready".to_string(),
        "approve_execution_runner_recovery_marker_cleanup_phase_present".to_string(),
        "recovery_marker_persistence_plan_ready".to_string(),
        "recovery_marker_persistence_ready".to_string(),
        "approve_execution_runner_recovery_marker_cleanup_phase_enabled".to_string(),
    ];
    let gate_states = [
        (
            "approve_execution_runner_phase_sequence_ready",
            phase_sequence_ready,
        ),
        (
            "approve_execution_runner_source_mutation_phase_ready",
            source_mutation_phase_ready,
        ),
        (
            "approve_execution_runner_recovery_marker_cleanup_phase_present",
            cleanup_phase_present,
        ),
        (
            "recovery_marker_persistence_plan_ready",
            marker_persistence_plan_ready,
        ),
        (
            "recovery_marker_persistence_ready",
            marker_persistence_ready,
        ),
        (
            "approve_execution_runner_recovery_marker_cleanup_phase_enabled",
            cleanup_phase_enabled,
        ),
    ];
    let mut passed_gates = Vec::new();
    let mut blocked_gates = Vec::new();
    for (gate, passed) in gate_states {
        if passed {
            passed_gates.push(gate.to_string());
        } else {
            blocked_gates.push(gate.to_string());
        }
    }

    ContractRepairApprovalApproveExecutionRunnerRecoveryMarkerCleanupPhaseReadinessDryRun {
        status: if cleanup_phase_ready {
            "approve_execution_runner_recovery_marker_cleanup_phase_readiness_ready".to_string()
        } else if cleanup_phase_dry_run.action == "approve"
            && cleanup_phase_present
            && marker_persistence_plan_ready
        {
            "approve_execution_runner_recovery_marker_cleanup_phase_readiness_ready_blocked"
                .to_string()
        } else {
            "approve_execution_runner_recovery_marker_cleanup_phase_readiness_blocked".to_string()
        },
        action: cleanup_phase_dry_run.action.clone(),
        approval_id: cleanup_phase_dry_run.approval_id.clone(),
        marker_key: cleanup_phase_dry_run.marker_key.clone(),
        file_name: cleanup_phase_dry_run.file_name.clone(),
        gate_name: "approve_execution_runner_recovery_marker_cleanup_phase_ready".to_string(),
        phase_sequence_ready,
        source_mutation_phase_enablement_ready,
        source_mutation_phase_ready,
        cleanup_phase_present,
        marker_persistence_plan_ready,
        marker_persistence_ready,
        cleanup_phase_enabled,
        cleanup_phase_enablement_ready,
        cleanup_phase_ready,
        would_clear_recovery_marker: false,
        would_continue_to_commit: false,
        would_return_success: false,
        would_touch_disk: false,
        would_unblock_transaction_commit: false,
        would_unblock_control_readiness: false,
        inherited_cleanup_phase_blockers: cleanup_phase_dry_run.blocked_by.clone(),
        inherited_cleanup_phase_enablement_blocked_gates: cleanup_phase_enablement_dry_run
            .blocked_gates
            .clone(),
        required_gates,
        passed_gates,
        blocked_gates,
    }
}

fn contract_repair_approval_approve_execution_runner_transaction_commit_phase_dry_run(
    phase_sequence_dry_run: &ContractRepairApprovalApproveExecutionRunnerBodyPhaseSequenceDryRun,
    cleanup_phase_dry_run: &ContractRepairApprovalApproveExecutionRunnerRecoveryMarkerCleanupPhaseDryRun,
    commit_gate: &ContractRepairApprovalApproveExecutionTransactionCommitGate,
    commit_phase_enabled: bool,
) -> ContractRepairApprovalApproveExecutionRunnerTransactionCommitPhaseDryRun {
    let phase_sequence_ready = phase_sequence_dry_run.phase_sequence_ready;
    let cleanup_phase_ready = cleanup_phase_dry_run.cleanup_phase_ready;
    let runner_plan_ready = commit_gate.runner_plan_ready;
    let runner_enabled = commit_gate.runner_enabled;
    let admission_ready = commit_gate.admission_ready;
    let commit_barrier_ready = commit_gate.commit_barrier_ready;
    let rollback_plan_ready = commit_gate.rollback_plan_ready;
    let recovery_marker_persistence_plan_ready = commit_gate.recovery_marker_persistence_plan_ready;
    let recovery_marker_persistence_ready = commit_gate.recovery_marker_persistence_ready;
    let commit_gate_enabled = commit_gate.commit_gate_enabled;
    let commit_ready = commit_gate.commit_ready;
    let commit_phase_ready =
        phase_sequence_ready && cleanup_phase_ready && commit_ready && commit_phase_enabled;
    let mut blocked_by = Vec::new();
    if !phase_sequence_ready {
        blocked_by.push("approve_execution_runner_phase_sequence_ready".to_string());
    }
    if !cleanup_phase_ready {
        blocked_by.push("approve_execution_runner_recovery_marker_cleanup_phase_ready".to_string());
    }
    if !runner_plan_ready {
        blocked_by.push("approve_execution_transaction_commit_plan_ready".to_string());
    }
    if !runner_enabled {
        blocked_by.push("approve_execution_transaction_runner_enabled".to_string());
    }
    if !admission_ready {
        blocked_by.push("approve_execution_admission_ready".to_string());
    }
    if !commit_barrier_ready {
        blocked_by.push("commit_barrier_ready".to_string());
    }
    if !rollback_plan_ready {
        blocked_by.push("rollback_plan_ready".to_string());
    }
    if !recovery_marker_persistence_plan_ready {
        blocked_by.push("recovery_marker_persistence_plan_ready".to_string());
    }
    if !recovery_marker_persistence_ready {
        blocked_by.push("recovery_marker_persistence_ready".to_string());
    }
    if !commit_gate_enabled {
        blocked_by.push("approve_execution_transaction_commit_enabled".to_string());
    }
    if !commit_phase_enabled {
        blocked_by.push("approve_execution_runner_transaction_commit_phase_enabled".to_string());
    }
    for gate in &cleanup_phase_dry_run.blocked_by {
        if !blocked_by.contains(gate) {
            blocked_by.push(gate.clone());
        }
    }

    ContractRepairApprovalApproveExecutionRunnerTransactionCommitPhaseDryRun {
        status: if commit_phase_ready {
            "approve_execution_runner_transaction_commit_phase_ready".to_string()
        } else if phase_sequence_dry_run.action == "approve" && runner_plan_ready {
            "approve_execution_runner_transaction_commit_phase_ready_blocked".to_string()
        } else {
            "approve_execution_runner_transaction_commit_phase_blocked".to_string()
        },
        action: phase_sequence_dry_run.action.clone(),
        approval_id: phase_sequence_dry_run.approval_id.clone(),
        phase_sequence_ready,
        cleanup_phase_ready,
        runner_plan_ready,
        runner_enabled,
        admission_ready,
        commit_barrier_ready,
        rollback_plan_ready,
        recovery_marker_persistence_plan_ready,
        recovery_marker_persistence_ready,
        commit_gate_enabled,
        commit_ready,
        commit_phase_enabled,
        commit_phase_ready,
        would_commit_transaction: false,
        would_return_success: false,
        would_touch_disk: false,
        blocked_by,
    }
}

fn contract_repair_approval_approve_execution_runner_transaction_commit_phase_enablement_dry_run(
    phase_sequence_dry_run: &ContractRepairApprovalApproveExecutionRunnerBodyPhaseSequenceDryRun,
    cleanup_phase_enablement_dry_run: &ContractRepairApprovalApproveExecutionRunnerRecoveryMarkerCleanupPhaseEnablementDryRun,
    cleanup_phase_dry_run: &ContractRepairApprovalApproveExecutionRunnerRecoveryMarkerCleanupPhaseDryRun,
    commit_gate: &ContractRepairApprovalApproveExecutionTransactionCommitGate,
    commit_phase_enabled: bool,
) -> ContractRepairApprovalApproveExecutionRunnerTransactionCommitPhaseEnablementDryRun {
    let phase_sequence_ready = phase_sequence_dry_run.phase_sequence_ready;
    let cleanup_phase_enablement_ready =
        cleanup_phase_enablement_dry_run.cleanup_phase_enablement_ready;
    let cleanup_phase_ready = cleanup_phase_dry_run.cleanup_phase_ready;
    let runner_plan_ready = commit_gate.runner_plan_ready;
    let runner_enabled = commit_gate.runner_enabled;
    let admission_ready = commit_gate.admission_ready;
    let commit_barrier_ready = commit_gate.commit_barrier_ready;
    let rollback_plan_ready = commit_gate.rollback_plan_ready;
    let recovery_marker_persistence_plan_ready = commit_gate.recovery_marker_persistence_plan_ready;
    let recovery_marker_persistence_ready = commit_gate.recovery_marker_persistence_ready;
    let commit_gate_enabled = commit_gate.commit_gate_enabled;
    let commit_ready = commit_gate.commit_ready;
    let enablement_prerequisites_ready = phase_sequence_ready
        && cleanup_phase_enablement_ready
        && cleanup_phase_ready
        && runner_plan_ready
        && runner_enabled
        && admission_ready
        && commit_barrier_ready
        && rollback_plan_ready
        && recovery_marker_persistence_plan_ready
        && recovery_marker_persistence_ready
        && commit_gate_enabled
        && commit_ready;
    let commit_phase_enablement_ready = enablement_prerequisites_ready && commit_phase_enabled;
    let required_gates = vec![
        "approve_execution_runner_phase_sequence_ready".to_string(),
        "approve_execution_runner_recovery_marker_cleanup_phase_enablement_ready".to_string(),
        "approve_execution_runner_recovery_marker_cleanup_phase_ready".to_string(),
        "approve_execution_transaction_commit_plan_ready".to_string(),
        "approve_execution_transaction_runner_enabled".to_string(),
        "approve_execution_admission_ready".to_string(),
        "commit_barrier_ready".to_string(),
        "rollback_plan_ready".to_string(),
        "recovery_marker_persistence_plan_ready".to_string(),
        "recovery_marker_persistence_ready".to_string(),
        "approve_execution_transaction_commit_enabled".to_string(),
        "approve_execution_transaction_commit_ready".to_string(),
        "approve_execution_runner_transaction_commit_phase_enabled".to_string(),
    ];
    let gate_states = [
        (
            "approve_execution_runner_phase_sequence_ready",
            phase_sequence_ready,
        ),
        (
            "approve_execution_runner_recovery_marker_cleanup_phase_enablement_ready",
            cleanup_phase_enablement_ready,
        ),
        (
            "approve_execution_runner_recovery_marker_cleanup_phase_ready",
            cleanup_phase_ready,
        ),
        (
            "approve_execution_transaction_commit_plan_ready",
            runner_plan_ready,
        ),
        (
            "approve_execution_transaction_runner_enabled",
            runner_enabled,
        ),
        ("approve_execution_admission_ready", admission_ready),
        ("commit_barrier_ready", commit_barrier_ready),
        ("rollback_plan_ready", rollback_plan_ready),
        (
            "recovery_marker_persistence_plan_ready",
            recovery_marker_persistence_plan_ready,
        ),
        (
            "recovery_marker_persistence_ready",
            recovery_marker_persistence_ready,
        ),
        (
            "approve_execution_transaction_commit_enabled",
            commit_gate_enabled,
        ),
        ("approve_execution_transaction_commit_ready", commit_ready),
        (
            "approve_execution_runner_transaction_commit_phase_enabled",
            commit_phase_enabled,
        ),
    ];
    let mut passed_gates = Vec::new();
    let mut blocked_gates = Vec::new();
    for (gate, passed) in gate_states {
        if passed {
            passed_gates.push(gate.to_string());
        } else {
            blocked_gates.push(gate.to_string());
        }
    }

    ContractRepairApprovalApproveExecutionRunnerTransactionCommitPhaseEnablementDryRun {
        status: if commit_phase_enablement_ready {
            "approve_execution_runner_transaction_commit_phase_enablement_ready".to_string()
        } else if phase_sequence_dry_run.action == "approve" {
            "approve_execution_runner_transaction_commit_phase_enablement_ready_blocked".to_string()
        } else {
            "approve_execution_runner_transaction_commit_phase_enablement_blocked".to_string()
        },
        action: phase_sequence_dry_run.action.clone(),
        approval_id: phase_sequence_dry_run.approval_id.clone(),
        switch_name: "approve_execution_runner_transaction_commit_phase_enabled".to_string(),
        phase_sequence_ready,
        cleanup_phase_enablement_ready,
        cleanup_phase_ready,
        runner_plan_ready,
        runner_enabled,
        admission_ready,
        commit_barrier_ready,
        rollback_plan_ready,
        recovery_marker_persistence_plan_ready,
        recovery_marker_persistence_ready,
        commit_gate_enabled,
        commit_ready,
        commit_phase_enabled,
        enablement_prerequisites_ready,
        commit_phase_enablement_ready,
        would_enable_commit_phase: false,
        would_commit_transaction: false,
        would_return_success: false,
        would_touch_disk: false,
        required_gates,
        passed_gates,
        blocked_gates,
    }
}

fn contract_repair_approval_approve_execution_runner_transaction_commit_phase_with_gate(
    commit_phase_dry_run: &ContractRepairApprovalApproveExecutionRunnerTransactionCommitPhaseDryRun,
    commit_phase_enablement_dry_run: &ContractRepairApprovalApproveExecutionRunnerTransactionCommitPhaseEnablementDryRun,
    commit_phase_enabled: bool,
) -> ContractRepairApprovalApproveExecutionRunnerTransactionCommitPhaseDryRun {
    let mut result = commit_phase_dry_run.clone();
    if !commit_phase_enabled {
        return result;
    }

    result.commit_phase_enabled = true;
    result
        .blocked_by
        .retain(|blocker| blocker != "approve_execution_runner_transaction_commit_phase_enabled");
    if !commit_phase_enablement_dry_run.enablement_prerequisites_ready {
        push_unique_blocker(
            &mut result.blocked_by,
            "approve_execution_runner_transaction_commit_phase_enablement_ready",
        );
    }

    let gate_states = [
        (
            "approve_execution_runner_phase_sequence_ready",
            commit_phase_dry_run.phase_sequence_ready,
        ),
        (
            "approve_execution_runner_recovery_marker_cleanup_phase_ready",
            commit_phase_dry_run.cleanup_phase_ready,
        ),
        (
            "approve_execution_transaction_commit_plan_ready",
            commit_phase_dry_run.runner_plan_ready,
        ),
        (
            "approve_execution_transaction_runner_enabled",
            commit_phase_dry_run.runner_enabled,
        ),
        (
            "approve_execution_admission_ready",
            commit_phase_dry_run.admission_ready,
        ),
        (
            "commit_barrier_ready",
            commit_phase_dry_run.commit_barrier_ready,
        ),
        (
            "rollback_plan_ready",
            commit_phase_dry_run.rollback_plan_ready,
        ),
        (
            "recovery_marker_persistence_plan_ready",
            commit_phase_dry_run.recovery_marker_persistence_plan_ready,
        ),
        (
            "recovery_marker_persistence_ready",
            commit_phase_dry_run.recovery_marker_persistence_ready,
        ),
        (
            "approve_execution_transaction_commit_enabled",
            commit_phase_dry_run.commit_gate_enabled,
        ),
        (
            "approve_execution_transaction_commit_ready",
            commit_phase_dry_run.commit_ready,
        ),
    ];
    for (gate, passed) in gate_states {
        if !passed {
            push_unique_blocker(&mut result.blocked_by, gate);
        }
    }
    if !result.blocked_by.is_empty() {
        result.commit_phase_ready = false;
        result.status = "approve_execution_runner_transaction_commit_phase_blocked".to_string();
        return result;
    }

    result.status = "approve_execution_runner_transaction_commit_phase_committed".to_string();
    result.commit_phase_ready = true;
    result.would_commit_transaction = true;
    result.would_return_success = false;
    result.would_touch_disk = false;
    result
}

fn contract_repair_approval_approve_execution_runner_transaction_commit_phase_readiness_dry_run(
    commit_phase_dry_run: &ContractRepairApprovalApproveExecutionRunnerTransactionCommitPhaseDryRun,
    commit_phase_enablement_dry_run: &ContractRepairApprovalApproveExecutionRunnerTransactionCommitPhaseEnablementDryRun,
) -> ContractRepairApprovalApproveExecutionRunnerTransactionCommitPhaseReadinessDryRun {
    let phase_sequence_ready = commit_phase_dry_run.phase_sequence_ready;
    let cleanup_phase_enablement_ready =
        commit_phase_enablement_dry_run.cleanup_phase_enablement_ready;
    let cleanup_phase_ready = commit_phase_dry_run.cleanup_phase_ready;
    let runner_plan_ready = commit_phase_dry_run.runner_plan_ready;
    let runner_enabled = commit_phase_dry_run.runner_enabled;
    let admission_ready = commit_phase_dry_run.admission_ready;
    let commit_barrier_ready = commit_phase_dry_run.commit_barrier_ready;
    let rollback_plan_ready = commit_phase_dry_run.rollback_plan_ready;
    let recovery_marker_persistence_plan_ready =
        commit_phase_dry_run.recovery_marker_persistence_plan_ready;
    let recovery_marker_persistence_ready = commit_phase_dry_run.recovery_marker_persistence_ready;
    let commit_gate_enabled = commit_phase_dry_run.commit_gate_enabled;
    let commit_ready = commit_phase_dry_run.commit_ready;
    let commit_phase_enabled = commit_phase_dry_run.commit_phase_enabled;
    let commit_phase_enablement_ready =
        commit_phase_enablement_dry_run.commit_phase_enablement_ready;
    let commit_phase_ready = commit_phase_dry_run.commit_phase_ready;
    let required_gates = vec![
        "approve_execution_runner_phase_sequence_ready".to_string(),
        "approve_execution_runner_recovery_marker_cleanup_phase_ready".to_string(),
        "approve_execution_transaction_commit_ready".to_string(),
        "approve_execution_runner_transaction_commit_phase_enabled".to_string(),
    ];
    let gate_states = [
        (
            "approve_execution_runner_phase_sequence_ready",
            phase_sequence_ready,
        ),
        (
            "approve_execution_runner_recovery_marker_cleanup_phase_ready",
            cleanup_phase_ready,
        ),
        ("approve_execution_transaction_commit_ready", commit_ready),
        (
            "approve_execution_runner_transaction_commit_phase_enabled",
            commit_phase_enabled,
        ),
    ];
    let mut passed_gates = Vec::new();
    let mut blocked_gates = Vec::new();
    for (gate, passed) in gate_states {
        if passed {
            passed_gates.push(gate.to_string());
        } else {
            blocked_gates.push(gate.to_string());
        }
    }

    ContractRepairApprovalApproveExecutionRunnerTransactionCommitPhaseReadinessDryRun {
        status: if commit_phase_ready {
            "approve_execution_runner_transaction_commit_phase_readiness_ready".to_string()
        } else if commit_phase_dry_run.action == "approve" && runner_plan_ready {
            "approve_execution_runner_transaction_commit_phase_readiness_ready_blocked".to_string()
        } else {
            "approve_execution_runner_transaction_commit_phase_readiness_blocked".to_string()
        },
        action: commit_phase_dry_run.action.clone(),
        approval_id: commit_phase_dry_run.approval_id.clone(),
        gate_name: "approve_execution_runner_transaction_commit_phase_ready".to_string(),
        phase_sequence_ready,
        cleanup_phase_enablement_ready,
        cleanup_phase_ready,
        runner_plan_ready,
        runner_enabled,
        admission_ready,
        commit_barrier_ready,
        rollback_plan_ready,
        recovery_marker_persistence_plan_ready,
        recovery_marker_persistence_ready,
        commit_gate_enabled,
        commit_ready,
        commit_phase_enabled,
        commit_phase_enablement_ready,
        commit_phase_ready,
        would_commit_transaction: false,
        would_return_success: false,
        would_touch_disk: false,
        would_unblock_rollback_execution: false,
        would_unblock_control_readiness: false,
        inherited_commit_phase_blockers: commit_phase_dry_run.blocked_by.clone(),
        inherited_commit_phase_enablement_blocked_gates: commit_phase_enablement_dry_run
            .blocked_gates
            .clone(),
        required_gates,
        passed_gates,
        blocked_gates,
    }
}

fn contract_repair_approval_approve_execution_runner_rollback_execution_phase_dry_run(
    phase_sequence_dry_run: &ContractRepairApprovalApproveExecutionRunnerBodyPhaseSequenceDryRun,
    commit_phase_dry_run: &ContractRepairApprovalApproveExecutionRunnerTransactionCommitPhaseDryRun,
    rollback_execution_enabled: bool,
) -> ContractRepairApprovalApproveExecutionRunnerRollbackExecutionPhaseDryRun {
    let phase_sequence_ready = phase_sequence_dry_run.phase_sequence_ready;
    let rollback_sequence_ready = phase_sequence_dry_run.rollback_sequence_ready;
    let commit_phase_ready = commit_phase_dry_run.commit_phase_ready;
    let rollback_order = phase_sequence_dry_run.rollback_order.clone();
    let rollback_phase_present = rollback_order
        .iter()
        .any(|phase| phase == "restore_contract_source")
        && rollback_order
            .iter()
            .any(|phase| phase == "restore_approval_record")
        && rollback_order
            .iter()
            .any(|phase| phase == "mark_recovery_marker_rolled_back");
    let rollback_plan_ready = commit_phase_dry_run.rollback_plan_ready && rollback_sequence_ready;
    let rollback_execution_ready = phase_sequence_ready
        && rollback_sequence_ready
        && commit_phase_ready
        && rollback_phase_present
        && rollback_plan_ready
        && rollback_execution_enabled;
    let mut blocked_by = Vec::new();
    if !phase_sequence_ready {
        blocked_by.push("approve_execution_runner_phase_sequence_ready".to_string());
    }
    if !rollback_sequence_ready {
        blocked_by.push("approve_execution_runner_rollback_sequence_ready".to_string());
    }
    if !commit_phase_ready {
        blocked_by.push("approve_execution_runner_transaction_commit_phase_ready".to_string());
    }
    if !rollback_phase_present {
        blocked_by.push("approve_execution_runner_rollback_phase_present".to_string());
    }
    if !rollback_plan_ready {
        blocked_by.push("rollback_plan_ready".to_string());
    }
    if !rollback_execution_enabled {
        blocked_by.push("approve_execution_runner_rollback_execution_enabled".to_string());
    }
    for gate in &commit_phase_dry_run.blocked_by {
        if !blocked_by.contains(gate) {
            blocked_by.push(gate.clone());
        }
    }

    ContractRepairApprovalApproveExecutionRunnerRollbackExecutionPhaseDryRun {
        status: if rollback_execution_ready {
            "approve_execution_runner_rollback_execution_phase_ready".to_string()
        } else if phase_sequence_dry_run.action == "approve"
            && rollback_phase_present
            && rollback_plan_ready
        {
            "approve_execution_runner_rollback_execution_phase_ready_blocked".to_string()
        } else {
            "approve_execution_runner_rollback_execution_phase_blocked".to_string()
        },
        action: phase_sequence_dry_run.action.clone(),
        approval_id: phase_sequence_dry_run.approval_id.clone(),
        phase_sequence_ready,
        rollback_sequence_ready,
        commit_phase_ready,
        rollback_phase_present,
        rollback_plan_ready,
        rollback_execution_enabled,
        rollback_execution_ready,
        rollback_order,
        would_restore_contract_source: false,
        would_restore_approval_record: false,
        would_mark_recovery_marker_rolled_back: false,
        would_rollback_on_error: false,
        would_return_success: false,
        would_touch_disk: false,
        blocked_by,
    }
}

fn contract_repair_approval_approve_execution_runner_rollback_execution_phase_enablement_dry_run(
    phase_sequence_dry_run: &ContractRepairApprovalApproveExecutionRunnerBodyPhaseSequenceDryRun,
    commit_phase_enablement_dry_run: &ContractRepairApprovalApproveExecutionRunnerTransactionCommitPhaseEnablementDryRun,
    commit_phase_dry_run: &ContractRepairApprovalApproveExecutionRunnerTransactionCommitPhaseDryRun,
    rollback_execution_enabled: bool,
) -> ContractRepairApprovalApproveExecutionRunnerRollbackExecutionPhaseEnablementDryRun {
    let phase_sequence_ready = phase_sequence_dry_run.phase_sequence_ready;
    let rollback_sequence_ready = phase_sequence_dry_run.rollback_sequence_ready;
    let commit_phase_enablement_ready =
        commit_phase_enablement_dry_run.commit_phase_enablement_ready;
    let commit_phase_ready = commit_phase_dry_run.commit_phase_ready;
    let rollback_order = phase_sequence_dry_run.rollback_order.clone();
    let rollback_phase_present = rollback_order
        .iter()
        .any(|phase| phase == "restore_contract_source")
        && rollback_order
            .iter()
            .any(|phase| phase == "restore_approval_record")
        && rollback_order
            .iter()
            .any(|phase| phase == "mark_recovery_marker_rolled_back");
    let rollback_plan_ready = commit_phase_dry_run.rollback_plan_ready && rollback_sequence_ready;
    let enablement_prerequisites_ready = phase_sequence_ready
        && rollback_sequence_ready
        && commit_phase_enablement_ready
        && commit_phase_ready
        && rollback_phase_present
        && rollback_plan_ready;
    let rollback_execution_enablement_ready =
        enablement_prerequisites_ready && rollback_execution_enabled;
    let required_gates = vec![
        "approve_execution_runner_phase_sequence_ready".to_string(),
        "approve_execution_runner_rollback_sequence_ready".to_string(),
        "approve_execution_runner_transaction_commit_phase_enablement_ready".to_string(),
        "approve_execution_runner_transaction_commit_phase_ready".to_string(),
        "approve_execution_runner_rollback_phase_present".to_string(),
        "rollback_plan_ready".to_string(),
        "approve_execution_runner_rollback_execution_enabled".to_string(),
    ];
    let gate_states = [
        (
            "approve_execution_runner_phase_sequence_ready",
            phase_sequence_ready,
        ),
        (
            "approve_execution_runner_rollback_sequence_ready",
            rollback_sequence_ready,
        ),
        (
            "approve_execution_runner_transaction_commit_phase_enablement_ready",
            commit_phase_enablement_ready,
        ),
        (
            "approve_execution_runner_transaction_commit_phase_ready",
            commit_phase_ready,
        ),
        (
            "approve_execution_runner_rollback_phase_present",
            rollback_phase_present,
        ),
        ("rollback_plan_ready", rollback_plan_ready),
        (
            "approve_execution_runner_rollback_execution_enabled",
            rollback_execution_enabled,
        ),
    ];
    let mut passed_gates = Vec::new();
    let mut blocked_gates = Vec::new();
    for (gate, passed) in gate_states {
        if passed {
            passed_gates.push(gate.to_string());
        } else {
            blocked_gates.push(gate.to_string());
        }
    }

    ContractRepairApprovalApproveExecutionRunnerRollbackExecutionPhaseEnablementDryRun {
        status: if rollback_execution_enablement_ready {
            "approve_execution_runner_rollback_execution_phase_enablement_ready".to_string()
        } else if phase_sequence_dry_run.action == "approve" {
            "approve_execution_runner_rollback_execution_phase_enablement_ready_blocked".to_string()
        } else {
            "approve_execution_runner_rollback_execution_phase_enablement_blocked".to_string()
        },
        action: phase_sequence_dry_run.action.clone(),
        approval_id: phase_sequence_dry_run.approval_id.clone(),
        switch_name: "approve_execution_runner_rollback_execution_enabled".to_string(),
        phase_sequence_ready,
        rollback_sequence_ready,
        commit_phase_enablement_ready,
        commit_phase_ready,
        rollback_phase_present,
        rollback_plan_ready,
        rollback_execution_enabled,
        enablement_prerequisites_ready,
        rollback_execution_enablement_ready,
        rollback_order,
        would_enable_rollback_execution: false,
        would_restore_contract_source: false,
        would_restore_approval_record: false,
        would_mark_recovery_marker_rolled_back: false,
        would_rollback_on_error: false,
        would_return_success: false,
        would_touch_disk: false,
        required_gates,
        passed_gates,
        blocked_gates,
    }
}

fn contract_repair_approval_approve_execution_runner_rollback_execution_phase_readiness_dry_run(
    rollback_execution_phase_dry_run: &ContractRepairApprovalApproveExecutionRunnerRollbackExecutionPhaseDryRun,
    rollback_execution_phase_enablement_dry_run: &ContractRepairApprovalApproveExecutionRunnerRollbackExecutionPhaseEnablementDryRun,
) -> ContractRepairApprovalApproveExecutionRunnerRollbackExecutionPhaseReadinessDryRun {
    let phase_sequence_ready = rollback_execution_phase_dry_run.phase_sequence_ready;
    let rollback_sequence_ready = rollback_execution_phase_dry_run.rollback_sequence_ready;
    let commit_phase_enablement_ready =
        rollback_execution_phase_enablement_dry_run.commit_phase_enablement_ready;
    let commit_phase_ready = rollback_execution_phase_dry_run.commit_phase_ready;
    let rollback_phase_present = rollback_execution_phase_dry_run.rollback_phase_present;
    let rollback_plan_ready = rollback_execution_phase_dry_run.rollback_plan_ready;
    let rollback_execution_enabled = rollback_execution_phase_dry_run.rollback_execution_enabled;
    let rollback_execution_enablement_ready =
        rollback_execution_phase_enablement_dry_run.rollback_execution_enablement_ready;
    let rollback_execution_ready = rollback_execution_phase_dry_run.rollback_execution_ready;
    let required_gates = vec![
        "approve_execution_runner_phase_sequence_ready".to_string(),
        "approve_execution_runner_rollback_sequence_ready".to_string(),
        "approve_execution_runner_transaction_commit_phase_ready".to_string(),
        "approve_execution_runner_rollback_phase_present".to_string(),
        "rollback_plan_ready".to_string(),
        "approve_execution_runner_rollback_execution_enabled".to_string(),
    ];
    let gate_states = [
        (
            "approve_execution_runner_phase_sequence_ready",
            phase_sequence_ready,
        ),
        (
            "approve_execution_runner_rollback_sequence_ready",
            rollback_sequence_ready,
        ),
        (
            "approve_execution_runner_transaction_commit_phase_ready",
            commit_phase_ready,
        ),
        (
            "approve_execution_runner_rollback_phase_present",
            rollback_phase_present,
        ),
        ("rollback_plan_ready", rollback_plan_ready),
        (
            "approve_execution_runner_rollback_execution_enabled",
            rollback_execution_enabled,
        ),
    ];
    let mut passed_gates = Vec::new();
    let mut blocked_gates = Vec::new();
    for (gate, passed) in gate_states {
        if passed {
            passed_gates.push(gate.to_string());
        } else {
            blocked_gates.push(gate.to_string());
        }
    }

    ContractRepairApprovalApproveExecutionRunnerRollbackExecutionPhaseReadinessDryRun {
        status: if rollback_execution_ready {
            "approve_execution_runner_rollback_execution_phase_readiness_ready".to_string()
        } else if rollback_execution_phase_dry_run.action == "approve"
            && rollback_phase_present
            && rollback_plan_ready
        {
            "approve_execution_runner_rollback_execution_phase_readiness_ready_blocked".to_string()
        } else {
            "approve_execution_runner_rollback_execution_phase_readiness_blocked".to_string()
        },
        action: rollback_execution_phase_dry_run.action.clone(),
        approval_id: rollback_execution_phase_dry_run.approval_id.clone(),
        gate_name: "approve_execution_runner_rollback_execution_phase_ready".to_string(),
        phase_sequence_ready,
        rollback_sequence_ready,
        commit_phase_enablement_ready,
        commit_phase_ready,
        rollback_phase_present,
        rollback_plan_ready,
        rollback_execution_enabled,
        rollback_execution_enablement_ready,
        rollback_execution_ready,
        rollback_order: rollback_execution_phase_dry_run.rollback_order.clone(),
        would_restore_contract_source: false,
        would_restore_approval_record: false,
        would_mark_recovery_marker_rolled_back: false,
        would_rollback_on_error: false,
        would_return_success: false,
        would_touch_disk: false,
        would_unblock_runner_activation: false,
        would_unblock_control_readiness: false,
        inherited_rollback_execution_blockers: rollback_execution_phase_dry_run.blocked_by.clone(),
        inherited_rollback_execution_enablement_blocked_gates:
            rollback_execution_phase_enablement_dry_run
                .blocked_gates
                .clone(),
        required_gates,
        passed_gates,
        blocked_gates,
    }
}

fn contract_repair_approval_approve_execution_runner_enablement_plan_dry_run(
    runner_attempt: &ContractRepairApprovalApproveExecutionRunnerAttempt,
    runner_outcome: &ContractRepairApprovalApproveExecutionRunnerOutcome,
    dispatch_gate: &ContractRepairApprovalApproveExecutionRunnerDispatchGate,
    call_dry_run: &ContractRepairApprovalApproveExecutionRunnerCallDryRun,
    body_dry_run: &ContractRepairApprovalApproveExecutionRunnerCallBodyDryRun,
    phase_sequence_dry_run: &ContractRepairApprovalApproveExecutionRunnerBodyPhaseSequenceDryRun,
    lifecycle_phase_dry_run: &ContractRepairApprovalApproveExecutionRunnerLifecyclePhaseDryRun,
    source_mutation_phase_dry_run: &ContractRepairApprovalApproveExecutionRunnerSourceMutationPhaseDryRun,
    cleanup_phase_dry_run: &ContractRepairApprovalApproveExecutionRunnerRecoveryMarkerCleanupPhaseDryRun,
    commit_phase_dry_run: &ContractRepairApprovalApproveExecutionRunnerTransactionCommitPhaseDryRun,
    rollback_phase_dry_run: &ContractRepairApprovalApproveExecutionRunnerRollbackExecutionPhaseDryRun,
    runner_activation_enabled: bool,
) -> ContractRepairApprovalApproveExecutionRunnerEnablementPlanDryRun {
    let structural_plan_ready = phase_sequence_dry_run.phase_sequence_ready
        && phase_sequence_dry_run.rollback_sequence_ready
        && lifecycle_phase_dry_run.lifecycle_phase_present
        && source_mutation_phase_dry_run.source_mutation_phase_present
        && cleanup_phase_dry_run.cleanup_phase_present
        && rollback_phase_dry_run.rollback_phase_present;
    let runner_control_ready = runner_attempt.runner_attempt_ready
        && runner_outcome.runner_execution_ready
        && dispatch_gate.dispatch_ready
        && call_dry_run.call_ready
        && body_dry_run.body_ready
        && phase_sequence_dry_run.phases_ready;
    let phase_chain_ready = lifecycle_phase_dry_run.lifecycle_phase_ready
        && source_mutation_phase_dry_run.source_mutation_phase_ready
        && cleanup_phase_dry_run.cleanup_phase_ready
        && commit_phase_dry_run.commit_phase_ready;
    let rollback_chain_ready = rollback_phase_dry_run.rollback_execution_ready;
    let side_effect_enablement_ready = lifecycle_phase_dry_run.lifecycle_effects_ready
        && source_mutation_phase_dry_run.contract_mutation_ready
        && cleanup_phase_dry_run.marker_persistence_ready
        && commit_phase_dry_run.commit_ready;
    let runner_activation_ready = structural_plan_ready
        && runner_control_ready
        && phase_chain_ready
        && rollback_chain_ready
        && side_effect_enablement_ready
        && runner_activation_enabled;
    let required_enablements = vec![
        "approve_execution_transaction_runner_enabled".to_string(),
        "approve_execution_runner_attempt_enabled".to_string(),
        "approve_execution_runner_execution_enabled".to_string(),
        "approve_execution_runner_route_dispatch_enabled".to_string(),
        "approve_execution_runner_call_enabled".to_string(),
        "approve_execution_runner_body_enabled".to_string(),
        "approve_execution_runner_phase_execution_enabled".to_string(),
        "approve_execution_runner_lifecycle_phase_enabled".to_string(),
        "contract_mutation_api_enabled".to_string(),
        "approve_execution_runner_source_mutation_phase_enabled".to_string(),
        "approve_execution_runner_recovery_marker_cleanup_phase_enabled".to_string(),
        "approve_execution_transaction_commit_enabled".to_string(),
        "approve_execution_runner_transaction_commit_phase_enabled".to_string(),
        "approve_execution_runner_rollback_execution_enabled".to_string(),
        "approve_execution_runner_activation_enabled".to_string(),
    ];
    let enablement_states = [
        (
            "approve_execution_transaction_runner_enabled",
            commit_phase_dry_run.runner_enabled,
        ),
        (
            "approve_execution_runner_attempt_enabled",
            runner_attempt.runner_attempt_enabled,
        ),
        (
            "approve_execution_runner_execution_enabled",
            runner_outcome.runner_execution_enabled,
        ),
        (
            "approve_execution_runner_route_dispatch_enabled",
            dispatch_gate.route_dispatch_enabled,
        ),
        (
            "approve_execution_runner_call_enabled",
            call_dry_run.runner_call_enabled,
        ),
        (
            "approve_execution_runner_body_enabled",
            body_dry_run.runner_body_enabled,
        ),
        (
            "approve_execution_runner_phase_execution_enabled",
            phase_sequence_dry_run.phase_execution_enabled,
        ),
        (
            "approve_execution_runner_lifecycle_phase_enabled",
            lifecycle_phase_dry_run.lifecycle_phase_enabled,
        ),
        (
            "contract_mutation_api_enabled",
            source_mutation_phase_dry_run.contract_mutation_api_enabled,
        ),
        (
            "approve_execution_runner_source_mutation_phase_enabled",
            source_mutation_phase_dry_run.source_mutation_phase_enabled,
        ),
        (
            "approve_execution_runner_recovery_marker_cleanup_phase_enabled",
            cleanup_phase_dry_run.cleanup_phase_enabled,
        ),
        (
            "approve_execution_transaction_commit_enabled",
            commit_phase_dry_run.commit_gate_enabled,
        ),
        (
            "approve_execution_runner_transaction_commit_phase_enabled",
            commit_phase_dry_run.commit_phase_enabled,
        ),
        (
            "approve_execution_runner_rollback_execution_enabled",
            rollback_phase_dry_run.rollback_execution_enabled,
        ),
        (
            "approve_execution_runner_activation_enabled",
            runner_activation_enabled,
        ),
    ];
    let mut passed_enablements = Vec::new();
    let mut blocked_enablements = Vec::new();
    for (gate, passed) in enablement_states {
        if passed {
            passed_enablements.push(gate.to_string());
        } else {
            blocked_enablements.push(gate.to_string());
        }
    }
    let mut blocked_by = Vec::new();
    if !structural_plan_ready {
        blocked_by.push("approve_execution_runner_structural_plan_ready".to_string());
    }
    if !runner_control_ready {
        blocked_by.push("approve_execution_runner_control_ready".to_string());
    }
    if !phase_chain_ready {
        blocked_by.push("approve_execution_runner_phase_chain_ready".to_string());
    }
    if !rollback_chain_ready {
        blocked_by.push("approve_execution_runner_rollback_chain_ready".to_string());
    }
    if !side_effect_enablement_ready {
        blocked_by.push("approve_execution_runner_side_effect_enablement_ready".to_string());
    }
    if !runner_activation_enabled {
        blocked_by.push("approve_execution_runner_activation_enabled".to_string());
    }
    for gate in &blocked_enablements {
        if !blocked_by.contains(gate) {
            blocked_by.push(gate.clone());
        }
    }
    for gate in &rollback_phase_dry_run.blocked_by {
        if !blocked_by.contains(gate) {
            blocked_by.push(gate.clone());
        }
    }

    ContractRepairApprovalApproveExecutionRunnerEnablementPlanDryRun {
        status: if runner_activation_ready {
            "approve_execution_runner_enablement_plan_ready".to_string()
        } else if runner_attempt.action == "approve" && structural_plan_ready {
            "approve_execution_runner_enablement_plan_ready_blocked".to_string()
        } else {
            "approve_execution_runner_enablement_plan_blocked".to_string()
        },
        action: runner_attempt.action.clone(),
        approval_id: runner_attempt.approval_id.clone(),
        structural_plan_ready,
        runner_control_ready,
        phase_chain_ready,
        rollback_chain_ready,
        side_effect_enablement_ready,
        runner_activation_enabled,
        runner_activation_ready,
        would_activate_runner: false,
        would_return_success: false,
        would_touch_disk: false,
        required_enablements,
        passed_enablements,
        blocked_enablements,
        blocked_by,
    }
}

fn contract_repair_approval_approve_execution_runner_activation_enablement_dry_run(
    enablement_plan_dry_run: &ContractRepairApprovalApproveExecutionRunnerEnablementPlanDryRun,
    rollback_execution_enablement_dry_run: &ContractRepairApprovalApproveExecutionRunnerRollbackExecutionPhaseEnablementDryRun,
) -> ContractRepairApprovalApproveExecutionRunnerActivationEnablementDryRun {
    let structural_plan_ready = enablement_plan_dry_run.structural_plan_ready;
    let runner_control_ready = enablement_plan_dry_run.runner_control_ready;
    let phase_chain_ready = enablement_plan_dry_run.phase_chain_ready;
    let rollback_chain_ready = enablement_plan_dry_run.rollback_chain_ready;
    let side_effect_enablement_ready = enablement_plan_dry_run.side_effect_enablement_ready;
    let rollback_execution_enablement_ready =
        rollback_execution_enablement_dry_run.rollback_execution_enablement_ready;
    let prior_enablements_ready = enablement_plan_dry_run
        .blocked_enablements
        .iter()
        .all(|gate| gate == "approve_execution_runner_activation_enabled");
    let runner_activation_enabled = enablement_plan_dry_run.runner_activation_enabled;
    let enablement_prerequisites_ready = structural_plan_ready
        && runner_control_ready
        && phase_chain_ready
        && rollback_chain_ready
        && side_effect_enablement_ready
        && rollback_execution_enablement_ready
        && prior_enablements_ready;
    let runner_activation_enablement_ready =
        enablement_prerequisites_ready && runner_activation_enabled;
    let required_gates = vec![
        "approve_execution_runner_structural_plan_ready".to_string(),
        "approve_execution_runner_control_ready".to_string(),
        "approve_execution_runner_phase_chain_ready".to_string(),
        "approve_execution_runner_rollback_chain_ready".to_string(),
        "approve_execution_runner_side_effect_enablement_ready".to_string(),
        "approve_execution_runner_rollback_execution_phase_enablement_ready".to_string(),
        "approve_execution_runner_prior_enablements_ready".to_string(),
        "approve_execution_runner_activation_enabled".to_string(),
    ];
    let gate_states = [
        (
            "approve_execution_runner_structural_plan_ready",
            structural_plan_ready,
        ),
        (
            "approve_execution_runner_control_ready",
            runner_control_ready,
        ),
        (
            "approve_execution_runner_phase_chain_ready",
            phase_chain_ready,
        ),
        (
            "approve_execution_runner_rollback_chain_ready",
            rollback_chain_ready,
        ),
        (
            "approve_execution_runner_side_effect_enablement_ready",
            side_effect_enablement_ready,
        ),
        (
            "approve_execution_runner_rollback_execution_phase_enablement_ready",
            rollback_execution_enablement_ready,
        ),
        (
            "approve_execution_runner_prior_enablements_ready",
            prior_enablements_ready,
        ),
        (
            "approve_execution_runner_activation_enabled",
            runner_activation_enabled,
        ),
    ];
    let mut passed_gates = Vec::new();
    let mut blocked_gates = Vec::new();
    for (gate, passed) in gate_states {
        if passed {
            passed_gates.push(gate.to_string());
        } else {
            blocked_gates.push(gate.to_string());
        }
    }

    ContractRepairApprovalApproveExecutionRunnerActivationEnablementDryRun {
        status: if runner_activation_enablement_ready {
            "approve_execution_runner_activation_enablement_ready".to_string()
        } else if enablement_plan_dry_run.action == "approve" && structural_plan_ready {
            "approve_execution_runner_activation_enablement_ready_blocked".to_string()
        } else {
            "approve_execution_runner_activation_enablement_blocked".to_string()
        },
        action: enablement_plan_dry_run.action.clone(),
        approval_id: enablement_plan_dry_run.approval_id.clone(),
        switch_name: "approve_execution_runner_activation_enabled".to_string(),
        structural_plan_ready,
        runner_control_ready,
        phase_chain_ready,
        rollback_chain_ready,
        side_effect_enablement_ready,
        rollback_execution_enablement_ready,
        prior_enablements_ready,
        runner_activation_enabled,
        enablement_prerequisites_ready,
        runner_activation_enablement_ready,
        would_enable_runner_activation: false,
        would_activate_runner: false,
        would_return_success: false,
        would_touch_disk: false,
        required_enablements: enablement_plan_dry_run.required_enablements.clone(),
        passed_enablements: enablement_plan_dry_run.passed_enablements.clone(),
        blocked_enablements: enablement_plan_dry_run.blocked_enablements.clone(),
        required_gates,
        passed_gates,
        blocked_gates,
    }
}

fn contract_repair_approval_approve_execution_runner_activation_enablement_readiness_dry_run(
    activation_enablement_dry_run: &ContractRepairApprovalApproveExecutionRunnerActivationEnablementDryRun,
) -> ContractRepairApprovalApproveExecutionRunnerActivationEnablementReadinessDryRun {
    let structural_plan_ready = activation_enablement_dry_run.structural_plan_ready;
    let runner_control_ready = activation_enablement_dry_run.runner_control_ready;
    let phase_chain_ready = activation_enablement_dry_run.phase_chain_ready;
    let rollback_chain_ready = activation_enablement_dry_run.rollback_chain_ready;
    let side_effect_enablement_ready = activation_enablement_dry_run.side_effect_enablement_ready;
    let rollback_execution_enablement_ready =
        activation_enablement_dry_run.rollback_execution_enablement_ready;
    let prior_enablements_ready = activation_enablement_dry_run.prior_enablements_ready;
    let runner_activation_enabled = activation_enablement_dry_run.runner_activation_enabled;
    let enablement_prerequisites_ready =
        activation_enablement_dry_run.enablement_prerequisites_ready;
    let runner_activation_enablement_ready =
        activation_enablement_dry_run.runner_activation_enablement_ready;
    let required_gates = activation_enablement_dry_run.required_gates.clone();
    let passed_gates = activation_enablement_dry_run.passed_gates.clone();
    let blocked_gates = activation_enablement_dry_run.blocked_gates.clone();

    ContractRepairApprovalApproveExecutionRunnerActivationEnablementReadinessDryRun {
        status: if runner_activation_enablement_ready {
            "approve_execution_runner_activation_enablement_readiness_ready".to_string()
        } else if activation_enablement_dry_run.action == "approve" && structural_plan_ready {
            "approve_execution_runner_activation_enablement_readiness_ready_blocked".to_string()
        } else {
            "approve_execution_runner_activation_enablement_readiness_blocked".to_string()
        },
        action: activation_enablement_dry_run.action.clone(),
        approval_id: activation_enablement_dry_run.approval_id.clone(),
        gate_name: "approve_execution_runner_activation_enablement_ready".to_string(),
        switch_name: activation_enablement_dry_run.switch_name.clone(),
        structural_plan_ready,
        runner_control_ready,
        phase_chain_ready,
        rollback_chain_ready,
        side_effect_enablement_ready,
        rollback_execution_enablement_ready,
        prior_enablements_ready,
        runner_activation_enabled,
        enablement_prerequisites_ready,
        runner_activation_enablement_ready,
        would_enable_runner_activation: false,
        would_activate_runner: false,
        would_return_success: false,
        would_touch_disk: false,
        would_unblock_activation_path: false,
        required_enablements: activation_enablement_dry_run.required_enablements.clone(),
        passed_enablements: activation_enablement_dry_run.passed_enablements.clone(),
        blocked_enablements: activation_enablement_dry_run.blocked_enablements.clone(),
        inherited_activation_enablement_blocked_gates: activation_enablement_dry_run
            .blocked_gates
            .clone(),
        required_gates,
        passed_gates,
        blocked_gates,
    }
}

fn contract_repair_approval_approve_execution_runner_activation_path_dry_run(
    activation_enablement_dry_run: &ContractRepairApprovalApproveExecutionRunnerActivationEnablementDryRun,
) -> ContractRepairApprovalApproveExecutionRunnerActivationPathDryRun {
    let structural_plan_ready = activation_enablement_dry_run.structural_plan_ready;
    let runner_control_ready = activation_enablement_dry_run.runner_control_ready;
    let phase_chain_ready = activation_enablement_dry_run.phase_chain_ready;
    let rollback_chain_ready = activation_enablement_dry_run.rollback_chain_ready;
    let side_effect_enablement_ready = activation_enablement_dry_run.side_effect_enablement_ready;
    let activation_enablement_ready =
        activation_enablement_dry_run.runner_activation_enablement_ready;
    let prior_enablements_ready = activation_enablement_dry_run.prior_enablements_ready;
    let atomic_activation_required = true;
    let activation_path_ready = structural_plan_ready
        && runner_control_ready
        && phase_chain_ready
        && rollback_chain_ready
        && side_effect_enablement_ready
        && activation_enablement_ready
        && prior_enablements_ready
        && atomic_activation_required;
    let activation_steps = activation_enablement_dry_run.required_enablements.clone();
    let required_gates = vec![
        "approve_execution_runner_structural_plan_ready".to_string(),
        "approve_execution_runner_control_ready".to_string(),
        "approve_execution_runner_phase_chain_ready".to_string(),
        "approve_execution_runner_rollback_chain_ready".to_string(),
        "approve_execution_runner_side_effect_enablement_ready".to_string(),
        "approve_execution_runner_activation_enablement_ready".to_string(),
        "approve_execution_runner_prior_enablements_ready".to_string(),
        "approve_execution_runner_atomic_activation_required".to_string(),
    ];
    let gate_states = [
        (
            "approve_execution_runner_structural_plan_ready",
            structural_plan_ready,
        ),
        (
            "approve_execution_runner_control_ready",
            runner_control_ready,
        ),
        (
            "approve_execution_runner_phase_chain_ready",
            phase_chain_ready,
        ),
        (
            "approve_execution_runner_rollback_chain_ready",
            rollback_chain_ready,
        ),
        (
            "approve_execution_runner_side_effect_enablement_ready",
            side_effect_enablement_ready,
        ),
        (
            "approve_execution_runner_activation_enablement_ready",
            activation_enablement_ready,
        ),
        (
            "approve_execution_runner_prior_enablements_ready",
            prior_enablements_ready,
        ),
        (
            "approve_execution_runner_atomic_activation_required",
            atomic_activation_required,
        ),
    ];
    let mut passed_gates = Vec::new();
    let mut blocked_gates = Vec::new();
    for (gate, passed) in gate_states {
        if passed {
            passed_gates.push(gate.to_string());
        } else {
            blocked_gates.push(gate.to_string());
        }
    }

    ContractRepairApprovalApproveExecutionRunnerActivationPathDryRun {
        status: if activation_path_ready {
            "approve_execution_runner_activation_path_ready".to_string()
        } else if activation_enablement_dry_run.action == "approve" && structural_plan_ready {
            "approve_execution_runner_activation_path_ready_blocked".to_string()
        } else {
            "approve_execution_runner_activation_path_blocked".to_string()
        },
        action: activation_enablement_dry_run.action.clone(),
        approval_id: activation_enablement_dry_run.approval_id.clone(),
        path_name: "approve_execution_runner_atomic_activation_path".to_string(),
        structural_plan_ready,
        runner_control_ready,
        phase_chain_ready,
        rollback_chain_ready,
        side_effect_enablement_ready,
        activation_enablement_ready,
        prior_enablements_ready,
        atomic_activation_required,
        activation_path_ready,
        activation_steps,
        would_enable_any_switch: false,
        would_activate_runner: false,
        would_return_success: false,
        would_touch_disk: false,
        required_enablements: activation_enablement_dry_run.required_enablements.clone(),
        passed_enablements: activation_enablement_dry_run.passed_enablements.clone(),
        blocked_enablements: activation_enablement_dry_run.blocked_enablements.clone(),
        required_gates,
        passed_gates,
        blocked_gates,
    }
}

fn contract_repair_approval_approve_execution_runner_activation_path_readiness_dry_run(
    activation_path_dry_run: &ContractRepairApprovalApproveExecutionRunnerActivationPathDryRun,
    activation_enablement_readiness_dry_run: &ContractRepairApprovalApproveExecutionRunnerActivationEnablementReadinessDryRun,
) -> ContractRepairApprovalApproveExecutionRunnerActivationPathReadinessDryRun {
    let structural_plan_ready = activation_path_dry_run.structural_plan_ready;
    let runner_control_ready = activation_path_dry_run.runner_control_ready;
    let phase_chain_ready = activation_path_dry_run.phase_chain_ready;
    let rollback_chain_ready = activation_path_dry_run.rollback_chain_ready;
    let side_effect_enablement_ready = activation_path_dry_run.side_effect_enablement_ready;
    let activation_enablement_ready = activation_path_dry_run.activation_enablement_ready;
    let activation_enablement_readiness_ready =
        activation_enablement_readiness_dry_run.runner_activation_enablement_ready;
    let prior_enablements_ready = activation_path_dry_run.prior_enablements_ready;
    let atomic_activation_required = activation_path_dry_run.atomic_activation_required;
    let activation_path_ready = activation_path_dry_run.activation_path_ready;
    let activation_steps = activation_path_dry_run.activation_steps.clone();
    let activation_step_count = activation_steps.len();
    let required_gates = activation_path_dry_run.required_gates.clone();
    let passed_gates = activation_path_dry_run.passed_gates.clone();
    let blocked_gates = activation_path_dry_run.blocked_gates.clone();

    ContractRepairApprovalApproveExecutionRunnerActivationPathReadinessDryRun {
        status: if activation_path_ready {
            "approve_execution_runner_activation_path_readiness_ready".to_string()
        } else if activation_path_dry_run.action == "approve" && atomic_activation_required {
            "approve_execution_runner_activation_path_readiness_ready_blocked".to_string()
        } else {
            "approve_execution_runner_activation_path_readiness_blocked".to_string()
        },
        action: activation_path_dry_run.action.clone(),
        approval_id: activation_path_dry_run.approval_id.clone(),
        gate_name: "approve_execution_runner_activation_path_ready".to_string(),
        path_name: activation_path_dry_run.path_name.clone(),
        structural_plan_ready,
        runner_control_ready,
        phase_chain_ready,
        rollback_chain_ready,
        side_effect_enablement_ready,
        activation_enablement_ready,
        activation_enablement_readiness_ready,
        prior_enablements_ready,
        atomic_activation_required,
        activation_path_ready,
        activation_step_count,
        activation_steps,
        would_enable_any_switch: false,
        would_activate_runner: false,
        would_return_success: false,
        would_touch_disk: false,
        would_unblock_activation_execution_plan: false,
        required_enablements: activation_path_dry_run.required_enablements.clone(),
        passed_enablements: activation_path_dry_run.passed_enablements.clone(),
        blocked_enablements: activation_path_dry_run.blocked_enablements.clone(),
        inherited_path_blocked_gates: activation_path_dry_run.blocked_gates.clone(),
        inherited_activation_enablement_readiness_blocked_gates:
            activation_enablement_readiness_dry_run
                .blocked_gates
                .clone(),
        required_gates,
        passed_gates,
        blocked_gates,
    }
}

fn contract_repair_approval_approve_execution_runner_activation_execution_plan_dry_run(
    activation_path_dry_run: &ContractRepairApprovalApproveExecutionRunnerActivationPathDryRun,
) -> ContractRepairApprovalApproveExecutionRunnerActivationExecutionPlanDryRun {
    let activation_path_ready = activation_path_dry_run.activation_path_ready;
    let atomic_activation_required = activation_path_dry_run.atomic_activation_required;
    let atomic_write_set = activation_path_dry_run.activation_steps.clone();
    let activation_step_count = atomic_write_set.len();
    let write_set_ready = !atomic_write_set.is_empty()
        && atomic_write_set == activation_path_dry_run.required_enablements;
    let all_or_nothing_guard_ready = atomic_activation_required;
    let rollback_boundary_ready = write_set_ready && all_or_nothing_guard_ready;
    let no_partial_activation_allowed = true;
    let activation_execution_plan_ready = activation_path_ready
        && write_set_ready
        && all_or_nothing_guard_ready
        && rollback_boundary_ready
        && no_partial_activation_allowed;
    let required_gates = vec![
        "approve_execution_runner_activation_path_ready".to_string(),
        "approve_execution_runner_atomic_write_set_ready".to_string(),
        "approve_execution_runner_all_or_nothing_guard_ready".to_string(),
        "approve_execution_runner_rollback_boundary_ready".to_string(),
        "approve_execution_runner_no_partial_activation_allowed".to_string(),
    ];
    let gate_states = [
        (
            "approve_execution_runner_activation_path_ready",
            activation_path_ready,
        ),
        (
            "approve_execution_runner_atomic_write_set_ready",
            write_set_ready,
        ),
        (
            "approve_execution_runner_all_or_nothing_guard_ready",
            all_or_nothing_guard_ready,
        ),
        (
            "approve_execution_runner_rollback_boundary_ready",
            rollback_boundary_ready,
        ),
        (
            "approve_execution_runner_no_partial_activation_allowed",
            no_partial_activation_allowed,
        ),
    ];
    let mut passed_gates = Vec::new();
    let mut blocked_gates = Vec::new();
    for (gate, passed) in gate_states {
        if passed {
            passed_gates.push(gate.to_string());
        } else {
            blocked_gates.push(gate.to_string());
        }
    }

    ContractRepairApprovalApproveExecutionRunnerActivationExecutionPlanDryRun {
        status: if activation_execution_plan_ready {
            "approve_execution_runner_activation_execution_plan_ready".to_string()
        } else if activation_path_dry_run.action == "approve"
            && write_set_ready
            && all_or_nothing_guard_ready
        {
            "approve_execution_runner_activation_execution_plan_ready_blocked".to_string()
        } else {
            "approve_execution_runner_activation_execution_plan_blocked".to_string()
        },
        action: activation_path_dry_run.action.clone(),
        approval_id: activation_path_dry_run.approval_id.clone(),
        plan_name: "approve_execution_runner_guarded_atomic_activation_execution_plan".to_string(),
        path_name: activation_path_dry_run.path_name.clone(),
        activation_path_ready,
        atomic_activation_required,
        write_set_ready,
        all_or_nothing_guard_ready,
        rollback_boundary_ready,
        no_partial_activation_allowed,
        activation_execution_plan_ready,
        activation_step_count,
        atomic_write_set,
        would_enable_any_switch: false,
        would_persist_activation_switches: false,
        would_activate_runner: false,
        would_return_success: false,
        would_touch_disk: false,
        required_enablements: activation_path_dry_run.required_enablements.clone(),
        passed_enablements: activation_path_dry_run.passed_enablements.clone(),
        blocked_enablements: activation_path_dry_run.blocked_enablements.clone(),
        inherited_path_blocked_gates: activation_path_dry_run.blocked_gates.clone(),
        required_gates,
        passed_gates,
        blocked_gates,
    }
}

fn contract_repair_approval_approve_execution_runner_activation_execution_plan_readiness_dry_run(
    activation_execution_plan_dry_run: &ContractRepairApprovalApproveExecutionRunnerActivationExecutionPlanDryRun,
    activation_path_readiness_dry_run: &ContractRepairApprovalApproveExecutionRunnerActivationPathReadinessDryRun,
) -> ContractRepairApprovalApproveExecutionRunnerActivationExecutionPlanReadinessDryRun {
    let activation_path_ready = activation_execution_plan_dry_run.activation_path_ready;
    let activation_path_readiness_ready = activation_path_readiness_dry_run.activation_path_ready;
    let atomic_activation_required = activation_execution_plan_dry_run.atomic_activation_required;
    let write_set_ready = activation_execution_plan_dry_run.write_set_ready;
    let all_or_nothing_guard_ready = activation_execution_plan_dry_run.all_or_nothing_guard_ready;
    let rollback_boundary_ready = activation_execution_plan_dry_run.rollback_boundary_ready;
    let no_partial_activation_allowed =
        activation_execution_plan_dry_run.no_partial_activation_allowed;
    let activation_execution_plan_ready =
        activation_execution_plan_dry_run.activation_execution_plan_ready;
    let atomic_write_set = activation_execution_plan_dry_run.atomic_write_set.clone();
    let activation_step_count = activation_execution_plan_dry_run.activation_step_count;
    let required_gates = activation_execution_plan_dry_run.required_gates.clone();
    let passed_gates = activation_execution_plan_dry_run.passed_gates.clone();
    let blocked_gates = activation_execution_plan_dry_run.blocked_gates.clone();

    ContractRepairApprovalApproveExecutionRunnerActivationExecutionPlanReadinessDryRun {
        status: if activation_execution_plan_ready {
            "approve_execution_runner_activation_execution_plan_readiness_ready".to_string()
        } else if activation_execution_plan_dry_run.action == "approve"
            && write_set_ready
            && all_or_nothing_guard_ready
            && rollback_boundary_ready
            && no_partial_activation_allowed
        {
            "approve_execution_runner_activation_execution_plan_readiness_ready_blocked".to_string()
        } else {
            "approve_execution_runner_activation_execution_plan_readiness_blocked".to_string()
        },
        action: activation_execution_plan_dry_run.action.clone(),
        approval_id: activation_execution_plan_dry_run.approval_id.clone(),
        gate_name: "approve_execution_runner_activation_execution_plan_ready".to_string(),
        plan_name: activation_execution_plan_dry_run.plan_name.clone(),
        path_name: activation_execution_plan_dry_run.path_name.clone(),
        activation_path_ready,
        activation_path_readiness_ready,
        atomic_activation_required,
        write_set_ready,
        all_or_nothing_guard_ready,
        rollback_boundary_ready,
        no_partial_activation_allowed,
        activation_execution_plan_ready,
        activation_step_count,
        atomic_write_set,
        would_enable_any_switch: false,
        would_persist_activation_switches: false,
        would_activate_runner: false,
        would_return_success: false,
        would_touch_disk: false,
        would_unblock_switch_transaction_proof: false,
        required_enablements: activation_execution_plan_dry_run
            .required_enablements
            .clone(),
        passed_enablements: activation_execution_plan_dry_run.passed_enablements.clone(),
        blocked_enablements: activation_execution_plan_dry_run
            .blocked_enablements
            .clone(),
        inherited_execution_plan_blocked_gates: activation_execution_plan_dry_run
            .blocked_gates
            .clone(),
        inherited_path_readiness_blocked_gates: activation_path_readiness_dry_run
            .blocked_gates
            .clone(),
        required_gates,
        passed_gates,
        blocked_gates,
    }
}

fn contract_repair_approval_approve_execution_runner_activation_switch_transaction_proof_dry_run(
    activation_execution_plan_dry_run: &ContractRepairApprovalApproveExecutionRunnerActivationExecutionPlanDryRun,
) -> ContractRepairApprovalApproveExecutionRunnerActivationSwitchTransactionProofDryRun {
    let activation_execution_plan_ready =
        activation_execution_plan_dry_run.activation_execution_plan_ready;
    let write_set_ready = activation_execution_plan_dry_run.write_set_ready;
    let all_or_nothing_guard_ready = activation_execution_plan_dry_run.all_or_nothing_guard_ready;
    let atomic_write_set = activation_execution_plan_dry_run.atomic_write_set.clone();
    let atomic_write_set_count = atomic_write_set.len();
    let simulated_failure_points = atomic_write_set.clone();
    let simulated_failure_point_count = simulated_failure_points.len();
    let rollback_actions = atomic_write_set
        .iter()
        .rev()
        .map(|switch_name| format!("disable:{switch_name}"))
        .collect::<Vec<_>>();
    let rollback_action_count = rollback_actions.len();
    let failure_probe_coverage_ready =
        atomic_write_set_count > 0 && simulated_failure_point_count == atomic_write_set_count;
    let rollback_action_coverage_ready =
        atomic_write_set_count > 0 && rollback_action_count == atomic_write_set_count;
    let partial_enabled_after_failure_count = if failure_probe_coverage_ready
        && rollback_action_coverage_ready
        && all_or_nothing_guard_ready
    {
        0
    } else {
        atomic_write_set_count
    };
    let partial_state_proof_ready = partial_enabled_after_failure_count == 0;
    let transaction_failure_proof_ready = activation_execution_plan_ready
        && write_set_ready
        && all_or_nothing_guard_ready
        && failure_probe_coverage_ready
        && rollback_action_coverage_ready
        && partial_state_proof_ready;
    let required_gates = vec![
        "approve_execution_runner_activation_execution_plan_ready".to_string(),
        "approve_execution_runner_atomic_write_set_ready".to_string(),
        "approve_execution_runner_all_or_nothing_guard_ready".to_string(),
        "approve_execution_runner_failure_probe_coverage_ready".to_string(),
        "approve_execution_runner_rollback_action_coverage_ready".to_string(),
        "approve_execution_runner_partial_state_proof_ready".to_string(),
    ];
    let gate_states = [
        (
            "approve_execution_runner_activation_execution_plan_ready",
            activation_execution_plan_ready,
        ),
        (
            "approve_execution_runner_atomic_write_set_ready",
            write_set_ready,
        ),
        (
            "approve_execution_runner_all_or_nothing_guard_ready",
            all_or_nothing_guard_ready,
        ),
        (
            "approve_execution_runner_failure_probe_coverage_ready",
            failure_probe_coverage_ready,
        ),
        (
            "approve_execution_runner_rollback_action_coverage_ready",
            rollback_action_coverage_ready,
        ),
        (
            "approve_execution_runner_partial_state_proof_ready",
            partial_state_proof_ready,
        ),
    ];
    let mut passed_gates = Vec::new();
    let mut blocked_gates = Vec::new();
    for (gate, passed) in gate_states {
        if passed {
            passed_gates.push(gate.to_string());
        } else {
            blocked_gates.push(gate.to_string());
        }
    }

    ContractRepairApprovalApproveExecutionRunnerActivationSwitchTransactionProofDryRun {
        status: if transaction_failure_proof_ready {
            "approve_execution_runner_activation_switch_transaction_proof_ready".to_string()
        } else if activation_execution_plan_dry_run.action == "approve"
            && write_set_ready
            && all_or_nothing_guard_ready
            && failure_probe_coverage_ready
            && rollback_action_coverage_ready
            && partial_state_proof_ready
        {
            "approve_execution_runner_activation_switch_transaction_proof_ready_blocked".to_string()
        } else {
            "approve_execution_runner_activation_switch_transaction_proof_blocked".to_string()
        },
        action: activation_execution_plan_dry_run.action.clone(),
        approval_id: activation_execution_plan_dry_run.approval_id.clone(),
        transaction_name: "approve_execution_runner_activation_switch_write_transaction"
            .to_string(),
        plan_name: activation_execution_plan_dry_run.plan_name.clone(),
        activation_execution_plan_ready,
        write_set_ready,
        all_or_nothing_guard_ready,
        failure_probe_coverage_ready,
        rollback_action_coverage_ready,
        partial_state_proof_ready,
        transaction_failure_proof_ready,
        atomic_write_set_count,
        simulated_failure_point_count,
        rollback_action_count,
        partial_enabled_after_failure_count,
        atomic_write_set,
        simulated_failure_points,
        rollback_actions,
        would_write_switches: false,
        would_persist_partial_state: false,
        would_commit_transaction: false,
        would_activate_runner: false,
        would_return_success: false,
        would_touch_disk: false,
        inherited_execution_plan_blocked_gates: activation_execution_plan_dry_run
            .blocked_gates
            .clone(),
        required_gates,
        passed_gates,
        blocked_gates,
    }
}

fn contract_repair_approval_approve_execution_runner_activation_switch_transaction_proof_readiness_dry_run(
    transaction_proof_dry_run: &ContractRepairApprovalApproveExecutionRunnerActivationSwitchTransactionProofDryRun,
    execution_plan_readiness_dry_run: &ContractRepairApprovalApproveExecutionRunnerActivationExecutionPlanReadinessDryRun,
) -> ContractRepairApprovalApproveExecutionRunnerActivationSwitchTransactionProofReadinessDryRun {
    let activation_execution_plan_ready = transaction_proof_dry_run.activation_execution_plan_ready;
    let activation_execution_plan_readiness_ready =
        execution_plan_readiness_dry_run.activation_execution_plan_ready;
    let write_set_ready = transaction_proof_dry_run.write_set_ready;
    let all_or_nothing_guard_ready = transaction_proof_dry_run.all_or_nothing_guard_ready;
    let failure_probe_coverage_ready = transaction_proof_dry_run.failure_probe_coverage_ready;
    let rollback_action_coverage_ready = transaction_proof_dry_run.rollback_action_coverage_ready;
    let partial_state_proof_ready = transaction_proof_dry_run.partial_state_proof_ready;
    let transaction_failure_proof_ready = transaction_proof_dry_run.transaction_failure_proof_ready;
    let required_gates = transaction_proof_dry_run.required_gates.clone();
    let passed_gates = transaction_proof_dry_run.passed_gates.clone();
    let blocked_gates = transaction_proof_dry_run.blocked_gates.clone();

    ContractRepairApprovalApproveExecutionRunnerActivationSwitchTransactionProofReadinessDryRun {
        status: if transaction_failure_proof_ready {
            "approve_execution_runner_activation_switch_transaction_proof_readiness_ready"
                .to_string()
        } else if transaction_proof_dry_run.action == "approve"
            && write_set_ready
            && all_or_nothing_guard_ready
            && failure_probe_coverage_ready
            && rollback_action_coverage_ready
            && partial_state_proof_ready
        {
            "approve_execution_runner_activation_switch_transaction_proof_readiness_ready_blocked"
                .to_string()
        } else {
            "approve_execution_runner_activation_switch_transaction_proof_readiness_blocked"
                .to_string()
        },
        action: transaction_proof_dry_run.action.clone(),
        approval_id: transaction_proof_dry_run.approval_id.clone(),
        gate_name: "approve_execution_runner_activation_switch_transaction_proof_ready".to_string(),
        transaction_name: transaction_proof_dry_run.transaction_name.clone(),
        plan_name: transaction_proof_dry_run.plan_name.clone(),
        activation_execution_plan_ready,
        activation_execution_plan_readiness_ready,
        write_set_ready,
        all_or_nothing_guard_ready,
        failure_probe_coverage_ready,
        rollback_action_coverage_ready,
        partial_state_proof_ready,
        transaction_failure_proof_ready,
        atomic_write_set_count: transaction_proof_dry_run.atomic_write_set_count,
        simulated_failure_point_count: transaction_proof_dry_run.simulated_failure_point_count,
        rollback_action_count: transaction_proof_dry_run.rollback_action_count,
        partial_enabled_after_failure_count: transaction_proof_dry_run
            .partial_enabled_after_failure_count,
        atomic_write_set: transaction_proof_dry_run.atomic_write_set.clone(),
        simulated_failure_points: transaction_proof_dry_run.simulated_failure_points.clone(),
        rollback_actions: transaction_proof_dry_run.rollback_actions.clone(),
        would_write_switches: false,
        would_persist_partial_state: false,
        would_commit_transaction: false,
        would_activate_runner: false,
        would_return_success: false,
        would_touch_disk: false,
        would_unblock_switch_write_transaction_enablement: false,
        inherited_transaction_proof_blocked_gates: transaction_proof_dry_run.blocked_gates.clone(),
        inherited_execution_plan_readiness_blocked_gates: execution_plan_readiness_dry_run
            .blocked_gates
            .clone(),
        required_gates,
        passed_gates,
        blocked_gates,
    }
}

fn contract_repair_approval_approve_execution_runner_activation_switch_write_transaction_enablement_dry_run(
    transaction_proof_dry_run: &ContractRepairApprovalApproveExecutionRunnerActivationSwitchTransactionProofDryRun,
    transaction_enabled: bool,
) -> ContractRepairApprovalApproveExecutionRunnerActivationSwitchWriteTransactionEnablementDryRun {
    let transaction_failure_proof_ready = transaction_proof_dry_run.transaction_failure_proof_ready;
    let write_set_ready = transaction_proof_dry_run.write_set_ready;
    let all_or_nothing_guard_ready = transaction_proof_dry_run.all_or_nothing_guard_ready;
    let failure_probe_coverage_ready = transaction_proof_dry_run.failure_probe_coverage_ready;
    let rollback_action_coverage_ready = transaction_proof_dry_run.rollback_action_coverage_ready;
    let partial_state_proof_ready = transaction_proof_dry_run.partial_state_proof_ready;
    let partial_enabled_after_failure_count =
        transaction_proof_dry_run.partial_enabled_after_failure_count;
    let enablement_prerequisites_ready = transaction_failure_proof_ready
        && write_set_ready
        && all_or_nothing_guard_ready
        && failure_probe_coverage_ready
        && rollback_action_coverage_ready
        && partial_state_proof_ready
        && partial_enabled_after_failure_count == 0;
    let switch_write_transaction_enablement_ready =
        enablement_prerequisites_ready && transaction_enabled;
    let required_gates = vec![
        "approve_execution_runner_activation_switch_transaction_proof_ready".to_string(),
        "approve_execution_runner_atomic_write_set_ready".to_string(),
        "approve_execution_runner_all_or_nothing_guard_ready".to_string(),
        "approve_execution_runner_failure_probe_coverage_ready".to_string(),
        "approve_execution_runner_rollback_action_coverage_ready".to_string(),
        "approve_execution_runner_partial_state_proof_ready".to_string(),
        "approve_execution_runner_activation_switch_write_transaction_enabled".to_string(),
    ];
    let gate_states = [
        (
            "approve_execution_runner_activation_switch_transaction_proof_ready",
            transaction_failure_proof_ready,
        ),
        (
            "approve_execution_runner_atomic_write_set_ready",
            write_set_ready,
        ),
        (
            "approve_execution_runner_all_or_nothing_guard_ready",
            all_or_nothing_guard_ready,
        ),
        (
            "approve_execution_runner_failure_probe_coverage_ready",
            failure_probe_coverage_ready,
        ),
        (
            "approve_execution_runner_rollback_action_coverage_ready",
            rollback_action_coverage_ready,
        ),
        (
            "approve_execution_runner_partial_state_proof_ready",
            partial_state_proof_ready,
        ),
        (
            "approve_execution_runner_activation_switch_write_transaction_enabled",
            transaction_enabled,
        ),
    ];
    let mut passed_gates = Vec::new();
    let mut blocked_gates = Vec::new();
    for (gate, passed) in gate_states {
        if passed {
            passed_gates.push(gate.to_string());
        } else {
            blocked_gates.push(gate.to_string());
        }
    }

    ContractRepairApprovalApproveExecutionRunnerActivationSwitchWriteTransactionEnablementDryRun {
        status: if switch_write_transaction_enablement_ready {
            "approve_execution_runner_activation_switch_write_transaction_enablement_ready"
                .to_string()
        } else if transaction_proof_dry_run.action == "approve"
            && write_set_ready
            && all_or_nothing_guard_ready
            && failure_probe_coverage_ready
            && rollback_action_coverage_ready
            && partial_state_proof_ready
        {
            "approve_execution_runner_activation_switch_write_transaction_enablement_ready_blocked"
                .to_string()
        } else {
            "approve_execution_runner_activation_switch_write_transaction_enablement_blocked"
                .to_string()
        },
        action: transaction_proof_dry_run.action.clone(),
        approval_id: transaction_proof_dry_run.approval_id.clone(),
        switch_name: "approve_execution_runner_activation_switch_write_transaction_enabled"
            .to_string(),
        transaction_name: transaction_proof_dry_run.transaction_name.clone(),
        transaction_failure_proof_ready,
        write_set_ready,
        all_or_nothing_guard_ready,
        failure_probe_coverage_ready,
        rollback_action_coverage_ready,
        partial_state_proof_ready,
        partial_enabled_after_failure_count,
        transaction_enabled,
        enablement_prerequisites_ready,
        switch_write_transaction_enablement_ready,
        atomic_write_set: transaction_proof_dry_run.atomic_write_set.clone(),
        rollback_actions: transaction_proof_dry_run.rollback_actions.clone(),
        would_enable_transaction: false,
        would_write_switches: false,
        would_persist_partial_state: false,
        would_commit_transaction: false,
        would_activate_runner: false,
        would_return_success: false,
        would_touch_disk: false,
        inherited_transaction_proof_blocked_gates: transaction_proof_dry_run.blocked_gates.clone(),
        required_gates,
        passed_gates,
        blocked_gates,
    }
}

fn contract_repair_approval_approve_execution_runner_activation_switch_write_transaction_enablement_readiness_dry_run(
    transaction_enablement_dry_run: &ContractRepairApprovalApproveExecutionRunnerActivationSwitchWriteTransactionEnablementDryRun,
    transaction_proof_readiness_dry_run: &ContractRepairApprovalApproveExecutionRunnerActivationSwitchTransactionProofReadinessDryRun,
) -> ContractRepairApprovalApproveExecutionRunnerActivationSwitchWriteTransactionEnablementReadinessDryRun{
    let transaction_failure_proof_ready =
        transaction_enablement_dry_run.transaction_failure_proof_ready;
    let transaction_failure_proof_readiness_ready =
        transaction_proof_readiness_dry_run.transaction_failure_proof_ready;
    let write_set_ready = transaction_enablement_dry_run.write_set_ready;
    let all_or_nothing_guard_ready = transaction_enablement_dry_run.all_or_nothing_guard_ready;
    let failure_probe_coverage_ready = transaction_enablement_dry_run.failure_probe_coverage_ready;
    let rollback_action_coverage_ready =
        transaction_enablement_dry_run.rollback_action_coverage_ready;
    let partial_state_proof_ready = transaction_enablement_dry_run.partial_state_proof_ready;
    let partial_enabled_after_failure_count =
        transaction_enablement_dry_run.partial_enabled_after_failure_count;
    let transaction_enabled = transaction_enablement_dry_run.transaction_enabled;
    let enablement_prerequisites_ready =
        transaction_enablement_dry_run.enablement_prerequisites_ready;
    let switch_write_transaction_enablement_ready =
        transaction_enablement_dry_run.switch_write_transaction_enablement_ready;
    let required_gates = transaction_enablement_dry_run.required_gates.clone();
    let passed_gates = transaction_enablement_dry_run.passed_gates.clone();
    let blocked_gates = transaction_enablement_dry_run.blocked_gates.clone();

    ContractRepairApprovalApproveExecutionRunnerActivationSwitchWriteTransactionEnablementReadinessDryRun {
        status: if switch_write_transaction_enablement_ready {
            "approve_execution_runner_activation_switch_write_transaction_enablement_readiness_ready"
                .to_string()
        } else if transaction_enablement_dry_run.action == "approve"
            && write_set_ready
            && all_or_nothing_guard_ready
            && failure_probe_coverage_ready
            && rollback_action_coverage_ready
            && partial_state_proof_ready
            && partial_enabled_after_failure_count == 0
        {
            "approve_execution_runner_activation_switch_write_transaction_enablement_readiness_ready_blocked"
                .to_string()
        } else {
            "approve_execution_runner_activation_switch_write_transaction_enablement_readiness_blocked"
                .to_string()
        },
        action: transaction_enablement_dry_run.action.clone(),
        approval_id: transaction_enablement_dry_run.approval_id.clone(),
        gate_name:
            "approve_execution_runner_activation_switch_write_transaction_enablement_ready"
                .to_string(),
        switch_name: transaction_enablement_dry_run.switch_name.clone(),
        transaction_name: transaction_enablement_dry_run.transaction_name.clone(),
        transaction_failure_proof_ready,
        transaction_failure_proof_readiness_ready,
        write_set_ready,
        all_or_nothing_guard_ready,
        failure_probe_coverage_ready,
        rollback_action_coverage_ready,
        partial_state_proof_ready,
        partial_enabled_after_failure_count,
        transaction_enabled,
        enablement_prerequisites_ready,
        switch_write_transaction_enablement_ready,
        atomic_write_set: transaction_enablement_dry_run.atomic_write_set.clone(),
        rollback_actions: transaction_enablement_dry_run.rollback_actions.clone(),
        would_enable_transaction: false,
        would_write_switches: false,
        would_persist_partial_state: false,
        would_commit_transaction: false,
        would_activate_runner: false,
        would_return_success: false,
        would_touch_disk: false,
        would_unblock_switch_write_transaction: false,
        inherited_enablement_blocked_gates: transaction_enablement_dry_run.blocked_gates.clone(),
        inherited_transaction_proof_readiness_blocked_gates: transaction_proof_readiness_dry_run
            .blocked_gates
            .clone(),
        required_gates,
        passed_gates,
        blocked_gates,
    }
}

fn contract_repair_approval_approve_execution_runner_activation_switch_write_transaction_dry_run(
    transaction_enablement_dry_run: &ContractRepairApprovalApproveExecutionRunnerActivationSwitchWriteTransactionEnablementDryRun,
) -> ContractRepairApprovalApproveExecutionRunnerActivationSwitchWriteTransactionDryRun {
    let transaction_failure_proof_ready =
        transaction_enablement_dry_run.transaction_failure_proof_ready;
    let write_set_ready = transaction_enablement_dry_run.write_set_ready;
    let all_or_nothing_guard_ready = transaction_enablement_dry_run.all_or_nothing_guard_ready;
    let failure_probe_coverage_ready = transaction_enablement_dry_run.failure_probe_coverage_ready;
    let rollback_action_coverage_ready =
        transaction_enablement_dry_run.rollback_action_coverage_ready;
    let partial_state_proof_ready = transaction_enablement_dry_run.partial_state_proof_ready;
    let partial_enabled_after_failure_count =
        transaction_enablement_dry_run.partial_enabled_after_failure_count;
    let transaction_enabled = transaction_enablement_dry_run.transaction_enabled;
    let transaction_prerequisites_ready = transaction_failure_proof_ready
        && write_set_ready
        && all_or_nothing_guard_ready
        && failure_probe_coverage_ready
        && rollback_action_coverage_ready
        && partial_state_proof_ready
        && partial_enabled_after_failure_count == 0;
    let switch_write_transaction_ready = transaction_prerequisites_ready && transaction_enabled;
    let atomic_write_set = transaction_enablement_dry_run.atomic_write_set.clone();
    let activation_switch_write_order = atomic_write_set.clone();
    let rollback_actions = transaction_enablement_dry_run.rollback_actions.clone();
    let required_gates = vec![
        "approve_execution_runner_activation_switch_transaction_proof_ready".to_string(),
        "approve_execution_runner_atomic_write_set_ready".to_string(),
        "approve_execution_runner_all_or_nothing_guard_ready".to_string(),
        "approve_execution_runner_failure_probe_coverage_ready".to_string(),
        "approve_execution_runner_rollback_action_coverage_ready".to_string(),
        "approve_execution_runner_partial_state_proof_ready".to_string(),
        "approve_execution_runner_activation_switch_write_transaction_enabled".to_string(),
    ];
    let gate_states = [
        (
            "approve_execution_runner_activation_switch_transaction_proof_ready",
            transaction_failure_proof_ready,
        ),
        (
            "approve_execution_runner_atomic_write_set_ready",
            write_set_ready,
        ),
        (
            "approve_execution_runner_all_or_nothing_guard_ready",
            all_or_nothing_guard_ready,
        ),
        (
            "approve_execution_runner_failure_probe_coverage_ready",
            failure_probe_coverage_ready,
        ),
        (
            "approve_execution_runner_rollback_action_coverage_ready",
            rollback_action_coverage_ready,
        ),
        (
            "approve_execution_runner_partial_state_proof_ready",
            partial_state_proof_ready,
        ),
        (
            "approve_execution_runner_activation_switch_write_transaction_enabled",
            transaction_enabled,
        ),
    ];
    let mut passed_gates = Vec::new();
    let mut blocked_gates = Vec::new();
    for (gate, passed) in gate_states {
        if passed {
            passed_gates.push(gate.to_string());
        } else {
            blocked_gates.push(gate.to_string());
        }
    }

    ContractRepairApprovalApproveExecutionRunnerActivationSwitchWriteTransactionDryRun {
        status: if switch_write_transaction_ready {
            "approve_execution_runner_activation_switch_write_transaction_ready".to_string()
        } else if transaction_enablement_dry_run.action == "approve"
            && write_set_ready
            && all_or_nothing_guard_ready
            && failure_probe_coverage_ready
            && rollback_action_coverage_ready
            && partial_state_proof_ready
        {
            "approve_execution_runner_activation_switch_write_transaction_ready_blocked".to_string()
        } else {
            "approve_execution_runner_activation_switch_write_transaction_blocked".to_string()
        },
        action: transaction_enablement_dry_run.action.clone(),
        approval_id: transaction_enablement_dry_run.approval_id.clone(),
        transaction_name: transaction_enablement_dry_run.transaction_name.clone(),
        transaction_failure_proof_ready,
        write_set_ready,
        all_or_nothing_guard_ready,
        failure_probe_coverage_ready,
        rollback_action_coverage_ready,
        partial_state_proof_ready,
        transaction_enabled,
        transaction_prerequisites_ready,
        switch_write_transaction_ready,
        partial_enabled_after_failure_count,
        activation_switch_write_order,
        atomic_write_set,
        rollback_actions,
        would_write_switches: false,
        would_persist_partial_state: false,
        would_commit_transaction: false,
        would_activate_runner: false,
        would_return_success: false,
        would_touch_disk: false,
        inherited_transaction_proof_blocked_gates: transaction_enablement_dry_run
            .inherited_transaction_proof_blocked_gates
            .clone(),
        required_gates,
        passed_gates,
        blocked_gates,
    }
}

fn contract_repair_approval_approve_execution_runner_activation_switch_write_transaction_readiness_dry_run(
    switch_write_transaction_dry_run: &ContractRepairApprovalApproveExecutionRunnerActivationSwitchWriteTransactionDryRun,
    transaction_enablement_readiness_dry_run: &ContractRepairApprovalApproveExecutionRunnerActivationSwitchWriteTransactionEnablementReadinessDryRun,
) -> ContractRepairApprovalApproveExecutionRunnerActivationSwitchWriteTransactionReadinessDryRun {
    let transaction_failure_proof_ready =
        switch_write_transaction_dry_run.transaction_failure_proof_ready;
    let switch_write_transaction_enablement_readiness_ready =
        transaction_enablement_readiness_dry_run.switch_write_transaction_enablement_ready;
    let write_set_ready = switch_write_transaction_dry_run.write_set_ready;
    let all_or_nothing_guard_ready = switch_write_transaction_dry_run.all_or_nothing_guard_ready;
    let failure_probe_coverage_ready =
        switch_write_transaction_dry_run.failure_probe_coverage_ready;
    let rollback_action_coverage_ready =
        switch_write_transaction_dry_run.rollback_action_coverage_ready;
    let partial_state_proof_ready = switch_write_transaction_dry_run.partial_state_proof_ready;
    let transaction_enabled = switch_write_transaction_dry_run.transaction_enabled;
    let transaction_prerequisites_ready =
        switch_write_transaction_dry_run.transaction_prerequisites_ready;
    let switch_write_transaction_ready =
        switch_write_transaction_dry_run.switch_write_transaction_ready;
    let partial_enabled_after_failure_count =
        switch_write_transaction_dry_run.partial_enabled_after_failure_count;
    let required_gates = switch_write_transaction_dry_run.required_gates.clone();
    let passed_gates = switch_write_transaction_dry_run.passed_gates.clone();
    let blocked_gates = switch_write_transaction_dry_run.blocked_gates.clone();

    ContractRepairApprovalApproveExecutionRunnerActivationSwitchWriteTransactionReadinessDryRun {
        status: if switch_write_transaction_ready {
            "approve_execution_runner_activation_switch_write_transaction_readiness_ready"
                .to_string()
        } else if switch_write_transaction_dry_run.action == "approve"
            && write_set_ready
            && all_or_nothing_guard_ready
            && failure_probe_coverage_ready
            && rollback_action_coverage_ready
            && partial_state_proof_ready
        {
            "approve_execution_runner_activation_switch_write_transaction_readiness_ready_blocked"
                .to_string()
        } else {
            "approve_execution_runner_activation_switch_write_transaction_readiness_blocked"
                .to_string()
        },
        action: switch_write_transaction_dry_run.action.clone(),
        approval_id: switch_write_transaction_dry_run.approval_id.clone(),
        gate_name: "approve_execution_runner_activation_switch_write_transaction_ready".to_string(),
        transaction_name: switch_write_transaction_dry_run.transaction_name.clone(),
        transaction_failure_proof_ready,
        switch_write_transaction_enablement_readiness_ready,
        write_set_ready,
        all_or_nothing_guard_ready,
        failure_probe_coverage_ready,
        rollback_action_coverage_ready,
        partial_state_proof_ready,
        transaction_enabled,
        transaction_prerequisites_ready,
        switch_write_transaction_ready,
        partial_enabled_after_failure_count,
        activation_switch_write_order: switch_write_transaction_dry_run
            .activation_switch_write_order
            .clone(),
        atomic_write_set: switch_write_transaction_dry_run.atomic_write_set.clone(),
        rollback_actions: switch_write_transaction_dry_run.rollback_actions.clone(),
        would_write_switches: false,
        would_persist_partial_state: false,
        would_commit_transaction: false,
        would_activate_runner: false,
        would_return_success: false,
        would_touch_disk: false,
        would_unblock_activation_transaction_admission: false,
        inherited_switch_write_transaction_blocked_gates: switch_write_transaction_dry_run
            .blocked_gates
            .clone(),
        inherited_enablement_readiness_blocked_gates: transaction_enablement_readiness_dry_run
            .blocked_gates
            .clone(),
        required_gates,
        passed_gates,
        blocked_gates,
    }
}

fn contract_repair_approval_approve_execution_runner_activation_transaction_admission_gate_dry_run(
    activation_path_dry_run: &ContractRepairApprovalApproveExecutionRunnerActivationPathDryRun,
    activation_execution_plan_dry_run: &ContractRepairApprovalApproveExecutionRunnerActivationExecutionPlanDryRun,
    transaction_proof_dry_run: &ContractRepairApprovalApproveExecutionRunnerActivationSwitchTransactionProofDryRun,
    transaction_enablement_dry_run: &ContractRepairApprovalApproveExecutionRunnerActivationSwitchWriteTransactionEnablementDryRun,
    switch_write_transaction_dry_run: &ContractRepairApprovalApproveExecutionRunnerActivationSwitchWriteTransactionDryRun,
) -> ContractRepairApprovalApproveExecutionRunnerActivationTransactionAdmissionGateDryRun {
    let activation_path_ready = activation_path_dry_run.activation_path_ready;
    let activation_execution_plan_ready =
        activation_execution_plan_dry_run.activation_execution_plan_ready;
    let transaction_failure_proof_ready = transaction_proof_dry_run.transaction_failure_proof_ready;
    let switch_write_transaction_enablement_ready =
        transaction_enablement_dry_run.switch_write_transaction_enablement_ready;
    let switch_write_transaction_ready =
        switch_write_transaction_dry_run.switch_write_transaction_ready;
    let transaction_enabled = switch_write_transaction_dry_run.transaction_enabled;
    let partial_enabled_after_failure_count =
        switch_write_transaction_dry_run.partial_enabled_after_failure_count;
    let activation_switch_write_order = switch_write_transaction_dry_run
        .activation_switch_write_order
        .clone();
    let atomic_write_set = switch_write_transaction_dry_run.atomic_write_set.clone();
    let rollback_actions = switch_write_transaction_dry_run.rollback_actions.clone();
    let transaction_shape_ready = switch_write_transaction_dry_run.write_set_ready
        && switch_write_transaction_dry_run.all_or_nothing_guard_ready
        && switch_write_transaction_dry_run.failure_probe_coverage_ready
        && switch_write_transaction_dry_run.rollback_action_coverage_ready
        && switch_write_transaction_dry_run.partial_state_proof_ready
        && partial_enabled_after_failure_count == 0
        && !activation_switch_write_order.is_empty()
        && !rollback_actions.is_empty();
    let activation_transaction_admission_ready = activation_path_ready
        && activation_execution_plan_ready
        && transaction_failure_proof_ready
        && switch_write_transaction_enablement_ready
        && switch_write_transaction_ready;
    let required_gates = vec![
        "approve_execution_runner_activation_path_ready".to_string(),
        "approve_execution_runner_activation_execution_plan_ready".to_string(),
        "approve_execution_runner_activation_switch_transaction_proof_ready".to_string(),
        "approve_execution_runner_activation_switch_write_transaction_enablement_ready".to_string(),
        "approve_execution_runner_activation_switch_write_transaction_ready".to_string(),
    ];
    let gate_states = [
        (
            "approve_execution_runner_activation_path_ready",
            activation_path_ready,
        ),
        (
            "approve_execution_runner_activation_execution_plan_ready",
            activation_execution_plan_ready,
        ),
        (
            "approve_execution_runner_activation_switch_transaction_proof_ready",
            transaction_failure_proof_ready,
        ),
        (
            "approve_execution_runner_activation_switch_write_transaction_enablement_ready",
            switch_write_transaction_enablement_ready,
        ),
        (
            "approve_execution_runner_activation_switch_write_transaction_ready",
            switch_write_transaction_ready,
        ),
    ];
    let mut passed_gates = Vec::new();
    let mut blocked_gates = Vec::new();
    for (gate, passed) in gate_states {
        if passed {
            passed_gates.push(gate.to_string());
        } else {
            blocked_gates.push(gate.to_string());
        }
    }

    ContractRepairApprovalApproveExecutionRunnerActivationTransactionAdmissionGateDryRun {
        status: if activation_transaction_admission_ready {
            "approve_execution_runner_activation_transaction_admission_ready".to_string()
        } else if switch_write_transaction_dry_run.action == "approve" && transaction_shape_ready {
            "approve_execution_runner_activation_transaction_admission_ready_blocked".to_string()
        } else {
            "approve_execution_runner_activation_transaction_admission_blocked".to_string()
        },
        action: switch_write_transaction_dry_run.action.clone(),
        approval_id: switch_write_transaction_dry_run.approval_id.clone(),
        gate_name: "approve_execution_runner_activation_transaction_admission_gate".to_string(),
        transaction_name: switch_write_transaction_dry_run.transaction_name.clone(),
        activation_path_ready,
        activation_execution_plan_ready,
        transaction_failure_proof_ready,
        switch_write_transaction_enablement_ready,
        switch_write_transaction_ready,
        transaction_enabled,
        transaction_shape_ready,
        activation_transaction_admission_ready,
        partial_enabled_after_failure_count,
        activation_switch_write_order,
        atomic_write_set,
        rollback_actions,
        would_admit_transaction: false,
        would_write_switches: false,
        would_persist_partial_state: false,
        would_commit_transaction: false,
        would_activate_runner: false,
        would_return_success: false,
        would_touch_disk: false,
        inherited_path_blocked_gates: activation_path_dry_run.blocked_gates.clone(),
        inherited_execution_plan_blocked_gates: activation_execution_plan_dry_run
            .blocked_gates
            .clone(),
        inherited_transaction_proof_blocked_gates: transaction_proof_dry_run.blocked_gates.clone(),
        inherited_transaction_enablement_blocked_gates: transaction_enablement_dry_run
            .blocked_gates
            .clone(),
        inherited_switch_write_transaction_blocked_gates: switch_write_transaction_dry_run
            .blocked_gates
            .clone(),
        required_gates,
        passed_gates,
        blocked_gates,
    }
}

fn contract_repair_approval_approve_execution_runner_activation_transaction_admission_readiness_dry_run(
    admission_gate_dry_run: &ContractRepairApprovalApproveExecutionRunnerActivationTransactionAdmissionGateDryRun,
    switch_write_transaction_readiness_dry_run: &ContractRepairApprovalApproveExecutionRunnerActivationSwitchWriteTransactionReadinessDryRun,
) -> ContractRepairApprovalApproveExecutionRunnerActivationTransactionAdmissionReadinessDryRun {
    let activation_path_ready = admission_gate_dry_run.activation_path_ready;
    let activation_execution_plan_ready = admission_gate_dry_run.activation_execution_plan_ready;
    let transaction_failure_proof_ready = admission_gate_dry_run.transaction_failure_proof_ready;
    let switch_write_transaction_enablement_ready =
        admission_gate_dry_run.switch_write_transaction_enablement_ready;
    let switch_write_transaction_ready = admission_gate_dry_run.switch_write_transaction_ready;
    let switch_write_transaction_readiness_ready =
        switch_write_transaction_readiness_dry_run.switch_write_transaction_ready;
    let transaction_enabled = admission_gate_dry_run.transaction_enabled;
    let transaction_shape_ready = admission_gate_dry_run.transaction_shape_ready;
    let activation_transaction_admission_ready =
        admission_gate_dry_run.activation_transaction_admission_ready;
    let partial_enabled_after_failure_count =
        admission_gate_dry_run.partial_enabled_after_failure_count;
    let required_gates = admission_gate_dry_run.required_gates.clone();
    let passed_gates = admission_gate_dry_run.passed_gates.clone();
    let blocked_gates = admission_gate_dry_run.blocked_gates.clone();

    ContractRepairApprovalApproveExecutionRunnerActivationTransactionAdmissionReadinessDryRun {
        status: if activation_transaction_admission_ready {
            "approve_execution_runner_activation_transaction_admission_readiness_ready".to_string()
        } else if admission_gate_dry_run.action == "approve" && transaction_shape_ready {
            "approve_execution_runner_activation_transaction_admission_readiness_ready_blocked"
                .to_string()
        } else {
            "approve_execution_runner_activation_transaction_admission_readiness_blocked"
                .to_string()
        },
        action: admission_gate_dry_run.action.clone(),
        approval_id: admission_gate_dry_run.approval_id.clone(),
        gate_name: "approve_execution_runner_activation_transaction_admission_ready".to_string(),
        source_gate_name: admission_gate_dry_run.gate_name.clone(),
        transaction_name: admission_gate_dry_run.transaction_name.clone(),
        activation_path_ready,
        activation_execution_plan_ready,
        transaction_failure_proof_ready,
        switch_write_transaction_enablement_ready,
        switch_write_transaction_ready,
        switch_write_transaction_readiness_ready,
        transaction_enabled,
        transaction_shape_ready,
        activation_transaction_admission_ready,
        partial_enabled_after_failure_count,
        activation_switch_write_order: admission_gate_dry_run.activation_switch_write_order.clone(),
        atomic_write_set: admission_gate_dry_run.atomic_write_set.clone(),
        rollback_actions: admission_gate_dry_run.rollback_actions.clone(),
        would_admit_transaction: false,
        would_write_switches: false,
        would_persist_partial_state: false,
        would_commit_transaction: false,
        would_activate_runner: false,
        would_return_success: false,
        would_touch_disk: false,
        would_unblock_activation_admission_handoff: false,
        inherited_admission_blocked_gates: admission_gate_dry_run.blocked_gates.clone(),
        inherited_switch_write_transaction_readiness_blocked_gates:
            switch_write_transaction_readiness_dry_run
                .blocked_gates
                .clone(),
        required_gates,
        passed_gates,
        blocked_gates,
    }
}

fn contract_repair_approval_approve_execution_runner_activation_admission_handoff_dry_run(
    enablement_plan_dry_run: &ContractRepairApprovalApproveExecutionRunnerEnablementPlanDryRun,
    activation_admission_gate_dry_run: &ContractRepairApprovalApproveExecutionRunnerActivationTransactionAdmissionGateDryRun,
) -> ContractRepairApprovalApproveExecutionRunnerActivationAdmissionHandoffDryRun {
    let structural_plan_ready = enablement_plan_dry_run.structural_plan_ready;
    let runner_control_ready = enablement_plan_dry_run.runner_control_ready;
    let phase_chain_ready = enablement_plan_dry_run.phase_chain_ready;
    let rollback_chain_ready = enablement_plan_dry_run.rollback_chain_ready;
    let side_effect_enablement_ready = enablement_plan_dry_run.side_effect_enablement_ready;
    let activation_admission_required = true;
    let activation_transaction_admission_ready =
        activation_admission_gate_dry_run.activation_transaction_admission_ready;
    let runner_activation_enabled = enablement_plan_dry_run.runner_activation_enabled;
    let activation_handoff_prerequisites_ready = structural_plan_ready
        && runner_control_ready
        && phase_chain_ready
        && rollback_chain_ready
        && side_effect_enablement_ready
        && activation_transaction_admission_ready;
    let runner_activation_handoff_ready =
        activation_handoff_prerequisites_ready && runner_activation_enabled;
    let activation_switch_write_order = activation_admission_gate_dry_run
        .activation_switch_write_order
        .clone();
    let required_gates = vec![
        "approve_execution_runner_structural_plan_ready".to_string(),
        "approve_execution_runner_control_ready".to_string(),
        "approve_execution_runner_phase_chain_ready".to_string(),
        "approve_execution_runner_rollback_chain_ready".to_string(),
        "approve_execution_runner_side_effect_enablement_ready".to_string(),
        "approve_execution_runner_activation_transaction_admission_ready".to_string(),
        "approve_execution_runner_activation_enabled".to_string(),
    ];
    let gate_states = [
        (
            "approve_execution_runner_structural_plan_ready",
            structural_plan_ready,
        ),
        (
            "approve_execution_runner_control_ready",
            runner_control_ready,
        ),
        (
            "approve_execution_runner_phase_chain_ready",
            phase_chain_ready,
        ),
        (
            "approve_execution_runner_rollback_chain_ready",
            rollback_chain_ready,
        ),
        (
            "approve_execution_runner_side_effect_enablement_ready",
            side_effect_enablement_ready,
        ),
        (
            "approve_execution_runner_activation_transaction_admission_ready",
            activation_transaction_admission_ready,
        ),
        (
            "approve_execution_runner_activation_enabled",
            runner_activation_enabled,
        ),
    ];
    let mut passed_gates = Vec::new();
    let mut blocked_gates = Vec::new();
    for (gate, passed) in gate_states {
        if passed {
            passed_gates.push(gate.to_string());
        } else {
            blocked_gates.push(gate.to_string());
        }
    }

    ContractRepairApprovalApproveExecutionRunnerActivationAdmissionHandoffDryRun {
        status: if runner_activation_handoff_ready {
            "approve_execution_runner_activation_admission_handoff_ready".to_string()
        } else if enablement_plan_dry_run.action == "approve"
            && structural_plan_ready
            && activation_admission_required
        {
            "approve_execution_runner_activation_admission_handoff_ready_blocked".to_string()
        } else {
            "approve_execution_runner_activation_admission_handoff_blocked".to_string()
        },
        action: enablement_plan_dry_run.action.clone(),
        approval_id: enablement_plan_dry_run.approval_id.clone(),
        handoff_name: "approve_execution_runner_activation_admission_handoff".to_string(),
        source_gate_name: activation_admission_gate_dry_run.gate_name.clone(),
        structural_plan_ready,
        runner_control_ready,
        phase_chain_ready,
        rollback_chain_ready,
        side_effect_enablement_ready,
        activation_admission_required,
        activation_transaction_admission_ready,
        runner_activation_enabled,
        activation_handoff_prerequisites_ready,
        runner_activation_handoff_ready,
        activation_switch_write_order,
        would_handoff_to_runner: false,
        would_activate_runner: false,
        would_return_success: false,
        would_touch_disk: false,
        inherited_activation_admission_blocked_gates: activation_admission_gate_dry_run
            .blocked_gates
            .clone(),
        inherited_runner_enablement_blocked_by: enablement_plan_dry_run.blocked_by.clone(),
        required_gates,
        passed_gates,
        blocked_gates,
    }
}

fn contract_repair_approval_approve_execution_runner_activation_admission_handoff_readiness_dry_run(
    handoff_dry_run: &ContractRepairApprovalApproveExecutionRunnerActivationAdmissionHandoffDryRun,
    admission_readiness_dry_run: &ContractRepairApprovalApproveExecutionRunnerActivationTransactionAdmissionReadinessDryRun,
) -> ContractRepairApprovalApproveExecutionRunnerActivationAdmissionHandoffReadinessDryRun {
    let structural_plan_ready = handoff_dry_run.structural_plan_ready;
    let runner_control_ready = handoff_dry_run.runner_control_ready;
    let phase_chain_ready = handoff_dry_run.phase_chain_ready;
    let rollback_chain_ready = handoff_dry_run.rollback_chain_ready;
    let side_effect_enablement_ready = handoff_dry_run.side_effect_enablement_ready;
    let activation_admission_required = handoff_dry_run.activation_admission_required;
    let activation_transaction_admission_ready =
        handoff_dry_run.activation_transaction_admission_ready;
    let activation_transaction_admission_readiness_ready =
        admission_readiness_dry_run.activation_transaction_admission_ready;
    let runner_activation_enabled = handoff_dry_run.runner_activation_enabled;
    let activation_handoff_prerequisites_ready =
        handoff_dry_run.activation_handoff_prerequisites_ready;
    let runner_activation_handoff_ready = handoff_dry_run.runner_activation_handoff_ready;
    let required_gates = handoff_dry_run.required_gates.clone();
    let passed_gates = handoff_dry_run.passed_gates.clone();
    let blocked_gates = handoff_dry_run.blocked_gates.clone();

    ContractRepairApprovalApproveExecutionRunnerActivationAdmissionHandoffReadinessDryRun {
        status: if runner_activation_handoff_ready {
            "approve_execution_runner_activation_admission_handoff_readiness_ready".to_string()
        } else if handoff_dry_run.action == "approve"
            && structural_plan_ready
            && activation_admission_required
        {
            "approve_execution_runner_activation_admission_handoff_readiness_ready_blocked"
                .to_string()
        } else {
            "approve_execution_runner_activation_admission_handoff_readiness_blocked".to_string()
        },
        action: handoff_dry_run.action.clone(),
        approval_id: handoff_dry_run.approval_id.clone(),
        gate_name: "approve_execution_runner_activation_admission_handoff_ready".to_string(),
        handoff_name: handoff_dry_run.handoff_name.clone(),
        source_gate_name: handoff_dry_run.source_gate_name.clone(),
        structural_plan_ready,
        runner_control_ready,
        phase_chain_ready,
        rollback_chain_ready,
        side_effect_enablement_ready,
        activation_admission_required,
        activation_transaction_admission_ready,
        activation_transaction_admission_readiness_ready,
        runner_activation_enabled,
        activation_handoff_prerequisites_ready,
        runner_activation_handoff_ready,
        activation_switch_write_order: handoff_dry_run.activation_switch_write_order.clone(),
        would_handoff_to_runner: false,
        would_activate_runner: false,
        would_return_success: false,
        would_touch_disk: false,
        would_unblock_handoff_enablement: false,
        inherited_handoff_blocked_gates: handoff_dry_run.blocked_gates.clone(),
        inherited_admission_readiness_blocked_gates: admission_readiness_dry_run
            .blocked_gates
            .clone(),
        inherited_runner_enablement_blocked_by: handoff_dry_run
            .inherited_runner_enablement_blocked_by
            .clone(),
        required_gates,
        passed_gates,
        blocked_gates,
    }
}

fn contract_repair_approval_approve_execution_runner_activation_handoff_enablement_dry_run(
    handoff_dry_run: &ContractRepairApprovalApproveExecutionRunnerActivationAdmissionHandoffDryRun,
) -> ContractRepairApprovalApproveExecutionRunnerActivationHandoffEnablementDryRun {
    let structural_plan_ready = handoff_dry_run.structural_plan_ready;
    let runner_control_ready = handoff_dry_run.runner_control_ready;
    let phase_chain_ready = handoff_dry_run.phase_chain_ready;
    let rollback_chain_ready = handoff_dry_run.rollback_chain_ready;
    let side_effect_enablement_ready = handoff_dry_run.side_effect_enablement_ready;
    let activation_admission_required = handoff_dry_run.activation_admission_required;
    let activation_transaction_admission_ready =
        handoff_dry_run.activation_transaction_admission_ready;
    let activation_handoff_prerequisites_ready =
        handoff_dry_run.activation_handoff_prerequisites_ready;
    let runner_activation_enabled = handoff_dry_run.runner_activation_enabled;
    let activation_handoff_enablement_ready =
        activation_handoff_prerequisites_ready && runner_activation_enabled;
    let activation_switch_write_order = handoff_dry_run.activation_switch_write_order.clone();
    let required_gates = handoff_dry_run.required_gates.clone();
    let passed_gates = handoff_dry_run.passed_gates.clone();
    let blocked_gates = handoff_dry_run.blocked_gates.clone();

    ContractRepairApprovalApproveExecutionRunnerActivationHandoffEnablementDryRun {
        status: if activation_handoff_enablement_ready {
            "approve_execution_runner_activation_handoff_enablement_ready".to_string()
        } else if handoff_dry_run.action == "approve"
            && structural_plan_ready
            && activation_admission_required
        {
            "approve_execution_runner_activation_handoff_enablement_ready_blocked".to_string()
        } else {
            "approve_execution_runner_activation_handoff_enablement_blocked".to_string()
        },
        action: handoff_dry_run.action.clone(),
        approval_id: handoff_dry_run.approval_id.clone(),
        switch_name: "approve_execution_runner_activation_enabled".to_string(),
        handoff_name: handoff_dry_run.handoff_name.clone(),
        structural_plan_ready,
        runner_control_ready,
        phase_chain_ready,
        rollback_chain_ready,
        side_effect_enablement_ready,
        activation_admission_required,
        activation_transaction_admission_ready,
        activation_handoff_prerequisites_ready,
        runner_activation_enabled,
        activation_handoff_enablement_ready,
        activation_switch_write_order,
        would_enable_runner_activation: false,
        would_handoff_to_runner: false,
        would_activate_runner: false,
        would_return_success: false,
        would_touch_disk: false,
        inherited_handoff_blocked_gates: handoff_dry_run.blocked_gates.clone(),
        required_gates,
        passed_gates,
        blocked_gates,
    }
}

fn contract_repair_approval_approve_execution_runner_activation_handoff_enablement_readiness_dry_run(
    handoff_enablement_dry_run: &ContractRepairApprovalApproveExecutionRunnerActivationHandoffEnablementDryRun,
    handoff_readiness_dry_run: &ContractRepairApprovalApproveExecutionRunnerActivationAdmissionHandoffReadinessDryRun,
) -> ContractRepairApprovalApproveExecutionRunnerActivationHandoffEnablementReadinessDryRun {
    let structural_plan_ready = handoff_enablement_dry_run.structural_plan_ready;
    let runner_control_ready = handoff_enablement_dry_run.runner_control_ready;
    let phase_chain_ready = handoff_enablement_dry_run.phase_chain_ready;
    let rollback_chain_ready = handoff_enablement_dry_run.rollback_chain_ready;
    let side_effect_enablement_ready = handoff_enablement_dry_run.side_effect_enablement_ready;
    let activation_admission_required = handoff_enablement_dry_run.activation_admission_required;
    let activation_transaction_admission_ready =
        handoff_enablement_dry_run.activation_transaction_admission_ready;
    let activation_admission_handoff_readiness_ready =
        handoff_readiness_dry_run.runner_activation_handoff_ready;
    let activation_handoff_prerequisites_ready =
        handoff_enablement_dry_run.activation_handoff_prerequisites_ready;
    let runner_activation_enabled = handoff_enablement_dry_run.runner_activation_enabled;
    let activation_handoff_enablement_ready =
        handoff_enablement_dry_run.activation_handoff_enablement_ready;
    let activation_switch_write_order = handoff_enablement_dry_run
        .activation_switch_write_order
        .clone();
    let required_gates = handoff_enablement_dry_run.required_gates.clone();
    let passed_gates = handoff_enablement_dry_run.passed_gates.clone();
    let blocked_gates = handoff_enablement_dry_run.blocked_gates.clone();

    ContractRepairApprovalApproveExecutionRunnerActivationHandoffEnablementReadinessDryRun {
        status: if activation_handoff_enablement_ready {
            "approve_execution_runner_activation_handoff_enablement_readiness_ready".to_string()
        } else if handoff_enablement_dry_run.action == "approve"
            && structural_plan_ready
            && activation_admission_required
        {
            "approve_execution_runner_activation_handoff_enablement_readiness_ready_blocked"
                .to_string()
        } else {
            "approve_execution_runner_activation_handoff_enablement_readiness_blocked".to_string()
        },
        action: handoff_enablement_dry_run.action.clone(),
        approval_id: handoff_enablement_dry_run.approval_id.clone(),
        gate_name: "approve_execution_runner_activation_handoff_enablement_ready".to_string(),
        switch_name: handoff_enablement_dry_run.switch_name.clone(),
        handoff_name: handoff_enablement_dry_run.handoff_name.clone(),
        structural_plan_ready,
        runner_control_ready,
        phase_chain_ready,
        rollback_chain_ready,
        side_effect_enablement_ready,
        activation_admission_required,
        activation_transaction_admission_ready,
        activation_admission_handoff_readiness_ready,
        activation_handoff_prerequisites_ready,
        runner_activation_enabled,
        activation_handoff_enablement_ready,
        activation_switch_write_order,
        would_unblock_handoff_attempt: false,
        would_enable_runner_activation: false,
        would_handoff_to_runner: false,
        would_activate_runner: false,
        would_return_success: false,
        would_touch_disk: false,
        inherited_enablement_blocked_gates: handoff_enablement_dry_run.blocked_gates.clone(),
        inherited_handoff_readiness_blocked_gates: handoff_readiness_dry_run.blocked_gates.clone(),
        required_gates,
        passed_gates,
        blocked_gates,
    }
}

fn contract_repair_approval_approve_execution_runner_activation_handoff_attempt_dry_run(
    handoff_enablement_dry_run: &ContractRepairApprovalApproveExecutionRunnerActivationHandoffEnablementDryRun,
) -> ContractRepairApprovalApproveExecutionRunnerActivationHandoffAttemptDryRun {
    let activation_handoff_enablement_ready =
        handoff_enablement_dry_run.activation_handoff_enablement_ready;
    let handoff_attempt_ready = activation_handoff_enablement_ready;
    let required_gates =
        vec!["approve_execution_runner_activation_handoff_enablement_ready".to_string()];
    let mut passed_gates = Vec::new();
    let mut blocked_gates = Vec::new();
    if activation_handoff_enablement_ready {
        passed_gates
            .push("approve_execution_runner_activation_handoff_enablement_ready".to_string());
    } else {
        blocked_gates
            .push("approve_execution_runner_activation_handoff_enablement_ready".to_string());
    }

    ContractRepairApprovalApproveExecutionRunnerActivationHandoffAttemptDryRun {
        status: if handoff_attempt_ready {
            "approve_execution_runner_activation_handoff_attempt_ready".to_string()
        } else if handoff_enablement_dry_run.action == "approve"
            && handoff_enablement_dry_run.structural_plan_ready
            && handoff_enablement_dry_run.activation_admission_required
        {
            "approve_execution_runner_activation_handoff_attempt_ready_blocked".to_string()
        } else {
            "approve_execution_runner_activation_handoff_attempt_blocked".to_string()
        },
        action: handoff_enablement_dry_run.action.clone(),
        approval_id: handoff_enablement_dry_run.approval_id.clone(),
        attempt_name: "approve_execution_runner_activation_handoff_attempt".to_string(),
        source_handoff_name: handoff_enablement_dry_run.handoff_name.clone(),
        source_switch_name: handoff_enablement_dry_run.switch_name.clone(),
        structural_plan_ready: handoff_enablement_dry_run.structural_plan_ready,
        runner_control_ready: handoff_enablement_dry_run.runner_control_ready,
        phase_chain_ready: handoff_enablement_dry_run.phase_chain_ready,
        rollback_chain_ready: handoff_enablement_dry_run.rollback_chain_ready,
        side_effect_enablement_ready: handoff_enablement_dry_run.side_effect_enablement_ready,
        activation_admission_required: handoff_enablement_dry_run.activation_admission_required,
        activation_transaction_admission_ready: handoff_enablement_dry_run
            .activation_transaction_admission_ready,
        activation_handoff_prerequisites_ready: handoff_enablement_dry_run
            .activation_handoff_prerequisites_ready,
        runner_activation_enabled: handoff_enablement_dry_run.runner_activation_enabled,
        activation_handoff_enablement_ready,
        handoff_attempt_ready,
        activation_switch_write_order: handoff_enablement_dry_run
            .activation_switch_write_order
            .clone(),
        would_start_handoff: false,
        would_handoff_to_runner: false,
        would_activate_runner: false,
        would_return_success: false,
        would_touch_disk: false,
        inherited_handoff_enablement_blocked_gates: handoff_enablement_dry_run
            .blocked_gates
            .clone(),
        required_gates,
        passed_gates,
        blocked_gates,
    }
}

fn contract_repair_approval_approve_execution_runner_activation_handoff_attempt_readiness_dry_run(
    handoff_attempt_dry_run: &ContractRepairApprovalApproveExecutionRunnerActivationHandoffAttemptDryRun,
    handoff_enablement_readiness_dry_run: &ContractRepairApprovalApproveExecutionRunnerActivationHandoffEnablementReadinessDryRun,
) -> ContractRepairApprovalApproveExecutionRunnerActivationHandoffAttemptReadinessDryRun {
    let structural_plan_ready = handoff_attempt_dry_run.structural_plan_ready;
    let runner_control_ready = handoff_attempt_dry_run.runner_control_ready;
    let phase_chain_ready = handoff_attempt_dry_run.phase_chain_ready;
    let rollback_chain_ready = handoff_attempt_dry_run.rollback_chain_ready;
    let side_effect_enablement_ready = handoff_attempt_dry_run.side_effect_enablement_ready;
    let activation_admission_required = handoff_attempt_dry_run.activation_admission_required;
    let activation_transaction_admission_ready =
        handoff_attempt_dry_run.activation_transaction_admission_ready;
    let activation_handoff_prerequisites_ready =
        handoff_attempt_dry_run.activation_handoff_prerequisites_ready;
    let runner_activation_enabled = handoff_attempt_dry_run.runner_activation_enabled;
    let activation_handoff_enablement_ready =
        handoff_attempt_dry_run.activation_handoff_enablement_ready;
    let activation_handoff_enablement_readiness_ready =
        handoff_enablement_readiness_dry_run.activation_handoff_enablement_ready;
    let handoff_attempt_ready = handoff_attempt_dry_run.handoff_attempt_ready;
    let activation_switch_write_order = handoff_attempt_dry_run
        .activation_switch_write_order
        .clone();
    let required_gates = handoff_attempt_dry_run.required_gates.clone();
    let passed_gates = handoff_attempt_dry_run.passed_gates.clone();
    let blocked_gates = handoff_attempt_dry_run.blocked_gates.clone();

    ContractRepairApprovalApproveExecutionRunnerActivationHandoffAttemptReadinessDryRun {
        status: if handoff_attempt_ready {
            "approve_execution_runner_activation_handoff_attempt_readiness_ready".to_string()
        } else if handoff_attempt_dry_run.action == "approve"
            && structural_plan_ready
            && activation_admission_required
        {
            "approve_execution_runner_activation_handoff_attempt_readiness_ready_blocked"
                .to_string()
        } else {
            "approve_execution_runner_activation_handoff_attempt_readiness_blocked".to_string()
        },
        action: handoff_attempt_dry_run.action.clone(),
        approval_id: handoff_attempt_dry_run.approval_id.clone(),
        gate_name: "approve_execution_runner_activation_handoff_attempt_ready".to_string(),
        attempt_name: handoff_attempt_dry_run.attempt_name.clone(),
        source_handoff_name: handoff_attempt_dry_run.source_handoff_name.clone(),
        source_switch_name: handoff_attempt_dry_run.source_switch_name.clone(),
        structural_plan_ready,
        runner_control_ready,
        phase_chain_ready,
        rollback_chain_ready,
        side_effect_enablement_ready,
        activation_admission_required,
        activation_transaction_admission_ready,
        activation_handoff_prerequisites_ready,
        runner_activation_enabled,
        activation_handoff_enablement_ready,
        activation_handoff_enablement_readiness_ready,
        handoff_attempt_ready,
        activation_switch_write_order,
        would_unblock_post_handoff_attempt: false,
        would_start_handoff: false,
        would_handoff_to_runner: false,
        would_activate_runner: false,
        would_return_success: false,
        would_touch_disk: false,
        inherited_attempt_blocked_gates: handoff_attempt_dry_run.blocked_gates.clone(),
        inherited_enablement_readiness_blocked_gates: handoff_enablement_readiness_dry_run
            .blocked_gates
            .clone(),
        inherited_handoff_enablement_blocked_gates: handoff_attempt_dry_run
            .inherited_handoff_enablement_blocked_gates
            .clone(),
        required_gates,
        passed_gates,
        blocked_gates,
    }
}

fn contract_repair_approval_approve_execution_runner_activation_post_handoff_attempt_dry_run(
    handoff_attempt_dry_run: &ContractRepairApprovalApproveExecutionRunnerActivationHandoffAttemptDryRun,
) -> ContractRepairApprovalApproveExecutionRunnerActivationPostHandoffAttemptDryRun {
    let handoff_attempt_ready = handoff_attempt_dry_run.handoff_attempt_ready;
    let activation_attempt_ready = handoff_attempt_ready;
    let required_gates =
        vec!["approve_execution_runner_activation_handoff_attempt_ready".to_string()];
    let mut passed_gates = Vec::new();
    let mut blocked_gates = Vec::new();
    if handoff_attempt_ready {
        passed_gates.push("approve_execution_runner_activation_handoff_attempt_ready".to_string());
    } else {
        blocked_gates.push("approve_execution_runner_activation_handoff_attempt_ready".to_string());
    }

    ContractRepairApprovalApproveExecutionRunnerActivationPostHandoffAttemptDryRun {
        status: if activation_attempt_ready {
            "approve_execution_runner_activation_post_handoff_attempt_ready".to_string()
        } else if handoff_attempt_dry_run.action == "approve"
            && handoff_attempt_dry_run.structural_plan_ready
            && handoff_attempt_dry_run.activation_admission_required
        {
            "approve_execution_runner_activation_post_handoff_attempt_ready_blocked".to_string()
        } else {
            "approve_execution_runner_activation_post_handoff_attempt_blocked".to_string()
        },
        action: handoff_attempt_dry_run.action.clone(),
        approval_id: handoff_attempt_dry_run.approval_id.clone(),
        attempt_name: "approve_execution_runner_activation_post_handoff_attempt".to_string(),
        source_attempt_name: handoff_attempt_dry_run.attempt_name.clone(),
        source_handoff_name: handoff_attempt_dry_run.source_handoff_name.clone(),
        source_switch_name: handoff_attempt_dry_run.source_switch_name.clone(),
        structural_plan_ready: handoff_attempt_dry_run.structural_plan_ready,
        runner_control_ready: handoff_attempt_dry_run.runner_control_ready,
        phase_chain_ready: handoff_attempt_dry_run.phase_chain_ready,
        rollback_chain_ready: handoff_attempt_dry_run.rollback_chain_ready,
        side_effect_enablement_ready: handoff_attempt_dry_run.side_effect_enablement_ready,
        activation_admission_required: handoff_attempt_dry_run.activation_admission_required,
        activation_transaction_admission_ready: handoff_attempt_dry_run
            .activation_transaction_admission_ready,
        activation_handoff_prerequisites_ready: handoff_attempt_dry_run
            .activation_handoff_prerequisites_ready,
        runner_activation_enabled: handoff_attempt_dry_run.runner_activation_enabled,
        activation_handoff_enablement_ready: handoff_attempt_dry_run
            .activation_handoff_enablement_ready,
        handoff_attempt_ready,
        activation_attempt_ready,
        activation_switch_write_order: handoff_attempt_dry_run
            .activation_switch_write_order
            .clone(),
        would_attempt_activation: false,
        would_activate_runner: false,
        would_return_success: false,
        would_touch_disk: false,
        inherited_handoff_attempt_blocked_gates: handoff_attempt_dry_run.blocked_gates.clone(),
        inherited_handoff_enablement_blocked_gates: handoff_attempt_dry_run
            .inherited_handoff_enablement_blocked_gates
            .clone(),
        required_gates,
        passed_gates,
        blocked_gates,
    }
}

fn contract_repair_approval_approve_execution_runner_activation_post_handoff_attempt_readiness_dry_run(
    post_handoff_attempt_dry_run: &ContractRepairApprovalApproveExecutionRunnerActivationPostHandoffAttemptDryRun,
    handoff_attempt_readiness_dry_run: &ContractRepairApprovalApproveExecutionRunnerActivationHandoffAttemptReadinessDryRun,
) -> ContractRepairApprovalApproveExecutionRunnerActivationPostHandoffAttemptReadinessDryRun {
    let structural_plan_ready = post_handoff_attempt_dry_run.structural_plan_ready;
    let runner_control_ready = post_handoff_attempt_dry_run.runner_control_ready;
    let phase_chain_ready = post_handoff_attempt_dry_run.phase_chain_ready;
    let rollback_chain_ready = post_handoff_attempt_dry_run.rollback_chain_ready;
    let side_effect_enablement_ready = post_handoff_attempt_dry_run.side_effect_enablement_ready;
    let activation_admission_required = post_handoff_attempt_dry_run.activation_admission_required;
    let activation_transaction_admission_ready =
        post_handoff_attempt_dry_run.activation_transaction_admission_ready;
    let activation_handoff_prerequisites_ready =
        post_handoff_attempt_dry_run.activation_handoff_prerequisites_ready;
    let runner_activation_enabled = post_handoff_attempt_dry_run.runner_activation_enabled;
    let activation_handoff_enablement_ready =
        post_handoff_attempt_dry_run.activation_handoff_enablement_ready;
    let handoff_attempt_ready = post_handoff_attempt_dry_run.handoff_attempt_ready;
    let handoff_attempt_readiness_ready = handoff_attempt_readiness_dry_run.handoff_attempt_ready;
    let activation_attempt_ready = post_handoff_attempt_dry_run.activation_attempt_ready;
    let activation_post_handoff_attempt_ready = activation_attempt_ready;
    let activation_switch_write_order = post_handoff_attempt_dry_run
        .activation_switch_write_order
        .clone();
    let required_gates = post_handoff_attempt_dry_run.required_gates.clone();
    let passed_gates = post_handoff_attempt_dry_run.passed_gates.clone();
    let blocked_gates = post_handoff_attempt_dry_run.blocked_gates.clone();

    ContractRepairApprovalApproveExecutionRunnerActivationPostHandoffAttemptReadinessDryRun {
        status: if activation_post_handoff_attempt_ready {
            "approve_execution_runner_activation_post_handoff_attempt_readiness_ready".to_string()
        } else if post_handoff_attempt_dry_run.action == "approve"
            && structural_plan_ready
            && activation_admission_required
        {
            "approve_execution_runner_activation_post_handoff_attempt_readiness_ready_blocked"
                .to_string()
        } else {
            "approve_execution_runner_activation_post_handoff_attempt_readiness_blocked".to_string()
        },
        action: post_handoff_attempt_dry_run.action.clone(),
        approval_id: post_handoff_attempt_dry_run.approval_id.clone(),
        gate_name: "approve_execution_runner_activation_post_handoff_attempt_ready".to_string(),
        attempt_name: post_handoff_attempt_dry_run.attempt_name.clone(),
        source_attempt_name: post_handoff_attempt_dry_run.source_attempt_name.clone(),
        source_handoff_name: post_handoff_attempt_dry_run.source_handoff_name.clone(),
        source_switch_name: post_handoff_attempt_dry_run.source_switch_name.clone(),
        structural_plan_ready,
        runner_control_ready,
        phase_chain_ready,
        rollback_chain_ready,
        side_effect_enablement_ready,
        activation_admission_required,
        activation_transaction_admission_ready,
        activation_handoff_prerequisites_ready,
        runner_activation_enabled,
        activation_handoff_enablement_ready,
        handoff_attempt_ready,
        handoff_attempt_readiness_ready,
        activation_attempt_ready,
        activation_post_handoff_attempt_ready,
        activation_switch_write_order,
        would_unblock_success_admission: false,
        would_attempt_activation: false,
        would_activate_runner: false,
        would_return_success: false,
        would_touch_disk: false,
        inherited_post_handoff_attempt_blocked_gates: post_handoff_attempt_dry_run
            .blocked_gates
            .clone(),
        inherited_handoff_attempt_readiness_blocked_gates: handoff_attempt_readiness_dry_run
            .blocked_gates
            .clone(),
        inherited_handoff_attempt_blocked_gates: post_handoff_attempt_dry_run
            .inherited_handoff_attempt_blocked_gates
            .clone(),
        inherited_handoff_enablement_blocked_gates: post_handoff_attempt_dry_run
            .inherited_handoff_enablement_blocked_gates
            .clone(),
        required_gates,
        passed_gates,
        blocked_gates,
    }
}

fn contract_repair_approval_approve_execution_runner_activation_success_admission_dry_run(
    post_handoff_attempt_dry_run: &ContractRepairApprovalApproveExecutionRunnerActivationPostHandoffAttemptDryRun,
) -> ContractRepairApprovalApproveExecutionRunnerActivationSuccessAdmissionDryRun {
    let activation_post_handoff_attempt_ready =
        post_handoff_attempt_dry_run.activation_attempt_ready;
    let activation_success_admission_ready = activation_post_handoff_attempt_ready;
    let required_gates =
        vec!["approve_execution_runner_activation_post_handoff_attempt_ready".to_string()];
    let mut passed_gates = Vec::new();
    let mut blocked_gates = Vec::new();
    if activation_post_handoff_attempt_ready {
        passed_gates
            .push("approve_execution_runner_activation_post_handoff_attempt_ready".to_string());
    } else {
        blocked_gates
            .push("approve_execution_runner_activation_post_handoff_attempt_ready".to_string());
    }

    ContractRepairApprovalApproveExecutionRunnerActivationSuccessAdmissionDryRun {
        status: if activation_success_admission_ready {
            "approve_execution_runner_activation_success_admission_ready".to_string()
        } else if post_handoff_attempt_dry_run.action == "approve"
            && post_handoff_attempt_dry_run.structural_plan_ready
            && post_handoff_attempt_dry_run.activation_admission_required
        {
            "approve_execution_runner_activation_success_admission_ready_blocked".to_string()
        } else {
            "approve_execution_runner_activation_success_admission_blocked".to_string()
        },
        action: post_handoff_attempt_dry_run.action.clone(),
        approval_id: post_handoff_attempt_dry_run.approval_id.clone(),
        admission_name: "approve_execution_runner_activation_success_admission".to_string(),
        source_attempt_name: post_handoff_attempt_dry_run.attempt_name.clone(),
        source_handoff_attempt_name: post_handoff_attempt_dry_run.source_attempt_name.clone(),
        source_handoff_name: post_handoff_attempt_dry_run.source_handoff_name.clone(),
        source_switch_name: post_handoff_attempt_dry_run.source_switch_name.clone(),
        structural_plan_ready: post_handoff_attempt_dry_run.structural_plan_ready,
        runner_control_ready: post_handoff_attempt_dry_run.runner_control_ready,
        phase_chain_ready: post_handoff_attempt_dry_run.phase_chain_ready,
        rollback_chain_ready: post_handoff_attempt_dry_run.rollback_chain_ready,
        side_effect_enablement_ready: post_handoff_attempt_dry_run.side_effect_enablement_ready,
        activation_admission_required: post_handoff_attempt_dry_run.activation_admission_required,
        activation_transaction_admission_ready: post_handoff_attempt_dry_run
            .activation_transaction_admission_ready,
        activation_handoff_prerequisites_ready: post_handoff_attempt_dry_run
            .activation_handoff_prerequisites_ready,
        runner_activation_enabled: post_handoff_attempt_dry_run.runner_activation_enabled,
        activation_handoff_enablement_ready: post_handoff_attempt_dry_run
            .activation_handoff_enablement_ready,
        handoff_attempt_ready: post_handoff_attempt_dry_run.handoff_attempt_ready,
        activation_attempt_ready: post_handoff_attempt_dry_run.activation_attempt_ready,
        activation_post_handoff_attempt_ready,
        activation_success_admission_ready,
        activation_switch_write_order: post_handoff_attempt_dry_run
            .activation_switch_write_order
            .clone(),
        would_admit_success: false,
        would_activate_runner: false,
        would_return_success: false,
        would_touch_disk: false,
        inherited_post_handoff_attempt_blocked_gates: post_handoff_attempt_dry_run
            .blocked_gates
            .clone(),
        inherited_handoff_attempt_blocked_gates: post_handoff_attempt_dry_run
            .inherited_handoff_attempt_blocked_gates
            .clone(),
        inherited_handoff_enablement_blocked_gates: post_handoff_attempt_dry_run
            .inherited_handoff_enablement_blocked_gates
            .clone(),
        required_gates,
        passed_gates,
        blocked_gates,
    }
}

fn contract_repair_approval_approve_execution_runner_activation_success_admission_readiness_dry_run(
    success_admission_dry_run: &ContractRepairApprovalApproveExecutionRunnerActivationSuccessAdmissionDryRun,
    post_handoff_attempt_readiness_dry_run: &ContractRepairApprovalApproveExecutionRunnerActivationPostHandoffAttemptReadinessDryRun,
) -> ContractRepairApprovalApproveExecutionRunnerActivationSuccessAdmissionReadinessDryRun {
    let structural_plan_ready = success_admission_dry_run.structural_plan_ready;
    let runner_control_ready = success_admission_dry_run.runner_control_ready;
    let phase_chain_ready = success_admission_dry_run.phase_chain_ready;
    let rollback_chain_ready = success_admission_dry_run.rollback_chain_ready;
    let side_effect_enablement_ready = success_admission_dry_run.side_effect_enablement_ready;
    let activation_admission_required = success_admission_dry_run.activation_admission_required;
    let activation_transaction_admission_ready =
        success_admission_dry_run.activation_transaction_admission_ready;
    let activation_handoff_prerequisites_ready =
        success_admission_dry_run.activation_handoff_prerequisites_ready;
    let runner_activation_enabled = success_admission_dry_run.runner_activation_enabled;
    let activation_handoff_enablement_ready =
        success_admission_dry_run.activation_handoff_enablement_ready;
    let handoff_attempt_ready = success_admission_dry_run.handoff_attempt_ready;
    let activation_attempt_ready = success_admission_dry_run.activation_attempt_ready;
    let activation_post_handoff_attempt_ready =
        success_admission_dry_run.activation_post_handoff_attempt_ready;
    let activation_post_handoff_attempt_readiness_ready =
        post_handoff_attempt_readiness_dry_run.activation_post_handoff_attempt_ready;
    let activation_success_admission_ready =
        success_admission_dry_run.activation_success_admission_ready;
    let activation_switch_write_order = success_admission_dry_run
        .activation_switch_write_order
        .clone();
    let required_gates = success_admission_dry_run.required_gates.clone();
    let passed_gates = success_admission_dry_run.passed_gates.clone();
    let blocked_gates = success_admission_dry_run.blocked_gates.clone();

    ContractRepairApprovalApproveExecutionRunnerActivationSuccessAdmissionReadinessDryRun {
        status: if activation_success_admission_ready {
            "approve_execution_runner_activation_success_admission_readiness_ready".to_string()
        } else if success_admission_dry_run.action == "approve"
            && structural_plan_ready
            && activation_admission_required
        {
            "approve_execution_runner_activation_success_admission_readiness_ready_blocked"
                .to_string()
        } else {
            "approve_execution_runner_activation_success_admission_readiness_blocked".to_string()
        },
        action: success_admission_dry_run.action.clone(),
        approval_id: success_admission_dry_run.approval_id.clone(),
        gate_name: "approve_execution_runner_activation_success_admission_ready".to_string(),
        admission_name: success_admission_dry_run.admission_name.clone(),
        source_attempt_name: success_admission_dry_run.source_attempt_name.clone(),
        source_handoff_attempt_name: success_admission_dry_run
            .source_handoff_attempt_name
            .clone(),
        source_handoff_name: success_admission_dry_run.source_handoff_name.clone(),
        source_switch_name: success_admission_dry_run.source_switch_name.clone(),
        structural_plan_ready,
        runner_control_ready,
        phase_chain_ready,
        rollback_chain_ready,
        side_effect_enablement_ready,
        activation_admission_required,
        activation_transaction_admission_ready,
        activation_handoff_prerequisites_ready,
        runner_activation_enabled,
        activation_handoff_enablement_ready,
        handoff_attempt_ready,
        activation_attempt_ready,
        activation_post_handoff_attempt_ready,
        activation_post_handoff_attempt_readiness_ready,
        activation_success_admission_ready,
        activation_switch_write_order,
        would_unblock_success_return: false,
        would_admit_success: false,
        would_activate_runner: false,
        would_return_success: false,
        would_touch_disk: false,
        inherited_success_admission_blocked_gates: success_admission_dry_run.blocked_gates.clone(),
        inherited_post_handoff_attempt_readiness_blocked_gates:
            post_handoff_attempt_readiness_dry_run.blocked_gates.clone(),
        inherited_post_handoff_attempt_blocked_gates: success_admission_dry_run
            .inherited_post_handoff_attempt_blocked_gates
            .clone(),
        inherited_handoff_attempt_blocked_gates: success_admission_dry_run
            .inherited_handoff_attempt_blocked_gates
            .clone(),
        inherited_handoff_enablement_blocked_gates: success_admission_dry_run
            .inherited_handoff_enablement_blocked_gates
            .clone(),
        required_gates,
        passed_gates,
        blocked_gates,
    }
}

fn contract_repair_approval_approve_execution_runner_activation_success_return_dry_run(
    success_admission_dry_run: &ContractRepairApprovalApproveExecutionRunnerActivationSuccessAdmissionDryRun,
) -> ContractRepairApprovalApproveExecutionRunnerActivationSuccessReturnDryRun {
    let activation_success_admission_ready =
        success_admission_dry_run.activation_success_admission_ready;
    let activation_success_return_ready = activation_success_admission_ready;
    let required_gates =
        vec!["approve_execution_runner_activation_success_admission_ready".to_string()];
    let mut passed_gates = Vec::new();
    let mut blocked_gates = Vec::new();
    if activation_success_admission_ready {
        passed_gates
            .push("approve_execution_runner_activation_success_admission_ready".to_string());
    } else {
        blocked_gates
            .push("approve_execution_runner_activation_success_admission_ready".to_string());
    }

    ContractRepairApprovalApproveExecutionRunnerActivationSuccessReturnDryRun {
        status: if activation_success_return_ready {
            "approve_execution_runner_activation_success_return_ready".to_string()
        } else if success_admission_dry_run.action == "approve"
            && success_admission_dry_run.structural_plan_ready
            && success_admission_dry_run.activation_admission_required
        {
            "approve_execution_runner_activation_success_return_ready_blocked".to_string()
        } else {
            "approve_execution_runner_activation_success_return_blocked".to_string()
        },
        action: success_admission_dry_run.action.clone(),
        approval_id: success_admission_dry_run.approval_id.clone(),
        return_name: "approve_execution_runner_activation_success_return".to_string(),
        source_admission_name: success_admission_dry_run.admission_name.clone(),
        source_attempt_name: success_admission_dry_run.source_attempt_name.clone(),
        source_handoff_attempt_name: success_admission_dry_run
            .source_handoff_attempt_name
            .clone(),
        source_handoff_name: success_admission_dry_run.source_handoff_name.clone(),
        source_switch_name: success_admission_dry_run.source_switch_name.clone(),
        structural_plan_ready: success_admission_dry_run.structural_plan_ready,
        runner_control_ready: success_admission_dry_run.runner_control_ready,
        phase_chain_ready: success_admission_dry_run.phase_chain_ready,
        rollback_chain_ready: success_admission_dry_run.rollback_chain_ready,
        side_effect_enablement_ready: success_admission_dry_run.side_effect_enablement_ready,
        activation_admission_required: success_admission_dry_run.activation_admission_required,
        activation_transaction_admission_ready: success_admission_dry_run
            .activation_transaction_admission_ready,
        activation_handoff_prerequisites_ready: success_admission_dry_run
            .activation_handoff_prerequisites_ready,
        runner_activation_enabled: success_admission_dry_run.runner_activation_enabled,
        activation_handoff_enablement_ready: success_admission_dry_run
            .activation_handoff_enablement_ready,
        handoff_attempt_ready: success_admission_dry_run.handoff_attempt_ready,
        activation_attempt_ready: success_admission_dry_run.activation_attempt_ready,
        activation_post_handoff_attempt_ready: success_admission_dry_run
            .activation_post_handoff_attempt_ready,
        activation_success_admission_ready,
        activation_success_return_ready,
        activation_switch_write_order: success_admission_dry_run
            .activation_switch_write_order
            .clone(),
        would_admit_success: false,
        would_activate_runner: false,
        would_return_success: false,
        would_touch_disk: false,
        inherited_success_admission_blocked_gates: success_admission_dry_run.blocked_gates.clone(),
        inherited_post_handoff_attempt_blocked_gates: success_admission_dry_run
            .inherited_post_handoff_attempt_blocked_gates
            .clone(),
        inherited_handoff_attempt_blocked_gates: success_admission_dry_run
            .inherited_handoff_attempt_blocked_gates
            .clone(),
        inherited_handoff_enablement_blocked_gates: success_admission_dry_run
            .inherited_handoff_enablement_blocked_gates
            .clone(),
        required_gates,
        passed_gates,
        blocked_gates,
    }
}

fn contract_repair_approval_approve_execution_runner_activation_success_return_readiness_dry_run(
    success_return_dry_run: &ContractRepairApprovalApproveExecutionRunnerActivationSuccessReturnDryRun,
    success_admission_readiness_dry_run: &ContractRepairApprovalApproveExecutionRunnerActivationSuccessAdmissionReadinessDryRun,
) -> ContractRepairApprovalApproveExecutionRunnerActivationSuccessReturnReadinessDryRun {
    let structural_plan_ready = success_return_dry_run.structural_plan_ready;
    let runner_control_ready = success_return_dry_run.runner_control_ready;
    let phase_chain_ready = success_return_dry_run.phase_chain_ready;
    let rollback_chain_ready = success_return_dry_run.rollback_chain_ready;
    let side_effect_enablement_ready = success_return_dry_run.side_effect_enablement_ready;
    let activation_admission_required = success_return_dry_run.activation_admission_required;
    let activation_transaction_admission_ready =
        success_return_dry_run.activation_transaction_admission_ready;
    let activation_handoff_prerequisites_ready =
        success_return_dry_run.activation_handoff_prerequisites_ready;
    let runner_activation_enabled = success_return_dry_run.runner_activation_enabled;
    let activation_handoff_enablement_ready =
        success_return_dry_run.activation_handoff_enablement_ready;
    let handoff_attempt_ready = success_return_dry_run.handoff_attempt_ready;
    let activation_attempt_ready = success_return_dry_run.activation_attempt_ready;
    let activation_post_handoff_attempt_ready =
        success_return_dry_run.activation_post_handoff_attempt_ready;
    let activation_success_admission_ready =
        success_return_dry_run.activation_success_admission_ready;
    let activation_success_admission_readiness_ready =
        success_admission_readiness_dry_run.activation_success_admission_ready;
    let activation_success_return_ready = success_return_dry_run.activation_success_return_ready;
    let activation_switch_write_order =
        success_return_dry_run.activation_switch_write_order.clone();
    let required_gates = success_return_dry_run.required_gates.clone();
    let passed_gates = success_return_dry_run.passed_gates.clone();
    let blocked_gates = success_return_dry_run.blocked_gates.clone();

    ContractRepairApprovalApproveExecutionRunnerActivationSuccessReturnReadinessDryRun {
        status: if activation_success_return_ready {
            "approve_execution_runner_activation_success_return_readiness_ready".to_string()
        } else if success_return_dry_run.action == "approve"
            && structural_plan_ready
            && activation_admission_required
        {
            "approve_execution_runner_activation_success_return_readiness_ready_blocked".to_string()
        } else {
            "approve_execution_runner_activation_success_return_readiness_blocked".to_string()
        },
        action: success_return_dry_run.action.clone(),
        approval_id: success_return_dry_run.approval_id.clone(),
        gate_name: "approve_execution_runner_activation_success_return_ready".to_string(),
        return_name: success_return_dry_run.return_name.clone(),
        source_admission_name: success_return_dry_run.source_admission_name.clone(),
        source_attempt_name: success_return_dry_run.source_attempt_name.clone(),
        source_handoff_attempt_name: success_return_dry_run.source_handoff_attempt_name.clone(),
        source_handoff_name: success_return_dry_run.source_handoff_name.clone(),
        source_switch_name: success_return_dry_run.source_switch_name.clone(),
        structural_plan_ready,
        runner_control_ready,
        phase_chain_ready,
        rollback_chain_ready,
        side_effect_enablement_ready,
        activation_admission_required,
        activation_transaction_admission_ready,
        activation_handoff_prerequisites_ready,
        runner_activation_enabled,
        activation_handoff_enablement_ready,
        handoff_attempt_ready,
        activation_attempt_ready,
        activation_post_handoff_attempt_ready,
        activation_success_admission_ready,
        activation_success_admission_readiness_ready,
        activation_success_return_ready,
        activation_switch_write_order,
        would_unblock_route_success: false,
        would_admit_success: false,
        would_activate_runner: false,
        would_return_success: false,
        would_touch_disk: false,
        inherited_success_return_blocked_gates: success_return_dry_run.blocked_gates.clone(),
        inherited_success_admission_readiness_blocked_gates: success_admission_readiness_dry_run
            .blocked_gates
            .clone(),
        inherited_success_admission_blocked_gates: success_return_dry_run
            .inherited_success_admission_blocked_gates
            .clone(),
        inherited_post_handoff_attempt_blocked_gates: success_return_dry_run
            .inherited_post_handoff_attempt_blocked_gates
            .clone(),
        inherited_handoff_attempt_blocked_gates: success_return_dry_run
            .inherited_handoff_attempt_blocked_gates
            .clone(),
        inherited_handoff_enablement_blocked_gates: success_return_dry_run
            .inherited_handoff_enablement_blocked_gates
            .clone(),
        required_gates,
        passed_gates,
        blocked_gates,
    }
}

fn contract_repair_approval_approve_execution_runner_route_success_readiness_dry_run(
    success_return_readiness_dry_run: &ContractRepairApprovalApproveExecutionRunnerActivationSuccessReturnReadinessDryRun,
    enablement_plan_dry_run: &ContractRepairApprovalApproveExecutionRunnerEnablementPlanDryRun,
) -> ContractRepairApprovalApproveExecutionRunnerRouteSuccessReadinessDryRun {
    let structural_plan_ready = enablement_plan_dry_run.structural_plan_ready;
    let runner_control_ready = enablement_plan_dry_run.runner_control_ready;
    let phase_chain_ready = enablement_plan_dry_run.phase_chain_ready;
    let rollback_chain_ready = enablement_plan_dry_run.rollback_chain_ready;
    let side_effect_enablement_ready = enablement_plan_dry_run.side_effect_enablement_ready;
    let runner_activation_enabled = enablement_plan_dry_run.runner_activation_enabled;
    let runner_activation_ready = enablement_plan_dry_run.runner_activation_ready;
    let activation_success_return_ready =
        success_return_readiness_dry_run.activation_success_return_ready;
    let activation_success_return_readiness_ready =
        success_return_readiness_dry_run.activation_success_return_ready;
    let enablement_plan_success_ready = enablement_plan_dry_run.runner_activation_ready;
    let route_success_ready =
        activation_success_return_readiness_ready && enablement_plan_success_ready;
    let gate_states = [
        (
            "approve_execution_runner_activation_success_return_ready",
            activation_success_return_ready,
        ),
        ("approve_runner_success", enablement_plan_success_ready),
    ];
    let required_gates = gate_states
        .iter()
        .map(|(gate, _)| (*gate).to_string())
        .collect::<Vec<_>>();
    let mut passed_gates = Vec::new();
    let mut blocked_gates = Vec::new();
    for (gate, passed) in gate_states {
        if passed {
            passed_gates.push(gate.to_string());
        } else {
            blocked_gates.push(gate.to_string());
        }
    }

    ContractRepairApprovalApproveExecutionRunnerRouteSuccessReadinessDryRun {
        status: if route_success_ready {
            "approve_execution_runner_route_success_readiness_ready".to_string()
        } else if success_return_readiness_dry_run.action == "approve" && structural_plan_ready {
            "approve_execution_runner_route_success_readiness_ready_blocked".to_string()
        } else {
            "approve_execution_runner_route_success_readiness_blocked".to_string()
        },
        action: success_return_readiness_dry_run.action.clone(),
        approval_id: success_return_readiness_dry_run.approval_id.clone(),
        gate_name: "approve_execution_runner_route_success_ready".to_string(),
        route_status_name: "review_approve_executed".to_string(),
        source_return_name: success_return_readiness_dry_run.return_name.clone(),
        source_enablement_plan_name: "approve_execution_runner_enablement_plan".to_string(),
        structural_plan_ready,
        runner_control_ready,
        phase_chain_ready,
        rollback_chain_ready,
        side_effect_enablement_ready,
        runner_activation_enabled,
        runner_activation_ready,
        activation_success_return_ready,
        activation_success_return_readiness_ready,
        enablement_plan_success_ready,
        route_success_ready,
        would_set_route_success: false,
        would_mark_review_approve_executed: false,
        would_activate_runner: false,
        would_return_success: false,
        would_touch_disk: false,
        inherited_success_return_readiness_blocked_gates: success_return_readiness_dry_run
            .blocked_gates
            .clone(),
        inherited_success_return_blocked_gates: success_return_readiness_dry_run
            .inherited_success_return_blocked_gates
            .clone(),
        inherited_enablement_plan_blocked_gates: enablement_plan_dry_run.blocked_by.clone(),
        inherited_enablement_plan_blocked_enablements: enablement_plan_dry_run
            .blocked_enablements
            .clone(),
        required_gates,
        passed_gates,
        blocked_gates,
    }
}

fn contract_repair_approval_approve_execution_runner_route_success_with_gate(
    route_success_readiness_dry_run: &ContractRepairApprovalApproveExecutionRunnerRouteSuccessReadinessDryRun,
    route_success_enabled: bool,
) -> ContractRepairApprovalApproveExecutionRunnerRouteSuccessReadinessDryRun {
    let mut result = route_success_readiness_dry_run.clone();
    if !route_success_enabled {
        return result;
    }

    let gate_states = [
        (
            "approve_execution_runner_structural_plan_ready",
            route_success_readiness_dry_run.structural_plan_ready,
        ),
        (
            "approve_execution_runner_control_ready",
            route_success_readiness_dry_run.runner_control_ready,
        ),
        (
            "approve_execution_runner_phase_chain_ready",
            route_success_readiness_dry_run.phase_chain_ready,
        ),
        (
            "approve_execution_runner_rollback_chain_ready",
            route_success_readiness_dry_run.rollback_chain_ready,
        ),
        (
            "approve_execution_runner_side_effect_enablement_ready",
            route_success_readiness_dry_run.side_effect_enablement_ready,
        ),
        (
            "approve_execution_runner_activation_enabled",
            route_success_readiness_dry_run.runner_activation_enabled,
        ),
        (
            "approve_execution_runner_activation_ready",
            route_success_readiness_dry_run.runner_activation_ready,
        ),
        (
            "approve_execution_runner_activation_success_return_ready",
            route_success_readiness_dry_run.activation_success_return_ready,
        ),
        (
            "approve_execution_runner_activation_success_return_readiness_ready",
            route_success_readiness_dry_run.activation_success_return_readiness_ready,
        ),
        (
            "approve_runner_success",
            route_success_readiness_dry_run.enablement_plan_success_ready,
        ),
    ];
    for (gate, passed) in gate_states {
        if !passed {
            push_unique_blocker(&mut result.blocked_gates, gate);
        }
    }
    if !result.blocked_gates.is_empty() {
        result.status = "approve_execution_runner_route_success_readiness_blocked".to_string();
        result.route_success_ready = false;
        return result;
    }

    result.status = "approve_execution_runner_route_success_dispatched".to_string();
    result.route_success_ready = true;
    result.would_set_route_success = true;
    result.would_mark_review_approve_executed = true;
    result.would_activate_runner = true;
    result.would_return_success = true;
    result.would_touch_disk = false;
    result
}

fn contract_repair_approval_approve_runner_success_from_route_success(
    route_success_readiness_dry_run: &ContractRepairApprovalApproveExecutionRunnerRouteSuccessReadinessDryRun,
) -> bool {
    route_success_readiness_dry_run.would_return_success
}

fn contract_repair_approval_approve_execution_runner_route_status_readiness_dry_run(
    action: &str,
    approval_id: &str,
    response_status: &str,
    route_status: &str,
    blocked_reasons: &[String],
    route_success_readiness_dry_run: &ContractRepairApprovalApproveExecutionRunnerRouteSuccessReadinessDryRun,
    decision_execution_preflight_requested: bool,
    review_execution_enabled: bool,
    approve_runner_success: bool,
) -> ContractRepairApprovalApproveExecutionRunnerRouteStatusReadinessDryRun {
    let route_success_ready = route_success_readiness_dry_run.route_success_ready;
    let route_status_ready =
        action == "approve" && decision_execution_preflight_requested && route_success_ready;
    let expected_http_status = if review_execution_enabled || approve_runner_success {
        200
    } else {
        423
    };
    let gate_states = [(
        "approve_execution_runner_route_success_ready",
        route_success_ready,
    )];
    let required_gates = gate_states
        .iter()
        .map(|(gate, _)| (*gate).to_string())
        .collect::<Vec<_>>();
    let mut passed_gates = Vec::new();
    let mut blocked_gates = Vec::new();
    for (gate, passed) in gate_states {
        if passed {
            passed_gates.push(gate.to_string());
        } else {
            blocked_gates.push(gate.to_string());
        }
    }

    ContractRepairApprovalApproveExecutionRunnerRouteStatusReadinessDryRun {
        status: if route_status_ready {
            "approve_execution_runner_route_status_readiness_ready".to_string()
        } else if action == "approve" && decision_execution_preflight_requested {
            "approve_execution_runner_route_status_readiness_ready_blocked".to_string()
        } else {
            "approve_execution_runner_route_status_readiness_blocked".to_string()
        },
        action: action.to_string(),
        approval_id: approval_id.to_string(),
        gate_name: "approve_execution_runner_route_status_ready".to_string(),
        current_response_status: response_status.to_string(),
        current_route_status: route_status.to_string(),
        target_response_status: "review_approve_executed".to_string(),
        target_route_status: "review_approve_executed".to_string(),
        expected_http_status,
        decision_execution_preflight_requested,
        review_execution_enabled,
        approve_runner_success,
        route_success_ready,
        route_status_ready,
        would_set_response_status: false,
        would_set_route_status: false,
        would_return_http_ok: false,
        would_touch_disk: false,
        inherited_route_success_blocked_gates: route_success_readiness_dry_run
            .blocked_gates
            .clone(),
        inherited_blocked_reasons: blocked_reasons.to_vec(),
        required_gates,
        passed_gates,
        blocked_gates,
    }
}

fn contract_repair_approval_approve_execution_formal_review_execution_readiness_dry_run(
    action: &str,
    approval_id: &str,
    review_request_enabled: bool,
    execution_gate: &ContractRepairApprovalReviewExecutionGate,
    approve_execution_gate: &ContractRepairApprovalApproveExecutionGate,
    route_success_readiness_dry_run: &ContractRepairApprovalApproveExecutionRunnerRouteSuccessReadinessDryRun,
    route_status_readiness_dry_run: &ContractRepairApprovalApproveExecutionRunnerRouteStatusReadinessDryRun,
    blocked_reasons: &[String],
    decision_execution_preflight_requested: bool,
    formal_approve_review_execution_enabled: bool,
    review_execution_enabled: bool,
) -> ContractRepairApprovalApproveExecutionFormalReviewExecutionReadinessDryRun {
    let approve_action = action == "approve";
    let review_execution_gate_clear = execution_gate.blocked_gates.is_empty();
    let approve_execution_ready = approve_execution_gate.approve_execution_ready;
    let route_success_ready = route_success_readiness_dry_run.route_success_ready;
    let upstream_ready = approve_action
        && decision_execution_preflight_requested
        && review_request_enabled
        && review_execution_gate_clear
        && approve_execution_ready
        && route_success_ready;
    let formal_review_execution_ready = upstream_ready && formal_approve_review_execution_enabled;
    let gate_states = [
        ("approve_action", approve_action),
        (
            "decision_execution_preflight_requested",
            decision_execution_preflight_requested,
        ),
        ("review_request_enabled", review_request_enabled),
        ("review_execution_gate_clear", review_execution_gate_clear),
        ("approve_execution_ready", approve_execution_ready),
        (
            "approve_execution_runner_route_success_ready",
            route_success_ready,
        ),
        (
            "formal_approve_review_execution_enabled",
            formal_approve_review_execution_enabled,
        ),
    ];
    let required_gates = gate_states
        .iter()
        .map(|(gate, _)| (*gate).to_string())
        .collect::<Vec<_>>();
    let mut passed_gates = Vec::new();
    let mut blocked_gates = Vec::new();
    for (gate, passed) in gate_states {
        if passed {
            passed_gates.push(gate.to_string());
        } else {
            blocked_gates.push(gate.to_string());
        }
    }

    ContractRepairApprovalApproveExecutionFormalReviewExecutionReadinessDryRun {
        status: if formal_review_execution_ready {
            "approve_execution_formal_review_execution_readiness_ready".to_string()
        } else if upstream_ready {
            "approve_execution_formal_review_execution_readiness_ready_blocked".to_string()
        } else {
            "approve_execution_formal_review_execution_readiness_blocked".to_string()
        },
        action: action.to_string(),
        approval_id: approval_id.to_string(),
        gate_name: "formal_approve_review_execution_ready".to_string(),
        decision_execution_preflight_requested,
        review_request_enabled,
        review_execution_gate_clear,
        approve_execution_ready,
        route_success_ready,
        formal_approve_review_execution_enabled,
        review_execution_enabled,
        formal_review_execution_ready,
        would_execute_decision: false,
        would_persist_approval_record: false,
        would_mutate_contract: false,
        would_emit_lifecycle_event: false,
        would_commit_transaction: false,
        would_return_http_ok: false,
        would_touch_disk: false,
        inherited_execution_blocked_gates: execution_gate.blocked_gates.clone(),
        inherited_approve_execution_blockers: approve_execution_gate.blocked_by.clone(),
        inherited_route_success_blocked_gates: route_success_readiness_dry_run
            .blocked_gates
            .clone(),
        inherited_route_status_blocked_gates: route_status_readiness_dry_run.blocked_gates.clone(),
        inherited_blocked_reasons: blocked_reasons.to_vec(),
        required_gates,
        passed_gates,
        blocked_gates,
    }
}

fn contract_repair_approval_approve_execution_final_atomic_readiness_dry_run(
    action: &str,
    approval_id: &str,
    record_write_dry_run: &ContractRepairApprovalRecordWriteDryRun,
    lifecycle_emission_gate: &ContractRepairApprovalLifecycleEmissionEnablementGate,
    contract_mutation_gate: &ContractRepairApprovalContractMutationEnablementGate,
    marker_persistence_gate: &ContractRepairApprovalApproveExecutionRecoveryMarkerPersistenceGate,
    transaction_commit_gate: &ContractRepairApprovalApproveExecutionTransactionCommitGate,
    route_success_readiness_dry_run: &ContractRepairApprovalApproveExecutionRunnerRouteSuccessReadinessDryRun,
    formal_review_execution_readiness_dry_run: &ContractRepairApprovalApproveExecutionFormalReviewExecutionReadinessDryRun,
    decision_execution_preflight_requested: bool,
    review_execution_enabled: bool,
) -> ContractRepairApprovalApproveExecutionFinalAtomicReadinessDryRun {
    let approve_action = action == "approve";
    let record_write_ready = record_write_dry_run.write_ready;
    let lifecycle_effects_ready = lifecycle_emission_gate.lifecycle_effects_ready;
    let contract_mutation_ready = contract_mutation_gate.mutation_ready;
    let recovery_marker_persistence_ready = marker_persistence_gate.persistence_ready;
    let transaction_commit_ready = transaction_commit_gate.commit_ready;
    let route_success_ready = route_success_readiness_dry_run.route_success_ready;
    let formal_review_execution_ready =
        formal_review_execution_readiness_dry_run.formal_review_execution_ready;
    let final_atomic_execution_ready = approve_action
        && decision_execution_preflight_requested
        && record_write_ready
        && lifecycle_effects_ready
        && contract_mutation_ready
        && recovery_marker_persistence_ready
        && transaction_commit_ready
        && route_success_ready
        && formal_review_execution_ready;
    let gate_states = [
        ("approve_action", approve_action),
        (
            "decision_execution_preflight_requested",
            decision_execution_preflight_requested,
        ),
        ("record_write_ready", record_write_ready),
        ("lifecycle_effects_ready", lifecycle_effects_ready),
        ("contract_mutation_ready", contract_mutation_ready),
        (
            "recovery_marker_persistence_ready",
            recovery_marker_persistence_ready,
        ),
        ("transaction_commit_ready", transaction_commit_ready),
        (
            "approve_execution_runner_route_success_ready",
            route_success_ready,
        ),
        (
            "formal_review_execution_ready",
            formal_review_execution_ready,
        ),
    ];
    let required_gates = gate_states
        .iter()
        .map(|(gate, _)| (*gate).to_string())
        .collect::<Vec<_>>();
    let mut passed_gates = Vec::new();
    let mut blocked_gates = Vec::new();
    for (gate, passed) in gate_states {
        if passed {
            passed_gates.push(gate.to_string());
        } else {
            blocked_gates.push(gate.to_string());
        }
    }

    ContractRepairApprovalApproveExecutionFinalAtomicReadinessDryRun {
        status: if final_atomic_execution_ready {
            "approve_execution_final_atomic_readiness_ready".to_string()
        } else if approve_action
            && decision_execution_preflight_requested
            && record_write_ready
            && lifecycle_effects_ready
            && contract_mutation_ready
            && recovery_marker_persistence_ready
            && transaction_commit_ready
            && route_success_ready
        {
            "approve_execution_final_atomic_readiness_ready_blocked".to_string()
        } else {
            "approve_execution_final_atomic_readiness_blocked".to_string()
        },
        action: action.to_string(),
        approval_id: approval_id.to_string(),
        gate_name: "approve_execution_final_atomic_ready".to_string(),
        decision_execution_preflight_requested,
        review_execution_enabled,
        record_write_ready,
        lifecycle_effects_ready,
        contract_mutation_ready,
        recovery_marker_persistence_ready,
        transaction_commit_ready,
        route_success_ready,
        formal_review_execution_ready,
        final_atomic_execution_ready,
        would_execute_decision: false,
        would_persist_approval_record: false,
        would_emit_lifecycle_event: false,
        would_mutate_contract: false,
        would_persist_recovery_marker: false,
        would_commit_transaction: false,
        would_return_http_ok: false,
        would_touch_disk: false,
        inherited_record_write_blockers: record_write_dry_run.blocked_by.clone(),
        inherited_lifecycle_effects_blocked_gates: lifecycle_emission_gate.blocked_gates.clone(),
        inherited_contract_mutation_blocked_gates: contract_mutation_gate.blocked_gates.clone(),
        inherited_recovery_marker_persistence_blocked_gates: marker_persistence_gate
            .blocked_gates
            .clone(),
        inherited_transaction_commit_blocked_gates: transaction_commit_gate.blocked_gates.clone(),
        inherited_route_success_blocked_gates: route_success_readiness_dry_run
            .blocked_gates
            .clone(),
        inherited_formal_review_execution_blocked_gates: formal_review_execution_readiness_dry_run
            .blocked_gates
            .clone(),
        required_gates,
        passed_gates,
        blocked_gates,
    }
}

fn contract_repair_approval_approve_execution_final_atomic_execution_plan_dry_run(
    action: &str,
    approval_id: &str,
    transaction_runner_dry_run: &ContractRepairApprovalApproveExecutionTransactionRunnerDryRun,
    cleanup_phase_readiness_dry_run: &ContractRepairApprovalApproveExecutionRunnerRecoveryMarkerCleanupPhaseReadinessDryRun,
    transaction_commit_phase_readiness_dry_run: &ContractRepairApprovalApproveExecutionRunnerTransactionCommitPhaseReadinessDryRun,
    rollback_execution_phase_readiness_dry_run: &ContractRepairApprovalApproveExecutionRunnerRollbackExecutionPhaseReadinessDryRun,
    final_atomic_readiness_dry_run: &ContractRepairApprovalApproveExecutionFinalAtomicReadinessDryRun,
    review_execution_enabled: bool,
) -> ContractRepairApprovalApproveExecutionFinalAtomicExecutionPlanDryRun {
    let execution_order = {
        let mut order = transaction_runner_dry_run.phase_order.clone();
        push_unique_blocker(&mut order, "commit_transaction");
        push_unique_blocker(&mut order, "dispatch_route_success");
        push_unique_blocker(&mut order, "return_review_approve_executed");
        order
    };
    let rollback_order = rollback_execution_phase_readiness_dry_run
        .rollback_order
        .clone();
    let execution_order_ready = [
        "write_recovery_marker",
        "transition_review_state",
        "persist_approval_record",
        "emit_lifecycle_event",
        "append_lifecycle_entry",
        "write_contract_source",
        "clear_recovery_marker",
        "commit_transaction",
        "dispatch_route_success",
        "return_review_approve_executed",
    ]
    .iter()
    .all(|phase| execution_order.iter().any(|entry| entry == phase));
    let rollback_order_ready = [
        "restore_contract_source",
        "restore_approval_record",
        "mark_recovery_marker_rolled_back",
    ]
    .iter()
    .all(|phase| rollback_order.iter().any(|entry| entry == phase));
    let transaction_runner_ready = transaction_runner_dry_run.commit_ready;
    let cleanup_phase_ready = cleanup_phase_readiness_dry_run.cleanup_phase_ready;
    let transaction_commit_phase_ready =
        transaction_commit_phase_readiness_dry_run.commit_phase_ready;
    let rollback_execution_ready =
        rollback_execution_phase_readiness_dry_run.rollback_execution_ready;
    let final_atomic_readiness_ready = final_atomic_readiness_dry_run.final_atomic_execution_ready;
    let formal_review_execution_ready =
        final_atomic_readiness_dry_run.formal_review_execution_ready;
    let commit_barrier_ready = transaction_runner_dry_run.commit_barrier_ready;
    let rollback_plan_ready = transaction_runner_dry_run.rollback_plan_ready
        && rollback_execution_phase_readiness_dry_run.rollback_plan_ready;
    let structural_plan_ready = action == "approve"
        && execution_order_ready
        && rollback_order_ready
        && transaction_runner_ready
        && cleanup_phase_ready
        && transaction_commit_phase_ready
        && rollback_execution_ready
        && commit_barrier_ready
        && rollback_plan_ready;
    let final_atomic_execution_plan_ready = structural_plan_ready && final_atomic_readiness_ready;
    let gate_states = [
        ("approve_action", action == "approve"),
        ("execution_order_ready", execution_order_ready),
        ("rollback_order_ready", rollback_order_ready),
        ("transaction_runner_ready", transaction_runner_ready),
        ("cleanup_phase_ready", cleanup_phase_ready),
        (
            "transaction_commit_phase_ready",
            transaction_commit_phase_ready,
        ),
        ("rollback_execution_ready", rollback_execution_ready),
        ("commit_barrier_ready", commit_barrier_ready),
        ("rollback_plan_ready", rollback_plan_ready),
        ("final_atomic_readiness_ready", final_atomic_readiness_ready),
        (
            "formal_review_execution_ready",
            formal_review_execution_ready,
        ),
    ];
    let required_gates = gate_states
        .iter()
        .map(|(gate, _)| (*gate).to_string())
        .collect::<Vec<_>>();
    let mut passed_gates = Vec::new();
    let mut blocked_gates = Vec::new();
    for (gate, passed) in gate_states {
        if passed {
            passed_gates.push(gate.to_string());
        } else {
            blocked_gates.push(gate.to_string());
        }
    }

    ContractRepairApprovalApproveExecutionFinalAtomicExecutionPlanDryRun {
        status: if final_atomic_execution_plan_ready {
            "approve_execution_final_atomic_execution_plan_ready".to_string()
        } else if structural_plan_ready {
            "approve_execution_final_atomic_execution_plan_ready_blocked".to_string()
        } else {
            "approve_execution_final_atomic_execution_plan_blocked".to_string()
        },
        action: action.to_string(),
        approval_id: approval_id.to_string(),
        plan_name: "approve_execution_final_atomic_execution_plan".to_string(),
        execution_order_ready,
        rollback_order_ready,
        transaction_runner_ready,
        cleanup_phase_ready,
        transaction_commit_phase_ready,
        rollback_execution_ready,
        final_atomic_readiness_ready,
        formal_review_execution_ready,
        review_execution_enabled,
        final_atomic_execution_plan_ready,
        partial_execution_allowed: false,
        recovery_marker_required: true,
        commit_barrier_ready,
        rollback_plan_ready,
        would_start_atomic_execution: false,
        would_persist_approval_record: false,
        would_mutate_contract: false,
        would_persist_recovery_marker: false,
        would_clear_recovery_marker: false,
        would_commit_transaction: false,
        would_rollback_on_error: false,
        would_return_http_ok: false,
        would_touch_disk: false,
        execution_order,
        rollback_order,
        inherited_runner_blockers: transaction_runner_dry_run.blocked_by.clone(),
        inherited_cleanup_phase_blocked_gates: cleanup_phase_readiness_dry_run
            .blocked_gates
            .clone(),
        inherited_transaction_commit_phase_blocked_gates:
            transaction_commit_phase_readiness_dry_run
                .blocked_gates
                .clone(),
        inherited_rollback_execution_blocked_gates: rollback_execution_phase_readiness_dry_run
            .blocked_gates
            .clone(),
        inherited_final_atomic_blocked_gates: final_atomic_readiness_dry_run.blocked_gates.clone(),
        required_gates,
        passed_gates,
        blocked_gates,
    }
}

fn contract_repair_approval_approve_execution_final_atomic_admission_gate_dry_run(
    action: &str,
    approval_id: &str,
    execution_plan_dry_run: &ContractRepairApprovalApproveExecutionFinalAtomicExecutionPlanDryRun,
) -> ContractRepairApprovalApproveExecutionFinalAtomicAdmissionGateDryRun {
    let execution_plan_structural_ready = execution_plan_dry_run.execution_order_ready
        && execution_plan_dry_run.rollback_order_ready
        && execution_plan_dry_run.transaction_runner_ready
        && execution_plan_dry_run.cleanup_phase_ready
        && execution_plan_dry_run.transaction_commit_phase_ready
        && execution_plan_dry_run.rollback_execution_ready
        && execution_plan_dry_run.commit_barrier_ready
        && execution_plan_dry_run.rollback_plan_ready
        && !execution_plan_dry_run.partial_execution_allowed
        && execution_plan_dry_run.recovery_marker_required;
    let final_atomic_execution_plan_ready =
        execution_plan_dry_run.final_atomic_execution_plan_ready;
    let final_atomic_readiness_ready = execution_plan_dry_run.final_atomic_readiness_ready;
    let formal_review_execution_ready = execution_plan_dry_run.formal_review_execution_ready;
    let review_execution_enabled = execution_plan_dry_run.review_execution_enabled;
    let admission_ready = execution_plan_structural_ready
        && final_atomic_execution_plan_ready
        && final_atomic_readiness_ready
        && formal_review_execution_ready;
    let gate_states = [
        (
            "execution_plan_structural_ready",
            execution_plan_structural_ready,
        ),
        (
            "final_atomic_execution_plan_ready",
            final_atomic_execution_plan_ready,
        ),
        ("final_atomic_readiness_ready", final_atomic_readiness_ready),
        (
            "formal_review_execution_ready",
            formal_review_execution_ready,
        ),
    ];
    let required_gates = gate_states
        .iter()
        .map(|(gate, _)| (*gate).to_string())
        .collect::<Vec<_>>();
    let mut passed_gates = Vec::new();
    let mut blocked_gates = Vec::new();
    for (gate, passed) in gate_states {
        if passed {
            passed_gates.push(gate.to_string());
        } else {
            blocked_gates.push(gate.to_string());
        }
    }

    ContractRepairApprovalApproveExecutionFinalAtomicAdmissionGateDryRun {
        status: if admission_ready {
            "approve_execution_final_atomic_admission_gate_ready".to_string()
        } else if execution_plan_structural_ready {
            "approve_execution_final_atomic_admission_gate_ready_blocked".to_string()
        } else {
            "approve_execution_final_atomic_admission_gate_blocked".to_string()
        },
        action: action.to_string(),
        approval_id: approval_id.to_string(),
        gate_name: "approve_execution_final_atomic_admission_gate".to_string(),
        execution_plan_structural_ready,
        execution_order_ready: execution_plan_dry_run.execution_order_ready,
        rollback_order_ready: execution_plan_dry_run.rollback_order_ready,
        final_atomic_execution_plan_ready,
        final_atomic_readiness_ready,
        formal_review_execution_ready,
        review_execution_enabled,
        partial_execution_allowed: execution_plan_dry_run.partial_execution_allowed,
        recovery_marker_required: execution_plan_dry_run.recovery_marker_required,
        commit_barrier_ready: execution_plan_dry_run.commit_barrier_ready,
        rollback_plan_ready: execution_plan_dry_run.rollback_plan_ready,
        admission_ready,
        would_enter_final_execution: false,
        would_execute_decision: false,
        would_return_http_ok: false,
        would_touch_disk: false,
        inherited_execution_plan_blocked_gates: execution_plan_dry_run.blocked_gates.clone(),
        inherited_execution_order: execution_plan_dry_run.execution_order.clone(),
        inherited_rollback_order: execution_plan_dry_run.rollback_order.clone(),
        required_gates,
        passed_gates,
        blocked_gates,
    }
}

fn contract_repair_approval_approve_execution_final_execution_entry_dry_run(
    action: &str,
    approval_id: &str,
    admission_gate_dry_run: &ContractRepairApprovalApproveExecutionFinalAtomicAdmissionGateDryRun,
    route_status_readiness_dry_run: &ContractRepairApprovalApproveExecutionRunnerRouteStatusReadinessDryRun,
    blocked_reasons: &[String],
    approve_final_execution_enabled: bool,
) -> ContractRepairApprovalApproveExecutionFinalExecutionEntryDryRun {
    let approve_action = action == "approve";
    let admission_ready = admission_gate_dry_run.admission_ready;
    let review_execution_enabled = admission_gate_dry_run.review_execution_enabled;
    let route_status_ready = route_status_readiness_dry_run.route_status_ready;
    let rollback_order_ready = [
        "restore_contract_source",
        "restore_approval_record",
        "mark_recovery_marker_rolled_back",
    ]
    .iter()
    .all(|phase| {
        admission_gate_dry_run
            .inherited_rollback_order
            .iter()
            .any(|entry| entry == phase)
    });
    let no_partial_execution_ready = !admission_gate_dry_run.partial_execution_allowed;
    let final_execution_entry_ready = approve_action
        && admission_ready
        && route_status_ready
        && rollback_order_ready
        && no_partial_execution_ready;
    let gate_states = [
        ("approve_action", approve_action),
        ("admission_ready", admission_ready),
        (
            "approve_execution_runner_route_status_ready",
            route_status_ready,
        ),
        ("rollback_order_ready", rollback_order_ready),
        ("no_partial_execution_ready", no_partial_execution_ready),
    ];
    let required_gates = gate_states
        .iter()
        .map(|(gate, _)| (*gate).to_string())
        .collect::<Vec<_>>();
    let mut passed_gates = Vec::new();
    let mut blocked_gates = Vec::new();
    for (gate, passed) in gate_states {
        if passed {
            passed_gates.push(gate.to_string());
        } else {
            blocked_gates.push(gate.to_string());
        }
    }

    ContractRepairApprovalApproveExecutionFinalExecutionEntryDryRun {
        status: if final_execution_entry_ready {
            "approve_execution_final_execution_entry_ready".to_string()
        } else if approve_action
            && admission_ready
            && rollback_order_ready
            && no_partial_execution_ready
        {
            "approve_execution_final_execution_entry_ready_blocked".to_string()
        } else {
            "approve_execution_final_execution_entry_blocked".to_string()
        },
        action: action.to_string(),
        approval_id: approval_id.to_string(),
        entry_name: "approve_execution_final_execution_entry".to_string(),
        admission_ready,
        review_execution_enabled,
        approve_final_execution_enabled,
        route_status_ready,
        rollback_order_ready,
        no_partial_execution_ready,
        final_execution_entry_ready,
        would_enter_final_execution: final_execution_entry_ready && approve_final_execution_enabled,
        would_execute_decision: final_execution_entry_ready && approve_final_execution_enabled,
        would_persist_approval_record: false,
        would_mutate_contract: false,
        would_persist_recovery_marker: false,
        would_clear_recovery_marker: false,
        would_commit_transaction: false,
        would_return_http_ok: final_execution_entry_ready && approve_final_execution_enabled,
        would_touch_disk: final_execution_entry_ready && approve_final_execution_enabled,
        inherited_admission_blocked_gates: admission_gate_dry_run.blocked_gates.clone(),
        inherited_execution_order: admission_gate_dry_run.inherited_execution_order.clone(),
        inherited_rollback_order: admission_gate_dry_run.inherited_rollback_order.clone(),
        inherited_route_status_blocked_gates: route_status_readiness_dry_run.blocked_gates.clone(),
        inherited_blocked_reasons: blocked_reasons.to_vec(),
        required_gates,
        passed_gates,
        blocked_gates,
    }
}

fn contract_repair_approval_approve_execution_final_execution_switch_readiness_dry_run(
    action: &str,
    approval_id: &str,
    final_execution_entry_dry_run: &ContractRepairApprovalApproveExecutionFinalExecutionEntryDryRun,
    record_write_dry_run: &ContractRepairApprovalRecordWriteDryRun,
    contract_mutation_gate: &ContractRepairApprovalContractMutationEnablementGate,
    recovery_marker_persistence_gate: &ContractRepairApprovalApproveExecutionRecoveryMarkerPersistenceGate,
    cleanup_phase_readiness_dry_run: &ContractRepairApprovalApproveExecutionRunnerRecoveryMarkerCleanupPhaseReadinessDryRun,
    transaction_commit_phase_readiness_dry_run: &ContractRepairApprovalApproveExecutionRunnerTransactionCommitPhaseReadinessDryRun,
    approve_final_execution_enabled: bool,
) -> ContractRepairApprovalApproveExecutionFinalExecutionSwitchReadinessDryRun {
    let final_execution_entry_ready = final_execution_entry_dry_run.final_execution_entry_ready;
    let record_write_ready = record_write_dry_run.write_ready;
    let contract_mutation_ready = contract_mutation_gate.mutation_ready;
    let recovery_marker_persistence_ready = recovery_marker_persistence_gate.persistence_ready;
    let cleanup_phase_ready = cleanup_phase_readiness_dry_run.cleanup_phase_ready;
    let transaction_commit_phase_ready =
        transaction_commit_phase_readiness_dry_run.commit_phase_ready;
    let route_status_ready = final_execution_entry_dry_run.route_status_ready;
    let rollback_order_ready = final_execution_entry_dry_run.rollback_order_ready;
    let no_partial_execution_ready = final_execution_entry_dry_run.no_partial_execution_ready;
    let final_execution_switch_ready = final_execution_entry_ready
        && record_write_ready
        && contract_mutation_ready
        && recovery_marker_persistence_ready
        && cleanup_phase_ready
        && transaction_commit_phase_ready
        && route_status_ready
        && rollback_order_ready
        && no_partial_execution_ready;
    let final_execution_switch_enabled =
        final_execution_switch_ready && approve_final_execution_enabled;
    let gate_states = [
        ("final_execution_entry_ready", final_execution_entry_ready),
        ("record_write_ready", record_write_ready),
        ("contract_mutation_ready", contract_mutation_ready),
        (
            "recovery_marker_persistence_ready",
            recovery_marker_persistence_ready,
        ),
        ("cleanup_phase_ready", cleanup_phase_ready),
        (
            "transaction_commit_phase_ready",
            transaction_commit_phase_ready,
        ),
        ("route_status_ready", route_status_ready),
        ("rollback_order_ready", rollback_order_ready),
        ("no_partial_execution_ready", no_partial_execution_ready),
    ];
    let required_gates = gate_states
        .iter()
        .map(|(gate, _)| (*gate).to_string())
        .collect::<Vec<_>>();
    let mut passed_gates = Vec::new();
    let mut blocked_gates = Vec::new();
    for (gate, passed) in gate_states {
        if passed {
            passed_gates.push(gate.to_string());
        } else {
            blocked_gates.push(gate.to_string());
        }
    }

    ContractRepairApprovalApproveExecutionFinalExecutionSwitchReadinessDryRun {
        status: if final_execution_switch_enabled {
            "approve_execution_final_execution_switch_readiness_ready".to_string()
        } else if final_execution_switch_ready {
            "approve_execution_final_execution_switch_readiness_ready_blocked".to_string()
        } else {
            "approve_execution_final_execution_switch_readiness_blocked".to_string()
        },
        action: action.to_string(),
        approval_id: approval_id.to_string(),
        switch_name: "approve_final_execution_enabled".to_string(),
        final_execution_entry_ready,
        approve_final_execution_enabled,
        record_write_ready,
        contract_mutation_ready,
        recovery_marker_persistence_ready,
        cleanup_phase_ready,
        transaction_commit_phase_ready,
        route_status_ready,
        rollback_order_ready,
        no_partial_execution_ready,
        final_execution_switch_ready,
        final_execution_switch_enabled,
        side_effect_replay_required: final_execution_switch_ready
            && !approve_final_execution_enabled,
        would_enable_final_execution: final_execution_switch_enabled,
        would_persist_approval_record: final_execution_switch_enabled,
        would_mutate_contract: final_execution_switch_enabled,
        would_persist_recovery_marker: final_execution_switch_enabled,
        would_clear_recovery_marker: final_execution_switch_enabled,
        would_commit_transaction: final_execution_switch_enabled,
        would_return_http_ok: final_execution_switch_enabled,
        would_touch_disk: final_execution_switch_enabled,
        replay_order: final_execution_entry_dry_run
            .inherited_execution_order
            .clone(),
        inherited_final_entry_blocked_gates: final_execution_entry_dry_run.blocked_gates.clone(),
        required_gates,
        passed_gates,
        blocked_gates,
    }
}

fn contract_repair_approval_approve_execution_final_execution_rollback_readiness_dry_run(
    action: &str,
    approval_id: &str,
    final_execution_switch_readiness_dry_run: &ContractRepairApprovalApproveExecutionFinalExecutionSwitchReadinessDryRun,
    rollback_execution_phase_readiness_dry_run: &ContractRepairApprovalApproveExecutionRunnerRollbackExecutionPhaseReadinessDryRun,
    approve_final_execution_enabled: bool,
) -> ContractRepairApprovalApproveExecutionFinalExecutionRollbackReadinessDryRun {
    let final_execution_switch_ready =
        final_execution_switch_readiness_dry_run.final_execution_switch_ready;
    let rollback_execution_ready =
        rollback_execution_phase_readiness_dry_run.rollback_execution_ready;
    let rollback_plan_ready = rollback_execution_phase_readiness_dry_run.rollback_plan_ready;
    let rollback_order = rollback_execution_phase_readiness_dry_run
        .rollback_order
        .clone();
    let rollback_order_ready = [
        "restore_contract_source",
        "restore_approval_record",
        "mark_recovery_marker_rolled_back",
    ]
    .iter()
    .all(|phase| rollback_order.iter().any(|entry| entry == phase));
    let failure_window_covered = final_execution_switch_ready
        && rollback_execution_ready
        && rollback_plan_ready
        && rollback_order_ready;
    let final_execution_rollback_ready = failure_window_covered;
    let gate_states = [
        ("final_execution_switch_ready", final_execution_switch_ready),
        ("rollback_execution_ready", rollback_execution_ready),
        ("rollback_plan_ready", rollback_plan_ready),
        ("rollback_order_ready", rollback_order_ready),
        ("failure_window_covered", failure_window_covered),
    ];
    let required_gates = gate_states
        .iter()
        .map(|(gate, _)| (*gate).to_string())
        .collect::<Vec<_>>();
    let mut passed_gates = Vec::new();
    let mut blocked_gates = Vec::new();
    for (gate, passed) in gate_states {
        if passed {
            passed_gates.push(gate.to_string());
        } else {
            blocked_gates.push(gate.to_string());
        }
    }

    ContractRepairApprovalApproveExecutionFinalExecutionRollbackReadinessDryRun {
        status: if final_execution_rollback_ready {
            "approve_execution_final_execution_rollback_readiness_ready".to_string()
        } else if final_execution_switch_ready {
            "approve_execution_final_execution_rollback_readiness_ready_blocked".to_string()
        } else {
            "approve_execution_final_execution_rollback_readiness_blocked".to_string()
        },
        action: action.to_string(),
        approval_id: approval_id.to_string(),
        gate_name: "approve_execution_final_execution_rollback_ready".to_string(),
        final_execution_switch_ready,
        approve_final_execution_enabled,
        rollback_execution_ready,
        rollback_plan_ready,
        rollback_order_ready,
        failure_window_covered,
        final_execution_rollback_ready,
        would_rollback_on_error: final_execution_rollback_ready && approve_final_execution_enabled,
        would_restore_contract_source: final_execution_rollback_ready
            && approve_final_execution_enabled,
        would_restore_approval_record: final_execution_rollback_ready
            && approve_final_execution_enabled,
        would_mark_recovery_marker_rolled_back: final_execution_rollback_ready
            && approve_final_execution_enabled,
        would_touch_disk: final_execution_rollback_ready && approve_final_execution_enabled,
        rollback_order,
        inherited_switch_blocked_gates: final_execution_switch_readiness_dry_run
            .blocked_gates
            .clone(),
        inherited_rollback_execution_blocked_gates: rollback_execution_phase_readiness_dry_run
            .blocked_gates
            .clone(),
        required_gates,
        passed_gates,
        blocked_gates,
    }
}

fn contract_repair_approval_final_execution_replay_phase_order() -> Vec<String> {
    vec![
        "write_recovery_marker".to_string(),
        "transition_review_state".to_string(),
        "persist_approval_record".to_string(),
        "emit_lifecycle_event".to_string(),
        "append_lifecycle_entry".to_string(),
        "write_contract_source".to_string(),
        "clear_recovery_marker".to_string(),
        "commit_transaction".to_string(),
        "dispatch_route_success".to_string(),
        "return_review_approve_executed".to_string(),
    ]
}

fn contract_repair_approval_final_execution_replay_phase_handler_routes() -> Vec<(String, String)> {
    vec![
        (
            "write_recovery_marker".to_string(),
            "approve_execution_recovery_marker_writer".to_string(),
        ),
        (
            "transition_review_state".to_string(),
            "contract_repair_review_transition".to_string(),
        ),
        (
            "persist_approval_record".to_string(),
            "contract_repair_approval_record_writer".to_string(),
        ),
        (
            "emit_lifecycle_event".to_string(),
            "contract_repair_approval_lifecycle_event_emitter".to_string(),
        ),
        (
            "append_lifecycle_entry".to_string(),
            "contract_repair_approval_lifecycle_entry_appender".to_string(),
        ),
        (
            "write_contract_source".to_string(),
            "contract_repair_approval_contract_source_writer".to_string(),
        ),
        (
            "clear_recovery_marker".to_string(),
            "approve_execution_recovery_marker_cleanup".to_string(),
        ),
        (
            "commit_transaction".to_string(),
            "approve_execution_transaction_commit_phase".to_string(),
        ),
        (
            "dispatch_route_success".to_string(),
            "approve_execution_route_success_dispatch".to_string(),
        ),
        (
            "return_review_approve_executed".to_string(),
            "approve_execution_success_return".to_string(),
        ),
    ]
}

fn contract_repair_approval_approve_execution_final_execution_replay_plan_dry_run(
    action: &str,
    approval_id: &str,
    final_execution_switch_readiness_dry_run: &ContractRepairApprovalApproveExecutionFinalExecutionSwitchReadinessDryRun,
    final_execution_rollback_readiness_dry_run: &ContractRepairApprovalApproveExecutionFinalExecutionRollbackReadinessDryRun,
    approve_final_execution_enabled: bool,
) -> ContractRepairApprovalApproveExecutionFinalExecutionReplayPlanDryRun {
    let final_execution_switch_ready =
        final_execution_switch_readiness_dry_run.final_execution_switch_ready;
    let final_execution_rollback_ready =
        final_execution_rollback_readiness_dry_run.final_execution_rollback_ready;
    let replay_order = final_execution_switch_readiness_dry_run
        .replay_order
        .clone();
    let required_replay_phases = contract_repair_approval_final_execution_replay_phase_order();
    let missing_replay_phases = required_replay_phases
        .iter()
        .filter(|phase| !replay_order.iter().any(|entry| entry == *phase))
        .cloned()
        .collect::<Vec<_>>();
    let replay_order_ready = missing_replay_phases.is_empty();
    let rollback_order = final_execution_rollback_readiness_dry_run
        .rollback_order
        .clone();
    let rollback_order_ready = final_execution_rollback_readiness_dry_run.rollback_order_ready;
    let replay_plan_ready = action == "approve"
        && final_execution_switch_ready
        && final_execution_rollback_ready
        && replay_order_ready
        && rollback_order_ready;
    let replay_enabled = replay_plan_ready && approve_final_execution_enabled;
    let gate_states = [
        ("final_execution_switch_ready", final_execution_switch_ready),
        (
            "final_execution_rollback_ready",
            final_execution_rollback_ready,
        ),
        ("replay_order_ready", replay_order_ready),
        ("rollback_order_ready", rollback_order_ready),
    ];
    let required_gates = gate_states
        .iter()
        .map(|(gate, _)| (*gate).to_string())
        .collect::<Vec<_>>();
    let mut passed_gates = Vec::new();
    let mut blocked_gates = Vec::new();
    for (gate, passed) in gate_states {
        if passed {
            passed_gates.push(gate.to_string());
        } else {
            blocked_gates.push(gate.to_string());
        }
    }

    ContractRepairApprovalApproveExecutionFinalExecutionReplayPlanDryRun {
        status: if replay_enabled {
            "approve_execution_final_execution_replay_plan_ready".to_string()
        } else if replay_plan_ready {
            "approve_execution_final_execution_replay_plan_ready_blocked".to_string()
        } else {
            "approve_execution_final_execution_replay_plan_blocked".to_string()
        },
        action: action.to_string(),
        approval_id: approval_id.to_string(),
        plan_name: "approve_execution_final_execution_replay_plan".to_string(),
        final_execution_switch_ready,
        final_execution_rollback_ready,
        approve_final_execution_enabled,
        replay_order_ready,
        rollback_order_ready,
        replay_plan_ready,
        replay_enabled,
        side_effect_replay_required: replay_plan_ready && !approve_final_execution_enabled,
        would_replay_side_effects: replay_enabled,
        would_enter_final_execution: replay_enabled,
        would_return_http_ok: replay_enabled,
        would_touch_disk: replay_enabled,
        replay_order,
        rollback_order,
        required_replay_phases,
        missing_replay_phases,
        inherited_switch_blocked_gates: final_execution_switch_readiness_dry_run
            .blocked_gates
            .clone(),
        inherited_rollback_blocked_gates: final_execution_rollback_readiness_dry_run
            .blocked_gates
            .clone(),
        required_gates,
        passed_gates,
        blocked_gates,
    }
}

fn contract_repair_approval_approve_execution_final_execution_replay_executor_dry_run(
    action: &str,
    approval_id: &str,
    replay_plan_dry_run: &ContractRepairApprovalApproveExecutionFinalExecutionReplayPlanDryRun,
    approve_final_execution_enabled: bool,
) -> ContractRepairApprovalApproveExecutionFinalExecutionReplayExecutorDryRun {
    let replay_plan_ready = replay_plan_dry_run.replay_plan_ready;
    let final_execution_rollback_ready = replay_plan_dry_run.final_execution_rollback_ready;
    let expected_replay_order = contract_repair_approval_final_execution_replay_phase_order();
    let replay_order = replay_plan_dry_run.replay_order.clone();
    let missing_executor_replay_phases = expected_replay_order
        .iter()
        .filter(|phase| !replay_order.iter().any(|entry| entry == *phase))
        .cloned()
        .collect::<Vec<_>>();
    let unexpected_executor_replay_phases = replay_order
        .iter()
        .filter(|phase| !expected_replay_order.iter().any(|entry| entry == *phase))
        .cloned()
        .collect::<Vec<_>>();
    let replay_executor_order_ready = replay_order == expected_replay_order;
    let replay_executor_ready = action == "approve"
        && replay_plan_ready
        && final_execution_rollback_ready
        && replay_executor_order_ready;
    let replay_executor_admitted = replay_executor_ready && approve_final_execution_enabled;
    let gate_states = [
        ("replay_plan_ready", replay_plan_ready),
        (
            "final_execution_rollback_ready",
            final_execution_rollback_ready,
        ),
        ("replay_executor_order_ready", replay_executor_order_ready),
        (
            "approve_final_execution_enabled",
            approve_final_execution_enabled,
        ),
    ];
    let required_gates = gate_states
        .iter()
        .map(|(gate, _)| (*gate).to_string())
        .collect::<Vec<_>>();
    let mut passed_gates = Vec::new();
    let mut blocked_gates = Vec::new();
    for (gate, passed) in gate_states {
        if passed {
            passed_gates.push(gate.to_string());
        } else {
            blocked_gates.push(gate.to_string());
        }
    }

    ContractRepairApprovalApproveExecutionFinalExecutionReplayExecutorDryRun {
        status: if replay_executor_admitted {
            "approve_execution_final_execution_replay_executor_ready".to_string()
        } else if replay_executor_ready {
            "approve_execution_final_execution_replay_executor_ready_blocked".to_string()
        } else {
            "approve_execution_final_execution_replay_executor_blocked".to_string()
        },
        action: action.to_string(),
        approval_id: approval_id.to_string(),
        executor_name: "approve_execution_final_execution_replay_executor".to_string(),
        replay_plan_ready,
        final_execution_rollback_ready,
        approve_final_execution_enabled,
        replay_executor_ready,
        replay_executor_admitted,
        replay_executor_order_ready,
        would_start_replay_executor: replay_executor_admitted,
        would_write_recovery_marker: replay_executor_admitted,
        would_persist_approval_record: replay_executor_admitted,
        would_emit_lifecycle_event: replay_executor_admitted,
        would_append_lifecycle_entry: replay_executor_admitted,
        would_write_contract_source: replay_executor_admitted,
        would_clear_recovery_marker: replay_executor_admitted,
        would_commit_transaction: replay_executor_admitted,
        would_dispatch_route_success: replay_executor_admitted,
        would_return_http_ok: replay_executor_admitted,
        would_touch_disk: replay_executor_admitted,
        expected_replay_order,
        missing_executor_replay_phases,
        unexpected_executor_replay_phases,
        replay_order,
        rollback_order: replay_plan_dry_run.rollback_order.clone(),
        inherited_replay_plan_blocked_gates: replay_plan_dry_run.blocked_gates.clone(),
        required_gates,
        passed_gates,
        blocked_gates,
    }
}

fn contract_repair_approval_approve_execution_final_execution_replay_executor_routing_dry_run(
    action: &str,
    approval_id: &str,
    replay_executor_dry_run: &ContractRepairApprovalApproveExecutionFinalExecutionReplayExecutorDryRun,
    approve_final_execution_enabled: bool,
) -> ContractRepairApprovalApproveExecutionFinalExecutionReplayExecutorRoutingDryRun {
    let expected_replay_order = contract_repair_approval_final_execution_replay_phase_order();
    let handler_route_pairs =
        contract_repair_approval_final_execution_replay_phase_handler_routes();
    let handler_phases = handler_route_pairs
        .iter()
        .map(|(phase, _)| phase.clone())
        .collect::<Vec<_>>();
    let handler_routes = handler_route_pairs
        .iter()
        .map(|(phase, handler)| format!("{phase}:{handler}"))
        .collect::<Vec<_>>();
    let missing_handler_phases = expected_replay_order
        .iter()
        .filter(|phase| !handler_phases.iter().any(|entry| entry == *phase))
        .cloned()
        .collect::<Vec<_>>();
    let unexpected_handler_phases = handler_phases
        .iter()
        .filter(|phase| !expected_replay_order.iter().any(|entry| entry == *phase))
        .cloned()
        .collect::<Vec<_>>();
    let handler_routing_ready = action == "approve"
        && handler_phases == expected_replay_order
        && missing_handler_phases.is_empty()
        && unexpected_handler_phases.is_empty();
    let replay_executor_ready = replay_executor_dry_run.replay_executor_ready;
    let replay_executor_order_ready = replay_executor_dry_run.replay_executor_order_ready;
    let replay_executor_admitted = replay_executor_dry_run.replay_executor_admitted;
    let routing_admitted = handler_routing_ready && replay_executor_admitted;
    let gate_states = [
        ("replay_executor_ready", replay_executor_ready),
        ("replay_executor_order_ready", replay_executor_order_ready),
        ("handler_routing_ready", handler_routing_ready),
        (
            "approve_final_execution_enabled",
            approve_final_execution_enabled,
        ),
    ];
    let required_gates = gate_states
        .iter()
        .map(|(gate, _)| (*gate).to_string())
        .collect::<Vec<_>>();
    let mut passed_gates = Vec::new();
    let mut blocked_gates = Vec::new();
    for (gate, passed) in gate_states {
        if passed {
            passed_gates.push(gate.to_string());
        } else {
            blocked_gates.push(gate.to_string());
        }
    }

    ContractRepairApprovalApproveExecutionFinalExecutionReplayExecutorRoutingDryRun {
        status: if routing_admitted {
            "approve_execution_final_execution_replay_executor_routing_ready".to_string()
        } else if handler_routing_ready && replay_executor_ready && replay_executor_order_ready {
            "approve_execution_final_execution_replay_executor_routing_ready_blocked".to_string()
        } else {
            "approve_execution_final_execution_replay_executor_routing_blocked".to_string()
        },
        action: action.to_string(),
        approval_id: approval_id.to_string(),
        routing_name: "approve_execution_final_execution_replay_executor_routing".to_string(),
        replay_executor_ready,
        replay_executor_order_ready,
        approve_final_execution_enabled,
        handler_routing_ready,
        replay_executor_admitted,
        routing_admitted,
        executor_routing_required: handler_routing_ready && !replay_executor_admitted,
        would_route_through_executor: routing_admitted,
        would_start_replay_executor: routing_admitted,
        would_touch_disk: routing_admitted,
        expected_replay_order,
        handler_phases,
        handler_routes,
        missing_handler_phases,
        unexpected_handler_phases,
        inherited_executor_blocked_gates: replay_executor_dry_run.blocked_gates.clone(),
        required_gates,
        passed_gates,
        blocked_gates,
    }
}

fn contract_repair_approval_approve_execution_final_execution_routed_write_handoff_dry_run(
    action: &str,
    approval_id: &str,
    routing_dry_run: &ContractRepairApprovalApproveExecutionFinalExecutionReplayExecutorRoutingDryRun,
    approve_final_execution_enabled: bool,
    routed_write_handoff_enabled: bool,
    legacy_inline_final_writes_enabled: bool,
) -> ContractRepairApprovalApproveExecutionFinalExecutionRoutedWriteHandoffDryRun {
    let handoff_phases = routing_dry_run.expected_replay_order.clone();
    let handler_routes = routing_dry_run.handler_routes.clone();
    let missing_handoff_phases = routing_dry_run.missing_handler_phases.clone();
    let unexpected_handoff_phases = routing_dry_run.unexpected_handler_phases.clone();
    let inline_final_write_gates = vec![
        "review_record_execution_enabled".to_string(),
        "contract_source_write_execution_enabled".to_string(),
        "recovery_marker_persistence_execution_enabled".to_string(),
        "recovery_marker_cleanup_phase_execution_enabled".to_string(),
        "transaction_commit_phase_execution_enabled".to_string(),
        "route_success_execution_enabled".to_string(),
    ];
    let replay_executor_ready = routing_dry_run.replay_executor_ready;
    let replay_executor_order_ready = routing_dry_run.replay_executor_order_ready;
    let handler_routing_ready = routing_dry_run.handler_routing_ready;
    let routing_ready = handler_routing_ready
        && replay_executor_ready
        && replay_executor_order_ready
        && missing_handoff_phases.is_empty()
        && unexpected_handoff_phases.is_empty();
    let legacy_inline_final_writes_blocked = !legacy_inline_final_writes_enabled;
    let routed_write_handoff_ready = action == "approve"
        && routing_ready
        && legacy_inline_final_writes_blocked
        && !inline_final_write_gates.is_empty();
    let routed_write_handoff_admitted = routed_write_handoff_ready
        && approve_final_execution_enabled
        && routed_write_handoff_enabled;
    let gate_states = [
        ("routing_ready", routing_ready),
        (
            "legacy_inline_final_writes_blocked",
            legacy_inline_final_writes_blocked,
        ),
        (
            "approve_final_execution_enabled",
            approve_final_execution_enabled,
        ),
        ("routed_write_handoff_enabled", routed_write_handoff_enabled),
    ];
    let required_gates = gate_states
        .iter()
        .map(|(gate, _)| (*gate).to_string())
        .collect::<Vec<_>>();
    let mut passed_gates = Vec::new();
    let mut blocked_gates = Vec::new();
    for (gate, passed) in gate_states {
        if passed {
            passed_gates.push(gate.to_string());
        } else {
            blocked_gates.push(gate.to_string());
        }
    }

    ContractRepairApprovalApproveExecutionFinalExecutionRoutedWriteHandoffDryRun {
        status: if routed_write_handoff_admitted {
            "approve_execution_final_execution_routed_write_handoff_ready".to_string()
        } else if routed_write_handoff_ready {
            "approve_execution_final_execution_routed_write_handoff_ready_blocked".to_string()
        } else {
            "approve_execution_final_execution_routed_write_handoff_blocked".to_string()
        },
        action: action.to_string(),
        approval_id: approval_id.to_string(),
        handoff_name: "approve_execution_final_execution_routed_write_handoff".to_string(),
        approve_final_execution_enabled,
        routed_write_handoff_enabled,
        legacy_inline_final_writes_enabled,
        replay_executor_ready,
        replay_executor_order_ready,
        handler_routing_ready,
        routing_ready,
        legacy_inline_final_writes_blocked,
        routed_write_handoff_ready,
        routed_write_handoff_admitted,
        would_route_writes_through_executor: routed_write_handoff_admitted,
        would_start_replay_executor: routed_write_handoff_admitted,
        would_write_recovery_marker: routed_write_handoff_admitted,
        would_persist_approval_record: routed_write_handoff_admitted,
        would_emit_lifecycle_event: routed_write_handoff_admitted,
        would_append_lifecycle_entry: routed_write_handoff_admitted,
        would_write_contract_source: routed_write_handoff_admitted,
        would_clear_recovery_marker: routed_write_handoff_admitted,
        would_commit_transaction: routed_write_handoff_admitted,
        would_dispatch_route_success: routed_write_handoff_admitted,
        would_return_http_ok: routed_write_handoff_admitted,
        would_touch_disk: routed_write_handoff_admitted,
        handoff_phases,
        handler_routes,
        inline_final_write_gates,
        missing_handoff_phases,
        unexpected_handoff_phases,
        inherited_routing_blocked_gates: routing_dry_run.blocked_gates.clone(),
        required_gates,
        passed_gates,
        blocked_gates,
    }
}

fn contract_repair_approval_approve_execution_final_execution_routed_handler_plan_dry_run(
    action: &str,
    approval_id: &str,
    handoff_dry_run: &ContractRepairApprovalApproveExecutionFinalExecutionRoutedWriteHandoffDryRun,
    recovery_marker_write_dry_run: &ContractRepairApprovalApproveExecutionRecoveryMarkerWriteDryRun,
    review_transition_dry_run: &ContractRepairApprovalReviewTransitionDryRun,
    record_write_dry_run: &ContractRepairApprovalRecordWriteDryRun,
    lifecycle_event_dry_run: &ContractRepairApprovalLifecycleEventDryRun,
    lifecycle_entry_append_dry_run: &ContractRepairApprovalLifecycleEntryAppendDryRun,
    contract_source_write_dry_run: &ContractRepairApprovalContractSourceWriteDryRun,
    cleanup_phase_readiness_dry_run: &ContractRepairApprovalApproveExecutionRunnerRecoveryMarkerCleanupPhaseReadinessDryRun,
    transaction_commit_phase_readiness_dry_run: &ContractRepairApprovalApproveExecutionRunnerTransactionCommitPhaseReadinessDryRun,
    route_success_readiness_dry_run: &ContractRepairApprovalApproveExecutionRunnerRouteSuccessReadinessDryRun,
) -> ContractRepairApprovalApproveExecutionFinalExecutionRoutedHandlerPlanDryRun {
    let handler_states = vec![
        (
            "write_recovery_marker".to_string(),
            recovery_marker_write_dry_run.write_ready,
        ),
        (
            "transition_review_state".to_string(),
            review_transition_dry_run.transition_ready,
        ),
        (
            "persist_approval_record".to_string(),
            record_write_dry_run.write_ready,
        ),
        (
            "emit_lifecycle_event".to_string(),
            lifecycle_event_dry_run.emission_ready,
        ),
        (
            "append_lifecycle_entry".to_string(),
            lifecycle_entry_append_dry_run.append_ready,
        ),
        (
            "write_contract_source".to_string(),
            contract_source_write_dry_run.write_ready,
        ),
        (
            "clear_recovery_marker".to_string(),
            cleanup_phase_readiness_dry_run.cleanup_phase_ready,
        ),
        (
            "commit_transaction".to_string(),
            transaction_commit_phase_readiness_dry_run.commit_phase_ready,
        ),
        (
            "dispatch_route_success".to_string(),
            route_success_readiness_dry_run.route_success_ready,
        ),
        (
            "return_review_approve_executed".to_string(),
            route_success_readiness_dry_run.route_success_ready,
        ),
    ];
    let handler_phases = handler_states
        .iter()
        .map(|(phase, _)| phase.clone())
        .collect::<Vec<_>>();
    let handler_readiness = handler_states
        .iter()
        .map(|(phase, ready)| format!("{phase}:{}", if *ready { "ready" } else { "blocked" }))
        .collect::<Vec<_>>();
    let ready_handlers = handler_states
        .iter()
        .filter(|(_, ready)| *ready)
        .map(|(phase, _)| phase.clone())
        .collect::<Vec<_>>();
    let blocked_handlers = handler_states
        .iter()
        .filter(|(_, ready)| !*ready)
        .map(|(phase, _)| phase.clone())
        .collect::<Vec<_>>();
    let handler_count = handler_states.len();
    let ready_handler_count = ready_handlers.len();
    let blocked_handler_count = blocked_handlers.len();
    let all_handlers_ready = blocked_handlers.is_empty();
    let handler_order_ready = handler_phases == handoff_dry_run.handoff_phases;
    let routed_write_handoff_ready = handoff_dry_run.routed_write_handoff_ready;
    let routed_write_handoff_admitted = handoff_dry_run.routed_write_handoff_admitted;
    let routing_ready = handoff_dry_run.routing_ready;
    let handler_execution_plan_ready = action == "approve"
        && routed_write_handoff_ready
        && routing_ready
        && handler_order_ready
        && all_handlers_ready;
    let would_execute_handlers = handler_execution_plan_ready && routed_write_handoff_admitted;
    let gate_states = [
        ("routed_write_handoff_ready", routed_write_handoff_ready),
        ("routing_ready", routing_ready),
        ("handler_order_ready", handler_order_ready),
        ("all_handlers_ready", all_handlers_ready),
        (
            "routed_write_handoff_admitted",
            routed_write_handoff_admitted,
        ),
    ];
    let required_gates = gate_states
        .iter()
        .map(|(gate, _)| (*gate).to_string())
        .collect::<Vec<_>>();
    let mut passed_gates = Vec::new();
    let mut blocked_gates = Vec::new();
    for (gate, passed) in gate_states {
        if passed {
            passed_gates.push(gate.to_string());
        } else {
            blocked_gates.push(gate.to_string());
        }
    }

    ContractRepairApprovalApproveExecutionFinalExecutionRoutedHandlerPlanDryRun {
        status: if would_execute_handlers {
            "approve_execution_final_execution_routed_handler_plan_ready".to_string()
        } else if handler_execution_plan_ready {
            "approve_execution_final_execution_routed_handler_plan_ready_blocked".to_string()
        } else {
            "approve_execution_final_execution_routed_handler_plan_blocked".to_string()
        },
        action: action.to_string(),
        approval_id: approval_id.to_string(),
        plan_name: "approve_execution_final_execution_routed_handler_plan".to_string(),
        routed_write_handoff_ready,
        routed_write_handoff_admitted,
        routing_ready,
        handler_execution_plan_ready,
        handler_count,
        ready_handler_count,
        blocked_handler_count,
        would_execute_handlers,
        would_start_replay_executor: would_execute_handlers,
        would_write_recovery_marker: would_execute_handlers,
        would_transition_review: would_execute_handlers,
        would_persist_approval_record: would_execute_handlers,
        would_emit_lifecycle_event: would_execute_handlers,
        would_append_lifecycle_entry: would_execute_handlers,
        would_write_contract_source: would_execute_handlers,
        would_clear_recovery_marker: would_execute_handlers,
        would_commit_transaction: would_execute_handlers,
        would_dispatch_route_success: would_execute_handlers,
        would_return_http_ok: would_execute_handlers,
        would_touch_disk: would_execute_handlers,
        handler_phases,
        handler_routes: handoff_dry_run.handler_routes.clone(),
        handler_readiness,
        ready_handlers,
        blocked_handlers,
        inherited_handoff_blocked_gates: handoff_dry_run.blocked_gates.clone(),
        required_gates,
        passed_gates,
        blocked_gates,
    }
}

fn contract_repair_approval_approve_execution_final_execution_routed_execution_attempt_dry_run(
    action: &str,
    approval_id: &str,
    handler_plan_dry_run: &ContractRepairApprovalApproveExecutionFinalExecutionRoutedHandlerPlanDryRun,
) -> ContractRepairApprovalApproveExecutionFinalExecutionRoutedExecutionAttemptDryRun {
    let handler_execution_plan_ready = handler_plan_dry_run.handler_execution_plan_ready;
    let routed_write_handoff_admitted = handler_plan_dry_run.routed_write_handoff_admitted;
    let no_blocked_handlers = handler_plan_dry_run.blocked_handler_count == 0;
    let execution_order_ready = handler_plan_dry_run.handler_phases
        == contract_repair_approval_final_execution_replay_phase_order();
    let execution_attempt_ready = action == "approve"
        && handler_execution_plan_ready
        && no_blocked_handlers
        && execution_order_ready;
    let execution_attempt_admitted = execution_attempt_ready && routed_write_handoff_admitted;
    let gate_states = [
        ("handler_execution_plan_ready", handler_execution_plan_ready),
        ("no_blocked_handlers", no_blocked_handlers),
        ("execution_order_ready", execution_order_ready),
        (
            "routed_write_handoff_admitted",
            routed_write_handoff_admitted,
        ),
    ];
    let required_gates = gate_states
        .iter()
        .map(|(gate, _)| (*gate).to_string())
        .collect::<Vec<_>>();
    let mut passed_gates = Vec::new();
    let mut blocked_gates = Vec::new();
    for (gate, passed) in gate_states {
        if passed {
            passed_gates.push(gate.to_string());
        } else {
            blocked_gates.push(gate.to_string());
        }
    }
    let execution_attempt_blocked_reason = blocked_gates
        .first()
        .cloned()
        .unwrap_or_else(|| "none".to_string());

    ContractRepairApprovalApproveExecutionFinalExecutionRoutedExecutionAttemptDryRun {
        status: if execution_attempt_admitted {
            "approve_execution_final_execution_routed_execution_attempt_ready".to_string()
        } else if execution_attempt_ready {
            "approve_execution_final_execution_routed_execution_attempt_ready_blocked".to_string()
        } else {
            "approve_execution_final_execution_routed_execution_attempt_blocked".to_string()
        },
        action: action.to_string(),
        approval_id: approval_id.to_string(),
        attempt_name: "approve_execution_final_execution_routed_execution_attempt".to_string(),
        handler_execution_plan_ready,
        routed_write_handoff_admitted,
        execution_attempt_ready,
        execution_attempt_admitted,
        execution_attempt_blocked_reason,
        handler_count: handler_plan_dry_run.handler_count,
        ready_handler_count: handler_plan_dry_run.ready_handler_count,
        blocked_handler_count: handler_plan_dry_run.blocked_handler_count,
        would_execute_handlers: execution_attempt_admitted,
        would_start_replay_executor: execution_attempt_admitted,
        would_write_recovery_marker: execution_attempt_admitted,
        would_transition_review: execution_attempt_admitted,
        would_persist_approval_record: execution_attempt_admitted,
        would_emit_lifecycle_event: execution_attempt_admitted,
        would_append_lifecycle_entry: execution_attempt_admitted,
        would_write_contract_source: execution_attempt_admitted,
        would_clear_recovery_marker: execution_attempt_admitted,
        would_commit_transaction: execution_attempt_admitted,
        would_dispatch_route_success: execution_attempt_admitted,
        would_return_http_ok: execution_attempt_admitted,
        would_touch_disk: execution_attempt_admitted,
        execution_order: handler_plan_dry_run.handler_phases.clone(),
        handler_readiness: handler_plan_dry_run.handler_readiness.clone(),
        inherited_handler_plan_blocked_gates: handler_plan_dry_run.blocked_gates.clone(),
        required_gates,
        passed_gates,
        blocked_gates,
    }
}

fn contract_repair_approval_approve_execution_final_execution_ordered_handler_execution_confirmation_dry_run(
    action: &str,
    approval_id: &str,
    final_execution_entry_dry_run: &ContractRepairApprovalApproveExecutionFinalExecutionEntryDryRun,
    final_execution_rollback_readiness_dry_run: &ContractRepairApprovalApproveExecutionFinalExecutionRollbackReadinessDryRun,
    execution_attempt_dry_run: &ContractRepairApprovalApproveExecutionFinalExecutionRoutedExecutionAttemptDryRun,
    ordered_handler_execution_connection_enabled: bool,
) -> ContractRepairApprovalApproveExecutionFinalExecutionOrderedHandlerExecutionConfirmationDryRun {
    let expected_execution_order = contract_repair_approval_final_execution_replay_phase_order();
    let execution_attempt_admitted = execution_attempt_dry_run.execution_attempt_admitted;
    let execution_order_ready =
        execution_attempt_dry_run.execution_order == expected_execution_order;
    let handler_readiness_ready = expected_execution_order.iter().all(|phase| {
        execution_attempt_dry_run
            .handler_readiness
            .iter()
            .any(|entry| entry == &format!("{phase}:ready"))
    });
    let handler_count = execution_attempt_dry_run.handler_count;
    let ready_handler_count = execution_attempt_dry_run.ready_handler_count;
    let blocked_handler_count = execution_attempt_dry_run.blocked_handler_count;
    let rollback_confirmation_ready =
        final_execution_rollback_readiness_dry_run.final_execution_rollback_ready;
    let no_partial_write_guard_ready = final_execution_entry_dry_run.no_partial_execution_ready;
    let ordered_handler_execution_ready = action == "approve"
        && execution_attempt_admitted
        && execution_order_ready
        && handler_readiness_ready
        && blocked_handler_count == 0;
    let ordered_handler_execution_connection_preflight_ready = ordered_handler_execution_ready
        && rollback_confirmation_ready
        && no_partial_write_guard_ready;
    let ordered_handler_execution_connected = ordered_handler_execution_connection_preflight_ready
        && ordered_handler_execution_connection_enabled;
    let ordered_handler_execution_confirmed =
        ordered_handler_execution_connection_preflight_ready && ordered_handler_execution_connected;
    let ordered_handler_execution_dry_run_ready =
        ordered_handler_execution_connection_preflight_ready;
    let dry_run_handler_execution_order = if ordered_handler_execution_dry_run_ready {
        execution_attempt_dry_run.execution_order.clone()
    } else {
        Vec::new()
    };
    let dry_run_handler_execution_count = dry_run_handler_execution_order.len();
    let dry_run_handler_execution_receipts = dry_run_handler_execution_order
        .iter()
        .map(|phase| format!("{phase}:dry_run_executed"))
        .collect::<Vec<_>>();
    let ordered_handler_execution_dry_run_complete = ordered_handler_execution_dry_run_ready
        && dry_run_handler_execution_order == expected_execution_order
        && dry_run_handler_execution_count == handler_count;
    let dry_run_handler_execution_effects_blocked =
        ordered_handler_execution_dry_run_complete && !ordered_handler_execution_connected;
    let gate_states = [
        ("execution_attempt_admitted", execution_attempt_admitted),
        ("execution_order_ready", execution_order_ready),
        ("handler_readiness_ready", handler_readiness_ready),
        ("no_blocked_handlers", blocked_handler_count == 0),
        ("rollback_confirmation_ready", rollback_confirmation_ready),
        ("no_partial_write_guard_ready", no_partial_write_guard_ready),
        (
            "ordered_handler_execution_connection_enabled",
            ordered_handler_execution_connection_enabled,
        ),
    ];
    let required_gates = gate_states
        .iter()
        .map(|(gate, _)| (*gate).to_string())
        .collect::<Vec<_>>();
    let mut passed_gates = Vec::new();
    let mut blocked_gates = Vec::new();
    for (gate, passed) in gate_states {
        if passed {
            passed_gates.push(gate.to_string());
        } else {
            blocked_gates.push(gate.to_string());
        }
    }
    let confirmation_blocked_reason = blocked_gates
        .first()
        .cloned()
        .unwrap_or_else(|| "none".to_string());
    let unconfirmed_handlers = if ordered_handler_execution_confirmed {
        Vec::new()
    } else {
        execution_attempt_dry_run.execution_order.clone()
    };

    ContractRepairApprovalApproveExecutionFinalExecutionOrderedHandlerExecutionConfirmationDryRun {
        status: if ordered_handler_execution_confirmed {
            "approve_execution_final_execution_ordered_handler_execution_confirmation_ready"
                .to_string()
        } else if ordered_handler_execution_connection_preflight_ready {
            "approve_execution_final_execution_ordered_handler_execution_confirmation_ready_blocked"
                .to_string()
        } else {
            "approve_execution_final_execution_ordered_handler_execution_confirmation_blocked"
                .to_string()
        },
        action: action.to_string(),
        approval_id: approval_id.to_string(),
        confirmation_name:
            "approve_execution_final_execution_ordered_handler_execution_confirmation".to_string(),
        execution_attempt_admitted,
        execution_order_ready,
        handler_readiness_ready,
        handler_count,
        ready_handler_count,
        blocked_handler_count,
        rollback_confirmation_ready,
        no_partial_write_guard_ready,
        ordered_handler_execution_ready,
        ordered_handler_execution_connection_preflight_ready,
        ordered_handler_execution_connection_enabled,
        ordered_handler_execution_connected,
        ordered_handler_execution_confirmed,
        ordered_handler_execution_dry_run_ready,
        ordered_handler_execution_dry_run_complete,
        dry_run_handler_execution_count,
        dry_run_handler_execution_effects_blocked,
        confirmation_blocked_reason,
        would_execute_handlers: ordered_handler_execution_confirmed,
        would_start_replay_executor: ordered_handler_execution_confirmed,
        would_write_recovery_marker: ordered_handler_execution_confirmed,
        would_transition_review: ordered_handler_execution_confirmed,
        would_persist_approval_record: ordered_handler_execution_confirmed,
        would_emit_lifecycle_event: ordered_handler_execution_confirmed,
        would_append_lifecycle_entry: ordered_handler_execution_confirmed,
        would_write_contract_source: ordered_handler_execution_confirmed,
        would_clear_recovery_marker: ordered_handler_execution_confirmed,
        would_commit_transaction: ordered_handler_execution_confirmed,
        would_dispatch_route_success: ordered_handler_execution_confirmed,
        would_return_http_ok: ordered_handler_execution_confirmed,
        would_touch_disk: ordered_handler_execution_confirmed,
        execution_order: execution_attempt_dry_run.execution_order.clone(),
        handler_readiness: execution_attempt_dry_run.handler_readiness.clone(),
        dry_run_handler_execution_order,
        dry_run_handler_execution_receipts,
        unconfirmed_handlers,
        inherited_attempt_blocked_gates: execution_attempt_dry_run.blocked_gates.clone(),
        required_gates,
        passed_gates,
        blocked_gates,
    }
}

fn contract_repair_approval_approve_execution_final_execution_routed_route_success_release_dry_run(
    action: &str,
    approval_id: &str,
    response_status: &str,
    route_status: &str,
    execution_attempt_dry_run: &ContractRepairApprovalApproveExecutionFinalExecutionRoutedExecutionAttemptDryRun,
    ordered_handler_execution_confirmation_dry_run: &ContractRepairApprovalApproveExecutionFinalExecutionOrderedHandlerExecutionConfirmationDryRun,
    legacy_inline_final_writes_enabled: bool,
    routed_route_success_release_application_enabled: bool,
) -> ContractRepairApprovalApproveExecutionFinalExecutionRoutedRouteSuccessReleaseDryRun {
    let execution_attempt_admitted = execution_attempt_dry_run.execution_attempt_admitted;
    let route_success_phase_ready = execution_attempt_dry_run
        .execution_order
        .iter()
        .any(|phase| phase == "dispatch_route_success")
        && execution_attempt_dry_run
            .execution_order
            .iter()
            .any(|phase| phase == "return_review_approve_executed")
        && execution_attempt_dry_run
            .handler_readiness
            .iter()
            .any(|entry| entry == "dispatch_route_success:ready")
        && execution_attempt_dry_run
            .handler_readiness
            .iter()
            .any(|entry| entry == "return_review_approve_executed:ready");
    let legacy_inline_final_writes_blocked = !legacy_inline_final_writes_enabled;
    let response_status_transition_ready = response_status == "review_decision_execution_blocked";
    let route_status_transition_ready = route_status == "review_decision_execution_blocked";
    let ordered_handler_execution_required =
        ordered_handler_execution_confirmation_dry_run.ordered_handler_execution_ready;
    let ordered_handler_execution_confirmed =
        ordered_handler_execution_confirmation_dry_run.ordered_handler_execution_confirmed;
    let response_status_connection_ready = response_status_transition_ready
        && route_status_transition_ready
        && ordered_handler_execution_required;
    let response_status_connection_blocked_reason =
        if response_status_connection_ready && !ordered_handler_execution_confirmed {
            "ordered_handler_execution_confirmed".to_string()
        } else if !response_status_transition_ready {
            "response_status_transition_ready".to_string()
        } else if !route_status_transition_ready {
            "route_status_transition_ready".to_string()
        } else {
            "none".to_string()
        };
    let routed_route_success_release_ready = action == "approve"
        && execution_attempt_admitted
        && route_success_phase_ready
        && legacy_inline_final_writes_blocked
        && response_status_transition_ready
        && route_status_transition_ready;
    let routed_route_success_release_connected =
        response_status_connection_ready && ordered_handler_execution_confirmed;
    let routed_route_success_release_admitted =
        routed_route_success_release_ready && routed_route_success_release_connected;
    let routed_route_success_release_application_ready = routed_route_success_release_admitted;
    let routed_route_success_release_applied = routed_route_success_release_application_ready
        && routed_route_success_release_application_enabled;
    let response_status_application_blocked_reason =
        if routed_route_success_release_application_ready
            && !routed_route_success_release_application_enabled
        {
            "routed_route_success_release_application_enabled".to_string()
        } else if !routed_route_success_release_admitted {
            "routed_route_success_release_admitted".to_string()
        } else {
            "none".to_string()
        };
    let gate_states = [
        ("execution_attempt_admitted", execution_attempt_admitted),
        ("route_success_phase_ready", route_success_phase_ready),
        (
            "legacy_inline_final_writes_blocked",
            legacy_inline_final_writes_blocked,
        ),
        (
            "response_status_transition_ready",
            response_status_transition_ready,
        ),
        (
            "route_status_transition_ready",
            route_status_transition_ready,
        ),
        (
            "ordered_handler_execution_confirmed",
            ordered_handler_execution_confirmed,
        ),
        (
            "routed_route_success_release_application_enabled",
            routed_route_success_release_application_enabled,
        ),
    ];
    let required_gates = gate_states
        .iter()
        .map(|(gate, _)| (*gate).to_string())
        .collect::<Vec<_>>();
    let mut passed_gates = Vec::new();
    let mut blocked_gates = Vec::new();
    for (gate, passed) in gate_states {
        if passed {
            passed_gates.push(gate.to_string());
        } else {
            blocked_gates.push(gate.to_string());
        }
    }

    ContractRepairApprovalApproveExecutionFinalExecutionRoutedRouteSuccessReleaseDryRun {
        status: if routed_route_success_release_applied {
            "approve_execution_final_execution_routed_route_success_release_ready".to_string()
        } else if routed_route_success_release_ready {
            "approve_execution_final_execution_routed_route_success_release_ready_blocked"
                .to_string()
        } else {
            "approve_execution_final_execution_routed_route_success_release_blocked".to_string()
        },
        action: action.to_string(),
        approval_id: approval_id.to_string(),
        release_name: "approve_execution_final_execution_routed_route_success_release".to_string(),
        execution_attempt_admitted,
        route_success_phase_ready,
        legacy_inline_final_writes_enabled,
        legacy_inline_final_writes_blocked,
        response_status_transition_ready,
        route_status_transition_ready,
        ordered_handler_execution_required,
        ordered_handler_execution_confirmed,
        response_status_connection_ready,
        response_status_connection_blocked_reason,
        routed_route_success_release_ready,
        routed_route_success_release_connected,
        routed_route_success_release_admitted,
        routed_route_success_release_application_ready,
        routed_route_success_release_application_enabled,
        routed_route_success_release_applied,
        response_status_application_blocked_reason,
        current_response_status: response_status.to_string(),
        current_route_status: route_status.to_string(),
        target_response_status: "review_approve_executed".to_string(),
        target_route_status: "review_approve_executed".to_string(),
        would_release_route_success: routed_route_success_release_admitted,
        would_set_response_status: routed_route_success_release_applied,
        would_set_route_status: routed_route_success_release_applied,
        would_return_http_ok: routed_route_success_release_applied,
        would_touch_disk: false,
        inherited_attempt_blocked_gates: execution_attempt_dry_run.blocked_gates.clone(),
        inherited_ordered_handler_execution_blocked_gates:
            ordered_handler_execution_confirmation_dry_run
                .blocked_gates
                .clone(),
        execution_order: execution_attempt_dry_run.execution_order.clone(),
        handler_readiness: execution_attempt_dry_run.handler_readiness.clone(),
        required_gates,
        passed_gates,
        blocked_gates,
    }
}

fn contract_repair_approval_approve_execution_final_execution_durable_writeback_bundle_dry_run(
    action: &str,
    approval_id: &str,
    response_application_success: bool,
    record_write_dry_run: &ContractRepairApprovalRecordWriteDryRun,
    approval_record_persistence_enabled: bool,
    recovery_marker_write_dry_run: &ContractRepairApprovalApproveExecutionRecoveryMarkerWriteDryRun,
    contract_source_write_dry_run: &ContractRepairApprovalContractSourceWriteDryRun,
    contract_source_write_enabled: bool,
    recovery_marker_persistence_gate: &ContractRepairApprovalApproveExecutionRecoveryMarkerPersistenceGate,
    transaction_commit_gate: &ContractRepairApprovalApproveExecutionTransactionCommitGate,
    cleanup_phase_readiness_dry_run: &ContractRepairApprovalApproveExecutionRunnerRecoveryMarkerCleanupPhaseReadinessDryRun,
    rollback_readiness_dry_run: &ContractRepairApprovalApproveExecutionFinalExecutionRollbackReadinessDryRun,
    final_execution_entry_dry_run: &ContractRepairApprovalApproveExecutionFinalExecutionEntryDryRun,
    route_success_release_dry_run: &ContractRepairApprovalApproveExecutionFinalExecutionRoutedRouteSuccessReleaseDryRun,
    durable_writeback_bundle_enabled: bool,
    durable_writeback_bundle_execution_enabled: bool,
    durable_writeback_bundle_disk_application_enabled: bool,
    durable_disk_application_helper_execution_connected: bool,
) -> ContractRepairApprovalApproveExecutionFinalExecutionDurableWritebackBundleDryRun {
    let approval_record_write_ready = record_write_dry_run.write_ready;
    let contract_source_write_ready = contract_source_write_dry_run.write_ready;
    let recovery_marker_persistence_ready = recovery_marker_persistence_gate.persistence_ready;
    let transaction_commit_ready = transaction_commit_gate.commit_ready;
    let recovery_marker_cleanup_ready = cleanup_phase_readiness_dry_run.cleanup_phase_ready;
    let rollback_ready = rollback_readiness_dry_run.final_execution_rollback_ready;
    let no_partial_write_guard_ready = final_execution_entry_dry_run.no_partial_execution_ready;
    let durable_writeback_bundle_ready = response_application_success
        && approval_record_write_ready
        && contract_source_write_ready
        && recovery_marker_persistence_ready
        && transaction_commit_ready
        && recovery_marker_cleanup_ready
        && rollback_ready
        && no_partial_write_guard_ready;
    let durable_writeback_bundle_admitted =
        durable_writeback_bundle_ready && durable_writeback_bundle_enabled;
    let durable_writeback_bundle_execution_admitted =
        durable_writeback_bundle_admitted && durable_writeback_bundle_execution_enabled;
    let durable_writeback_bundle_disk_application_admitted =
        durable_writeback_bundle_execution_admitted
            && durable_writeback_bundle_disk_application_enabled;
    let dry_run_durable_execution_order = route_success_release_dry_run
        .execution_order
        .iter()
        .filter(|phase| {
            phase.as_str() != "dispatch_route_success"
                && phase.as_str() != "return_review_approve_executed"
        })
        .cloned()
        .collect::<Vec<_>>();
    let dry_run_execution_ready = durable_writeback_bundle_admitted;
    let dry_run_execution_count = dry_run_durable_execution_order.len();
    let dry_run_execution_complete = dry_run_execution_ready && dry_run_execution_count == 8;
    let dry_run_execution_effects_blocked = !durable_writeback_bundle_disk_application_admitted;
    let dry_run_durable_execution_receipts = if dry_run_execution_ready {
        dry_run_durable_execution_order
            .iter()
            .map(|phase| format!("{phase}:dry_run_executed"))
            .collect::<Vec<_>>()
    } else {
        Vec::new()
    };
    let dry_run_rollback_order = rollback_readiness_dry_run.rollback_order.clone();
    let rollback_dry_run_ready = durable_writeback_bundle_admitted && rollback_ready;
    let rollback_dry_run_count = dry_run_rollback_order.len();
    let rollback_dry_run_complete = rollback_dry_run_ready
        && [
            "restore_contract_source",
            "restore_approval_record",
            "mark_recovery_marker_rolled_back",
        ]
        .iter()
        .all(|phase| dry_run_rollback_order.iter().any(|entry| entry == phase));
    let rollback_dry_run_effects_blocked = !durable_writeback_bundle_disk_application_admitted;
    let dry_run_rollback_receipts = if rollback_dry_run_ready {
        dry_run_rollback_order
            .iter()
            .map(|phase| format!("{phase}:dry_run_rollback_ready"))
            .collect::<Vec<_>>()
    } else {
        Vec::new()
    };
    let rollback_coverage_pairs = vec![
        "write_contract_source->restore_contract_source".to_string(),
        "persist_approval_record->restore_approval_record".to_string(),
        "write_recovery_marker->mark_recovery_marker_rolled_back".to_string(),
    ];
    let mut uncovered_durable_phases = Vec::new();
    for (durable_phase, rollback_phase) in [
        ("write_contract_source", "restore_contract_source"),
        ("persist_approval_record", "restore_approval_record"),
        ("write_recovery_marker", "mark_recovery_marker_rolled_back"),
    ] {
        if dry_run_durable_execution_order
            .iter()
            .any(|phase| phase == durable_phase)
            && !dry_run_rollback_order
                .iter()
                .any(|phase| phase == rollback_phase)
        {
            uncovered_durable_phases.push(durable_phase.to_string());
        }
    }
    let rollback_coverage_ready = rollback_dry_run_complete
        && uncovered_durable_phases.is_empty()
        && rollback_coverage_pairs.len() == 3;
    let durable_execution_rollback_barrier_ready =
        dry_run_execution_complete && rollback_coverage_ready;
    let approval_record_disk_write_planned =
        durable_writeback_bundle_execution_admitted && approval_record_write_ready;
    let contract_source_disk_write_planned =
        durable_writeback_bundle_execution_admitted && contract_source_write_ready;
    let recovery_marker_disk_write_planned =
        durable_writeback_bundle_execution_admitted && recovery_marker_persistence_ready;
    let recovery_marker_cleanup_planned =
        durable_writeback_bundle_execution_admitted && recovery_marker_cleanup_ready;
    let transaction_commit_planned =
        durable_writeback_bundle_execution_admitted && transaction_commit_ready;
    let disk_application_effect_count = [
        approval_record_disk_write_planned,
        contract_source_disk_write_planned,
        recovery_marker_disk_write_planned,
        recovery_marker_cleanup_planned,
        transaction_commit_planned,
    ]
    .iter()
    .filter(|planned| **planned)
    .count();
    let disk_application_plan_ready = durable_execution_rollback_barrier_ready
        && disk_application_effect_count == 5
        && contract_source_write_dry_run.source_digest_before.is_some()
        && contract_source_write_dry_run.source_digest_after.is_some();
    let disk_application_plan_complete = disk_application_plan_ready;
    let disk_application_effects_blocked = !durable_writeback_bundle_disk_application_admitted;
    let disk_application_plan_receipts = if disk_application_plan_ready {
        vec![
            "persist_approval_record:planned".to_string(),
            "write_contract_source:planned".to_string(),
            "write_recovery_marker:planned".to_string(),
            "clear_recovery_marker:planned".to_string(),
            "commit_transaction:planned".to_string(),
        ]
    } else {
        Vec::new()
    };
    let disk_application_atomic_write_set = if disk_application_plan_ready {
        vec![
            "persist_approval_record".to_string(),
            "write_contract_source".to_string(),
            "write_recovery_marker".to_string(),
            "clear_recovery_marker".to_string(),
            "commit_transaction".to_string(),
        ]
    } else {
        Vec::new()
    };
    let disk_application_atomic_write_set_count = disk_application_atomic_write_set.len();
    let disk_application_atomic_write_set_ready =
        disk_application_plan_ready && disk_application_atomic_write_set_count == 5;
    let disk_application_simulated_failure_points = disk_application_atomic_write_set
        .iter()
        .map(|effect| format!("after:{effect}"))
        .collect::<Vec<_>>();
    let disk_application_simulated_failure_point_count =
        disk_application_simulated_failure_points.len();
    let disk_application_failure_probe_coverage_ready = disk_application_atomic_write_set_ready
        && disk_application_simulated_failure_point_count
            == disk_application_atomic_write_set_count;
    let mut disk_application_rollback_actions = if disk_application_plan_ready {
        dry_run_rollback_order.clone()
    } else {
        Vec::new()
    };
    let disk_application_cleanup_recovery_action_ready =
        recovery_marker_cleanup_planned && disk_application_atomic_write_set_ready;
    if disk_application_cleanup_recovery_action_ready {
        disk_application_rollback_actions.push("restore_recovery_marker".to_string());
    }
    let disk_application_commit_terminal_verification_ready =
        transaction_commit_planned && disk_application_atomic_write_set_ready;
    if disk_application_commit_terminal_verification_ready {
        disk_application_rollback_actions.push("verify_committed_transaction".to_string());
    }
    let disk_application_rollback_action_count = disk_application_rollback_actions.len();
    let disk_application_rollback_action_coverage_ready = disk_application_atomic_write_set_ready
        && disk_application_rollback_action_count == disk_application_atomic_write_set_count;
    let disk_application_terminal_recovery_proof_ready =
        disk_application_cleanup_recovery_action_ready
            && disk_application_commit_terminal_verification_ready;
    let disk_application_all_or_nothing_guard_ready = disk_application_failure_probe_coverage_ready
        && disk_application_rollback_action_coverage_ready
        && disk_application_terminal_recovery_proof_ready
        && transaction_commit_planned;
    let disk_application_partial_enabled_after_failure_count =
        if disk_application_all_or_nothing_guard_ready {
            0
        } else {
            disk_application_atomic_write_set_count
                .saturating_sub(disk_application_rollback_action_count)
        };
    let disk_application_partial_state_proof_ready =
        disk_application_partial_enabled_after_failure_count == 0;
    let disk_application_transaction_failure_proof_ready = disk_application_plan_ready
        && disk_application_atomic_write_set_ready
        && disk_application_all_or_nothing_guard_ready
        && disk_application_failure_probe_coverage_ready
        && disk_application_rollback_action_coverage_ready
        && disk_application_terminal_recovery_proof_ready
        && disk_application_partial_state_proof_ready;
    let disk_application_transaction_proof_ready = disk_application_transaction_failure_proof_ready;
    let disk_application_executor_name =
        "approve_execution_durable_disk_application_executor".to_string();
    let disk_application_executor_handler_routes =
        if disk_application_transaction_failure_proof_ready {
            vec![
                "persist_approval_record->contract_repair_approval_record_write_with_gate"
                    .to_string(),
                "write_contract_source->contract_repair_approval_contract_source_write_with_gate"
                    .to_string(),
                "write_recovery_marker->contract_repair_approval_approve_execution_recovery_marker_write_with_gate".to_string(),
                "clear_recovery_marker->contract_repair_approval_approve_execution_runner_recovery_marker_cleanup_phase_with_gate".to_string(),
                "commit_transaction->contract_repair_approval_approve_execution_runner_transaction_commit_phase_with_gate".to_string(),
            ]
        } else {
            Vec::new()
        };
    let disk_application_executor_handler_count = disk_application_executor_handler_routes.len();
    let disk_application_executor_ready_handlers = disk_application_executor_handler_routes
        .iter()
        .map(|route| {
            route
                .split_once("->")
                .map(|(effect, _)| effect.to_string())
                .unwrap_or_else(|| route.clone())
        })
        .collect::<Vec<_>>();
    let disk_application_executor_ready_handler_count =
        disk_application_executor_ready_handlers.len();
    let disk_application_executor_blocked_handlers = Vec::new();
    let disk_application_executor_blocked_handler_count =
        disk_application_executor_blocked_handlers.len();
    let disk_application_executor_ready = disk_application_transaction_failure_proof_ready
        && disk_application_executor_handler_count == 5
        && disk_application_executor_ready_handler_count == 5
        && disk_application_executor_blocked_handler_count == 0;
    let disk_application_executor_admitted =
        disk_application_executor_ready && durable_writeback_bundle_disk_application_admitted;
    let disk_application_executor_effects_blocked = !disk_application_executor_admitted;
    let disk_application_executor_blocked_reason = if !disk_application_executor_ready {
        "disk_application_executor_ready".to_string()
    } else if !durable_writeback_bundle_disk_application_enabled {
        "approve_final_execution_durable_writeback_bundle_disk_application_enabled".to_string()
    } else {
        "none".to_string()
    };
    let disk_application_endpoint_helper_name =
        "contract_repair_approval_execute_durable_disk_application_handlers".to_string();
    let disk_application_endpoint_helper_wired = true;
    let disk_application_endpoint_helper_required_inputs = vec![
        "store_dir".to_string(),
        "graph_store_dir".to_string(),
        "approval_record".to_string(),
        "recovery_marker_write".to_string(),
        "runner_dry_run".to_string(),
        "contract_source_resolution".to_string(),
        "contract_patch_plan".to_string(),
        "contract_patch_apply".to_string(),
        "recovery_marker_cleanup".to_string(),
        "transaction_commit".to_string(),
    ];
    let disk_application_endpoint_helper_ready_inputs = if disk_application_executor_ready {
        disk_application_endpoint_helper_required_inputs.clone()
    } else {
        Vec::new()
    };
    let disk_application_endpoint_helper_blocked_inputs = if disk_application_executor_ready {
        Vec::new()
    } else {
        disk_application_endpoint_helper_required_inputs.clone()
    };
    let disk_application_endpoint_helper_admission_ready =
        disk_application_endpoint_helper_wired && disk_application_executor_ready;
    let disk_application_endpoint_helper_admitted = disk_application_endpoint_helper_admission_ready
        && durable_writeback_bundle_disk_application_admitted;
    let disk_application_endpoint_helper_execution_connected =
        durable_disk_application_helper_execution_connected;
    let disk_application_endpoint_helper_would_execute =
        disk_application_endpoint_helper_execution_connected
            && disk_application_endpoint_helper_admitted;
    let disk_application_endpoint_helper_execution_blocked =
        !disk_application_endpoint_helper_would_execute;
    let disk_application_endpoint_helper_blocked_reason = if !disk_application_endpoint_helper_wired
    {
        "disk_application_endpoint_helper_wired".to_string()
    } else if !disk_application_endpoint_helper_admission_ready {
        "disk_application_endpoint_helper_admission_ready".to_string()
    } else if !durable_writeback_bundle_disk_application_enabled {
        "approve_final_execution_durable_writeback_bundle_disk_application_enabled".to_string()
    } else {
        "none".to_string()
    };
    let disk_application_endpoint_helper_execution_blocked_reason =
        if !disk_application_endpoint_helper_execution_connected {
            "disk_application_endpoint_helper_execution_connected".to_string()
        } else if disk_application_endpoint_helper_execution_blocked {
            disk_application_endpoint_helper_blocked_reason.clone()
        } else {
            "none".to_string()
        };
    let durable_writeback_bundle_execution_preflight_ready =
        dry_run_execution_complete && durable_execution_rollback_barrier_ready;
    let gate_states = [
        ("response_application_success", response_application_success),
        ("approval_record_write_ready", approval_record_write_ready),
        ("contract_source_write_ready", contract_source_write_ready),
        (
            "recovery_marker_persistence_ready",
            recovery_marker_persistence_ready,
        ),
        ("transaction_commit_ready", transaction_commit_ready),
        (
            "recovery_marker_cleanup_ready",
            recovery_marker_cleanup_ready,
        ),
        ("rollback_ready", rollback_ready),
        ("no_partial_write_guard_ready", no_partial_write_guard_ready),
        (
            "durable_execution_rollback_barrier_ready",
            durable_execution_rollback_barrier_ready,
        ),
        ("disk_application_plan_ready", disk_application_plan_ready),
        (
            "disk_application_transaction_failure_proof_ready",
            disk_application_transaction_failure_proof_ready,
        ),
        (
            "disk_application_executor_ready",
            disk_application_executor_ready,
        ),
        (
            "disk_application_endpoint_helper_admission_ready",
            disk_application_endpoint_helper_admission_ready,
        ),
        (
            "disk_application_endpoint_helper_execution_connected",
            disk_application_endpoint_helper_execution_connected,
        ),
        (
            "approve_final_execution_durable_writeback_bundle_enabled",
            durable_writeback_bundle_enabled,
        ),
        (
            "approve_final_execution_durable_writeback_bundle_execution_enabled",
            durable_writeback_bundle_execution_enabled,
        ),
        (
            "approve_final_execution_durable_writeback_bundle_disk_application_enabled",
            durable_writeback_bundle_disk_application_enabled,
        ),
    ];
    let required_gates = gate_states
        .iter()
        .map(|(gate, _)| (*gate).to_string())
        .collect::<Vec<_>>();
    let mut passed_gates = Vec::new();
    let mut blocked_gates = Vec::new();
    for (gate, passed) in gate_states {
        if passed {
            passed_gates.push(gate.to_string());
        } else {
            blocked_gates.push(gate.to_string());
        }
    }
    let bundle_blocked_reason = blocked_gates
        .first()
        .cloned()
        .unwrap_or_else(|| "none".to_string());

    ContractRepairApprovalApproveExecutionFinalExecutionDurableWritebackBundleDryRun {
        status: if durable_writeback_bundle_disk_application_admitted {
            "approve_execution_final_execution_durable_writeback_bundle_ready".to_string()
        } else if durable_writeback_bundle_execution_admitted {
            "approve_execution_final_execution_durable_writeback_bundle_ready_blocked".to_string()
        } else {
            "approve_execution_final_execution_durable_writeback_bundle_blocked".to_string()
        },
        action: action.to_string(),
        approval_id: approval_id.to_string(),
        bundle_name: "approve_execution_final_execution_durable_writeback_bundle".to_string(),
        response_application_success,
        approval_record_write_ready,
        approval_record_persistence_enabled,
        contract_source_write_ready,
        contract_source_write_enabled,
        recovery_marker_persistence_ready,
        transaction_commit_ready,
        recovery_marker_cleanup_ready,
        rollback_ready,
        no_partial_write_guard_ready,
        durable_writeback_bundle_ready,
        durable_writeback_bundle_enabled,
        durable_writeback_bundle_admitted,
        durable_writeback_bundle_execution_enabled,
        durable_writeback_bundle_execution_admitted,
        durable_writeback_bundle_disk_application_enabled,
        durable_writeback_bundle_disk_application_admitted,
        durable_writeback_bundle_execution_preflight_ready,
        dry_run_execution_ready,
        dry_run_execution_complete,
        dry_run_execution_count,
        dry_run_execution_effects_blocked,
        rollback_dry_run_ready,
        rollback_dry_run_complete,
        rollback_dry_run_count,
        rollback_dry_run_effects_blocked,
        rollback_coverage_ready,
        durable_execution_rollback_barrier_ready,
        disk_application_plan_ready,
        disk_application_plan_complete,
        disk_application_effect_count,
        disk_application_effects_blocked,
        approval_record_disk_write_planned,
        contract_source_disk_write_planned,
        recovery_marker_disk_write_planned,
        recovery_marker_cleanup_planned,
        transaction_commit_planned,
        approval_record_file_name: record_write_dry_run.file_name.clone(),
        recovery_marker_file_name: recovery_marker_write_dry_run.file_name.clone(),
        contract_source_digest_before: contract_source_write_dry_run.source_digest_before.clone(),
        contract_source_digest_after: contract_source_write_dry_run.source_digest_after.clone(),
        disk_application_transaction_proof_ready,
        disk_application_atomic_write_set_ready,
        disk_application_all_or_nothing_guard_ready,
        disk_application_failure_probe_coverage_ready,
        disk_application_rollback_action_coverage_ready,
        disk_application_cleanup_recovery_action_ready,
        disk_application_commit_terminal_verification_ready,
        disk_application_terminal_recovery_proof_ready,
        disk_application_partial_state_proof_ready,
        disk_application_transaction_failure_proof_ready,
        disk_application_executor_name,
        disk_application_executor_ready,
        disk_application_executor_admitted,
        disk_application_executor_effects_blocked,
        disk_application_executor_blocked_reason,
        disk_application_endpoint_helper_name,
        disk_application_endpoint_helper_wired,
        disk_application_endpoint_helper_admission_ready,
        disk_application_endpoint_helper_admitted,
        disk_application_endpoint_helper_execution_connected,
        disk_application_endpoint_helper_execution_blocked,
        disk_application_endpoint_helper_would_execute,
        disk_application_endpoint_helper_blocked_reason,
        disk_application_endpoint_helper_execution_blocked_reason,
        disk_application_executor_handler_count,
        disk_application_executor_ready_handler_count,
        disk_application_executor_blocked_handler_count,
        disk_application_atomic_write_set_count,
        disk_application_simulated_failure_point_count,
        disk_application_rollback_action_count,
        disk_application_partial_enabled_after_failure_count,
        bundle_blocked_reason,
        would_persist_approval_record: durable_writeback_bundle_disk_application_admitted,
        would_write_contract_source: durable_writeback_bundle_disk_application_admitted,
        would_persist_recovery_marker: durable_writeback_bundle_disk_application_admitted,
        would_clear_recovery_marker: durable_writeback_bundle_disk_application_admitted,
        would_commit_transaction: durable_writeback_bundle_disk_application_admitted,
        would_touch_disk: durable_writeback_bundle_disk_application_admitted,
        execution_order: route_success_release_dry_run.execution_order.clone(),
        rollback_order: rollback_readiness_dry_run.rollback_order.clone(),
        dry_run_durable_execution_order,
        dry_run_durable_execution_receipts,
        dry_run_rollback_order,
        dry_run_rollback_receipts,
        rollback_coverage_pairs,
        uncovered_durable_phases,
        disk_application_plan_receipts,
        disk_application_atomic_write_set,
        disk_application_simulated_failure_points,
        disk_application_rollback_actions,
        disk_application_executor_handler_routes,
        disk_application_executor_ready_handlers,
        disk_application_executor_blocked_handlers,
        disk_application_endpoint_helper_required_inputs,
        disk_application_endpoint_helper_ready_inputs,
        disk_application_endpoint_helper_blocked_inputs,
        required_gates,
        passed_gates,
        blocked_gates,
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
struct ContractRepairApprovalDurableDiskApplicationExecution {
    status: String,
    executor_name: String,
    executor_admitted: bool,
    recovery_marker_written: bool,
    approval_record_persisted: bool,
    contract_source_written: bool,
    recovery_marker_cleared: bool,
    transaction_committed: bool,
    rollback_executed: bool,
    would_touch_disk: bool,
    execution_receipts: Vec<String>,
    rollback_receipts: Vec<String>,
    blocked_by: Vec<String>,
}

#[allow(dead_code)]
fn contract_repair_approval_durable_disk_application_execution_locked_response(
    durable_writeback_bundle: &ContractRepairApprovalApproveExecutionFinalExecutionDurableWritebackBundleDryRun,
) -> ContractRepairApprovalApproveExecutionDurableDiskApplicationExecution {
    let mut blocked_by = Vec::new();
    if durable_writeback_bundle.disk_application_endpoint_helper_execution_blocked_reason != "none"
    {
        blocked_by.push(
            durable_writeback_bundle
                .disk_application_endpoint_helper_execution_blocked_reason
                .clone(),
        );
    }

    ContractRepairApprovalApproveExecutionDurableDiskApplicationExecution {
        status: if durable_writeback_bundle.disk_application_endpoint_helper_would_execute {
            "approve_execution_durable_disk_application_endpoint_helper_ready".to_string()
        } else {
            "approve_execution_durable_disk_application_endpoint_helper_execution_blocked"
                .to_string()
        },
        executor_name: durable_writeback_bundle
            .disk_application_endpoint_helper_name
            .clone(),
        endpoint_helper_execution_connected: durable_writeback_bundle
            .disk_application_endpoint_helper_execution_connected,
        endpoint_helper_would_execute: durable_writeback_bundle
            .disk_application_endpoint_helper_would_execute,
        executor_admitted: durable_writeback_bundle.disk_application_executor_admitted,
        recovery_marker_written: false,
        approval_record_persisted: false,
        contract_source_written: false,
        recovery_marker_cleared: false,
        transaction_committed: false,
        rollback_executed: false,
        would_touch_disk: false,
        execution_receipts: Vec::new(),
        rollback_receipts: Vec::new(),
        blocked_by,
    }
}

#[allow(dead_code)]
fn contract_repair_approval_durable_disk_application_execution_response(
    execution: &ContractRepairApprovalDurableDiskApplicationExecution,
    endpoint_helper_execution_connected: bool,
    endpoint_helper_would_execute: bool,
) -> ContractRepairApprovalApproveExecutionDurableDiskApplicationExecution {
    ContractRepairApprovalApproveExecutionDurableDiskApplicationExecution {
        status: execution.status.clone(),
        executor_name: execution.executor_name.clone(),
        endpoint_helper_execution_connected,
        endpoint_helper_would_execute,
        executor_admitted: execution.executor_admitted,
        recovery_marker_written: execution.recovery_marker_written,
        approval_record_persisted: execution.approval_record_persisted,
        contract_source_written: execution.contract_source_written,
        recovery_marker_cleared: execution.recovery_marker_cleared,
        transaction_committed: execution.transaction_committed,
        rollback_executed: execution.rollback_executed,
        would_touch_disk: execution.would_touch_disk,
        execution_receipts: execution.execution_receipts.clone(),
        rollback_receipts: execution.rollback_receipts.clone(),
        blocked_by: execution.blocked_by.clone(),
    }
}

#[allow(dead_code)]
async fn contract_repair_approval_rollback_durable_disk_application_partial_state(
    store_dir: &FsPath,
    graph_store_dir: &FsPath,
    approval_id: &str,
    source_ref: &ContractRepairApprovalContractSourceRef,
    approval_record_before_execution: Option<&ContractRepairApprovalRecordPreview>,
    contract_source_before_execution: Option<&serde_json::Value>,
    marker_file_name: &str,
    execution: &mut ContractRepairApprovalDurableDiskApplicationExecution,
    failure_phase: &str,
) {
    let mut rollback_receipts = Vec::new();

    if execution.contract_source_written {
        let source_path =
            contract_repair_approval_contract_source_path(graph_store_dir, source_ref);
        if let Some(source_before_execution) = contract_source_before_execution {
            match crate::runtime_persistence::atomic_write_json(
                &source_path,
                source_before_execution,
            )
            .await
            {
                Ok(()) => rollback_receipts.push("restore_contract_source:executed".to_string()),
                Err(_) => {
                    rollback_receipts.push("restore_contract_source:failed".to_string());
                    push_unique_blocker(&mut execution.blocked_by, "restore_contract_source");
                }
            }
        } else {
            rollback_receipts.push("restore_contract_source:missing_snapshot".to_string());
            push_unique_blocker(
                &mut execution.blocked_by,
                "restore_contract_source_snapshot",
            );
        }
    }

    if execution.approval_record_persisted {
        match approval_record_before_execution {
            Some(record_before_execution) => {
                match persist_contract_repair_approval_record(store_dir, record_before_execution)
                    .await
                {
                    Ok(()) => {
                        rollback_receipts.push("restore_approval_record:executed".to_string())
                    }
                    Err(_) => {
                        rollback_receipts.push("restore_approval_record:failed".to_string());
                        push_unique_blocker(&mut execution.blocked_by, "restore_approval_record");
                    }
                }
            }
            None => {
                let record_path = contract_repair_approval_record_path(store_dir, approval_id);
                match fs::remove_file(&record_path).await {
                    Ok(()) => rollback_receipts
                        .push("restore_approval_record:removed_created_record".to_string()),
                    Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                        rollback_receipts.push("restore_approval_record:already_absent".to_string())
                    }
                    Err(_) => {
                        rollback_receipts.push("restore_approval_record:failed".to_string());
                        push_unique_blocker(&mut execution.blocked_by, "restore_approval_record");
                    }
                }
            }
        }
    }

    if execution.recovery_marker_written && !execution.recovery_marker_cleared {
        let marker_path = store_dir.join(marker_file_name);
        match fs::remove_file(&marker_path).await {
            Ok(()) => {
                rollback_receipts.push("mark_recovery_marker_rolled_back:executed".to_string())
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => rollback_receipts
                .push("mark_recovery_marker_rolled_back:already_absent".to_string()),
            Err(_) => {
                rollback_receipts.push("mark_recovery_marker_rolled_back:failed".to_string());
                push_unique_blocker(
                    &mut execution.blocked_by,
                    "mark_recovery_marker_rolled_back",
                );
            }
        }
    }

    if !rollback_receipts.is_empty() {
        rollback_receipts.push(format!("rollback_completed:{failure_phase}"));
        execution.rollback_executed = true;
        execution.rollback_receipts.extend(rollback_receipts);
        execution.would_touch_disk = true;
    }
}

#[allow(dead_code)]
async fn contract_repair_approval_execute_durable_disk_application_handlers(
    store_dir: &FsPath,
    graph_store_dir: &FsPath,
    approval_record: &ContractRepairApprovalRecordPreview,
    marker_write_dry_run: &ContractRepairApprovalApproveExecutionRecoveryMarkerWriteDryRun,
    idempotency_precheck: &ContractRepairApprovalApproveExecutionRecoveryMarkerIdempotencyPrecheck,
    runner_dry_run: &ContractRepairApprovalApproveExecutionTransactionRunnerDryRun,
    source_ref: &ContractRepairApprovalContractSourceRef,
    source_resolution: &ContractRepairApprovalContractSourceResolutionDryRun,
    patch_plan: &ContractRepairApprovalContractPatchPlanPreview,
    patch_apply_dry_run: &ContractRepairApprovalContractPatchApplyDryRun,
    cleanup_phase_dry_run: &ContractRepairApprovalApproveExecutionRunnerRecoveryMarkerCleanupPhaseDryRun,
    cleanup_phase_enablement_dry_run: &ContractRepairApprovalApproveExecutionRunnerRecoveryMarkerCleanupPhaseEnablementDryRun,
    commit_phase_dry_run: &ContractRepairApprovalApproveExecutionRunnerTransactionCommitPhaseDryRun,
    commit_phase_enablement_dry_run: &ContractRepairApprovalApproveExecutionRunnerTransactionCommitPhaseEnablementDryRun,
    disk_application_executor_admitted: bool,
) -> ContractRepairApprovalDurableDiskApplicationExecution {
    let mut execution = ContractRepairApprovalDurableDiskApplicationExecution {
        status: "approve_execution_durable_disk_application_executor_blocked".to_string(),
        executor_name: "approve_execution_durable_disk_application_executor".to_string(),
        executor_admitted: disk_application_executor_admitted,
        recovery_marker_written: false,
        approval_record_persisted: false,
        contract_source_written: false,
        recovery_marker_cleared: false,
        transaction_committed: false,
        rollback_executed: false,
        would_touch_disk: false,
        execution_receipts: Vec::new(),
        rollback_receipts: Vec::new(),
        blocked_by: Vec::new(),
    };

    if !disk_application_executor_admitted {
        push_unique_blocker(
            &mut execution.blocked_by,
            "disk_application_executor_admitted",
        );
        return execution;
    }

    let approval_record_before_execution =
        load_contract_repair_approval_record_from_disk(store_dir, &approval_record.approval_id)
            .await
            .ok()
            .flatten();
    let source_path = contract_repair_approval_contract_source_path(graph_store_dir, source_ref);
    let contract_source_before_execution = crate::runtime_persistence::read_to_string_bounded(
        &source_path,
        crate::runtime_persistence::MAX_BOUNDED_JSON_READ_BYTES,
    )
    .await
    .ok()
    .and_then(|content| serde_json::from_str::<serde_json::Value>(&content).ok());

    let marker_write_result =
        contract_repair_approval_approve_execution_recovery_marker_write_with_gate(
            store_dir,
            marker_write_dry_run,
            idempotency_precheck,
            runner_dry_run,
            true,
        )
        .await;
    if !marker_write_result.would_write_marker {
        push_unique_blocker(&mut execution.blocked_by, "write_recovery_marker");
        for blocker in &marker_write_result.blocked_by {
            push_unique_blocker(&mut execution.blocked_by, blocker);
        }
        execution.status =
            "approve_execution_durable_disk_application_executor_write_recovery_marker_blocked"
                .to_string();
        return execution;
    }
    execution.recovery_marker_written = true;
    execution.would_touch_disk = true;
    execution
        .execution_receipts
        .push("write_recovery_marker:executed".to_string());

    if let Err(error) = persist_contract_repair_approval_record(store_dir, approval_record).await {
        push_unique_blocker(&mut execution.blocked_by, "persist_approval_record");
        execution
            .execution_receipts
            .push(format!("persist_approval_record:failed:{error}"));
        execution.status =
            "approve_execution_durable_disk_application_executor_persist_approval_record_failed"
                .to_string();
        contract_repair_approval_rollback_durable_disk_application_partial_state(
            store_dir,
            graph_store_dir,
            &approval_record.approval_id,
            source_ref,
            approval_record_before_execution.as_ref(),
            contract_source_before_execution.as_ref(),
            &marker_write_dry_run.file_name,
            &mut execution,
            "persist_approval_record",
        )
        .await;
        return execution;
    }
    execution.approval_record_persisted = true;
    execution
        .execution_receipts
        .push("persist_approval_record:executed".to_string());

    let source_write_result = contract_repair_approval_contract_source_write_with_gate(
        graph_store_dir,
        source_ref,
        source_resolution,
        patch_plan,
        patch_apply_dry_run,
        true,
    )
    .await;
    if !source_write_result.would_write_source {
        push_unique_blocker(&mut execution.blocked_by, "write_contract_source");
        for blocker in &source_write_result.blocked_by {
            push_unique_blocker(&mut execution.blocked_by, blocker);
        }
        execution.status =
            "approve_execution_durable_disk_application_executor_write_contract_source_blocked"
                .to_string();
        contract_repair_approval_rollback_durable_disk_application_partial_state(
            store_dir,
            graph_store_dir,
            &approval_record.approval_id,
            source_ref,
            approval_record_before_execution.as_ref(),
            contract_source_before_execution.as_ref(),
            &marker_write_dry_run.file_name,
            &mut execution,
            "write_contract_source",
        )
        .await;
        return execution;
    }
    execution.contract_source_written = true;
    execution
        .execution_receipts
        .push("write_contract_source:executed".to_string());

    let cleanup_result =
        contract_repair_approval_approve_execution_runner_recovery_marker_cleanup_phase_with_gate(
            store_dir,
            cleanup_phase_dry_run,
            cleanup_phase_enablement_dry_run,
            true,
        )
        .await;
    if !cleanup_result.would_clear_recovery_marker {
        push_unique_blocker(&mut execution.blocked_by, "clear_recovery_marker");
        for blocker in &cleanup_result.blocked_by {
            push_unique_blocker(&mut execution.blocked_by, blocker);
        }
        execution.status =
            "approve_execution_durable_disk_application_executor_clear_recovery_marker_blocked"
                .to_string();
        contract_repair_approval_rollback_durable_disk_application_partial_state(
            store_dir,
            graph_store_dir,
            &approval_record.approval_id,
            source_ref,
            approval_record_before_execution.as_ref(),
            contract_source_before_execution.as_ref(),
            &marker_write_dry_run.file_name,
            &mut execution,
            "clear_recovery_marker",
        )
        .await;
        return execution;
    }
    execution.recovery_marker_cleared = true;
    execution
        .execution_receipts
        .push("clear_recovery_marker:executed".to_string());

    let commit_result =
        contract_repair_approval_approve_execution_runner_transaction_commit_phase_with_gate(
            commit_phase_dry_run,
            commit_phase_enablement_dry_run,
            true,
        );
    if !commit_result.would_commit_transaction {
        push_unique_blocker(&mut execution.blocked_by, "commit_transaction");
        for blocker in &commit_result.blocked_by {
            push_unique_blocker(&mut execution.blocked_by, blocker);
        }
        execution.status =
            "approve_execution_durable_disk_application_executor_commit_transaction_blocked"
                .to_string();
        contract_repair_approval_rollback_durable_disk_application_partial_state(
            store_dir,
            graph_store_dir,
            &approval_record.approval_id,
            source_ref,
            approval_record_before_execution.as_ref(),
            contract_source_before_execution.as_ref(),
            &marker_write_dry_run.file_name,
            &mut execution,
            "commit_transaction",
        )
        .await;
        return execution;
    }
    execution.transaction_committed = true;
    execution
        .execution_receipts
        .push("commit_transaction:executed".to_string());
    execution.status = "approve_execution_durable_disk_application_executor_committed".to_string();
    execution
}

fn contract_repair_approval_approve_execution_decision_lock_summary_dry_run(
    action: &str,
    approval_id: &str,
    response_status: &str,
    route_status: &str,
    blocked_reasons: &[String],
    route_status_readiness_dry_run: &ContractRepairApprovalApproveExecutionRunnerRouteStatusReadinessDryRun,
    final_execution_entry_dry_run: &ContractRepairApprovalApproveExecutionFinalExecutionEntryDryRun,
    decision_execution_preflight_requested: bool,
    review_execution_enabled: bool,
    approve_final_execution_enabled: bool,
    approve_runner_success: bool,
    routed_route_success_release_applied: bool,
    durable_writeback_would_touch_disk: bool,
) -> ContractRepairApprovalApproveExecutionDecisionLockSummaryDryRun {
    let route_status_ready = route_status_readiness_dry_run.route_status_ready;
    let final_execution_entry_ready = final_execution_entry_dry_run.final_execution_entry_ready;
    let final_execution_switch_enabled =
        final_execution_entry_ready && approve_final_execution_enabled;
    let final_response_application_success =
        approve_runner_success || routed_route_success_release_applied;
    let final_execution_locked = !(response_status == "review_approve_executed"
        && route_status == "review_approve_executed"
        && route_status_ready
        && final_execution_switch_enabled
        && final_response_application_success);
    let primary_blocked_reason = if final_execution_entry_ready && !approve_final_execution_enabled
    {
        "approve_final_execution_enabled".to_string()
    } else if final_execution_entry_ready
        && final_execution_switch_enabled
        && !final_response_application_success
    {
        "routed_route_success_release_applied".to_string()
    } else {
        blocked_reasons.first().cloned().unwrap_or_else(|| {
            if final_execution_locked {
                "decision_execution_locked".to_string()
            } else {
                "none".to_string()
            }
        })
    };
    let expected_http_status = if !final_execution_locked
        && (review_execution_enabled || final_response_application_success)
    {
        200
    } else {
        route_status_readiness_dry_run.expected_http_status
    };

    ContractRepairApprovalApproveExecutionDecisionLockSummaryDryRun {
        status: if final_execution_locked {
            if decision_execution_preflight_requested {
                "approve_execution_decision_lock_summary_ready_blocked".to_string()
            } else {
                "approve_execution_decision_lock_summary_blocked".to_string()
            }
        } else {
            "approve_execution_decision_lock_summary_ready".to_string()
        },
        action: action.to_string(),
        approval_id: approval_id.to_string(),
        response_status: response_status.to_string(),
        route_status: route_status.to_string(),
        target_response_status: "review_approve_executed".to_string(),
        target_route_status: "review_approve_executed".to_string(),
        expected_http_status,
        decision_execution_preflight_requested,
        review_execution_enabled,
        approve_runner_success,
        routed_route_success_release_applied,
        final_response_application_success,
        route_status_ready,
        final_execution_entry_ready,
        approve_final_execution_enabled,
        final_execution_switch_enabled,
        final_execution_locked,
        primary_blocked_reason,
        blocked_reason_count: blocked_reasons.len(),
        would_execute_decision: !final_execution_locked
            && (review_execution_enabled || final_response_application_success),
        would_mutate_contract: false,
        would_return_http_ok: !final_execution_locked
            && (review_execution_enabled || final_response_application_success),
        would_touch_disk: !final_execution_locked
            && (review_execution_enabled || final_response_application_success)
            && durable_writeback_would_touch_disk,
        inherited_route_status_blocked_gates: route_status_readiness_dry_run.blocked_gates.clone(),
        inherited_final_execution_entry_blocked_gates: final_execution_entry_dry_run
            .blocked_gates
            .clone(),
        inherited_blocked_reasons: blocked_reasons.to_vec(),
    }
}

fn contract_repair_approval_approve_execution_runner_control_readiness_dry_run(
    runner_attempt: &ContractRepairApprovalApproveExecutionRunnerAttempt,
    runner_outcome: &ContractRepairApprovalApproveExecutionRunnerOutcome,
    dispatch_gate: &ContractRepairApprovalApproveExecutionRunnerDispatchGate,
    call_dry_run: &ContractRepairApprovalApproveExecutionRunnerCallDryRun,
    body_dry_run: &ContractRepairApprovalApproveExecutionRunnerCallBodyDryRun,
    phase_sequence_dry_run: &ContractRepairApprovalApproveExecutionRunnerBodyPhaseSequenceDryRun,
) -> ContractRepairApprovalApproveExecutionRunnerControlReadinessDryRun {
    let runner_attempt_ready = runner_attempt.runner_attempt_ready;
    let runner_execution_ready = runner_outcome.runner_execution_ready;
    let dispatch_ready = dispatch_gate.dispatch_ready;
    let call_ready = call_dry_run.call_ready;
    let body_ready = body_dry_run.body_ready;
    let phases_ready = phase_sequence_dry_run.phases_ready;
    let runner_control_ready = runner_attempt_ready
        && runner_execution_ready
        && dispatch_ready
        && call_ready
        && body_ready
        && phases_ready;
    let gate_states = [
        (
            "approve_execution_runner_attempt_ready",
            runner_attempt_ready,
        ),
        (
            "approve_execution_runner_execution_ready",
            runner_execution_ready,
        ),
        ("approve_execution_runner_dispatch_ready", dispatch_ready),
        ("approve_execution_runner_call_ready", call_ready),
        ("approve_execution_runner_body_ready", body_ready),
        ("approve_execution_runner_phases_ready", phases_ready),
    ];
    let required_gates = gate_states
        .iter()
        .map(|(gate, _)| (*gate).to_string())
        .collect::<Vec<_>>();
    let mut passed_gates = Vec::new();
    let mut blocked_gates = Vec::new();
    for (gate, passed) in gate_states {
        if passed {
            passed_gates.push(gate.to_string());
        } else {
            blocked_gates.push(gate.to_string());
        }
    }

    ContractRepairApprovalApproveExecutionRunnerControlReadinessDryRun {
        status: if runner_control_ready {
            "approve_execution_runner_control_readiness_ready".to_string()
        } else if runner_attempt.action == "approve" {
            "approve_execution_runner_control_readiness_ready_blocked".to_string()
        } else {
            "approve_execution_runner_control_readiness_blocked".to_string()
        },
        action: runner_attempt.action.clone(),
        approval_id: runner_attempt.approval_id.clone(),
        gate_name: "approve_execution_runner_control_ready".to_string(),
        runner_attempt_ready,
        runner_execution_ready,
        dispatch_ready,
        call_ready,
        body_ready,
        phases_ready,
        runner_control_ready,
        would_unblock_activation_control: false,
        would_activate_runner: false,
        would_return_success: false,
        would_touch_disk: false,
        inherited_runner_attempt_blockers: runner_attempt.blocked_by.clone(),
        inherited_runner_outcome_blockers: runner_outcome.blocked_by.clone(),
        inherited_dispatch_blockers: dispatch_gate.blocked_gates.clone(),
        inherited_call_blockers: call_dry_run.blocked_by.clone(),
        inherited_body_blockers: body_dry_run.blocked_by.clone(),
        inherited_phase_sequence_blockers: phase_sequence_dry_run.blocked_by.clone(),
        required_gates,
        passed_gates,
        blocked_gates,
    }
}

fn target_review_state_for_action(action: &str) -> RuntimeApprovalReviewState {
    match action {
        "claim" => RuntimeApprovalReviewState::UnderReview,
        "approve" => RuntimeApprovalReviewState::Approved,
        "reject" => RuntimeApprovalReviewState::Rejected,
        _ => RuntimeApprovalReviewState::Pending,
    }
}

fn contract_repair_approval_persistence_plan_preview(
    preview: &ContractRepairApprovalRecordPreview,
) -> ContractRepairApprovalPersistencePlanPreview {
    ContractRepairApprovalPersistencePlanPreview {
        status: "persistence_plan_preview_only".to_string(),
        persistence_enabled: false,
        would_write_record: false,
        store_kind: "contract_repair_approval_records".to_string(),
        record_kind: "contract_repair_approval".to_string(),
        record_key: preview.approval_id.clone(),
        idempotency_key: preview.idempotency_key.clone(),
        record_source_kind: "transient_preview_cache".to_string(),
        blocked_by: vec!["approval_persistence_enabled".to_string()],
    }
}

fn contract_repair_approval_persistence_path_preview(
    persistence_plan: &ContractRepairApprovalPersistencePlanPreview,
) -> ContractRepairApprovalPersistencePathPreview {
    let path_segment = sanitize_storage_path_segment(&persistence_plan.record_key);
    ContractRepairApprovalPersistencePathPreview {
        status: "persistence_path_preview_only".to_string(),
        store_kind: persistence_plan.store_kind.clone(),
        record_key: persistence_plan.record_key.clone(),
        file_name: format!("{path_segment}.json"),
        path_segment,
        atomic_write_required: true,
        would_touch_disk: false,
        blocked_by: vec![
            "approval_persistence_enabled".to_string(),
            "contract_repair_approval_store_ready".to_string(),
        ],
    }
}

fn contract_repair_approval_record_snapshot_preview(
    preview: &ContractRepairApprovalRecordPreview,
    action: &str,
    reviewer_id: &str,
    reason: &str,
) -> ContractRepairApprovalRecordSnapshotPreview {
    ContractRepairApprovalRecordSnapshotPreview {
        status: "record_snapshot_preview_only".to_string(),
        approval_id: preview.approval_id.clone(),
        record_kind: "contract_repair_approval".to_string(),
        target_path: preview.target_path.clone(),
        target_kind: preview.target_kind.clone(),
        changed_fields: preview.changed_fields.clone(),
        patch_payload: preview.patch_payload.clone(),
        contract_source_ref: preview.contract_source_ref.clone(),
        review_state: target_review_state_for_action(action),
        reviewer_id: reviewer_id.to_string(),
        review_reason: reason.to_string(),
        idempotency_key: preview.idempotency_key.clone(),
        persistence_enabled: false,
        would_write_record: false,
    }
}

async fn contract_repair_approval_storage_readiness_gate(
    store_dir: &FsPath,
    persistence_plan: &ContractRepairApprovalPersistencePlanPreview,
    path_preview: &ContractRepairApprovalPersistencePathPreview,
    snapshot: &ContractRepairApprovalRecordSnapshotPreview,
) -> ContractRepairApprovalStorageReadinessGate {
    let store_ready = fs::metadata(store_dir)
        .await
        .map(|metadata| metadata.is_dir())
        .unwrap_or(false);
    let mut ready_gates = vec![
        "record_schema_preview_ready".to_string(),
        "idempotency_key_ready".to_string(),
        "persistence_path_preview_ready".to_string(),
        "record_snapshot_preview_ready".to_string(),
    ];
    if store_ready {
        ready_gates.push("contract_repair_approval_store_ready".to_string());
    }
    let mut blocked_gates = vec!["approval_persistence_enabled".to_string()];
    if !store_ready {
        blocked_gates.push("contract_repair_approval_store_ready".to_string());
    }
    ContractRepairApprovalStorageReadinessGate {
        status: "blocked".to_string(),
        persistence_enabled: false,
        store_ready,
        schema_ready: snapshot.record_kind == "contract_repair_approval"
            && !snapshot.target_path.trim().is_empty()
            && !snapshot.target_kind.trim().is_empty()
            && !snapshot.changed_fields.is_empty(),
        idempotency_ready: !persistence_plan.idempotency_key.trim().is_empty()
            && persistence_plan.idempotency_key.starts_with("sha256:"),
        snapshot_ready: snapshot.status == "record_snapshot_preview_only"
            && !snapshot.approval_id.trim().is_empty()
            && snapshot.approval_id == persistence_plan.record_key
            && path_preview.record_key == persistence_plan.record_key
            && !path_preview.path_segment.trim().is_empty()
            && path_preview.file_name.ends_with(".json"),
        ready_gates,
        blocked_gates,
    }
}

fn contract_repair_approval_storage_dry_run_preview(
    persistence_plan: &ContractRepairApprovalPersistencePlanPreview,
    readiness_gate: &ContractRepairApprovalStorageReadinessGate,
) -> ContractRepairApprovalStorageDryRunPreview {
    ContractRepairApprovalStorageDryRunPreview {
        status: "dry_run_blocked".to_string(),
        adapter_kind: "contract_repair_approval_store_adapter".to_string(),
        store_kind: persistence_plan.store_kind.clone(),
        record_key: persistence_plan.record_key.clone(),
        would_write: false,
        accepted_by_adapter: readiness_gate.schema_ready
            && readiness_gate.idempotency_ready
            && readiness_gate.snapshot_ready,
        readiness_status: readiness_gate.status.clone(),
        blocked_by: readiness_gate.blocked_gates.clone(),
    }
}

async fn contract_repair_approval_idempotency_precheck(
    store_dir: &FsPath,
    persistence_plan: &ContractRepairApprovalPersistencePlanPreview,
) -> Result<ContractRepairApprovalIdempotencyPrecheck, (StatusCode, String)> {
    let existing_record =
        load_contract_repair_approval_record_from_disk(store_dir, &persistence_plan.record_key)
            .await?;
    let existing_record_found = existing_record.is_some();
    let conflict_detected = existing_record
        .as_ref()
        .is_some_and(|record| record.idempotency_key != persistence_plan.idempotency_key);
    Ok(ContractRepairApprovalIdempotencyPrecheck {
        status: "precheck_checked_blocked".to_string(),
        idempotency_key: persistence_plan.idempotency_key.clone(),
        candidate_record_key: persistence_plan.record_key.clone(),
        store_lookup_enabled: true,
        existing_record_found,
        conflict_detected,
        safe_to_write: false,
        blocked_by: vec!["approval_persistence_enabled".to_string()],
    })
}

#[derive(Debug, Deserialize)]
struct ContractRepairReviewerGrantFile {
    #[serde(default)]
    policy_version: String,
    #[serde(default)]
    grants: Vec<ContractRepairReviewerGrant>,
}

#[derive(Debug, Deserialize)]
struct ContractRepairReviewerGrant {
    subject: String,
    role: String,
}

async fn contract_repair_approval_reviewer_authorization_precheck(
    grants_path: &FsPath,
    user_id: &auth::UserId,
    reviewer_id: &str,
) -> ContractRepairApprovalReviewerAuthorizationPrecheck {
    let identity_format_valid = reviewer_identity_format_valid(reviewer_id);
    let auth_subject = format!("user:{}", user_id.0);
    let identity_matches_auth_subject = identity_format_valid && reviewer_id.trim() == auth_subject;
    let (grant_source, role_granted, grant_source_valid) =
        contract_repair_reviewer_grant_lookup(grants_path, &auth_subject).await;
    let authorized = identity_format_valid && identity_matches_auth_subject && role_granted;
    let mut blocked_by = Vec::new();
    if !role_granted {
        blocked_by.push("formal_reviewer_role_grant_missing".to_string());
    }
    if !grant_source_valid {
        blocked_by.push("formal_reviewer_role_grant_source_invalid".to_string());
    }
    if !identity_format_valid {
        blocked_by.push("reviewer_identity_format_valid".to_string());
    }
    if !identity_matches_auth_subject {
        blocked_by.push("reviewer_identity_matches_auth_subject".to_string());
    }
    ContractRepairApprovalReviewerAuthorizationPrecheck {
        status: if authorized {
            "authorization_precheck_authorized".to_string()
        } else {
            "authorization_precheck_denied".to_string()
        },
        policy_version: CONTRACT_REPAIR_REVIEWER_POLICY_VERSION.to_string(),
        required_role: CONTRACT_REPAIR_REVIEWER_REQUIRED_ROLE.to_string(),
        grant_source,
        reviewer_id: reviewer_id.to_string(),
        auth_subject,
        identity_format_valid,
        identity_matches_auth_subject,
        role_policy_available: true,
        authorized,
        blocked_by,
    }
}

async fn contract_repair_reviewer_grant_lookup(
    grants_path: &FsPath,
    auth_subject: &str,
) -> (String, bool, bool) {
    let content = match fs::read_to_string(grants_path).await {
        Ok(content) => content,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return ("not_configured".to_string(), false, true);
        }
        Err(_) => return ("file_unreadable".to_string(), false, false),
    };
    let grant_file = match serde_json::from_str::<ContractRepairReviewerGrantFile>(&content) {
        Ok(grant_file) => grant_file,
        Err(_) => return ("file_invalid".to_string(), false, false),
    };
    if grant_file.policy_version.trim() != CONTRACT_REPAIR_REVIEWER_POLICY_VERSION {
        return ("file_policy_version_mismatch".to_string(), false, false);
    }
    let role_granted = grant_file.grants.iter().any(|grant| {
        grant.subject.trim() == auth_subject
            && grant.role.trim() == CONTRACT_REPAIR_REVIEWER_REQUIRED_ROLE
    });
    let file_name = grants_path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("contract-repair-reviewer-grants.json");
    (format!("file:{file_name}"), role_granted, true)
}

fn contract_repair_approval_review_transition_dry_run(
    preview: &ContractRepairApprovalRecordPreview,
    snapshot: &ContractRepairApprovalRecordSnapshotPreview,
    gate: &ContractRepairApprovalReviewExecutionGate,
    transition_enabled: bool,
) -> ContractRepairApprovalReviewTransitionDryRun {
    let local_blockers: Vec<String> = gate
        .blocked_gates
        .iter()
        .filter(|gate| {
            !matches!(
                gate.as_str(),
                "review_workflow_enabled"
                    | "approval_persistence_enabled"
                    | "lifecycle_event_emission_enabled"
                    | "contract_mutation_api_enabled"
            )
        })
        .cloned()
        .collect();
    let transition_ready = local_blockers.is_empty();
    let transition_executed = transition_enabled && transition_ready;
    ContractRepairApprovalReviewTransitionDryRun {
        status: if transition_executed {
            "transition_executed".to_string()
        } else if transition_ready {
            "transition_dry_run_ready_blocked".to_string()
        } else {
            "transition_dry_run_blocked".to_string()
        },
        approval_id: preview.approval_id.clone(),
        from_review_state: preview.review_state,
        target_review_state: snapshot.review_state,
        reviewer_id: snapshot.reviewer_id.clone(),
        reason_code: "review_transition_preview_only".to_string(),
        sequence_no_preview: (preview.lifecycle.len() as u64).saturating_add(1),
        transition_ready,
        would_transition: transition_executed,
        blocked_by: gate.blocked_gates.clone(),
    }
}

fn contract_repair_approval_record_write_dry_run(
    persistence_plan: &ContractRepairApprovalPersistencePlanPreview,
    path_preview: &ContractRepairApprovalPersistencePathPreview,
    storage_readiness_gate: &ContractRepairApprovalStorageReadinessGate,
    idempotency_precheck: &ContractRepairApprovalIdempotencyPrecheck,
    transition_dry_run: &ContractRepairApprovalReviewTransitionDryRun,
    write_enabled: bool,
) -> ContractRepairApprovalRecordWriteDryRun {
    let idempotency_precheck_passed =
        idempotency_precheck.store_lookup_enabled && !idempotency_precheck.conflict_detected;
    let write_ready = transition_dry_run.transition_ready
        && storage_readiness_gate.store_ready
        && storage_readiness_gate.schema_ready
        && storage_readiness_gate.idempotency_ready
        && storage_readiness_gate.snapshot_ready
        && idempotency_precheck_passed;
    let write_executed = write_enabled && write_ready;
    let mut blocked_by = vec!["approval_persistence_enabled".to_string()];
    if write_executed {
        blocked_by.clear();
    }
    if !transition_dry_run.transition_ready {
        blocked_by.push("review_transition_ready".to_string());
    }
    if !storage_readiness_gate.store_ready {
        blocked_by.push("contract_repair_approval_store_ready".to_string());
    }
    if !storage_readiness_gate.schema_ready {
        blocked_by.push("record_schema_preview_ready".to_string());
    }
    if !storage_readiness_gate.idempotency_ready {
        blocked_by.push("idempotency_key_ready".to_string());
    }
    if !storage_readiness_gate.snapshot_ready {
        blocked_by.push("record_snapshot_preview_ready".to_string());
    }
    if !idempotency_precheck_passed {
        blocked_by.push("idempotency_precheck_passed".to_string());
    }
    ContractRepairApprovalRecordWriteDryRun {
        status: if write_executed {
            "record_write_executed".to_string()
        } else if write_ready {
            "record_write_dry_run_ready_blocked".to_string()
        } else {
            "record_write_dry_run_blocked".to_string()
        },
        adapter_kind: "contract_repair_approval_record_writer".to_string(),
        store_kind: persistence_plan.store_kind.clone(),
        record_key: persistence_plan.record_key.clone(),
        file_name: path_preview.file_name.clone(),
        transition_ready: transition_dry_run.transition_ready,
        storage_ready: storage_readiness_gate.store_ready,
        schema_ready: storage_readiness_gate.schema_ready,
        idempotency_ready: storage_readiness_gate.idempotency_ready,
        snapshot_ready: storage_readiness_gate.snapshot_ready,
        idempotency_precheck_passed,
        write_ready,
        would_write: write_executed,
        blocked_by,
    }
}

fn contract_repair_approval_lifecycle_event_dry_run(
    snapshot: &ContractRepairApprovalRecordSnapshotPreview,
    action: &str,
    transition_dry_run: &ContractRepairApprovalReviewTransitionDryRun,
    emission_enabled: bool,
) -> ContractRepairApprovalLifecycleEventDryRun {
    let event_id = format!("contract-repair-review-{action}:{}", snapshot.approval_id);
    let event_kind = "contract_repair_approval_review".to_string();
    let sequence_no = transition_dry_run.sequence_no_preview;
    let event_payload_ready = !event_id.trim().is_empty()
        && !event_kind.trim().is_empty()
        && !snapshot.reviewer_id.trim().is_empty()
        && !snapshot.approval_id.trim().is_empty()
        && sequence_no > 0;
    let emission_ready = transition_dry_run.transition_ready && event_payload_ready;
    let emission_executed =
        emission_enabled && emission_ready && transition_dry_run.would_transition;
    let mut blocked_by = if transition_dry_run.would_transition {
        vec!["lifecycle_event_emission_enabled".to_string()]
    } else if transition_dry_run.transition_ready {
        let lifecycle_blockers: Vec<String> = transition_dry_run
            .blocked_by
            .iter()
            .filter(|gate| gate.as_str() == "lifecycle_event_emission_enabled")
            .cloned()
            .collect();
        if lifecycle_blockers.is_empty() {
            vec![
                "review_workflow_enabled".to_string(),
                "approval_persistence_enabled".to_string(),
            ]
        } else {
            lifecycle_blockers
        }
    } else {
        transition_dry_run.blocked_by.clone()
    };
    if emission_executed {
        blocked_by.clear();
    }
    ContractRepairApprovalLifecycleEventDryRun {
        status: if emission_executed {
            "lifecycle_emitted".to_string()
        } else if transition_dry_run.would_transition {
            "lifecycle_emit_blocked".to_string()
        } else if emission_ready {
            "lifecycle_dry_run_emission_ready_blocked".to_string()
        } else if transition_dry_run.transition_ready {
            "lifecycle_dry_run_ready_blocked".to_string()
        } else {
            "lifecycle_dry_run_blocked".to_string()
        },
        event_id,
        event_kind,
        target_review_state: snapshot.review_state,
        actor_id: snapshot.reviewer_id.clone(),
        reason_code: "review_transition_preview_only".to_string(),
        sequence_no,
        transition_ready: transition_dry_run.transition_ready,
        event_payload_ready,
        emission_ready,
        would_emit: emission_executed,
        blocked_by,
    }
}

fn contract_repair_approval_lifecycle_entry_append_dry_run(
    lifecycle_event_dry_run: &ContractRepairApprovalLifecycleEventDryRun,
    append_enabled: bool,
) -> ContractRepairApprovalLifecycleEntryAppendDryRun {
    let entry_ready = lifecycle_event_dry_run.event_payload_ready
        && !lifecycle_event_dry_run.event_id.trim().is_empty()
        && lifecycle_event_dry_run.sequence_no > 0;
    let append_ready = entry_ready && lifecycle_event_dry_run.emission_ready;
    let append_executed = append_enabled && append_ready && lifecycle_event_dry_run.would_emit;
    let mut blocked_by = Vec::new();
    if !entry_ready {
        blocked_by.push("lifecycle_entry_payload_ready".to_string());
    }
    if !lifecycle_event_dry_run.emission_ready {
        blocked_by.push("lifecycle_event_emission_ready".to_string());
    }
    if !lifecycle_event_dry_run.would_emit {
        blocked_by.push("lifecycle_event_emission_enabled".to_string());
    }
    if append_executed {
        blocked_by.clear();
    }

    ContractRepairApprovalLifecycleEntryAppendDryRun {
        status: if append_executed {
            "lifecycle_entry_appended".to_string()
        } else if append_ready {
            "lifecycle_entry_append_ready_blocked".to_string()
        } else {
            "lifecycle_entry_append_blocked".to_string()
        },
        event_id: lifecycle_event_dry_run.event_id.clone(),
        review_state: lifecycle_event_dry_run.target_review_state,
        sequence_no: lifecycle_event_dry_run.sequence_no,
        entry_ready,
        emission_ready: lifecycle_event_dry_run.emission_ready,
        append_ready,
        would_append: append_executed,
        blocked_by,
    }
}

fn contract_repair_approval_lifecycle_event_with_gate(
    lifecycle_event_dry_run: &ContractRepairApprovalLifecycleEventDryRun,
    lifecycle_effects_enabled: bool,
) -> ContractRepairApprovalLifecycleEventDryRun {
    let mut result = lifecycle_event_dry_run.clone();
    if !lifecycle_effects_enabled || lifecycle_event_dry_run.would_emit {
        return result;
    }

    let non_enablement_blocked = lifecycle_event_dry_run
        .blocked_by
        .iter()
        .any(|blocker| blocker != "lifecycle_event_emission_enabled");
    if !lifecycle_event_dry_run.emission_ready || non_enablement_blocked {
        if !lifecycle_event_dry_run.emission_ready {
            push_unique_blocker(&mut result.blocked_by, "lifecycle_event_emission_ready");
        }
        return result;
    }

    result.status = "lifecycle_emitted".to_string();
    result.would_emit = true;
    result.blocked_by.clear();
    result
}

fn contract_repair_approval_lifecycle_entry_append_with_gate(
    lifecycle_event_dry_run: &ContractRepairApprovalLifecycleEventDryRun,
    lifecycle_entry_append_dry_run: &ContractRepairApprovalLifecycleEntryAppendDryRun,
    lifecycle_effects_enabled: bool,
) -> ContractRepairApprovalLifecycleEntryAppendDryRun {
    let mut result = lifecycle_entry_append_dry_run.clone();
    if !lifecycle_effects_enabled || lifecycle_entry_append_dry_run.would_append {
        return result;
    }

    if !lifecycle_event_dry_run.would_emit {
        push_unique_blocker(&mut result.blocked_by, "lifecycle_event_emission_enabled");
        return result;
    }
    if !lifecycle_entry_append_dry_run.append_ready {
        push_unique_blocker(&mut result.blocked_by, "lifecycle_entry_append_ready");
        return result;
    }

    result.status = "lifecycle_entry_appended".to_string();
    result.would_append = true;
    result.blocked_by.clear();
    result
}

fn contract_repair_approval_lifecycle_emission_enablement_gate(
    lifecycle_event_dry_run: &ContractRepairApprovalLifecycleEventDryRun,
    lifecycle_entry_append_dry_run: &ContractRepairApprovalLifecycleEntryAppendDryRun,
) -> ContractRepairApprovalLifecycleEmissionEnablementGate {
    let transition_ready = lifecycle_event_dry_run.transition_ready;
    let event_payload_ready = lifecycle_event_dry_run.event_payload_ready;
    let emission_ready = lifecycle_event_dry_run.emission_ready;
    let entry_append_ready = lifecycle_entry_append_dry_run.append_ready;
    let lifecycle_emission_plan_ready =
        transition_ready && event_payload_ready && emission_ready && entry_append_ready;
    let lifecycle_event_emission_enabled = lifecycle_event_dry_run.would_emit;
    let lifecycle_entry_append_enabled = lifecycle_entry_append_dry_run.would_append;
    let lifecycle_effects_ready = lifecycle_emission_plan_ready
        && lifecycle_event_emission_enabled
        && lifecycle_entry_append_enabled;
    let mut passed_gates = Vec::new();
    if transition_ready {
        passed_gates.push("approved_review_transition_ready".to_string());
    }
    if event_payload_ready {
        passed_gates.push("lifecycle_event_payload_ready".to_string());
    }
    if emission_ready {
        passed_gates.push("lifecycle_event_emission_ready".to_string());
    }
    if entry_append_ready {
        passed_gates.push("lifecycle_entry_append_ready".to_string());
    }
    if lifecycle_event_emission_enabled {
        passed_gates.push("lifecycle_event_emission_enabled".to_string());
    }
    if lifecycle_entry_append_enabled {
        passed_gates.push("lifecycle_entry_append_enabled".to_string());
    }
    let mut blocked_gates = Vec::new();
    if !transition_ready {
        blocked_gates.push("approved_review_transition_ready".to_string());
    }
    if !event_payload_ready {
        blocked_gates.push("lifecycle_event_payload_ready".to_string());
    }
    if !emission_ready {
        blocked_gates.push("lifecycle_event_emission_ready".to_string());
    }
    if !entry_append_ready {
        blocked_gates.push("lifecycle_entry_append_ready".to_string());
    }
    if !lifecycle_event_emission_enabled {
        blocked_gates.push("lifecycle_event_emission_enabled".to_string());
    }
    if !lifecycle_entry_append_enabled {
        blocked_gates.push("lifecycle_entry_append_enabled".to_string());
    }

    ContractRepairApprovalLifecycleEmissionEnablementGate {
        status: if lifecycle_effects_ready {
            "lifecycle_emission_enablement_ready".to_string()
        } else if lifecycle_emission_plan_ready {
            "lifecycle_emission_enablement_ready_blocked".to_string()
        } else {
            "lifecycle_emission_enablement_blocked".to_string()
        },
        event_id: lifecycle_event_dry_run.event_id.clone(),
        transition_ready,
        event_payload_ready,
        emission_ready,
        entry_append_ready,
        lifecycle_emission_plan_ready,
        lifecycle_event_emission_enabled,
        lifecycle_entry_append_enabled,
        lifecycle_effects_ready,
        would_emit: lifecycle_event_dry_run.would_emit,
        would_append: lifecycle_entry_append_dry_run.would_append,
        would_touch_lifecycle_log: lifecycle_event_dry_run.would_emit
            || lifecycle_entry_append_dry_run.would_append,
        required_gates: vec![
            "approved_review_transition_ready".to_string(),
            "lifecycle_event_payload_ready".to_string(),
            "lifecycle_event_emission_ready".to_string(),
            "lifecycle_entry_append_ready".to_string(),
            "lifecycle_event_emission_enabled".to_string(),
            "lifecycle_entry_append_enabled".to_string(),
        ],
        passed_gates,
        blocked_gates,
    }
}

fn contract_repair_approval_approve_execution_lifecycle_effects_readiness_dry_run(
    action: &str,
    approval_id: &str,
    lifecycle_enablement_gate: &ContractRepairApprovalLifecycleEmissionEnablementGate,
) -> ContractRepairApprovalApproveExecutionLifecycleEffectsReadinessDryRun {
    let transition_ready = lifecycle_enablement_gate.transition_ready;
    let event_payload_ready = lifecycle_enablement_gate.event_payload_ready;
    let emission_ready = lifecycle_enablement_gate.emission_ready;
    let entry_append_ready = lifecycle_enablement_gate.entry_append_ready;
    let lifecycle_emission_plan_ready = lifecycle_enablement_gate.lifecycle_emission_plan_ready;
    let lifecycle_event_emission_enabled =
        lifecycle_enablement_gate.lifecycle_event_emission_enabled;
    let lifecycle_entry_append_enabled = lifecycle_enablement_gate.lifecycle_entry_append_enabled;
    let lifecycle_effects_ready = lifecycle_enablement_gate.lifecycle_effects_ready;
    let required_gates = vec![
        "approved_review_transition_ready".to_string(),
        "lifecycle_event_payload_ready".to_string(),
        "lifecycle_event_emission_ready".to_string(),
        "lifecycle_entry_append_ready".to_string(),
        "lifecycle_event_emission_enabled".to_string(),
        "lifecycle_entry_append_enabled".to_string(),
    ];
    let gate_states = [
        ("approved_review_transition_ready", transition_ready),
        ("lifecycle_event_payload_ready", event_payload_ready),
        ("lifecycle_event_emission_ready", emission_ready),
        ("lifecycle_entry_append_ready", entry_append_ready),
        (
            "lifecycle_event_emission_enabled",
            lifecycle_event_emission_enabled,
        ),
        (
            "lifecycle_entry_append_enabled",
            lifecycle_entry_append_enabled,
        ),
    ];
    let mut passed_gates = Vec::new();
    let mut blocked_gates = Vec::new();
    for (gate, passed) in gate_states {
        if passed {
            passed_gates.push(gate.to_string());
        } else {
            blocked_gates.push(gate.to_string());
        }
    }

    ContractRepairApprovalApproveExecutionLifecycleEffectsReadinessDryRun {
        status: if lifecycle_effects_ready {
            "approve_execution_lifecycle_effects_readiness_ready".to_string()
        } else if action == "approve" && lifecycle_emission_plan_ready {
            "approve_execution_lifecycle_effects_readiness_ready_blocked".to_string()
        } else {
            "approve_execution_lifecycle_effects_readiness_blocked".to_string()
        },
        action: action.to_string(),
        approval_id: approval_id.to_string(),
        gate_name: "lifecycle_effects_ready".to_string(),
        event_id: lifecycle_enablement_gate.event_id.clone(),
        transition_ready,
        event_payload_ready,
        emission_ready,
        entry_append_ready,
        lifecycle_emission_plan_ready,
        lifecycle_event_emission_enabled,
        lifecycle_entry_append_enabled,
        lifecycle_effects_ready,
        would_emit: false,
        would_append: false,
        would_touch_lifecycle_log: false,
        would_unblock_atomic_side_effects: false,
        inherited_lifecycle_emission_blocked_gates: lifecycle_enablement_gate.blocked_gates.clone(),
        required_gates,
        passed_gates,
        blocked_gates,
    }
}

fn contract_repair_approval_approve_execution_lifecycle_event_emission_enablement_dry_run(
    action: &str,
    approval_id: &str,
    lifecycle_enablement_gate: &ContractRepairApprovalLifecycleEmissionEnablementGate,
) -> ContractRepairApprovalApproveExecutionLifecycleEventEmissionEnablementDryRun {
    let transition_ready = lifecycle_enablement_gate.transition_ready;
    let event_payload_ready = lifecycle_enablement_gate.event_payload_ready;
    let emission_ready = lifecycle_enablement_gate.emission_ready;
    let entry_append_ready = lifecycle_enablement_gate.entry_append_ready;
    let lifecycle_emission_plan_ready = lifecycle_enablement_gate.lifecycle_emission_plan_ready;
    let lifecycle_event_emission_enabled =
        lifecycle_enablement_gate.lifecycle_event_emission_enabled;
    let lifecycle_event_emission_enablement_ready =
        lifecycle_emission_plan_ready && lifecycle_event_emission_enabled;
    let required_gates = vec![
        "lifecycle_emission_plan_ready".to_string(),
        "lifecycle_event_emission_enabled".to_string(),
    ];
    let gate_states = [
        (
            "lifecycle_emission_plan_ready",
            lifecycle_emission_plan_ready,
        ),
        (
            "lifecycle_event_emission_enabled",
            lifecycle_event_emission_enabled,
        ),
    ];
    let mut passed_gates = Vec::new();
    let mut blocked_gates = Vec::new();
    for (gate, passed) in gate_states {
        if passed {
            passed_gates.push(gate.to_string());
        } else {
            blocked_gates.push(gate.to_string());
        }
    }

    ContractRepairApprovalApproveExecutionLifecycleEventEmissionEnablementDryRun {
        status: if lifecycle_event_emission_enablement_ready {
            "approve_execution_lifecycle_event_emission_enablement_ready".to_string()
        } else if action == "approve" && lifecycle_emission_plan_ready {
            "approve_execution_lifecycle_event_emission_enablement_ready_blocked".to_string()
        } else {
            "approve_execution_lifecycle_event_emission_enablement_blocked".to_string()
        },
        action: action.to_string(),
        approval_id: approval_id.to_string(),
        switch_name: "lifecycle_event_emission_enabled".to_string(),
        event_id: lifecycle_enablement_gate.event_id.clone(),
        transition_ready,
        event_payload_ready,
        emission_ready,
        entry_append_ready,
        lifecycle_emission_plan_ready,
        lifecycle_event_emission_enabled,
        lifecycle_event_emission_enablement_ready,
        would_enable_lifecycle_event_emission: false,
        would_emit: false,
        would_append: false,
        would_touch_lifecycle_log: false,
        would_unblock_lifecycle_effects: false,
        inherited_lifecycle_emission_blocked_gates: lifecycle_enablement_gate.blocked_gates.clone(),
        required_gates,
        passed_gates,
        blocked_gates,
    }
}

fn contract_repair_approval_approve_execution_lifecycle_entry_append_enablement_dry_run(
    action: &str,
    approval_id: &str,
    lifecycle_enablement_gate: &ContractRepairApprovalLifecycleEmissionEnablementGate,
) -> ContractRepairApprovalApproveExecutionLifecycleEntryAppendEnablementDryRun {
    let transition_ready = lifecycle_enablement_gate.transition_ready;
    let event_payload_ready = lifecycle_enablement_gate.event_payload_ready;
    let emission_ready = lifecycle_enablement_gate.emission_ready;
    let entry_append_ready = lifecycle_enablement_gate.entry_append_ready;
    let lifecycle_emission_plan_ready = lifecycle_enablement_gate.lifecycle_emission_plan_ready;
    let lifecycle_entry_append_enabled = lifecycle_enablement_gate.lifecycle_entry_append_enabled;
    let lifecycle_entry_append_enablement_ready =
        lifecycle_emission_plan_ready && lifecycle_entry_append_enabled;
    let required_gates = vec![
        "lifecycle_emission_plan_ready".to_string(),
        "lifecycle_entry_append_enabled".to_string(),
    ];
    let gate_states = [
        (
            "lifecycle_emission_plan_ready",
            lifecycle_emission_plan_ready,
        ),
        (
            "lifecycle_entry_append_enabled",
            lifecycle_entry_append_enabled,
        ),
    ];
    let mut passed_gates = Vec::new();
    let mut blocked_gates = Vec::new();
    for (gate, passed) in gate_states {
        if passed {
            passed_gates.push(gate.to_string());
        } else {
            blocked_gates.push(gate.to_string());
        }
    }

    ContractRepairApprovalApproveExecutionLifecycleEntryAppendEnablementDryRun {
        status: if lifecycle_entry_append_enablement_ready {
            "approve_execution_lifecycle_entry_append_enablement_ready".to_string()
        } else if action == "approve" && lifecycle_emission_plan_ready {
            "approve_execution_lifecycle_entry_append_enablement_ready_blocked".to_string()
        } else {
            "approve_execution_lifecycle_entry_append_enablement_blocked".to_string()
        },
        action: action.to_string(),
        approval_id: approval_id.to_string(),
        switch_name: "lifecycle_entry_append_enabled".to_string(),
        event_id: lifecycle_enablement_gate.event_id.clone(),
        transition_ready,
        event_payload_ready,
        emission_ready,
        entry_append_ready,
        lifecycle_emission_plan_ready,
        lifecycle_entry_append_enabled,
        lifecycle_entry_append_enablement_ready,
        would_enable_lifecycle_entry_append: false,
        would_emit: false,
        would_append: false,
        would_touch_lifecycle_log: false,
        would_unblock_lifecycle_effects: false,
        inherited_lifecycle_emission_blocked_gates: lifecycle_enablement_gate.blocked_gates.clone(),
        required_gates,
        passed_gates,
        blocked_gates,
    }
}

fn contract_repair_approval_lifecycle_entry_preview(
    append_dry_run: &ContractRepairApprovalLifecycleEntryAppendDryRun,
) -> ContractRepairApprovalLifecycleEntryPreview {
    ContractRepairApprovalLifecycleEntryPreview {
        event_id: append_dry_run.event_id.clone(),
        review_state: append_dry_run.review_state,
        sequence_no: append_dry_run.sequence_no,
    }
}

fn contract_repair_approval_contract_writeback_dry_run(
    snapshot: &ContractRepairApprovalRecordSnapshotPreview,
    action: &str,
    transition_dry_run: &ContractRepairApprovalReviewTransitionDryRun,
    lifecycle_entry_append_dry_run: &ContractRepairApprovalLifecycleEntryAppendDryRun,
    source_resolution_dry_run: &ContractRepairApprovalContractSourceResolutionDryRun,
    patch_plan: ContractRepairApprovalContractPatchPlanPreview,
    patch_apply_dry_run: &ContractRepairApprovalContractPatchApplyDryRun,
    source_write_dry_run: &ContractRepairApprovalContractSourceWriteDryRun,
) -> ContractRepairApprovalContractWritebackDryRun {
    let patch_ready =
        !snapshot.target_path.trim().is_empty() && !snapshot.changed_fields.is_empty();
    let missing_patch_fields = snapshot
        .changed_fields
        .iter()
        .filter(|field_name| !snapshot.patch_payload.contains_key(field_name.as_str()))
        .cloned()
        .collect::<Vec<_>>();
    let patch_payload_ready = missing_patch_fields.is_empty()
        && snapshot
            .changed_fields
            .iter()
            .all(|field_name| snapshot.patch_payload.contains_key(field_name.as_str()));
    let missing_contract_source_fields =
        contract_source_ref_missing_fields(&snapshot.contract_source_ref);
    let contract_source_ready = missing_contract_source_fields.is_empty();
    let eligible_after_approval = action == "approve";
    let lifecycle_append_ready = lifecycle_entry_append_dry_run.append_ready;
    let writeback_ready = eligible_after_approval
        && transition_dry_run.transition_ready
        && patch_ready
        && patch_plan.contract_patch_ready
        && patch_payload_ready
        && contract_source_ready
        && source_resolution_dry_run.resolved
        && patch_apply_dry_run.apply_ready
        && source_write_dry_run.write_ready
        && lifecycle_append_ready;
    let mut blocked_by = Vec::new();
    if !eligible_after_approval {
        blocked_by.push("approved_review_state_required".to_string());
    }
    if !transition_dry_run.transition_ready {
        blocked_by.push("review_transition_ready".to_string());
    }
    if !patch_ready {
        blocked_by.push("contract_patch_target_ready".to_string());
    }
    if !patch_plan.contract_patch_ready {
        blocked_by.push("contract_patch_plan_ready".to_string());
    }
    if !patch_payload_ready {
        blocked_by.push("contract_patch_payload_ready".to_string());
    }
    if !contract_source_ready {
        blocked_by.push("contract_source_ref_ready".to_string());
    }
    if !source_resolution_dry_run.resolved {
        blocked_by.push("contract_source_resolved".to_string());
    }
    if !patch_apply_dry_run.apply_ready {
        blocked_by.push("contract_patch_apply_ready".to_string());
    }
    if !source_write_dry_run.write_ready {
        blocked_by.push("contract_source_write_ready".to_string());
    }
    if !lifecycle_append_ready {
        blocked_by.push("lifecycle_entry_append_ready".to_string());
    }
    blocked_by.push("lifecycle_event_emission_enabled".to_string());
    blocked_by.push("contract_mutation_api_enabled".to_string());

    ContractRepairApprovalContractWritebackDryRun {
        status: if writeback_ready {
            "contract_writeback_dry_run_ready_blocked".to_string()
        } else {
            "contract_writeback_dry_run_blocked".to_string()
        },
        patch_kind: contract_writeback_patch_kind(&snapshot.target_kind).to_string(),
        target_path: snapshot.target_path.clone(),
        target_kind: snapshot.target_kind.clone(),
        changed_fields: snapshot.changed_fields.clone(),
        patch_payload: snapshot.patch_payload.clone(),
        contract_source_ref: snapshot.contract_source_ref.clone(),
        source_resolution: source_resolution_dry_run.clone(),
        patch_plan,
        patch_apply_dry_run: patch_apply_dry_run.clone(),
        source_write_dry_run: source_write_dry_run.clone(),
        eligible_after_approval,
        patch_ready,
        patch_payload_ready,
        contract_source_ready,
        transition_ready: transition_dry_run.transition_ready,
        lifecycle_append_ready,
        writeback_ready,
        missing_patch_fields,
        missing_contract_source_fields,
        would_mutate_contract: false,
        blocked_by,
    }
}

fn contract_repair_approval_contract_mutation_enablement_gate(
    lifecycle_enablement_gate: &ContractRepairApprovalLifecycleEmissionEnablementGate,
    contract_writeback_dry_run: &ContractRepairApprovalContractWritebackDryRun,
    contract_mutation_api_enabled: bool,
) -> ContractRepairApprovalContractMutationEnablementGate {
    let writeback_plan_ready = contract_writeback_dry_run.writeback_ready;
    let source_write_ready = contract_writeback_dry_run.source_write_dry_run.write_ready;
    let lifecycle_effects_ready = lifecycle_enablement_gate.lifecycle_effects_ready;
    let mutation_ready = writeback_plan_ready
        && source_write_ready
        && lifecycle_effects_ready
        && contract_mutation_api_enabled;
    let mut passed_gates = Vec::new();
    if writeback_plan_ready {
        passed_gates.push("contract_writeback_ready".to_string());
    }
    if source_write_ready {
        passed_gates.push("contract_source_write_ready".to_string());
    }
    if lifecycle_effects_ready {
        passed_gates.push("lifecycle_effects_ready".to_string());
    }
    if contract_mutation_api_enabled {
        passed_gates.push("contract_mutation_api_enabled".to_string());
    }
    let mut blocked_gates = Vec::new();
    if !writeback_plan_ready {
        blocked_gates.push("contract_writeback_ready".to_string());
    }
    if !source_write_ready {
        blocked_gates.push("contract_source_write_ready".to_string());
    }
    if !lifecycle_effects_ready {
        blocked_gates.push("lifecycle_effects_ready".to_string());
    }
    if !contract_mutation_api_enabled {
        blocked_gates.push("contract_mutation_api_enabled".to_string());
    }

    ContractRepairApprovalContractMutationEnablementGate {
        status: if mutation_ready {
            "contract_mutation_enablement_ready".to_string()
        } else if writeback_plan_ready && source_write_ready {
            "contract_mutation_enablement_ready_blocked".to_string()
        } else {
            "contract_mutation_enablement_blocked".to_string()
        },
        target_path: contract_writeback_dry_run.target_path.clone(),
        target_kind: contract_writeback_dry_run.target_kind.clone(),
        source_path: contract_writeback_dry_run
            .source_write_dry_run
            .source_path
            .clone(),
        writeback_plan_ready,
        source_write_ready,
        lifecycle_effects_ready,
        contract_mutation_api_enabled,
        mutation_ready,
        would_mutate_contract: false,
        would_write_source: false,
        would_touch_disk: false,
        required_gates: vec![
            "contract_writeback_ready".to_string(),
            "contract_source_write_ready".to_string(),
            "lifecycle_effects_ready".to_string(),
            "contract_mutation_api_enabled".to_string(),
        ],
        passed_gates,
        blocked_gates,
    }
}

fn contract_repair_approval_approve_execution_contract_mutation_readiness_dry_run(
    action: &str,
    approval_id: &str,
    contract_mutation_gate: &ContractRepairApprovalContractMutationEnablementGate,
) -> ContractRepairApprovalApproveExecutionContractMutationReadinessDryRun {
    let writeback_plan_ready = contract_mutation_gate.writeback_plan_ready;
    let source_write_ready = contract_mutation_gate.source_write_ready;
    let lifecycle_effects_ready = contract_mutation_gate.lifecycle_effects_ready;
    let contract_mutation_api_enabled = contract_mutation_gate.contract_mutation_api_enabled;
    let contract_mutation_ready = contract_mutation_gate.mutation_ready;
    let required_gates = vec![
        "contract_writeback_ready".to_string(),
        "contract_source_write_ready".to_string(),
        "lifecycle_effects_ready".to_string(),
        "contract_mutation_api_enabled".to_string(),
    ];
    let gate_states = [
        ("contract_writeback_ready", writeback_plan_ready),
        ("contract_source_write_ready", source_write_ready),
        ("lifecycle_effects_ready", lifecycle_effects_ready),
        (
            "contract_mutation_api_enabled",
            contract_mutation_api_enabled,
        ),
    ];
    let mut passed_gates = Vec::new();
    let mut blocked_gates = Vec::new();
    for (gate, passed) in gate_states {
        if passed {
            passed_gates.push(gate.to_string());
        } else {
            blocked_gates.push(gate.to_string());
        }
    }

    ContractRepairApprovalApproveExecutionContractMutationReadinessDryRun {
        status: if contract_mutation_ready {
            "approve_execution_contract_mutation_readiness_ready".to_string()
        } else if action == "approve" && writeback_plan_ready && source_write_ready {
            "approve_execution_contract_mutation_readiness_ready_blocked".to_string()
        } else {
            "approve_execution_contract_mutation_readiness_blocked".to_string()
        },
        action: action.to_string(),
        approval_id: approval_id.to_string(),
        gate_name: "contract_mutation_ready".to_string(),
        target_path: contract_mutation_gate.target_path.clone(),
        target_kind: contract_mutation_gate.target_kind.clone(),
        source_path: contract_mutation_gate.source_path.clone(),
        writeback_plan_ready,
        source_write_ready,
        lifecycle_effects_ready,
        contract_mutation_api_enabled,
        contract_mutation_ready,
        would_mutate_contract: false,
        would_write_source: false,
        would_touch_disk: false,
        would_unblock_atomic_side_effects: false,
        inherited_contract_mutation_blocked_gates: contract_mutation_gate.blocked_gates.clone(),
        required_gates,
        passed_gates,
        blocked_gates,
    }
}

async fn contract_repair_approval_contract_source_resolution_dry_run(
    graph_store_dir: &FsPath,
    source_ref: &ContractRepairApprovalContractSourceRef,
) -> ContractRepairApprovalContractSourceResolutionDryRun {
    let source_kind_supported = source_ref.source_kind == "v4_machine_graph_contract";
    let source_path = contract_repair_approval_contract_source_path(graph_store_dir, source_ref);
    let mut blocked_by = Vec::new();
    if !source_kind_supported {
        blocked_by.push("contract_source_kind_supported".to_string());
    }
    let missing_source_fields = contract_source_ref_missing_fields(source_ref);
    if !missing_source_fields.is_empty() {
        blocked_by.push("contract_source_ref_ready".to_string());
    }

    let mut source_exists = false;
    let mut source_id_match = false;
    let mut version_match = false;
    let mut artifact_digest_match = source_ref.artifact_digest.is_none();
    let mut contract_shape_ready = false;

    if source_kind_supported && missing_source_fields.is_empty() {
        match crate::runtime_persistence::read_to_string_bounded(
            &source_path,
            crate::runtime_persistence::MAX_BOUNDED_JSON_READ_BYTES,
        )
        .await
        {
            Ok(content) => match serde_json::from_str::<serde_json::Value>(&content) {
                Ok(source_json) => {
                    source_exists = true;
                    source_id_match = contract_source_json_graph_id(&source_json).as_deref()
                        == Some(source_ref.source_id.as_str());
                    version_match = contract_source_json_graph_version(&source_json).as_deref()
                        == Some(source_ref.version.as_str());
                    artifact_digest_match = match source_ref.artifact_digest.as_deref() {
                        Some(expected) => {
                            contract_source_json_artifact_digest(&source_json).as_deref()
                                == Some(expected)
                        }
                        None => true,
                    };
                    contract_shape_ready = source_json
                        .get("machines")
                        .and_then(serde_json::Value::as_array)
                        .is_some()
                        && source_json
                            .get("event_catalog")
                            .and_then(serde_json::Value::as_object)
                            .is_some();
                }
                Err(_) => blocked_by.push("contract_source_json_valid".to_string()),
            },
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(_) => blocked_by.push("contract_source_readable".to_string()),
        }
    }

    if !source_exists {
        blocked_by.push("contract_source_exists".to_string());
    }
    if !source_id_match {
        blocked_by.push("contract_source_id_match".to_string());
    }
    if !version_match {
        blocked_by.push("contract_source_version_match".to_string());
    }
    if !artifact_digest_match {
        blocked_by.push("contract_source_artifact_digest_match".to_string());
    }
    if !contract_shape_ready {
        blocked_by.push("contract_source_shape_ready".to_string());
    }

    let resolved = source_kind_supported
        && source_exists
        && source_id_match
        && version_match
        && artifact_digest_match
        && contract_shape_ready
        && missing_source_fields.is_empty();

    ContractRepairApprovalContractSourceResolutionDryRun {
        status: if resolved {
            "contract_source_resolved".to_string()
        } else {
            "contract_source_resolution_blocked".to_string()
        },
        source_kind: source_ref.source_kind.clone(),
        source_id: source_ref.source_id.clone(),
        version: source_ref.version.clone(),
        artifact_digest: source_ref.artifact_digest.clone(),
        source_path: source_path.to_string_lossy().to_string(),
        source_kind_supported,
        source_exists,
        source_id_match,
        version_match,
        artifact_digest_match,
        contract_shape_ready,
        resolved,
        blocked_by,
    }
}

async fn contract_repair_approval_contract_patch_apply_dry_run(
    graph_store_dir: &FsPath,
    source_ref: &ContractRepairApprovalContractSourceRef,
    source_resolution: &ContractRepairApprovalContractSourceResolutionDryRun,
    patch_plan: &ContractRepairApprovalContractPatchPlanPreview,
) -> ContractRepairApprovalContractPatchApplyDryRun {
    let mut blocked_by = Vec::new();
    if !source_resolution.resolved {
        push_unique_blocker(&mut blocked_by, "contract_source_resolved");
    }
    if !patch_plan.contract_patch_ready {
        push_unique_blocker(&mut blocked_by, "contract_patch_plan_ready");
    }

    let mut applied_selectors = Vec::new();
    let mut applied_operation_count = 0;
    if source_resolution.resolved && patch_plan.contract_patch_ready {
        let source_path =
            contract_repair_approval_contract_source_path(graph_store_dir, source_ref);
        match crate::runtime_persistence::read_to_string_bounded(
            &source_path,
            crate::runtime_persistence::MAX_BOUNDED_JSON_READ_BYTES,
        )
        .await
        {
            Ok(content) => match serde_json::from_str::<serde_json::Value>(&content) {
                Ok(mut source_json) => {
                    for operation in &patch_plan.operations {
                        if contract_repair_approval_apply_contract_patch_operation(
                            &mut source_json,
                            patch_plan,
                            operation,
                            &mut blocked_by,
                        ) {
                            applied_operation_count += 1;
                            applied_selectors.push(operation.selector.clone());
                        }
                    }
                }
                Err(_) => push_unique_blocker(&mut blocked_by, "contract_source_json_valid"),
            },
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                push_unique_blocker(&mut blocked_by, "contract_source_exists");
            }
            Err(_) => push_unique_blocker(&mut blocked_by, "contract_source_readable"),
        }
    }

    let apply_ready = source_resolution.resolved
        && patch_plan.contract_patch_ready
        && applied_operation_count == patch_plan.operations.len()
        && !patch_plan.operations.is_empty()
        && blocked_by.is_empty();

    ContractRepairApprovalContractPatchApplyDryRun {
        status: if apply_ready {
            "contract_patch_apply_ready_blocked".to_string()
        } else {
            "contract_patch_apply_blocked".to_string()
        },
        target_kind: patch_plan.target_kind.clone(),
        target_path: patch_plan.target_path.clone(),
        source_resolved: source_resolution.resolved,
        patch_plan_ready: patch_plan.contract_patch_ready,
        apply_ready,
        operation_count: patch_plan.operations.len(),
        applied_operation_count,
        applied_selectors,
        would_persist_source: false,
        blocked_by,
    }
}

#[cfg(test)]
async fn contract_repair_approval_contract_source_write_dry_run(
    graph_store_dir: &FsPath,
    source_ref: &ContractRepairApprovalContractSourceRef,
    source_resolution: &ContractRepairApprovalContractSourceResolutionDryRun,
    patch_plan: &ContractRepairApprovalContractPatchPlanPreview,
    patch_apply_dry_run: &ContractRepairApprovalContractPatchApplyDryRun,
) -> ContractRepairApprovalContractSourceWriteDryRun {
    contract_repair_approval_contract_source_write_with_gate(
        graph_store_dir,
        source_ref,
        source_resolution,
        patch_plan,
        patch_apply_dry_run,
        false,
    )
    .await
}

async fn contract_repair_approval_contract_source_write_with_gate(
    graph_store_dir: &FsPath,
    source_ref: &ContractRepairApprovalContractSourceRef,
    source_resolution: &ContractRepairApprovalContractSourceResolutionDryRun,
    patch_plan: &ContractRepairApprovalContractPatchPlanPreview,
    patch_apply_dry_run: &ContractRepairApprovalContractPatchApplyDryRun,
    source_write_enabled: bool,
) -> ContractRepairApprovalContractSourceWriteDryRun {
    let source_path = contract_repair_approval_contract_source_path(graph_store_dir, source_ref);
    let temp_file_name = source_path
        .file_name()
        .and_then(|name| name.to_str())
        .map(|file_name| format!("{file_name}.tmp"))
        .unwrap_or_else(|| "contract-source.json.tmp".to_string());
    let mut blocked_by = Vec::new();
    if !source_resolution.resolved {
        push_unique_blocker(&mut blocked_by, "contract_source_resolved");
    }
    if !patch_apply_dry_run.apply_ready {
        push_unique_blocker(&mut blocked_by, "contract_patch_apply_ready");
    }

    let mut source_digest_before = None;
    let mut source_digest_after = None;
    let mut patched_source_json = None;
    if source_resolution.resolved && patch_apply_dry_run.apply_ready {
        match crate::runtime_persistence::read_to_string_bounded(
            &source_path,
            crate::runtime_persistence::MAX_BOUNDED_JSON_READ_BYTES,
        )
        .await
        {
            Ok(content) => match serde_json::from_str::<serde_json::Value>(&content) {
                Ok(source_json) => {
                    match canonical_json_sha256_digest(&source_json) {
                        Ok(digest) => {
                            source_digest_before = Some(format!("sha256:{}", digest.value));
                        }
                        Err(_) => push_unique_blocker(
                            &mut blocked_by,
                            "contract_source_digest_before_ready",
                        ),
                    }
                    let mut patched_json = source_json.clone();
                    for operation in &patch_plan.operations {
                        contract_repair_approval_apply_contract_patch_operation(
                            &mut patched_json,
                            patch_plan,
                            operation,
                            &mut blocked_by,
                        );
                    }
                    match canonical_json_sha256_digest(&patched_json) {
                        Ok(digest) => {
                            source_digest_after = Some(format!("sha256:{}", digest.value));
                        }
                        Err(_) => push_unique_blocker(
                            &mut blocked_by,
                            "contract_source_digest_after_ready",
                        ),
                    }
                    patched_source_json = Some(patched_json);
                }
                Err(_) => push_unique_blocker(&mut blocked_by, "contract_source_json_valid"),
            },
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                push_unique_blocker(&mut blocked_by, "contract_source_exists");
            }
            Err(_) => push_unique_blocker(&mut blocked_by, "contract_source_readable"),
        }
    }

    let prepared_write_ready = source_resolution.resolved
        && patch_apply_dry_run.apply_ready
        && source_digest_before.is_some()
        && source_digest_after.is_some()
        && blocked_by.is_empty();
    let mut would_write_source = false;
    let mut would_touch_disk = false;
    if prepared_write_ready && source_write_enabled {
        if let Some(patched_json) = patched_source_json.as_ref() {
            match crate::runtime_persistence::atomic_write_json(&source_path, patched_json).await {
                Ok(()) => {
                    would_write_source = true;
                    would_touch_disk = true;
                }
                Err(_) => push_unique_blocker(&mut blocked_by, "contract_source_atomic_write"),
            }
        } else {
            push_unique_blocker(&mut blocked_by, "contract_source_patched_json_ready");
        }
    }
    let write_ready = prepared_write_ready && (!source_write_enabled || would_write_source);

    ContractRepairApprovalContractSourceWriteDryRun {
        status: if would_write_source {
            "contract_source_written".to_string()
        } else if write_ready {
            "contract_source_write_ready_blocked".to_string()
        } else {
            "contract_source_write_blocked".to_string()
        },
        source_path: source_path.to_string_lossy().to_string(),
        temp_file_name,
        source_resolved: source_resolution.resolved,
        patch_apply_ready: patch_apply_dry_run.apply_ready,
        write_ready,
        atomic_write_required: true,
        operation_count: patch_plan.operations.len(),
        source_digest_before,
        source_digest_after,
        would_write_source,
        would_touch_disk,
        blocked_by,
    }
}

fn contract_repair_approval_apply_contract_patch_operation(
    source_json: &mut serde_json::Value,
    patch_plan: &ContractRepairApprovalContractPatchPlanPreview,
    operation: &ContractRepairApprovalContractPatchOperationPreview,
    blocked_by: &mut Vec<String>,
) -> bool {
    if operation.operation != "upsert_contract_field" {
        push_unique_blocker(blocked_by, "contract_patch_apply_operation_supported");
        return false;
    }
    let path_segments = patch_plan
        .target_path
        .split('/')
        .filter(|segment| !segment.trim().is_empty())
        .collect::<Vec<_>>();

    match patch_plan.target_kind.as_str() {
        "memory_field" => contract_repair_approval_apply_memory_field_patch(
            source_json,
            &path_segments,
            operation,
            blocked_by,
        ),
        "event_payload_schema" => contract_repair_approval_apply_event_payload_schema_patch(
            source_json,
            &path_segments,
            operation,
            blocked_by,
        ),
        "event_catalog_entry" => contract_repair_approval_apply_event_catalog_entry_patch(
            source_json,
            &path_segments,
            operation,
            blocked_by,
        ),
        "event_boundary" => contract_repair_approval_apply_event_boundary_patch(
            source_json,
            &path_segments,
            operation,
            blocked_by,
        ),
        _ => {
            push_unique_blocker(blocked_by, "contract_patch_apply_target_supported");
            false
        }
    }
}

fn contract_repair_approval_apply_memory_field_patch(
    source_json: &mut serde_json::Value,
    path_segments: &[&str],
    operation: &ContractRepairApprovalContractPatchOperationPreview,
    blocked_by: &mut Vec<String>,
) -> bool {
    let Some(machine_id) = path_segments.get(1).copied() else {
        push_unique_blocker(blocked_by, "contract_patch_target_identity_ready");
        return false;
    };
    let Some(field_name) = path_segments.last().copied() else {
        push_unique_blocker(blocked_by, "contract_patch_target_identity_ready");
        return false;
    };
    let Some(machines) = source_json
        .get_mut("machines")
        .and_then(serde_json::Value::as_array_mut)
    else {
        push_unique_blocker(blocked_by, "contract_source_machines_array_ready");
        return false;
    };
    let Some(machine_index) = machines.iter().position(|machine| {
        machine
            .get("machine_id")
            .and_then(serde_json::Value::as_str)
            == Some(machine_id)
    }) else {
        push_unique_blocker(blocked_by, "contract_patch_apply_machine_found");
        return false;
    };
    let Some(machine_object) = machines[machine_index].as_object_mut() else {
        push_unique_blocker(blocked_by, "contract_patch_apply_machine_object_ready");
        return false;
    };
    let memory_value = machine_object
        .entry("memory".to_string())
        .or_insert_with(|| serde_json::Value::Array(Vec::new()));
    let Some(memory_fields) = memory_value.as_array_mut() else {
        push_unique_blocker(blocked_by, "contract_patch_apply_memory_array_ready");
        return false;
    };
    let field_index = match memory_fields
        .iter()
        .position(|field| field.get("name").and_then(serde_json::Value::as_str) == Some(field_name))
    {
        Some(index) => index,
        None => {
            memory_fields.push(json!({ "name": field_name }));
            memory_fields.len() - 1
        }
    };
    let Some(field_object) = memory_fields[field_index].as_object_mut() else {
        push_unique_blocker(blocked_by, "contract_patch_apply_memory_field_object_ready");
        return false;
    };
    field_object.insert(operation.field_name.clone(), operation.value.clone());
    true
}

fn contract_repair_approval_apply_event_payload_schema_patch(
    source_json: &mut serde_json::Value,
    path_segments: &[&str],
    operation: &ContractRepairApprovalContractPatchOperationPreview,
    blocked_by: &mut Vec<String>,
) -> bool {
    let Some(event_type) = path_segments.get(1).copied() else {
        push_unique_blocker(blocked_by, "contract_patch_target_identity_ready");
        return false;
    };
    let Some(field_name) = path_segments.last().copied() else {
        push_unique_blocker(blocked_by, "contract_patch_target_identity_ready");
        return false;
    };
    let Some(event_object) =
        contract_repair_approval_event_catalog_event_object_mut(source_json, event_type, false)
    else {
        push_unique_blocker(blocked_by, "contract_patch_apply_event_found");
        return false;
    };
    let payload_fields_value = event_object
        .entry("payload_fields".to_string())
        .or_insert_with(|| serde_json::Value::Array(Vec::new()));
    let Some(payload_fields) = payload_fields_value.as_array_mut() else {
        push_unique_blocker(
            blocked_by,
            "contract_patch_apply_payload_fields_array_ready",
        );
        return false;
    };
    let field_index = match payload_fields
        .iter()
        .position(|field| field.get("name").and_then(serde_json::Value::as_str) == Some(field_name))
    {
        Some(index) => index,
        None => {
            payload_fields.push(json!({ "name": field_name }));
            payload_fields.len() - 1
        }
    };
    let Some(field_object) = payload_fields[field_index].as_object_mut() else {
        push_unique_blocker(
            blocked_by,
            "contract_patch_apply_payload_field_object_ready",
        );
        return false;
    };
    field_object.insert(operation.field_name.clone(), operation.value.clone());
    true
}

fn contract_repair_approval_apply_event_catalog_entry_patch(
    source_json: &mut serde_json::Value,
    path_segments: &[&str],
    operation: &ContractRepairApprovalContractPatchOperationPreview,
    blocked_by: &mut Vec<String>,
) -> bool {
    let Some(event_type) = path_segments.get(1).copied() else {
        push_unique_blocker(blocked_by, "contract_patch_target_identity_ready");
        return false;
    };
    let Some(event_object) =
        contract_repair_approval_event_catalog_event_object_mut(source_json, event_type, true)
    else {
        push_unique_blocker(blocked_by, "contract_patch_apply_event_catalog_ready");
        return false;
    };
    event_object.insert(operation.field_name.clone(), operation.value.clone());
    true
}

fn contract_repair_approval_apply_event_boundary_patch(
    source_json: &mut serde_json::Value,
    path_segments: &[&str],
    operation: &ContractRepairApprovalContractPatchOperationPreview,
    blocked_by: &mut Vec<String>,
) -> bool {
    let Some(event_type) = path_segments.get(1).copied() else {
        push_unique_blocker(blocked_by, "contract_patch_target_identity_ready");
        return false;
    };
    let Some(event_object) =
        contract_repair_approval_event_catalog_event_object_mut(source_json, event_type, false)
    else {
        push_unique_blocker(blocked_by, "contract_patch_apply_event_found");
        return false;
    };
    let boundary_value = event_object
        .entry("boundary".to_string())
        .or_insert_with(|| json!({}));
    let Some(boundary_object) = boundary_value.as_object_mut() else {
        push_unique_blocker(blocked_by, "contract_patch_apply_boundary_object_ready");
        return false;
    };
    boundary_object.insert(operation.field_name.clone(), operation.value.clone());
    true
}

fn contract_repair_approval_event_catalog_event_object_mut<'a>(
    source_json: &'a mut serde_json::Value,
    event_type: &str,
    create_missing: bool,
) -> Option<&'a mut serde_json::Map<String, serde_json::Value>> {
    let event_catalog = source_json
        .get_mut("event_catalog")
        .and_then(serde_json::Value::as_object_mut)?;
    let events_value = event_catalog
        .entry("events".to_string())
        .or_insert_with(|| serde_json::Value::Array(Vec::new()));
    let events = events_value.as_array_mut()?;
    let event_index = match events.iter().position(|event| {
        event.get("event_type").and_then(serde_json::Value::as_str) == Some(event_type)
    }) {
        Some(index) => index,
        None if create_missing => {
            events.push(json!({ "event_type": event_type }));
            events.len() - 1
        }
        None => return None,
    };
    events[event_index].as_object_mut()
}

fn push_unique_blocker(blocked_by: &mut Vec<String>, blocker: &str) {
    if !blocked_by.iter().any(|existing| existing == blocker) {
        blocked_by.push(blocker.to_string());
    }
}

fn contract_repair_approval_contract_source_path(
    graph_store_dir: &FsPath,
    source_ref: &ContractRepairApprovalContractSourceRef,
) -> PathBuf {
    graph_store_dir.join(format!(
        "{}.json",
        sanitize_storage_path_segment(&source_ref.source_id)
    ))
}

fn contract_source_json_graph_id(source_json: &serde_json::Value) -> Option<String> {
    source_json
        .get("graph_id")
        .and_then(serde_json::Value::as_str)
        .or_else(|| {
            source_json
                .get("metadata")
                .and_then(|metadata| metadata.get("graph_id"))
                .and_then(serde_json::Value::as_str)
        })
        .map(str::to_string)
}

fn contract_source_json_graph_version(source_json: &serde_json::Value) -> Option<String> {
    source_json
        .get("metadata")
        .and_then(|metadata| metadata.get("graph_version"))
        .and_then(serde_json::Value::as_str)
        .or_else(|| {
            source_json
                .get("graph_version")
                .and_then(serde_json::Value::as_str)
        })
        .map(str::to_string)
}

fn contract_source_json_artifact_digest(source_json: &serde_json::Value) -> Option<String> {
    source_json
        .get("metadata")
        .and_then(|metadata| metadata.get("artifact_digest"))
        .and_then(serde_json::Value::as_str)
        .or_else(|| {
            source_json
                .get("metadata")
                .and_then(|metadata| metadata.get("artifact_hash"))
                .and_then(serde_json::Value::as_str)
        })
        .or_else(|| {
            source_json
                .get("contract_artifact_hash")
                .and_then(serde_json::Value::as_str)
        })
        .map(str::to_string)
}

fn contract_repair_approval_contract_patch_plan_preview(
    snapshot: &ContractRepairApprovalRecordSnapshotPreview,
) -> ContractRepairApprovalContractPatchPlanPreview {
    let target_kind = snapshot.target_kind.as_str();
    let plan_kind = contract_writeback_patch_kind(target_kind).to_string();
    let path_segments = snapshot
        .target_path
        .split('/')
        .filter(|segment| !segment.trim().is_empty())
        .collect::<Vec<_>>();
    let evidence_sample_patch = target_kind == "event_instance_payload";
    let mut blocked_by = Vec::new();
    let mut unsupported_fields = Vec::new();
    let identity_ready = contract_patch_target_identity_ready(target_kind, &path_segments);
    if !identity_ready {
        blocked_by.push("contract_patch_target_identity_ready".to_string());
    }
    if evidence_sample_patch {
        blocked_by.push("contract_patch_target_kind_supported".to_string());
    }

    let mut operations = Vec::new();
    for field_name in &snapshot.changed_fields {
        if !contract_patch_supported_field(target_kind, field_name) {
            unsupported_fields.push(field_name.clone());
            continue;
        }
        if let Some(value) = snapshot.patch_payload.get(field_name) {
            operations.push(ContractRepairApprovalContractPatchOperationPreview {
                operation: if evidence_sample_patch {
                    "update_evidence_sample_field".to_string()
                } else {
                    "upsert_contract_field".to_string()
                },
                domain: contract_patch_domain(target_kind).to_string(),
                selector: contract_patch_selector(target_kind, &path_segments),
                field_name: field_name.clone(),
                value: value.clone(),
            });
        }
    }

    if !unsupported_fields.is_empty() {
        blocked_by.push("contract_patch_fields_supported".to_string());
    }
    if operations.len() != snapshot.changed_fields.len() {
        blocked_by.push("contract_patch_operations_cover_changed_fields".to_string());
    }

    let contract_patch_ready = identity_ready
        && !evidence_sample_patch
        && !snapshot.changed_fields.is_empty()
        && unsupported_fields.is_empty()
        && operations.len() == snapshot.changed_fields.len();

    ContractRepairApprovalContractPatchPlanPreview {
        status: if contract_patch_ready {
            "contract_patch_plan_ready".to_string()
        } else {
            "contract_patch_plan_blocked".to_string()
        },
        plan_kind,
        target_kind: snapshot.target_kind.clone(),
        target_path: snapshot.target_path.clone(),
        contract_patch_ready,
        evidence_sample_patch,
        operations,
        unsupported_fields,
        blocked_by,
    }
}

fn contract_patch_target_identity_ready(target_kind: &str, path_segments: &[&str]) -> bool {
    match target_kind {
        "memory_field" => {
            path_segments.first() == Some(&"memory_schema")
                && path_segments.get(1).is_some()
                && path_segments.last().is_some()
                && path_segments.len() >= 3
        }
        "event_payload_schema" => {
            path_segments.first() == Some(&"event_catalog")
                && path_segments.get(1).is_some()
                && path_segments.last().is_some()
                && path_segments.len() >= 3
        }
        "event_boundary" | "event_catalog_entry" => {
            path_segments.first() == Some(&"event_catalog") && path_segments.get(1).is_some()
        }
        "event_instance_payload" => {
            path_segments.first() == Some(&"event_catalog")
                && path_segments.get(1).is_some()
                && path_segments.last().is_some()
                && path_segments.len() >= 4
        }
        _ => false,
    }
}

fn contract_patch_supported_field(target_kind: &str, field_name: &str) -> bool {
    let supported_fields: &[&str] = match target_kind {
        "memory_field" => &["type_name", "nullable", "default_value"],
        "event_payload_schema" => &["type_name", "required", "nullable"],
        "event_boundary" => &["source", "machine_id"],
        "event_catalog_entry" => &["source_kind", "scope"],
        "event_instance_payload" => &["payload_value"],
        _ => &[],
    };
    supported_fields.contains(&field_name)
}

fn contract_patch_domain(target_kind: &str) -> &'static str {
    match target_kind {
        "memory_field" => "memory_schema",
        "event_instance_payload" => "event_payload_sample",
        _ => "event_catalog",
    }
}

fn contract_patch_selector(target_kind: &str, path_segments: &[&str]) -> String {
    match target_kind {
        "memory_field" => {
            let machine_id = path_segments.get(1).copied().unwrap_or("missing");
            let field_name = path_segments.last().copied().unwrap_or("missing");
            format!("machines[machine_id={machine_id}].memory[name={field_name}]")
        }
        "event_payload_schema" => {
            let event_type = path_segments.get(1).copied().unwrap_or("missing");
            let field_name = path_segments.last().copied().unwrap_or("missing");
            format!(
                "event_catalog.events[event_type={event_type}].payload_fields[name={field_name}]"
            )
        }
        "event_boundary" => {
            let event_type = path_segments.get(1).copied().unwrap_or("missing");
            format!("event_catalog.events[event_type={event_type}].boundary")
        }
        "event_instance_payload" => {
            let event_type = path_segments.get(1).copied().unwrap_or("missing");
            let instance_id = path_segments.last().copied().unwrap_or("missing");
            format!("event_instances[event_type={event_type}][instance_id={instance_id}]")
        }
        _ => {
            let event_type = path_segments.get(1).copied().unwrap_or("missing");
            format!("event_catalog.events[event_type={event_type}]")
        }
    }
}

fn contract_source_ref_missing_fields(
    source_ref: &ContractRepairApprovalContractSourceRef,
) -> Vec<String> {
    let mut missing_fields = Vec::new();
    if source_ref.source_kind.trim().is_empty() {
        missing_fields.push("source_kind".to_string());
    }
    if source_ref.source_id.trim().is_empty() {
        missing_fields.push("source_id".to_string());
    }
    if source_ref.version.trim().is_empty() {
        missing_fields.push("version".to_string());
    }
    missing_fields
}

fn contract_writeback_patch_kind(target_kind: &str) -> &'static str {
    match target_kind {
        "memory_field" => "memory_schema_patch",
        "event_instance_payload" => "event_payload_instance_patch",
        "event_payload_schema" => "event_payload_schema_patch",
        "event_boundary" => "event_boundary_patch",
        _ => "event_catalog_entry_patch",
    }
}

async fn visible_contract_repair_approval_records(
    state: &AppState,
    user_id: &auth::UserId,
) -> Result<Vec<ContractRepairApprovalRecordPreview>, (StatusCode, String)> {
    let scoped_prefix = auth::scoped_key(user_id, "");
    let mut by_id = BTreeMap::new();
    for record in list_contract_repair_approval_records_from_disk(
        state.contract_repair_approval_store_dir.as_ref(),
    )
    .await?
    {
        by_id.insert(record.approval_id.clone(), record);
    }
    let previews = state.contract_repair_approval_previews.read().await;
    for (key, record) in previews.iter() {
        if user_id.0 == 0 || key.starts_with(&scoped_prefix) {
            by_id
                .entry(record.approval_id.clone())
                .or_insert_with(|| record.clone());
        }
    }
    let mut records = by_id.into_values().collect::<Vec<_>>();
    records.sort_by(|left, right| left.approval_id.cmp(&right.approval_id));
    Ok(records)
}

fn mark_contract_repair_approval_record_persisted(
    record: &mut ContractRepairApprovalRecordPreview,
) {
    record.status = "approval_request_persisted".to_string();
    record.would_persist = true;
    record.persistence_enabled = true;
}

fn contract_repair_approval_record_path(store_dir: &FsPath, approval_id: &str) -> PathBuf {
    store_dir.join(format!(
        "{}.json",
        sanitize_storage_path_segment(approval_id)
    ))
}

async fn persist_contract_repair_approval_record(
    store_dir: &FsPath,
    record: &ContractRepairApprovalRecordPreview,
) -> std::io::Result<()> {
    fs::create_dir_all(store_dir).await?;
    let path = contract_repair_approval_record_path(store_dir, &record.approval_id);
    crate::runtime_persistence::atomic_write_json(&path, record).await
}

async fn load_contract_repair_approval_record_from_disk(
    store_dir: &FsPath,
    approval_id: &str,
) -> Result<Option<ContractRepairApprovalRecordPreview>, (StatusCode, String)> {
    let path = contract_repair_approval_record_path(store_dir, approval_id);
    match fs::read(&path).await {
        Ok(bytes) => serde_json::from_slice(&bytes)
            .map(Some)
            .map_err(|error| internal_error(anyhow::anyhow!("{}", error))),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(internal_error(anyhow::Error::new(error))),
    }
}

async fn list_contract_repair_approval_records_from_disk(
    store_dir: &FsPath,
) -> Result<Vec<ContractRepairApprovalRecordPreview>, (StatusCode, String)> {
    if !fs::try_exists(store_dir)
        .await
        .map_err(|error| internal_error(anyhow::Error::new(error)))?
    {
        return Ok(Vec::new());
    }
    let mut entries = fs::read_dir(store_dir)
        .await
        .map_err(|error| internal_error(anyhow::Error::new(error)))?;
    let mut records = Vec::new();
    while let Some(entry) = entries
        .next_entry()
        .await
        .map_err(|error| internal_error(anyhow::Error::new(error)))?
    {
        let path = entry.path();
        if path.extension().and_then(|value| value.to_str()) != Some("json") {
            continue;
        }
        let bytes = fs::read(&path)
            .await
            .map_err(|error| internal_error(anyhow::Error::new(error)))?;
        let record = serde_json::from_slice(&bytes)
            .map_err(|error| internal_error(anyhow::anyhow!("{}", error)))?;
        records.push(record);
    }
    Ok(records)
}

fn contract_repair_approval_record_preview(
    request: &CreateContractRepairApprovalRequest,
) -> Result<ContractRepairApprovalRecordPreview, (StatusCode, String)> {
    let mut changed_fields = request.changed_fields.clone();
    changed_fields.sort();
    let identity_input = json!({
        "payload_kind": request.payload_kind,
        "request_id": request.request_id,
        "target_kind": request.target_kind,
        "target_path": request.target_path,
        "changed_fields": changed_fields,
        "patch_payload": request.patch_payload,
        "contract_source_ref": request.contract_source_ref,
    });
    let digest = canonical_json_sha256_digest(&identity_input)
        .map_err(|error| internal_error(anyhow::Error::new(error)))?;
    let idempotency_key = format!("sha256:{}", digest.value);
    let approval_suffix = digest.value.chars().take(16).collect::<String>();

    Ok(ContractRepairApprovalRecordPreview {
        status: "approval_record_preview_only".to_string(),
        approval_id: format!("contract-repair-apr-{approval_suffix}"),
        idempotency_key,
        target_path: request.target_path.clone(),
        target_kind: request.target_kind.clone(),
        changed_fields,
        patch_payload: request.patch_payload.clone(),
        contract_source_ref: request.contract_source_ref.clone(),
        review_state: RuntimeApprovalReviewState::Pending,
        reviewers_required: 1,
        transient_review_status: "not_claimed".to_string(),
        transient_review_action: None,
        transient_reviewer_id: None,
        transient_review_reason: None,
        lifecycle: Vec::new(),
        would_persist: false,
        persistence_enabled: false,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_stamp() -> u128 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system time should be after unix epoch")
            .as_nanos()
    }

    fn valid_request() -> CreateContractRepairApprovalRequest {
        CreateContractRepairApprovalRequest {
            status: "body_preview_only".to_string(),
            payload_kind: CONTRACT_REPAIR_PAYLOAD_KIND.to_string(),
            request_id:
                "approval-request:repair-draft:memory_schema/decision.dual_ma/cross_up/last_signal_at"
                    .to_string(),
            target_path: "memory_schema/decision.dual_ma/cross_up/last_signal_at".to_string(),
            target_kind: "memory_field".to_string(),
            changed_fields: vec!["type_name".to_string()],
            patch_payload: BTreeMap::from([(
                "type_name".to_string(),
                serde_json::Value::String("time?".to_string()),
            )]),
            contract_source_ref: ContractRepairApprovalContractSourceRef {
                source_kind: "v4_machine_graph_contract".to_string(),
                source_id: "graph:dual-ma".to_string(),
                version: "v4-test".to_string(),
                artifact_digest: Some("sha256:test-contract".to_string()),
            },
            mutation_enabled: false,
            review_required: true,
        }
    }

    fn ready_source_resolution() -> ContractRepairApprovalContractSourceResolutionDryRun {
        ContractRepairApprovalContractSourceResolutionDryRun {
            status: "contract_source_resolved".to_string(),
            source_kind: "v4_machine_graph_contract".to_string(),
            source_id: "graph:dual-ma".to_string(),
            version: "v4-test".to_string(),
            artifact_digest: Some("sha256:test-contract".to_string()),
            source_path: "graphs/graph_dual-ma.json".to_string(),
            source_kind_supported: true,
            source_exists: true,
            source_id_match: true,
            version_match: true,
            artifact_digest_match: true,
            contract_shape_ready: true,
            resolved: true,
            blocked_by: Vec::new(),
        }
    }

    fn blocked_source_resolution() -> ContractRepairApprovalContractSourceResolutionDryRun {
        ContractRepairApprovalContractSourceResolutionDryRun {
            status: "contract_source_resolution_blocked".to_string(),
            source_kind: String::new(),
            source_id: String::new(),
            version: String::new(),
            artifact_digest: None,
            source_path: "graphs/missing.json".to_string(),
            source_kind_supported: false,
            source_exists: false,
            source_id_match: false,
            version_match: false,
            artifact_digest_match: true,
            contract_shape_ready: false,
            resolved: false,
            blocked_by: vec!["contract_source_ref_ready".to_string()],
        }
    }

    fn ready_patch_apply_dry_run() -> ContractRepairApprovalContractPatchApplyDryRun {
        ContractRepairApprovalContractPatchApplyDryRun {
            status: "contract_patch_apply_ready_blocked".to_string(),
            target_kind: "memory_field".to_string(),
            target_path: "memory_schema/decision.dual_ma/cross_up/last_signal_at".to_string(),
            source_resolved: true,
            patch_plan_ready: true,
            apply_ready: true,
            operation_count: 1,
            applied_operation_count: 1,
            applied_selectors: vec![
                "machines[machine_id=decision.dual_ma].memory[name=last_signal_at]".to_string(),
            ],
            would_persist_source: false,
            blocked_by: Vec::new(),
        }
    }

    fn blocked_patch_apply_dry_run() -> ContractRepairApprovalContractPatchApplyDryRun {
        ContractRepairApprovalContractPatchApplyDryRun {
            status: "contract_patch_apply_blocked".to_string(),
            target_kind: "memory_field".to_string(),
            target_path: "memory_schema/decision.dual_ma/cross_up/last_signal_at".to_string(),
            source_resolved: false,
            patch_plan_ready: false,
            apply_ready: false,
            operation_count: 0,
            applied_operation_count: 0,
            applied_selectors: Vec::new(),
            would_persist_source: false,
            blocked_by: vec!["contract_source_resolved".to_string()],
        }
    }

    fn ready_source_write_dry_run() -> ContractRepairApprovalContractSourceWriteDryRun {
        ContractRepairApprovalContractSourceWriteDryRun {
            status: "contract_source_write_ready_blocked".to_string(),
            source_path: "graphs/graph_dual-ma.json".to_string(),
            temp_file_name: "graph_dual-ma.json.tmp".to_string(),
            source_resolved: true,
            patch_apply_ready: true,
            write_ready: true,
            atomic_write_required: true,
            operation_count: 1,
            source_digest_before: Some("sha256:before".to_string()),
            source_digest_after: Some("sha256:after".to_string()),
            would_write_source: false,
            would_touch_disk: false,
            blocked_by: Vec::new(),
        }
    }

    fn blocked_source_write_dry_run() -> ContractRepairApprovalContractSourceWriteDryRun {
        ContractRepairApprovalContractSourceWriteDryRun {
            status: "contract_source_write_blocked".to_string(),
            source_path: "graphs/missing.json".to_string(),
            temp_file_name: "missing.json.tmp".to_string(),
            source_resolved: false,
            patch_apply_ready: false,
            write_ready: false,
            atomic_write_required: true,
            operation_count: 0,
            source_digest_before: None,
            source_digest_after: None,
            would_write_source: false,
            would_touch_disk: false,
            blocked_by: vec!["contract_patch_apply_ready".to_string()],
        }
    }

    #[tokio::test]
    async fn contract_repair_approval_source_resolution_reads_graph_contract_source() {
        let source_ref = ContractRepairApprovalContractSourceRef {
            source_kind: "v4_machine_graph_contract".to_string(),
            source_id: format!("graph-dual-ma-{}", test_stamp()),
            version: "v4-test".to_string(),
            artifact_digest: Some("sha256:test-contract".to_string()),
        };
        let graph_store_dir = std::env::temp_dir().join(format!(
            "quantpilot-contract-repair-source-resolution-{}",
            test_stamp()
        ));
        std::fs::create_dir_all(&graph_store_dir).expect("graph store should be created");
        let source_path =
            contract_repair_approval_contract_source_path(&graph_store_dir, &source_ref);
        std::fs::write(
            &source_path,
            json!({
                "schema_version": "quantpilot/machine-graph-contract/v1",
                "graph_id": source_ref.source_id.clone(),
                "machines": [
                    {
                        "machine_id": "decision.dual_ma",
                        "memory": []
                    }
                ],
                "event_catalog": {
                    "events": []
                },
                "metadata": {
                    "graph_version": "v4-test",
                    "artifact_hash": "sha256:test-contract"
                }
            })
            .to_string(),
        )
        .expect("source graph should be written");

        let resolution = contract_repair_approval_contract_source_resolution_dry_run(
            &graph_store_dir,
            &source_ref,
        )
        .await;

        assert_eq!(resolution.status, "contract_source_resolved");
        assert!(resolution.source_kind_supported);
        assert!(resolution.source_exists);
        assert!(resolution.source_id_match);
        assert!(resolution.version_match);
        assert!(resolution.artifact_digest_match);
        assert!(resolution.contract_shape_ready);
        assert!(resolution.resolved);
        assert!(resolution.blocked_by.is_empty());
    }

    #[tokio::test]
    async fn contract_repair_approval_patch_apply_dry_run_applies_memory_field_without_persisting()
    {
        let mut request = valid_request();
        request.contract_source_ref.source_id = format!("graph-dual-ma-{}", test_stamp());
        let preview = contract_repair_approval_record_preview(&request).unwrap();
        let snapshot = contract_repair_approval_record_snapshot_preview(
            &preview,
            "approve",
            "reviewer-a",
            "preview looks safe",
        );
        let graph_store_dir = std::env::temp_dir().join(format!(
            "quantpilot-contract-repair-patch-apply-{}",
            test_stamp()
        ));
        std::fs::create_dir_all(&graph_store_dir).expect("graph store should be created");
        let source_path = contract_repair_approval_contract_source_path(
            &graph_store_dir,
            &snapshot.contract_source_ref,
        );
        let source_json = json!({
            "schema_version": "quantpilot/machine-graph-contract/v1",
            "graph_id": snapshot.contract_source_ref.source_id.clone(),
            "machines": [
                {
                    "machine_id": "decision.dual_ma",
                    "memory": []
                }
            ],
            "event_catalog": {
                "events": []
            },
            "metadata": {
                "graph_version": "v4-test",
                "artifact_hash": "sha256:test-contract"
            }
        });
        std::fs::write(&source_path, source_json.to_string())
            .expect("source graph should be written");
        let resolution = contract_repair_approval_contract_source_resolution_dry_run(
            &graph_store_dir,
            &snapshot.contract_source_ref,
        )
        .await;
        let patch_plan = contract_repair_approval_contract_patch_plan_preview(&snapshot);

        let apply_dry_run = contract_repair_approval_contract_patch_apply_dry_run(
            &graph_store_dir,
            &snapshot.contract_source_ref,
            &resolution,
            &patch_plan,
        )
        .await;

        assert_eq!(apply_dry_run.status, "contract_patch_apply_ready_blocked");
        assert!(apply_dry_run.source_resolved);
        assert!(apply_dry_run.patch_plan_ready);
        assert!(apply_dry_run.apply_ready);
        assert_eq!(apply_dry_run.operation_count, 1);
        assert_eq!(apply_dry_run.applied_operation_count, 1);
        assert_eq!(
            apply_dry_run.applied_selectors,
            vec!["machines[machine_id=decision.dual_ma].memory[name=last_signal_at]".to_string()]
        );
        assert!(!apply_dry_run.would_persist_source);
        assert!(apply_dry_run.blocked_by.is_empty());
        let persisted_source = std::fs::read_to_string(&source_path)
            .expect("source graph should remain readable after dry-run");
        let persisted_json: serde_json::Value =
            serde_json::from_str(&persisted_source).expect("persisted source should stay json");
        assert_eq!(persisted_json, source_json);
    }

    #[tokio::test]
    async fn contract_repair_approval_source_write_dry_run_prepares_digest_without_writing() {
        let mut request = valid_request();
        request.contract_source_ref.source_id = format!("graph-dual-ma-{}", test_stamp());
        let preview = contract_repair_approval_record_preview(&request).unwrap();
        let snapshot = contract_repair_approval_record_snapshot_preview(
            &preview,
            "approve",
            "reviewer-a",
            "preview looks safe",
        );
        let graph_store_dir = std::env::temp_dir().join(format!(
            "quantpilot-contract-repair-source-write-{}",
            test_stamp()
        ));
        std::fs::create_dir_all(&graph_store_dir).expect("graph store should be created");
        let source_path = contract_repair_approval_contract_source_path(
            &graph_store_dir,
            &snapshot.contract_source_ref,
        );
        let source_json = json!({
            "schema_version": "quantpilot/machine-graph-contract/v1",
            "graph_id": snapshot.contract_source_ref.source_id.clone(),
            "machines": [
                {
                    "machine_id": "decision.dual_ma",
                    "memory": []
                }
            ],
            "event_catalog": {
                "events": []
            },
            "metadata": {
                "graph_version": "v4-test",
                "artifact_hash": "sha256:test-contract"
            }
        });
        std::fs::write(&source_path, source_json.to_string())
            .expect("source graph should be written");
        let resolution = contract_repair_approval_contract_source_resolution_dry_run(
            &graph_store_dir,
            &snapshot.contract_source_ref,
        )
        .await;
        let patch_plan = contract_repair_approval_contract_patch_plan_preview(&snapshot);
        let apply_dry_run = contract_repair_approval_contract_patch_apply_dry_run(
            &graph_store_dir,
            &snapshot.contract_source_ref,
            &resolution,
            &patch_plan,
        )
        .await;

        let source_write_dry_run = contract_repair_approval_contract_source_write_dry_run(
            &graph_store_dir,
            &snapshot.contract_source_ref,
            &resolution,
            &patch_plan,
            &apply_dry_run,
        )
        .await;

        assert_eq!(
            source_write_dry_run.status,
            "contract_source_write_ready_blocked"
        );
        assert!(source_write_dry_run.source_resolved);
        assert!(source_write_dry_run.patch_apply_ready);
        assert!(source_write_dry_run.write_ready);
        assert!(source_write_dry_run.atomic_write_required);
        assert_eq!(source_write_dry_run.operation_count, 1);
        assert!(source_write_dry_run
            .source_digest_before
            .as_deref()
            .unwrap_or_default()
            .starts_with("sha256:"));
        assert!(source_write_dry_run
            .source_digest_after
            .as_deref()
            .unwrap_or_default()
            .starts_with("sha256:"));
        assert_ne!(
            source_write_dry_run.source_digest_before,
            source_write_dry_run.source_digest_after
        );
        assert!(!source_write_dry_run.would_write_source);
        assert!(!source_write_dry_run.would_touch_disk);
        assert!(source_write_dry_run.blocked_by.is_empty());
        let persisted_source = std::fs::read_to_string(&source_path)
            .expect("source graph should remain readable after dry-run");
        let persisted_json: serde_json::Value =
            serde_json::from_str(&persisted_source).expect("persisted source should stay json");
        assert_eq!(persisted_json, source_json);
    }

    #[tokio::test]
    async fn contract_repair_approval_source_write_gate_persists_patched_source_when_enabled() {
        let mut request = valid_request();
        request.contract_source_ref.source_id = format!("graph-dual-ma-{}", test_stamp());
        let preview = contract_repair_approval_record_preview(&request).unwrap();
        let snapshot = contract_repair_approval_record_snapshot_preview(
            &preview,
            "approve",
            "reviewer-a",
            "preview looks safe",
        );
        let graph_store_dir = std::env::temp_dir().join(format!(
            "quantpilot-contract-repair-source-write-execute-{}",
            test_stamp()
        ));
        std::fs::create_dir_all(&graph_store_dir).expect("graph store should be created");
        let source_path = contract_repair_approval_contract_source_path(
            &graph_store_dir,
            &snapshot.contract_source_ref,
        );
        let source_json = json!({
            "schema_version": "quantpilot/machine-graph-contract/v1",
            "graph_id": snapshot.contract_source_ref.source_id.clone(),
            "machines": [
                {
                    "machine_id": "decision.dual_ma",
                    "memory": []
                }
            ],
            "event_catalog": {
                "events": []
            },
            "metadata": {
                "graph_version": "v4-test",
                "artifact_hash": "sha256:test-contract"
            }
        });
        std::fs::write(&source_path, source_json.to_string())
            .expect("source graph should be written");
        let resolution = contract_repair_approval_contract_source_resolution_dry_run(
            &graph_store_dir,
            &snapshot.contract_source_ref,
        )
        .await;
        let patch_plan = contract_repair_approval_contract_patch_plan_preview(&snapshot);
        let apply_dry_run = contract_repair_approval_contract_patch_apply_dry_run(
            &graph_store_dir,
            &snapshot.contract_source_ref,
            &resolution,
            &patch_plan,
        )
        .await;

        let source_write_result = contract_repair_approval_contract_source_write_with_gate(
            &graph_store_dir,
            &snapshot.contract_source_ref,
            &resolution,
            &patch_plan,
            &apply_dry_run,
            true,
        )
        .await;

        assert_eq!(source_write_result.status, "contract_source_written");
        assert!(source_write_result.source_resolved);
        assert!(source_write_result.patch_apply_ready);
        assert!(source_write_result.write_ready);
        assert!(source_write_result.atomic_write_required);
        assert_eq!(source_write_result.operation_count, 1);
        assert!(source_write_result.would_write_source);
        assert!(source_write_result.would_touch_disk);
        assert!(source_write_result.blocked_by.is_empty());
        assert_ne!(
            source_write_result.source_digest_before,
            source_write_result.source_digest_after
        );

        let persisted_source = std::fs::read_to_string(&source_path)
            .expect("source graph should remain readable after write");
        let persisted_json: serde_json::Value =
            serde_json::from_str(&persisted_source).expect("persisted source should stay json");
        assert_ne!(persisted_json, source_json);
        assert_eq!(
            persisted_json["machines"][0]["memory"][0]["name"],
            "last_signal_at"
        );
        assert_eq!(
            persisted_json["machines"][0]["memory"][0]["type_name"],
            "time?"
        );
    }

    #[tokio::test]
    async fn contract_repair_approval_recovery_marker_write_gate_persists_marker_when_enabled() {
        let mut request = valid_request();
        request.request_id = format!("approval-request:repair-draft:marker-{}", test_stamp());
        let preview = contract_repair_approval_record_preview(&request).unwrap();
        let approval_id = preview.approval_id.clone();
        let store_dir = std::env::temp_dir().join(format!(
            "quantpilot-contract-repair-recovery-marker-{}",
            test_stamp()
        ));
        std::fs::create_dir_all(&store_dir).expect("approval store should be created");
        let persistence_plan = contract_repair_approval_persistence_plan_preview(&preview);
        let persistence_path = contract_repair_approval_persistence_path_preview(&persistence_plan);
        let record_snapshot = contract_repair_approval_record_snapshot_preview(
            &preview,
            "approve",
            "user:0",
            "approve marker write",
        );
        let storage_readiness_gate = contract_repair_approval_storage_readiness_gate(
            &store_dir,
            &persistence_plan,
            &persistence_path,
            &record_snapshot,
        )
        .await;
        let runner_dry_run = ContractRepairApprovalApproveExecutionTransactionRunnerDryRun {
            status: "approve_execution_transaction_runner_ready".to_string(),
            action: "approve".to_string(),
            approval_id: approval_id.clone(),
            runner_enabled: true,
            admission_ready: true,
            transaction_plan_ready: true,
            commit_barrier_ready: true,
            recovery_marker_ready: true,
            rollback_plan_ready: true,
            commit_ready: true,
            would_start_runner: false,
            would_write_recovery_marker: false,
            would_commit_transaction: false,
            would_rollback_on_error: false,
            phase_order: vec![
                "write_recovery_marker".to_string(),
                "transition_review_state".to_string(),
                "persist_approval_record".to_string(),
                "emit_lifecycle_event".to_string(),
                "append_lifecycle_entry".to_string(),
                "write_contract_source".to_string(),
                "clear_recovery_marker".to_string(),
            ],
            rollback_order: vec![
                "restore_contract_source".to_string(),
                "restore_approval_record".to_string(),
                "mark_recovery_marker_rolled_back".to_string(),
            ],
            blocked_by: Vec::new(),
        };
        let marker_write_dry_run =
            contract_repair_approval_approve_execution_recovery_marker_write_dry_run(
                &approval_id,
                &storage_readiness_gate,
                &runner_dry_run,
            );
        let idempotency_precheck =
            contract_repair_approval_approve_execution_recovery_marker_idempotency_precheck(
                &store_dir,
                &marker_write_dry_run,
            )
            .await;

        let marker_write_result =
            contract_repair_approval_approve_execution_recovery_marker_write_with_gate(
                &store_dir,
                &marker_write_dry_run,
                &idempotency_precheck,
                &runner_dry_run,
                true,
            )
            .await;

        assert_eq!(
            marker_write_result.status,
            "approve_execution_recovery_marker_written"
        );
        assert!(marker_write_result.write_ready);
        assert!(marker_write_result.runner_ready);
        assert!(marker_write_result.would_write_marker);
        assert!(marker_write_result.would_touch_disk);
        assert!(marker_write_result.blocked_by.is_empty());

        let marker_path = store_dir.join(&marker_write_result.file_name);
        let persisted_marker = std::fs::read_to_string(&marker_path)
            .expect("recovery marker should be persisted when gate is enabled");
        let persisted_json: serde_json::Value =
            serde_json::from_str(&persisted_marker).expect("marker should be valid json");
        assert_eq!(persisted_json["approval_id"], approval_id);
        assert_eq!(
            persisted_json["marker_kind"],
            "approve_execution_recovery_marker"
        );
        assert_eq!(persisted_json["phase_order"][0], "write_recovery_marker");
        assert_eq!(
            persisted_json["rollback_order"][0],
            "restore_contract_source"
        );
    }

    #[tokio::test]
    async fn contract_repair_approval_durable_disk_executor_handlers_apply_happy_path() {
        let mut request = valid_request();
        request.request_id = format!(
            "approval-request:repair-draft:durable-executor-{}",
            test_stamp()
        );
        request.contract_source_ref.source_id = format!("graph-dual-ma-{}", test_stamp());
        let preview = contract_repair_approval_record_preview(&request).unwrap();
        let approval_id = preview.approval_id.clone();
        let mut durable_record = preview.clone();
        durable_record.review_state = RuntimeApprovalReviewState::Approved;
        durable_record.transient_review_status = "review_approve_executed".to_string();
        durable_record.transient_review_action = Some("approve".to_string());
        durable_record.transient_reviewer_id = Some("user:0".to_string());
        durable_record.transient_review_reason =
            Some("durable disk executor happy path".to_string());

        let store_dir = std::env::temp_dir().join(format!(
            "quantpilot-contract-repair-durable-executor-store-{}",
            test_stamp()
        ));
        let graph_store_dir = std::env::temp_dir().join(format!(
            "quantpilot-contract-repair-durable-executor-graph-{}",
            test_stamp()
        ));
        std::fs::create_dir_all(&store_dir).expect("approval store should be created");
        std::fs::create_dir_all(&graph_store_dir).expect("graph store should be created");
        let record_snapshot = contract_repair_approval_record_snapshot_preview(
            &preview,
            "approve",
            "user:0",
            "durable disk executor happy path",
        );
        let source_path = contract_repair_approval_contract_source_path(
            &graph_store_dir,
            &record_snapshot.contract_source_ref,
        );
        let source_json = json!({
            "schema_version": "quantpilot/machine-graph-contract/v1",
            "graph_id": record_snapshot.contract_source_ref.source_id.clone(),
            "machines": [
                {
                    "machine_id": "decision.dual_ma",
                    "memory": []
                }
            ],
            "event_catalog": {
                "events": []
            },
            "metadata": {
                "graph_version": "v4-test",
                "artifact_hash": "sha256:test-contract"
            }
        });
        std::fs::write(&source_path, source_json.to_string())
            .expect("source graph should be written");

        let persistence_plan = contract_repair_approval_persistence_plan_preview(&preview);
        let persistence_path = contract_repair_approval_persistence_path_preview(&persistence_plan);
        let storage_readiness_gate = contract_repair_approval_storage_readiness_gate(
            &store_dir,
            &persistence_plan,
            &persistence_path,
            &record_snapshot,
        )
        .await;
        let runner_dry_run = ContractRepairApprovalApproveExecutionTransactionRunnerDryRun {
            status: "approve_execution_transaction_runner_ready".to_string(),
            action: "approve".to_string(),
            approval_id: approval_id.clone(),
            runner_enabled: true,
            admission_ready: true,
            transaction_plan_ready: true,
            commit_barrier_ready: true,
            recovery_marker_ready: true,
            rollback_plan_ready: true,
            commit_ready: true,
            would_start_runner: false,
            would_write_recovery_marker: false,
            would_commit_transaction: false,
            would_rollback_on_error: false,
            phase_order: vec![
                "write_recovery_marker".to_string(),
                "transition_review_state".to_string(),
                "persist_approval_record".to_string(),
                "emit_lifecycle_event".to_string(),
                "append_lifecycle_entry".to_string(),
                "write_contract_source".to_string(),
                "clear_recovery_marker".to_string(),
            ],
            rollback_order: vec![
                "restore_contract_source".to_string(),
                "restore_approval_record".to_string(),
                "mark_recovery_marker_rolled_back".to_string(),
            ],
            blocked_by: Vec::new(),
        };
        let marker_write_dry_run =
            contract_repair_approval_approve_execution_recovery_marker_write_dry_run(
                &approval_id,
                &storage_readiness_gate,
                &runner_dry_run,
            );
        let idempotency_precheck =
            contract_repair_approval_approve_execution_recovery_marker_idempotency_precheck(
                &store_dir,
                &marker_write_dry_run,
            )
            .await;

        let source_resolution = contract_repair_approval_contract_source_resolution_dry_run(
            &graph_store_dir,
            &record_snapshot.contract_source_ref,
        )
        .await;
        let patch_plan = contract_repair_approval_contract_patch_plan_preview(&record_snapshot);
        let patch_apply_dry_run = contract_repair_approval_contract_patch_apply_dry_run(
            &graph_store_dir,
            &record_snapshot.contract_source_ref,
            &source_resolution,
            &patch_plan,
        )
        .await;
        let cleanup_phase_dry_run =
            ContractRepairApprovalApproveExecutionRunnerRecoveryMarkerCleanupPhaseDryRun {
                status: "approve_execution_runner_recovery_marker_cleanup_phase_ready_blocked"
                    .to_string(),
                action: "approve".to_string(),
                approval_id: approval_id.clone(),
                marker_key: marker_write_dry_run.marker_key.clone(),
                file_name: marker_write_dry_run.file_name.clone(),
                phase_sequence_ready: true,
                source_mutation_phase_ready: true,
                cleanup_phase_present: true,
                marker_persistence_plan_ready: true,
                marker_persistence_ready: true,
                cleanup_phase_enabled: false,
                cleanup_phase_ready: false,
                would_clear_recovery_marker: false,
                would_continue_to_commit: false,
                would_return_success: false,
                would_touch_disk: false,
                blocked_by: vec![
                    "approve_execution_runner_recovery_marker_cleanup_phase_enabled".to_string(),
                ],
            };
        let cleanup_phase_enablement_dry_run =
            ContractRepairApprovalApproveExecutionRunnerRecoveryMarkerCleanupPhaseEnablementDryRun {
                status:
                    "approve_execution_runner_recovery_marker_cleanup_phase_enablement_ready"
                        .to_string(),
                action: "approve".to_string(),
                approval_id: approval_id.clone(),
                marker_key: marker_write_dry_run.marker_key.clone(),
                file_name: marker_write_dry_run.file_name.clone(),
                switch_name:
                    "approve_execution_runner_recovery_marker_cleanup_phase_enabled".to_string(),
                phase_sequence_ready: true,
                source_mutation_phase_enablement_ready: true,
                source_mutation_phase_ready: true,
                cleanup_phase_present: true,
                marker_persistence_plan_ready: true,
                marker_persistence_ready: true,
                cleanup_phase_enabled: true,
                enablement_prerequisites_ready: true,
                cleanup_phase_enablement_ready: true,
                would_enable_cleanup_phase: false,
                would_clear_recovery_marker: false,
                would_continue_to_commit: false,
                would_return_success: false,
                would_touch_disk: false,
                required_gates: Vec::new(),
                passed_gates: Vec::new(),
                blocked_gates: Vec::new(),
            };

        let commit_phase_dry_run =
            ContractRepairApprovalApproveExecutionRunnerTransactionCommitPhaseDryRun {
                status: "approve_execution_runner_transaction_commit_phase_ready_blocked"
                    .to_string(),
                action: "approve".to_string(),
                approval_id: approval_id.clone(),
                phase_sequence_ready: true,
                cleanup_phase_ready: true,
                runner_plan_ready: true,
                runner_enabled: true,
                admission_ready: true,
                commit_barrier_ready: true,
                rollback_plan_ready: true,
                recovery_marker_persistence_plan_ready: true,
                recovery_marker_persistence_ready: true,
                commit_gate_enabled: true,
                commit_ready: true,
                commit_phase_enabled: false,
                commit_phase_ready: false,
                would_commit_transaction: false,
                would_return_success: false,
                would_touch_disk: false,
                blocked_by: vec![
                    "approve_execution_runner_transaction_commit_phase_enabled".to_string()
                ],
            };
        let commit_phase_enablement_dry_run =
            ContractRepairApprovalApproveExecutionRunnerTransactionCommitPhaseEnablementDryRun {
                status: "approve_execution_runner_transaction_commit_phase_enablement_ready"
                    .to_string(),
                action: "approve".to_string(),
                approval_id: approval_id.clone(),
                switch_name: "approve_execution_runner_transaction_commit_phase_enabled"
                    .to_string(),
                phase_sequence_ready: true,
                cleanup_phase_enablement_ready: true,
                cleanup_phase_ready: true,
                runner_plan_ready: true,
                runner_enabled: true,
                admission_ready: true,
                commit_barrier_ready: true,
                rollback_plan_ready: true,
                recovery_marker_persistence_plan_ready: true,
                recovery_marker_persistence_ready: true,
                commit_gate_enabled: true,
                commit_ready: true,
                commit_phase_enabled: true,
                enablement_prerequisites_ready: true,
                commit_phase_enablement_ready: true,
                would_enable_commit_phase: false,
                would_commit_transaction: false,
                would_return_success: false,
                would_touch_disk: false,
                required_gates: Vec::new(),
                passed_gates: Vec::new(),
                blocked_gates: Vec::new(),
            };
        let execution = contract_repair_approval_execute_durable_disk_application_handlers(
            &store_dir,
            &graph_store_dir,
            &durable_record,
            &marker_write_dry_run,
            &idempotency_precheck,
            &runner_dry_run,
            &record_snapshot.contract_source_ref,
            &source_resolution,
            &patch_plan,
            &patch_apply_dry_run,
            &cleanup_phase_dry_run,
            &cleanup_phase_enablement_dry_run,
            &commit_phase_dry_run,
            &commit_phase_enablement_dry_run,
            true,
        )
        .await;
        assert_eq!(
            execution.status,
            "approve_execution_durable_disk_application_executor_committed"
        );
        assert_eq!(
            execution.executor_name,
            "approve_execution_durable_disk_application_executor"
        );
        assert!(execution.executor_admitted);
        assert!(execution.recovery_marker_written);
        assert!(execution.approval_record_persisted);
        assert!(execution.contract_source_written);
        assert!(execution.recovery_marker_cleared);
        assert!(execution.transaction_committed);
        assert!(!execution.rollback_executed);
        assert!(execution.would_touch_disk);
        assert!(execution.blocked_by.is_empty());
        assert!(execution.rollback_receipts.is_empty());
        assert!(execution
            .execution_receipts
            .contains(&"write_recovery_marker:executed".to_string()));
        assert!(execution
            .execution_receipts
            .contains(&"persist_approval_record:executed".to_string()));
        assert!(execution
            .execution_receipts
            .contains(&"write_contract_source:executed".to_string()));
        assert!(execution
            .execution_receipts
            .contains(&"clear_recovery_marker:executed".to_string()));
        assert!(execution
            .execution_receipts
            .contains(&"commit_transaction:executed".to_string()));
        assert!(!store_dir.join(&marker_write_dry_run.file_name).exists());

        let persisted_record =
            load_contract_repair_approval_record_from_disk(&store_dir, &approval_id)
                .await
                .expect("record load should succeed")
                .expect("approval record should exist on disk");
        assert_eq!(
            persisted_record.review_state,
            RuntimeApprovalReviewState::Approved
        );
        assert_eq!(
            persisted_record.transient_review_status,
            "review_approve_executed"
        );

        let persisted_source = std::fs::read_to_string(&source_path)
            .expect("source graph should remain readable after durable executor");
        let persisted_json: serde_json::Value =
            serde_json::from_str(&persisted_source).expect("persisted source should stay json");
        assert_eq!(
            persisted_json["machines"][0]["memory"][0]["name"],
            "last_signal_at"
        );
        assert_eq!(
            persisted_json["machines"][0]["memory"][0]["type_name"],
            "time?"
        );
    }

    #[tokio::test]
    async fn contract_repair_approval_durable_disk_executor_rolls_back_record_and_marker_when_source_write_blocks(
    ) {
        let mut request = valid_request();
        request.request_id = format!(
            "approval-request:repair-draft:durable-executor-rollback-{}",
            test_stamp()
        );
        request.contract_source_ref.source_id = format!("graph-dual-ma-{}", test_stamp());
        let preview = contract_repair_approval_record_preview(&request).unwrap();
        let approval_id = preview.approval_id.clone();
        let mut durable_record = preview.clone();
        durable_record.review_state = RuntimeApprovalReviewState::Approved;
        durable_record.transient_review_status = "review_approve_executed".to_string();
        durable_record.transient_review_action = Some("approve".to_string());
        durable_record.transient_reviewer_id = Some("user:0".to_string());
        durable_record.transient_review_reason =
            Some("durable disk executor rollback path".to_string());

        let store_dir = std::env::temp_dir().join(format!(
            "quantpilot-contract-repair-durable-executor-rollback-store-{}",
            test_stamp()
        ));
        let graph_store_dir = std::env::temp_dir().join(format!(
            "quantpilot-contract-repair-durable-executor-rollback-graph-{}",
            test_stamp()
        ));
        std::fs::create_dir_all(&store_dir).expect("approval store should be created");
        std::fs::create_dir_all(&graph_store_dir).expect("graph store should be created");
        let record_snapshot = contract_repair_approval_record_snapshot_preview(
            &preview,
            "approve",
            "user:0",
            "durable disk executor rollback path",
        );
        let source_path = contract_repair_approval_contract_source_path(
            &graph_store_dir,
            &record_snapshot.contract_source_ref,
        );
        let source_json = json!({
            "schema_version": "quantpilot/machine-graph-contract/v1",
            "graph_id": record_snapshot.contract_source_ref.source_id.clone(),
            "machines": [
                {
                    "machine_id": "decision.dual_ma",
                    "memory": []
                }
            ],
            "event_catalog": {
                "events": []
            },
            "metadata": {
                "graph_version": "v4-test",
                "artifact_hash": "sha256:test-contract"
            }
        });
        std::fs::write(&source_path, source_json.to_string())
            .expect("source graph should be written");

        let persistence_plan = contract_repair_approval_persistence_plan_preview(&preview);
        let persistence_path = contract_repair_approval_persistence_path_preview(&persistence_plan);
        let storage_readiness_gate = contract_repair_approval_storage_readiness_gate(
            &store_dir,
            &persistence_plan,
            &persistence_path,
            &record_snapshot,
        )
        .await;
        let runner_dry_run = ContractRepairApprovalApproveExecutionTransactionRunnerDryRun {
            status: "approve_execution_transaction_runner_ready".to_string(),
            action: "approve".to_string(),
            approval_id: approval_id.clone(),
            runner_enabled: true,
            admission_ready: true,
            transaction_plan_ready: true,
            commit_barrier_ready: true,
            recovery_marker_ready: true,
            rollback_plan_ready: true,
            commit_ready: true,
            would_start_runner: false,
            would_write_recovery_marker: false,
            would_commit_transaction: false,
            would_rollback_on_error: false,
            phase_order: vec![
                "write_recovery_marker".to_string(),
                "transition_review_state".to_string(),
                "persist_approval_record".to_string(),
                "emit_lifecycle_event".to_string(),
                "append_lifecycle_entry".to_string(),
                "write_contract_source".to_string(),
                "clear_recovery_marker".to_string(),
            ],
            rollback_order: vec![
                "restore_contract_source".to_string(),
                "restore_approval_record".to_string(),
                "mark_recovery_marker_rolled_back".to_string(),
            ],
            blocked_by: Vec::new(),
        };
        let marker_write_dry_run =
            contract_repair_approval_approve_execution_recovery_marker_write_dry_run(
                &approval_id,
                &storage_readiness_gate,
                &runner_dry_run,
            );
        let idempotency_precheck =
            contract_repair_approval_approve_execution_recovery_marker_idempotency_precheck(
                &store_dir,
                &marker_write_dry_run,
            )
            .await;

        let source_resolution = contract_repair_approval_contract_source_resolution_dry_run(
            &graph_store_dir,
            &record_snapshot.contract_source_ref,
        )
        .await;
        let patch_plan = contract_repair_approval_contract_patch_plan_preview(&record_snapshot);
        let patch_apply_dry_run = contract_repair_approval_contract_patch_apply_dry_run(
            &graph_store_dir,
            &record_snapshot.contract_source_ref,
            &source_resolution,
            &patch_plan,
        )
        .await;
        let blocked_patch_apply_dry_run = ContractRepairApprovalContractPatchApplyDryRun {
            status: "contract_patch_apply_blocked".to_string(),
            apply_ready: false,
            applied_operation_count: 0,
            applied_selectors: Vec::new(),
            blocked_by: vec!["forced_source_write_failure".to_string()],
            ..patch_apply_dry_run
        };
        let cleanup_phase_dry_run =
            ContractRepairApprovalApproveExecutionRunnerRecoveryMarkerCleanupPhaseDryRun {
                status: "approve_execution_runner_recovery_marker_cleanup_phase_ready_blocked"
                    .to_string(),
                action: "approve".to_string(),
                approval_id: approval_id.clone(),
                marker_key: marker_write_dry_run.marker_key.clone(),
                file_name: marker_write_dry_run.file_name.clone(),
                phase_sequence_ready: true,
                source_mutation_phase_ready: true,
                cleanup_phase_present: true,
                marker_persistence_plan_ready: true,
                marker_persistence_ready: true,
                cleanup_phase_enabled: false,
                cleanup_phase_ready: false,
                would_clear_recovery_marker: false,
                would_continue_to_commit: false,
                would_return_success: false,
                would_touch_disk: false,
                blocked_by: vec![
                    "approve_execution_runner_recovery_marker_cleanup_phase_enabled".to_string(),
                ],
            };
        let cleanup_phase_enablement_dry_run =
            ContractRepairApprovalApproveExecutionRunnerRecoveryMarkerCleanupPhaseEnablementDryRun {
                status:
                    "approve_execution_runner_recovery_marker_cleanup_phase_enablement_ready"
                        .to_string(),
                action: "approve".to_string(),
                approval_id: approval_id.clone(),
                marker_key: marker_write_dry_run.marker_key.clone(),
                file_name: marker_write_dry_run.file_name.clone(),
                switch_name:
                    "approve_execution_runner_recovery_marker_cleanup_phase_enabled".to_string(),
                phase_sequence_ready: true,
                source_mutation_phase_enablement_ready: true,
                source_mutation_phase_ready: true,
                cleanup_phase_present: true,
                marker_persistence_plan_ready: true,
                marker_persistence_ready: true,
                cleanup_phase_enabled: true,
                enablement_prerequisites_ready: true,
                cleanup_phase_enablement_ready: true,
                would_enable_cleanup_phase: false,
                would_clear_recovery_marker: false,
                would_continue_to_commit: false,
                would_return_success: false,
                would_touch_disk: false,
                required_gates: Vec::new(),
                passed_gates: Vec::new(),
                blocked_gates: Vec::new(),
            };
        let commit_phase_dry_run =
            ContractRepairApprovalApproveExecutionRunnerTransactionCommitPhaseDryRun {
                status: "approve_execution_runner_transaction_commit_phase_ready_blocked"
                    .to_string(),
                action: "approve".to_string(),
                approval_id: approval_id.clone(),
                phase_sequence_ready: true,
                cleanup_phase_ready: true,
                runner_plan_ready: true,
                runner_enabled: true,
                admission_ready: true,
                commit_barrier_ready: true,
                rollback_plan_ready: true,
                recovery_marker_persistence_plan_ready: true,
                recovery_marker_persistence_ready: true,
                commit_gate_enabled: true,
                commit_ready: true,
                commit_phase_enabled: false,
                commit_phase_ready: false,
                would_commit_transaction: false,
                would_return_success: false,
                would_touch_disk: false,
                blocked_by: vec![
                    "approve_execution_runner_transaction_commit_phase_enabled".to_string()
                ],
            };
        let commit_phase_enablement_dry_run =
            ContractRepairApprovalApproveExecutionRunnerTransactionCommitPhaseEnablementDryRun {
                status: "approve_execution_runner_transaction_commit_phase_enablement_ready"
                    .to_string(),
                action: "approve".to_string(),
                approval_id: approval_id.clone(),
                switch_name: "approve_execution_runner_transaction_commit_phase_enabled"
                    .to_string(),
                phase_sequence_ready: true,
                cleanup_phase_enablement_ready: true,
                cleanup_phase_ready: true,
                runner_plan_ready: true,
                runner_enabled: true,
                admission_ready: true,
                commit_barrier_ready: true,
                rollback_plan_ready: true,
                recovery_marker_persistence_plan_ready: true,
                recovery_marker_persistence_ready: true,
                commit_gate_enabled: true,
                commit_ready: true,
                commit_phase_enabled: true,
                enablement_prerequisites_ready: true,
                commit_phase_enablement_ready: true,
                would_enable_commit_phase: false,
                would_commit_transaction: false,
                would_return_success: false,
                would_touch_disk: false,
                required_gates: Vec::new(),
                passed_gates: Vec::new(),
                blocked_gates: Vec::new(),
            };
        let execution = contract_repair_approval_execute_durable_disk_application_handlers(
            &store_dir,
            &graph_store_dir,
            &durable_record,
            &marker_write_dry_run,
            &idempotency_precheck,
            &runner_dry_run,
            &record_snapshot.contract_source_ref,
            &source_resolution,
            &patch_plan,
            &blocked_patch_apply_dry_run,
            &cleanup_phase_dry_run,
            &cleanup_phase_enablement_dry_run,
            &commit_phase_dry_run,
            &commit_phase_enablement_dry_run,
            true,
        )
        .await;

        assert_eq!(
            execution.status,
            "approve_execution_durable_disk_application_executor_write_contract_source_blocked"
        );
        assert!(execution.recovery_marker_written);
        assert!(execution.approval_record_persisted);
        assert!(!execution.contract_source_written);
        assert!(!execution.recovery_marker_cleared);
        assert!(!execution.transaction_committed);
        assert!(execution.rollback_executed);
        assert!(execution.would_touch_disk);
        assert!(execution
            .blocked_by
            .contains(&"write_contract_source".to_string()));
        assert!(execution
            .blocked_by
            .contains(&"contract_patch_apply_ready".to_string()));
        assert!(execution
            .rollback_receipts
            .contains(&"restore_approval_record:removed_created_record".to_string()));
        assert!(execution
            .rollback_receipts
            .contains(&"mark_recovery_marker_rolled_back:executed".to_string()));
        assert!(execution
            .rollback_receipts
            .contains(&"rollback_completed:write_contract_source".to_string()));
        assert!(!store_dir.join(&marker_write_dry_run.file_name).exists());
        assert!(
            load_contract_repair_approval_record_from_disk(&store_dir, &approval_id)
                .await
                .expect("record load after rollback should succeed")
                .is_none()
        );
        let persisted_source = std::fs::read_to_string(&source_path)
            .expect("source graph should remain readable after rollback");
        let persisted_json: serde_json::Value =
            serde_json::from_str(&persisted_source).expect("persisted source should stay json");
        assert_eq!(persisted_json, source_json);
    }

    #[tokio::test]
    async fn contract_repair_approval_durable_disk_executor_restores_source_record_and_marker_when_cleanup_blocks(
    ) {
        let mut request = valid_request();
        request.request_id = format!(
            "approval-request:repair-draft:durable-executor-cleanup-rollback-{}",
            test_stamp()
        );
        request.contract_source_ref.source_id = format!("graph-dual-ma-{}", test_stamp());
        let preview = contract_repair_approval_record_preview(&request).unwrap();
        let approval_id = preview.approval_id.clone();
        let mut durable_record = preview.clone();
        durable_record.review_state = RuntimeApprovalReviewState::Approved;
        durable_record.transient_review_status = "review_approve_executed".to_string();
        durable_record.transient_review_action = Some("approve".to_string());
        durable_record.transient_reviewer_id = Some("user:0".to_string());
        durable_record.transient_review_reason =
            Some("durable disk executor cleanup rollback path".to_string());

        let store_dir = std::env::temp_dir().join(format!(
            "quantpilot-contract-repair-durable-executor-cleanup-rollback-store-{}",
            test_stamp()
        ));
        let graph_store_dir = std::env::temp_dir().join(format!(
            "quantpilot-contract-repair-durable-executor-cleanup-rollback-graph-{}",
            test_stamp()
        ));
        std::fs::create_dir_all(&store_dir).expect("approval store should be created");
        std::fs::create_dir_all(&graph_store_dir).expect("graph store should be created");
        let record_snapshot = contract_repair_approval_record_snapshot_preview(
            &preview,
            "approve",
            "user:0",
            "durable disk executor cleanup rollback path",
        );
        let source_path = contract_repair_approval_contract_source_path(
            &graph_store_dir,
            &record_snapshot.contract_source_ref,
        );
        let source_json = json!({
            "schema_version": "quantpilot/machine-graph-contract/v1",
            "graph_id": record_snapshot.contract_source_ref.source_id.clone(),
            "machines": [
                {
                    "machine_id": "decision.dual_ma",
                    "memory": []
                }
            ],
            "event_catalog": {
                "events": []
            },
            "metadata": {
                "graph_version": "v4-test",
                "artifact_hash": "sha256:test-contract"
            }
        });
        std::fs::write(&source_path, source_json.to_string())
            .expect("source graph should be written");

        let persistence_plan = contract_repair_approval_persistence_plan_preview(&preview);
        let persistence_path = contract_repair_approval_persistence_path_preview(&persistence_plan);
        let storage_readiness_gate = contract_repair_approval_storage_readiness_gate(
            &store_dir,
            &persistence_plan,
            &persistence_path,
            &record_snapshot,
        )
        .await;
        let runner_dry_run = ContractRepairApprovalApproveExecutionTransactionRunnerDryRun {
            status: "approve_execution_transaction_runner_ready".to_string(),
            action: "approve".to_string(),
            approval_id: approval_id.clone(),
            runner_enabled: true,
            admission_ready: true,
            transaction_plan_ready: true,
            commit_barrier_ready: true,
            recovery_marker_ready: true,
            rollback_plan_ready: true,
            commit_ready: true,
            would_start_runner: false,
            would_write_recovery_marker: false,
            would_commit_transaction: false,
            would_rollback_on_error: false,
            phase_order: vec![
                "write_recovery_marker".to_string(),
                "transition_review_state".to_string(),
                "persist_approval_record".to_string(),
                "emit_lifecycle_event".to_string(),
                "append_lifecycle_entry".to_string(),
                "write_contract_source".to_string(),
                "clear_recovery_marker".to_string(),
            ],
            rollback_order: vec![
                "restore_contract_source".to_string(),
                "restore_approval_record".to_string(),
                "mark_recovery_marker_rolled_back".to_string(),
            ],
            blocked_by: Vec::new(),
        };
        let marker_write_dry_run =
            contract_repair_approval_approve_execution_recovery_marker_write_dry_run(
                &approval_id,
                &storage_readiness_gate,
                &runner_dry_run,
            );
        let idempotency_precheck =
            contract_repair_approval_approve_execution_recovery_marker_idempotency_precheck(
                &store_dir,
                &marker_write_dry_run,
            )
            .await;

        let source_resolution = contract_repair_approval_contract_source_resolution_dry_run(
            &graph_store_dir,
            &record_snapshot.contract_source_ref,
        )
        .await;
        let patch_plan = contract_repair_approval_contract_patch_plan_preview(&record_snapshot);
        let patch_apply_dry_run = contract_repair_approval_contract_patch_apply_dry_run(
            &graph_store_dir,
            &record_snapshot.contract_source_ref,
            &source_resolution,
            &patch_plan,
        )
        .await;
        let cleanup_missing_file_name = format!("missing-{}", marker_write_dry_run.file_name);
        let cleanup_phase_dry_run =
            ContractRepairApprovalApproveExecutionRunnerRecoveryMarkerCleanupPhaseDryRun {
                status: "approve_execution_runner_recovery_marker_cleanup_phase_ready_blocked"
                    .to_string(),
                action: "approve".to_string(),
                approval_id: approval_id.clone(),
                marker_key: marker_write_dry_run.marker_key.clone(),
                file_name: cleanup_missing_file_name.clone(),
                phase_sequence_ready: true,
                source_mutation_phase_ready: true,
                cleanup_phase_present: true,
                marker_persistence_plan_ready: true,
                marker_persistence_ready: true,
                cleanup_phase_enabled: false,
                cleanup_phase_ready: false,
                would_clear_recovery_marker: false,
                would_continue_to_commit: false,
                would_return_success: false,
                would_touch_disk: false,
                blocked_by: vec![
                    "approve_execution_runner_recovery_marker_cleanup_phase_enabled".to_string(),
                ],
            };
        let cleanup_phase_enablement_dry_run =
            ContractRepairApprovalApproveExecutionRunnerRecoveryMarkerCleanupPhaseEnablementDryRun {
                status:
                    "approve_execution_runner_recovery_marker_cleanup_phase_enablement_ready"
                        .to_string(),
                action: "approve".to_string(),
                approval_id: approval_id.clone(),
                marker_key: marker_write_dry_run.marker_key.clone(),
                file_name: cleanup_missing_file_name,
                switch_name:
                    "approve_execution_runner_recovery_marker_cleanup_phase_enabled".to_string(),
                phase_sequence_ready: true,
                source_mutation_phase_enablement_ready: true,
                source_mutation_phase_ready: true,
                cleanup_phase_present: true,
                marker_persistence_plan_ready: true,
                marker_persistence_ready: true,
                cleanup_phase_enabled: true,
                enablement_prerequisites_ready: true,
                cleanup_phase_enablement_ready: true,
                would_enable_cleanup_phase: false,
                would_clear_recovery_marker: false,
                would_continue_to_commit: false,
                would_return_success: false,
                would_touch_disk: false,
                required_gates: Vec::new(),
                passed_gates: Vec::new(),
                blocked_gates: Vec::new(),
            };
        let commit_phase_dry_run =
            ContractRepairApprovalApproveExecutionRunnerTransactionCommitPhaseDryRun {
                status: "approve_execution_runner_transaction_commit_phase_ready_blocked"
                    .to_string(),
                action: "approve".to_string(),
                approval_id: approval_id.clone(),
                phase_sequence_ready: true,
                cleanup_phase_ready: true,
                runner_plan_ready: true,
                runner_enabled: true,
                admission_ready: true,
                commit_barrier_ready: true,
                rollback_plan_ready: true,
                recovery_marker_persistence_plan_ready: true,
                recovery_marker_persistence_ready: true,
                commit_gate_enabled: true,
                commit_ready: true,
                commit_phase_enabled: false,
                commit_phase_ready: false,
                would_commit_transaction: false,
                would_return_success: false,
                would_touch_disk: false,
                blocked_by: vec![
                    "approve_execution_runner_transaction_commit_phase_enabled".to_string()
                ],
            };
        let commit_phase_enablement_dry_run =
            ContractRepairApprovalApproveExecutionRunnerTransactionCommitPhaseEnablementDryRun {
                status: "approve_execution_runner_transaction_commit_phase_enablement_ready"
                    .to_string(),
                action: "approve".to_string(),
                approval_id: approval_id.clone(),
                switch_name: "approve_execution_runner_transaction_commit_phase_enabled"
                    .to_string(),
                phase_sequence_ready: true,
                cleanup_phase_enablement_ready: true,
                cleanup_phase_ready: true,
                runner_plan_ready: true,
                runner_enabled: true,
                admission_ready: true,
                commit_barrier_ready: true,
                rollback_plan_ready: true,
                recovery_marker_persistence_plan_ready: true,
                recovery_marker_persistence_ready: true,
                commit_gate_enabled: true,
                commit_ready: true,
                commit_phase_enabled: true,
                enablement_prerequisites_ready: true,
                commit_phase_enablement_ready: true,
                would_enable_commit_phase: false,
                would_commit_transaction: false,
                would_return_success: false,
                would_touch_disk: false,
                required_gates: Vec::new(),
                passed_gates: Vec::new(),
                blocked_gates: Vec::new(),
            };
        let execution = contract_repair_approval_execute_durable_disk_application_handlers(
            &store_dir,
            &graph_store_dir,
            &durable_record,
            &marker_write_dry_run,
            &idempotency_precheck,
            &runner_dry_run,
            &record_snapshot.contract_source_ref,
            &source_resolution,
            &patch_plan,
            &patch_apply_dry_run,
            &cleanup_phase_dry_run,
            &cleanup_phase_enablement_dry_run,
            &commit_phase_dry_run,
            &commit_phase_enablement_dry_run,
            true,
        )
        .await;

        assert_eq!(
            execution.status,
            "approve_execution_durable_disk_application_executor_clear_recovery_marker_blocked"
        );
        assert!(execution.recovery_marker_written);
        assert!(execution.approval_record_persisted);
        assert!(execution.contract_source_written);
        assert!(!execution.recovery_marker_cleared);
        assert!(!execution.transaction_committed);
        assert!(execution.rollback_executed);
        assert!(execution.would_touch_disk);
        assert!(execution
            .execution_receipts
            .contains(&"write_contract_source:executed".to_string()));
        assert!(execution
            .blocked_by
            .contains(&"clear_recovery_marker".to_string()));
        assert!(execution
            .blocked_by
            .contains(&"recovery_marker_file_exists".to_string()));
        assert!(execution
            .rollback_receipts
            .contains(&"restore_contract_source:executed".to_string()));
        assert!(execution
            .rollback_receipts
            .contains(&"restore_approval_record:removed_created_record".to_string()));
        assert!(execution
            .rollback_receipts
            .contains(&"mark_recovery_marker_rolled_back:executed".to_string()));
        assert!(execution
            .rollback_receipts
            .contains(&"rollback_completed:clear_recovery_marker".to_string()));
        assert!(!store_dir.join(&marker_write_dry_run.file_name).exists());
        assert!(
            load_contract_repair_approval_record_from_disk(&store_dir, &approval_id)
                .await
                .expect("record load after cleanup rollback should succeed")
                .is_none()
        );
        let persisted_source = std::fs::read_to_string(&source_path)
            .expect("source graph should remain readable after cleanup rollback");
        let persisted_json: serde_json::Value =
            serde_json::from_str(&persisted_source).expect("persisted source should stay json");
        assert_eq!(persisted_json, source_json);
    }

    #[tokio::test]
    async fn contract_repair_approval_durable_disk_executor_restores_source_and_record_when_commit_blocks(
    ) {
        let mut request = valid_request();
        request.request_id = format!(
            "approval-request:repair-draft:durable-executor-commit-rollback-{}",
            test_stamp()
        );
        request.contract_source_ref.source_id = format!("graph-dual-ma-{}", test_stamp());
        let preview = contract_repair_approval_record_preview(&request).unwrap();
        let approval_id = preview.approval_id.clone();
        let mut durable_record = preview.clone();
        durable_record.review_state = RuntimeApprovalReviewState::Approved;
        durable_record.transient_review_status = "review_approve_executed".to_string();
        durable_record.transient_review_action = Some("approve".to_string());
        durable_record.transient_reviewer_id = Some("user:0".to_string());
        durable_record.transient_review_reason =
            Some("durable disk executor commit rollback path".to_string());

        let store_dir = std::env::temp_dir().join(format!(
            "quantpilot-contract-repair-durable-executor-commit-rollback-store-{}",
            test_stamp()
        ));
        let graph_store_dir = std::env::temp_dir().join(format!(
            "quantpilot-contract-repair-durable-executor-commit-rollback-graph-{}",
            test_stamp()
        ));
        std::fs::create_dir_all(&store_dir).expect("approval store should be created");
        std::fs::create_dir_all(&graph_store_dir).expect("graph store should be created");
        let record_snapshot = contract_repair_approval_record_snapshot_preview(
            &preview,
            "approve",
            "user:0",
            "durable disk executor commit rollback path",
        );
        let source_path = contract_repair_approval_contract_source_path(
            &graph_store_dir,
            &record_snapshot.contract_source_ref,
        );
        let source_json = json!({
            "schema_version": "quantpilot/machine-graph-contract/v1",
            "graph_id": record_snapshot.contract_source_ref.source_id.clone(),
            "machines": [
                {
                    "machine_id": "decision.dual_ma",
                    "memory": []
                }
            ],
            "event_catalog": {
                "events": []
            },
            "metadata": {
                "graph_version": "v4-test",
                "artifact_hash": "sha256:test-contract"
            }
        });
        std::fs::write(&source_path, source_json.to_string())
            .expect("source graph should be written");

        let persistence_plan = contract_repair_approval_persistence_plan_preview(&preview);
        let persistence_path = contract_repair_approval_persistence_path_preview(&persistence_plan);
        let storage_readiness_gate = contract_repair_approval_storage_readiness_gate(
            &store_dir,
            &persistence_plan,
            &persistence_path,
            &record_snapshot,
        )
        .await;
        let runner_dry_run = ContractRepairApprovalApproveExecutionTransactionRunnerDryRun {
            status: "approve_execution_transaction_runner_ready".to_string(),
            action: "approve".to_string(),
            approval_id: approval_id.clone(),
            runner_enabled: true,
            admission_ready: true,
            transaction_plan_ready: true,
            commit_barrier_ready: true,
            recovery_marker_ready: true,
            rollback_plan_ready: true,
            commit_ready: true,
            would_start_runner: false,
            would_write_recovery_marker: false,
            would_commit_transaction: false,
            would_rollback_on_error: false,
            phase_order: vec![
                "write_recovery_marker".to_string(),
                "transition_review_state".to_string(),
                "persist_approval_record".to_string(),
                "emit_lifecycle_event".to_string(),
                "append_lifecycle_entry".to_string(),
                "write_contract_source".to_string(),
                "clear_recovery_marker".to_string(),
            ],
            rollback_order: vec![
                "restore_contract_source".to_string(),
                "restore_approval_record".to_string(),
                "mark_recovery_marker_rolled_back".to_string(),
            ],
            blocked_by: Vec::new(),
        };
        let marker_write_dry_run =
            contract_repair_approval_approve_execution_recovery_marker_write_dry_run(
                &approval_id,
                &storage_readiness_gate,
                &runner_dry_run,
            );
        let idempotency_precheck =
            contract_repair_approval_approve_execution_recovery_marker_idempotency_precheck(
                &store_dir,
                &marker_write_dry_run,
            )
            .await;

        let source_resolution = contract_repair_approval_contract_source_resolution_dry_run(
            &graph_store_dir,
            &record_snapshot.contract_source_ref,
        )
        .await;
        let patch_plan = contract_repair_approval_contract_patch_plan_preview(&record_snapshot);
        let patch_apply_dry_run = contract_repair_approval_contract_patch_apply_dry_run(
            &graph_store_dir,
            &record_snapshot.contract_source_ref,
            &source_resolution,
            &patch_plan,
        )
        .await;
        let cleanup_phase_dry_run =
            ContractRepairApprovalApproveExecutionRunnerRecoveryMarkerCleanupPhaseDryRun {
                status: "approve_execution_runner_recovery_marker_cleanup_phase_ready_blocked"
                    .to_string(),
                action: "approve".to_string(),
                approval_id: approval_id.clone(),
                marker_key: marker_write_dry_run.marker_key.clone(),
                file_name: marker_write_dry_run.file_name.clone(),
                phase_sequence_ready: true,
                source_mutation_phase_ready: true,
                cleanup_phase_present: true,
                marker_persistence_plan_ready: true,
                marker_persistence_ready: true,
                cleanup_phase_enabled: false,
                cleanup_phase_ready: false,
                would_clear_recovery_marker: false,
                would_continue_to_commit: false,
                would_return_success: false,
                would_touch_disk: false,
                blocked_by: vec![
                    "approve_execution_runner_recovery_marker_cleanup_phase_enabled".to_string(),
                ],
            };
        let cleanup_phase_enablement_dry_run =
            ContractRepairApprovalApproveExecutionRunnerRecoveryMarkerCleanupPhaseEnablementDryRun {
                status:
                    "approve_execution_runner_recovery_marker_cleanup_phase_enablement_ready"
                        .to_string(),
                action: "approve".to_string(),
                approval_id: approval_id.clone(),
                marker_key: marker_write_dry_run.marker_key.clone(),
                file_name: marker_write_dry_run.file_name.clone(),
                switch_name:
                    "approve_execution_runner_recovery_marker_cleanup_phase_enabled".to_string(),
                phase_sequence_ready: true,
                source_mutation_phase_enablement_ready: true,
                source_mutation_phase_ready: true,
                cleanup_phase_present: true,
                marker_persistence_plan_ready: true,
                marker_persistence_ready: true,
                cleanup_phase_enabled: true,
                enablement_prerequisites_ready: true,
                cleanup_phase_enablement_ready: true,
                would_enable_cleanup_phase: false,
                would_clear_recovery_marker: false,
                would_continue_to_commit: false,
                would_return_success: false,
                would_touch_disk: false,
                required_gates: Vec::new(),
                passed_gates: Vec::new(),
                blocked_gates: Vec::new(),
            };
        let commit_phase_dry_run =
            ContractRepairApprovalApproveExecutionRunnerTransactionCommitPhaseDryRun {
                status: "approve_execution_runner_transaction_commit_phase_ready_blocked"
                    .to_string(),
                action: "approve".to_string(),
                approval_id: approval_id.clone(),
                phase_sequence_ready: true,
                cleanup_phase_ready: true,
                runner_plan_ready: true,
                runner_enabled: true,
                admission_ready: true,
                commit_barrier_ready: true,
                rollback_plan_ready: true,
                recovery_marker_persistence_plan_ready: true,
                recovery_marker_persistence_ready: true,
                commit_gate_enabled: true,
                commit_ready: false,
                commit_phase_enabled: false,
                commit_phase_ready: false,
                would_commit_transaction: false,
                would_return_success: false,
                would_touch_disk: false,
                blocked_by: vec![
                    "approve_execution_transaction_commit_ready".to_string(),
                    "approve_execution_runner_transaction_commit_phase_enabled".to_string(),
                ],
            };
        let commit_phase_enablement_dry_run =
            ContractRepairApprovalApproveExecutionRunnerTransactionCommitPhaseEnablementDryRun {
                status:
                    "approve_execution_runner_transaction_commit_phase_enablement_ready_blocked"
                        .to_string(),
                action: "approve".to_string(),
                approval_id: approval_id.clone(),
                switch_name: "approve_execution_runner_transaction_commit_phase_enabled"
                    .to_string(),
                phase_sequence_ready: true,
                cleanup_phase_enablement_ready: true,
                cleanup_phase_ready: true,
                runner_plan_ready: true,
                runner_enabled: true,
                admission_ready: true,
                commit_barrier_ready: true,
                rollback_plan_ready: true,
                recovery_marker_persistence_plan_ready: true,
                recovery_marker_persistence_ready: true,
                commit_gate_enabled: true,
                commit_ready: false,
                commit_phase_enabled: true,
                enablement_prerequisites_ready: false,
                commit_phase_enablement_ready: false,
                would_enable_commit_phase: false,
                would_commit_transaction: false,
                would_return_success: false,
                would_touch_disk: false,
                required_gates: Vec::new(),
                passed_gates: Vec::new(),
                blocked_gates: vec!["approve_execution_transaction_commit_ready".to_string()],
            };
        let execution = contract_repair_approval_execute_durable_disk_application_handlers(
            &store_dir,
            &graph_store_dir,
            &durable_record,
            &marker_write_dry_run,
            &idempotency_precheck,
            &runner_dry_run,
            &record_snapshot.contract_source_ref,
            &source_resolution,
            &patch_plan,
            &patch_apply_dry_run,
            &cleanup_phase_dry_run,
            &cleanup_phase_enablement_dry_run,
            &commit_phase_dry_run,
            &commit_phase_enablement_dry_run,
            true,
        )
        .await;

        assert_eq!(
            execution.status,
            "approve_execution_durable_disk_application_executor_commit_transaction_blocked"
        );
        assert!(execution.recovery_marker_written);
        assert!(execution.approval_record_persisted);
        assert!(execution.contract_source_written);
        assert!(execution.recovery_marker_cleared);
        assert!(!execution.transaction_committed);
        assert!(execution.rollback_executed);
        assert!(execution.would_touch_disk);
        assert!(execution
            .execution_receipts
            .contains(&"clear_recovery_marker:executed".to_string()));
        assert!(execution
            .blocked_by
            .contains(&"commit_transaction".to_string()));
        assert!(execution
            .blocked_by
            .contains(&"approve_execution_transaction_commit_ready".to_string()));
        assert!(execution
            .rollback_receipts
            .contains(&"restore_contract_source:executed".to_string()));
        assert!(execution
            .rollback_receipts
            .contains(&"restore_approval_record:removed_created_record".to_string()));
        assert!(execution
            .rollback_receipts
            .contains(&"rollback_completed:commit_transaction".to_string()));
        assert!(!execution
            .rollback_receipts
            .contains(&"mark_recovery_marker_rolled_back:executed".to_string()));
        let endpoint_execution =
            contract_repair_approval_durable_disk_application_execution_response(
                &execution, true, true,
            );
        assert_eq!(
            endpoint_execution.status,
            "approve_execution_durable_disk_application_executor_commit_transaction_blocked"
        );
        assert!(endpoint_execution.endpoint_helper_execution_connected);
        assert!(endpoint_execution.endpoint_helper_would_execute);
        assert!(endpoint_execution.executor_admitted);
        assert!(endpoint_execution.recovery_marker_written);
        assert!(endpoint_execution.approval_record_persisted);
        assert!(endpoint_execution.contract_source_written);
        assert!(endpoint_execution.recovery_marker_cleared);
        assert!(!endpoint_execution.transaction_committed);
        assert!(endpoint_execution.rollback_executed);
        assert!(endpoint_execution.would_touch_disk);
        assert!(endpoint_execution
            .blocked_by
            .contains(&"commit_transaction".to_string()));
        assert!(endpoint_execution
            .rollback_receipts
            .contains(&"restore_contract_source:executed".to_string()));
        assert!(endpoint_execution
            .rollback_receipts
            .contains(&"restore_approval_record:removed_created_record".to_string()));
        assert!(endpoint_execution
            .rollback_receipts
            .contains(&"rollback_completed:commit_transaction".to_string()));
        assert!(!store_dir.join(&marker_write_dry_run.file_name).exists());
        assert!(
            load_contract_repair_approval_record_from_disk(&store_dir, &approval_id)
                .await
                .expect("record load after commit rollback should succeed")
                .is_none()
        );
        let persisted_source = std::fs::read_to_string(&source_path)
            .expect("source graph should remain readable after commit rollback");
        let persisted_json: serde_json::Value =
            serde_json::from_str(&persisted_source).expect("persisted source should stay json");
        assert_eq!(persisted_json, source_json);
    }

    #[tokio::test]
    async fn contract_repair_approval_recovery_marker_cleanup_gate_deletes_marker_when_enabled() {
        let approval_id = format!("contract-repair-apr-cleanup-{}", test_stamp());
        let store_dir = std::env::temp_dir().join(format!(
            "quantpilot-contract-repair-cleanup-marker-{}",
            test_stamp()
        ));
        std::fs::create_dir_all(&store_dir).expect("approval store should be created");
        let file_name = format!("{approval_id}-recovery-marker.json");
        let marker_path = store_dir.join(&file_name);
        std::fs::write(
            &marker_path,
            r#"{"schema_version":"quantpilot/contract-repair-approve-execution-recovery-marker/v1"}"#,
        )
        .expect("marker file should be writable");

        let cleanup_phase_dry_run =
            ContractRepairApprovalApproveExecutionRunnerRecoveryMarkerCleanupPhaseDryRun {
                status: "approve_execution_runner_recovery_marker_cleanup_phase_ready_blocked"
                    .to_string(),
                action: "approve".to_string(),
                approval_id: approval_id.clone(),
                marker_key: approval_id.clone(),
                file_name: file_name.clone(),
                phase_sequence_ready: true,
                source_mutation_phase_ready: true,
                cleanup_phase_present: true,
                marker_persistence_plan_ready: true,
                marker_persistence_ready: true,
                cleanup_phase_enabled: false,
                cleanup_phase_ready: false,
                would_clear_recovery_marker: false,
                would_continue_to_commit: false,
                would_return_success: false,
                would_touch_disk: false,
                blocked_by: vec![
                    "approve_execution_runner_recovery_marker_cleanup_phase_enabled".to_string(),
                ],
            };
        let cleanup_phase_enablement_dry_run =
            ContractRepairApprovalApproveExecutionRunnerRecoveryMarkerCleanupPhaseEnablementDryRun {
                status:
                    "approve_execution_runner_recovery_marker_cleanup_phase_enablement_ready_blocked"
                        .to_string(),
                action: "approve".to_string(),
                approval_id: approval_id.clone(),
                marker_key: approval_id.clone(),
                file_name,
                switch_name:
                    "approve_execution_runner_recovery_marker_cleanup_phase_enabled".to_string(),
                phase_sequence_ready: true,
                source_mutation_phase_enablement_ready: true,
                source_mutation_phase_ready: true,
                cleanup_phase_present: true,
                marker_persistence_plan_ready: true,
                marker_persistence_ready: true,
                cleanup_phase_enabled: false,
                enablement_prerequisites_ready: true,
                cleanup_phase_enablement_ready: false,
                would_enable_cleanup_phase: false,
                would_clear_recovery_marker: false,
                would_continue_to_commit: false,
                would_return_success: false,
                would_touch_disk: false,
                required_gates: vec![
                    "approve_execution_runner_phase_sequence_ready".to_string(),
                    "approve_execution_runner_source_mutation_phase_enablement_ready".to_string(),
                    "approve_execution_runner_source_mutation_phase_ready".to_string(),
                    "approve_execution_runner_recovery_marker_cleanup_phase_present".to_string(),
                    "recovery_marker_persistence_plan_ready".to_string(),
                    "recovery_marker_persistence_ready".to_string(),
                    "approve_execution_runner_recovery_marker_cleanup_phase_enabled".to_string(),
                ],
                passed_gates: vec![
                    "approve_execution_runner_phase_sequence_ready".to_string(),
                    "approve_execution_runner_source_mutation_phase_enablement_ready".to_string(),
                    "approve_execution_runner_source_mutation_phase_ready".to_string(),
                    "approve_execution_runner_recovery_marker_cleanup_phase_present".to_string(),
                    "recovery_marker_persistence_plan_ready".to_string(),
                    "recovery_marker_persistence_ready".to_string(),
                ],
                blocked_gates: vec![
                    "approve_execution_runner_recovery_marker_cleanup_phase_enabled".to_string(),
                ],
            };

        let locked_result =
            contract_repair_approval_approve_execution_runner_recovery_marker_cleanup_phase_with_gate(
                &store_dir,
                &cleanup_phase_dry_run,
                &cleanup_phase_enablement_dry_run,
                false,
            )
            .await;

        assert_eq!(locked_result, cleanup_phase_dry_run);
        assert!(marker_path.exists());

        let cleanup_result =
            contract_repair_approval_approve_execution_runner_recovery_marker_cleanup_phase_with_gate(
                &store_dir,
                &cleanup_phase_dry_run,
                &cleanup_phase_enablement_dry_run,
                true,
            )
            .await;

        assert_eq!(
            cleanup_result.status,
            "approve_execution_runner_recovery_marker_cleanup_phase_cleared"
        );
        assert!(cleanup_result.cleanup_phase_enabled);
        assert!(cleanup_result.cleanup_phase_ready);
        assert!(cleanup_result.would_clear_recovery_marker);
        assert!(cleanup_result.would_continue_to_commit);
        assert!(cleanup_result.would_touch_disk);
        assert!(!cleanup_result.would_return_success);
        assert!(cleanup_result.blocked_by.is_empty());
        assert!(!marker_path.exists());
    }

    #[test]
    fn contract_repair_approval_transaction_commit_gate_and_phase_commit_when_enabled() {
        let approval_id = "contract-repair-apr-commit-test".to_string();
        let commit_gate = ContractRepairApprovalApproveExecutionTransactionCommitGate {
            status: "approve_execution_transaction_commit_ready_blocked".to_string(),
            action: "approve".to_string(),
            approval_id: approval_id.clone(),
            runner_plan_ready: true,
            runner_enabled: true,
            admission_ready: true,
            commit_barrier_ready: true,
            rollback_plan_ready: true,
            recovery_marker_persistence_plan_ready: true,
            recovery_marker_persistence_ready: true,
            commit_gate_enabled: false,
            commit_ready: false,
            would_start_runner: false,
            would_persist_recovery_marker: false,
            would_commit_transaction: false,
            would_touch_disk: false,
            required_gates: vec![
                "approve_execution_transaction_runner_enabled".to_string(),
                "approve_execution_admission_ready".to_string(),
                "commit_barrier_ready".to_string(),
                "rollback_plan_ready".to_string(),
                "recovery_marker_persistence_plan_ready".to_string(),
                "recovery_marker_persistence_ready".to_string(),
                "approve_execution_transaction_commit_enabled".to_string(),
            ],
            passed_gates: vec![
                "approve_execution_transaction_runner_enabled".to_string(),
                "approve_execution_admission_ready".to_string(),
                "commit_barrier_ready".to_string(),
                "rollback_plan_ready".to_string(),
                "recovery_marker_persistence_plan_ready".to_string(),
                "recovery_marker_persistence_ready".to_string(),
            ],
            blocked_gates: vec!["approve_execution_transaction_commit_enabled".to_string()],
        };

        let locked_gate =
            contract_repair_approval_approve_execution_transaction_commit_gate_with_gate(
                &commit_gate,
                false,
            );
        assert_eq!(locked_gate, commit_gate);

        let committed_gate =
            contract_repair_approval_approve_execution_transaction_commit_gate_with_gate(
                &commit_gate,
                true,
            );
        assert_eq!(
            committed_gate.status,
            "approve_execution_transaction_commit_ready"
        );
        assert!(committed_gate.commit_gate_enabled);
        assert!(committed_gate.commit_ready);
        assert!(committed_gate.would_commit_transaction);
        assert!(!committed_gate.would_touch_disk);
        assert!(committed_gate.blocked_gates.is_empty());

        let commit_phase_dry_run =
            ContractRepairApprovalApproveExecutionRunnerTransactionCommitPhaseDryRun {
                status: "approve_execution_runner_transaction_commit_phase_ready_blocked"
                    .to_string(),
                action: "approve".to_string(),
                approval_id: approval_id.clone(),
                phase_sequence_ready: true,
                cleanup_phase_ready: true,
                runner_plan_ready: true,
                runner_enabled: true,
                admission_ready: true,
                commit_barrier_ready: true,
                rollback_plan_ready: true,
                recovery_marker_persistence_plan_ready: true,
                recovery_marker_persistence_ready: true,
                commit_gate_enabled: true,
                commit_ready: true,
                commit_phase_enabled: false,
                commit_phase_ready: false,
                would_commit_transaction: false,
                would_return_success: false,
                would_touch_disk: false,
                blocked_by: vec![
                    "approve_execution_runner_transaction_commit_phase_enabled".to_string()
                ],
            };
        let commit_phase_enablement_dry_run =
            ContractRepairApprovalApproveExecutionRunnerTransactionCommitPhaseEnablementDryRun {
                status:
                    "approve_execution_runner_transaction_commit_phase_enablement_ready_blocked"
                        .to_string(),
                action: "approve".to_string(),
                approval_id,
                switch_name: "approve_execution_runner_transaction_commit_phase_enabled"
                    .to_string(),
                phase_sequence_ready: true,
                cleanup_phase_enablement_ready: true,
                cleanup_phase_ready: true,
                runner_plan_ready: true,
                runner_enabled: true,
                admission_ready: true,
                commit_barrier_ready: true,
                rollback_plan_ready: true,
                recovery_marker_persistence_plan_ready: true,
                recovery_marker_persistence_ready: true,
                commit_gate_enabled: true,
                commit_ready: true,
                commit_phase_enabled: false,
                enablement_prerequisites_ready: true,
                commit_phase_enablement_ready: false,
                would_enable_commit_phase: false,
                would_commit_transaction: false,
                would_return_success: false,
                would_touch_disk: false,
                required_gates: vec![
                    "approve_execution_runner_phase_sequence_ready".to_string(),
                    "approve_execution_runner_recovery_marker_cleanup_phase_enablement_ready"
                        .to_string(),
                    "approve_execution_runner_recovery_marker_cleanup_phase_ready".to_string(),
                    "approve_execution_transaction_commit_plan_ready".to_string(),
                    "approve_execution_transaction_runner_enabled".to_string(),
                    "approve_execution_admission_ready".to_string(),
                    "commit_barrier_ready".to_string(),
                    "rollback_plan_ready".to_string(),
                    "recovery_marker_persistence_plan_ready".to_string(),
                    "recovery_marker_persistence_ready".to_string(),
                    "approve_execution_transaction_commit_enabled".to_string(),
                    "approve_execution_transaction_commit_ready".to_string(),
                    "approve_execution_runner_transaction_commit_phase_enabled".to_string(),
                ],
                passed_gates: vec![
                    "approve_execution_runner_phase_sequence_ready".to_string(),
                    "approve_execution_runner_recovery_marker_cleanup_phase_enablement_ready"
                        .to_string(),
                    "approve_execution_runner_recovery_marker_cleanup_phase_ready".to_string(),
                    "approve_execution_transaction_commit_plan_ready".to_string(),
                    "approve_execution_transaction_runner_enabled".to_string(),
                    "approve_execution_admission_ready".to_string(),
                    "commit_barrier_ready".to_string(),
                    "rollback_plan_ready".to_string(),
                    "recovery_marker_persistence_plan_ready".to_string(),
                    "recovery_marker_persistence_ready".to_string(),
                    "approve_execution_transaction_commit_enabled".to_string(),
                    "approve_execution_transaction_commit_ready".to_string(),
                ],
                blocked_gates: vec![
                    "approve_execution_runner_transaction_commit_phase_enabled".to_string()
                ],
            };

        let locked_phase =
            contract_repair_approval_approve_execution_runner_transaction_commit_phase_with_gate(
                &commit_phase_dry_run,
                &commit_phase_enablement_dry_run,
                false,
            );
        assert_eq!(locked_phase, commit_phase_dry_run);

        let committed_phase =
            contract_repair_approval_approve_execution_runner_transaction_commit_phase_with_gate(
                &commit_phase_dry_run,
                &commit_phase_enablement_dry_run,
                true,
            );
        assert_eq!(
            committed_phase.status,
            "approve_execution_runner_transaction_commit_phase_committed"
        );
        assert!(committed_phase.commit_phase_enabled);
        assert!(committed_phase.commit_phase_ready);
        assert!(committed_phase.would_commit_transaction);
        assert!(!committed_phase.would_return_success);
        assert!(!committed_phase.would_touch_disk);
        assert!(committed_phase.blocked_by.is_empty());
    }

    fn contract_repair_approval_ready_route_success_dry_run_for_test(
        approval_id: &str,
    ) -> ContractRepairApprovalApproveExecutionRunnerRouteSuccessReadinessDryRun {
        ContractRepairApprovalApproveExecutionRunnerRouteSuccessReadinessDryRun {
            status: "approve_execution_runner_route_success_readiness_ready".to_string(),
            action: "approve".to_string(),
            approval_id: approval_id.to_string(),
            gate_name: "approve_execution_runner_route_success_ready".to_string(),
            route_status_name: "review_approve_executed".to_string(),
            source_return_name: "approve_execution_runner_activation_success_return".to_string(),
            source_enablement_plan_name: "approve_execution_runner_enablement_plan".to_string(),
            structural_plan_ready: true,
            runner_control_ready: true,
            phase_chain_ready: true,
            rollback_chain_ready: true,
            side_effect_enablement_ready: true,
            runner_activation_enabled: true,
            runner_activation_ready: true,
            activation_success_return_ready: true,
            activation_success_return_readiness_ready: true,
            enablement_plan_success_ready: true,
            route_success_ready: true,
            would_set_route_success: false,
            would_mark_review_approve_executed: false,
            would_activate_runner: false,
            would_return_success: false,
            would_touch_disk: false,
            inherited_success_return_readiness_blocked_gates: Vec::new(),
            inherited_success_return_blocked_gates: Vec::new(),
            inherited_enablement_plan_blocked_gates: Vec::new(),
            inherited_enablement_plan_blocked_enablements: Vec::new(),
            required_gates: vec![
                "approve_execution_runner_activation_success_return_ready".to_string(),
                "approve_runner_success".to_string(),
            ],
            passed_gates: vec![
                "approve_execution_runner_activation_success_return_ready".to_string(),
                "approve_runner_success".to_string(),
            ],
            blocked_gates: Vec::new(),
        }
    }

    #[test]
    fn contract_repair_approval_final_atomic_readiness_waits_for_formal_review_execution() {
        let approval_id = "contract-repair-apr-final-atomic-test";
        let record_write_dry_run = ContractRepairApprovalRecordWriteDryRun {
            status: "record_write_executed".to_string(),
            adapter_kind: "contract_repair_approval_record_writer".to_string(),
            store_kind: CONTRACT_REPAIR_APPROVAL_STORE_KIND.to_string(),
            record_key: approval_id.to_string(),
            file_name: format!("{approval_id}.json"),
            transition_ready: true,
            storage_ready: true,
            schema_ready: true,
            idempotency_ready: true,
            snapshot_ready: true,
            idempotency_precheck_passed: true,
            write_ready: true,
            would_write: true,
            blocked_by: Vec::new(),
        };
        let lifecycle_emission_gate = ContractRepairApprovalLifecycleEmissionEnablementGate {
            status: "lifecycle_emission_enablement_ready".to_string(),
            event_id: format!("contract-repair-review-approve:{approval_id}"),
            transition_ready: true,
            event_payload_ready: true,
            emission_ready: true,
            entry_append_ready: true,
            lifecycle_emission_plan_ready: true,
            lifecycle_event_emission_enabled: true,
            lifecycle_entry_append_enabled: true,
            lifecycle_effects_ready: true,
            would_emit: true,
            would_append: true,
            would_touch_lifecycle_log: true,
            required_gates: Vec::new(),
            passed_gates: Vec::new(),
            blocked_gates: Vec::new(),
        };
        let contract_mutation_gate = ContractRepairApprovalContractMutationEnablementGate {
            status: "contract_mutation_enablement_ready".to_string(),
            target_path: "memory_schema/decision.dual_ma/cross_up/last_signal_at".to_string(),
            target_kind: "memory_field".to_string(),
            source_path: "graphs/graph_dual-ma.json".to_string(),
            writeback_plan_ready: true,
            source_write_ready: true,
            lifecycle_effects_ready: true,
            contract_mutation_api_enabled: true,
            mutation_ready: true,
            would_mutate_contract: false,
            would_write_source: false,
            would_touch_disk: false,
            required_gates: Vec::new(),
            passed_gates: Vec::new(),
            blocked_gates: Vec::new(),
        };
        let marker_persistence_gate =
            ContractRepairApprovalApproveExecutionRecoveryMarkerPersistenceGate {
                status: "approve_execution_recovery_marker_persistence_ready".to_string(),
                marker_key: approval_id.to_string(),
                file_name: format!("{approval_id}.recovery-marker.json"),
                marker_persistence_plan_ready: true,
                marker_write_ready: true,
                idempotency_checked: true,
                no_existing_marker_conflict: true,
                runner_ready: true,
                marker_persistence_enabled: true,
                persistence_ready: true,
                would_persist_marker: false,
                would_touch_disk: false,
                required_gates: Vec::new(),
                passed_gates: Vec::new(),
                blocked_gates: Vec::new(),
            };
        let transaction_commit_gate = ContractRepairApprovalApproveExecutionTransactionCommitGate {
            status: "approve_execution_transaction_commit_ready".to_string(),
            action: "approve".to_string(),
            approval_id: approval_id.to_string(),
            runner_plan_ready: true,
            runner_enabled: true,
            admission_ready: true,
            commit_barrier_ready: true,
            rollback_plan_ready: true,
            recovery_marker_persistence_plan_ready: true,
            recovery_marker_persistence_ready: true,
            commit_gate_enabled: true,
            commit_ready: true,
            would_start_runner: false,
            would_persist_recovery_marker: false,
            would_commit_transaction: false,
            would_touch_disk: false,
            required_gates: Vec::new(),
            passed_gates: Vec::new(),
            blocked_gates: Vec::new(),
        };
        let route_success_readiness_dry_run =
            contract_repair_approval_ready_route_success_dry_run_for_test(approval_id);
        let formal_review_execution_readiness_dry_run =
            ContractRepairApprovalApproveExecutionFormalReviewExecutionReadinessDryRun {
                status: "approve_execution_formal_review_execution_readiness_ready_blocked"
                    .to_string(),
                action: "approve".to_string(),
                approval_id: approval_id.to_string(),
                gate_name: "formal_approve_review_execution_ready".to_string(),
                decision_execution_preflight_requested: true,
                review_request_enabled: true,
                review_execution_gate_clear: true,
                approve_execution_ready: true,
                route_success_ready: true,
                formal_approve_review_execution_enabled: false,
                review_execution_enabled: false,
                formal_review_execution_ready: false,
                would_execute_decision: false,
                would_persist_approval_record: false,
                would_mutate_contract: false,
                would_emit_lifecycle_event: false,
                would_commit_transaction: false,
                would_return_http_ok: false,
                would_touch_disk: false,
                inherited_execution_blocked_gates: Vec::new(),
                inherited_approve_execution_blockers: Vec::new(),
                inherited_route_success_blocked_gates: Vec::new(),
                inherited_route_status_blocked_gates: vec!["review_execution_enabled".to_string()],
                inherited_blocked_reasons: vec!["approve_execution_not_enabled".to_string()],
                required_gates: Vec::new(),
                passed_gates: Vec::new(),
                blocked_gates: vec!["formal_approve_review_execution_enabled".to_string()],
            };

        let final_atomic_readiness =
            contract_repair_approval_approve_execution_final_atomic_readiness_dry_run(
                "approve",
                approval_id,
                &record_write_dry_run,
                &lifecycle_emission_gate,
                &contract_mutation_gate,
                &marker_persistence_gate,
                &transaction_commit_gate,
                &route_success_readiness_dry_run,
                &formal_review_execution_readiness_dry_run,
                true,
                false,
            );

        assert_eq!(
            final_atomic_readiness.status,
            "approve_execution_final_atomic_readiness_ready_blocked"
        );
        assert!(final_atomic_readiness.record_write_ready);
        assert!(final_atomic_readiness.lifecycle_effects_ready);
        assert!(final_atomic_readiness.contract_mutation_ready);
        assert!(final_atomic_readiness.recovery_marker_persistence_ready);
        assert!(final_atomic_readiness.transaction_commit_ready);
        assert!(final_atomic_readiness.route_success_ready);
        assert!(!final_atomic_readiness.formal_review_execution_ready);
        assert!(!final_atomic_readiness.review_execution_enabled);
        assert!(!final_atomic_readiness.final_atomic_execution_ready);
        assert!(!final_atomic_readiness.would_execute_decision);
        assert!(!final_atomic_readiness.would_persist_approval_record);
        assert!(!final_atomic_readiness.would_mutate_contract);
        assert!(!final_atomic_readiness.would_persist_recovery_marker);
        assert!(!final_atomic_readiness.would_commit_transaction);
        assert!(!final_atomic_readiness.would_return_http_ok);
        assert!(!final_atomic_readiness.would_touch_disk);
        assert!(final_atomic_readiness
            .blocked_gates
            .contains(&"formal_review_execution_ready".to_string()));
        assert!(final_atomic_readiness
            .inherited_record_write_blockers
            .is_empty());
        assert!(final_atomic_readiness
            .inherited_contract_mutation_blocked_gates
            .is_empty());
        assert!(final_atomic_readiness
            .inherited_transaction_commit_blocked_gates
            .is_empty());
        assert!(final_atomic_readiness
            .inherited_formal_review_execution_blocked_gates
            .contains(&"formal_approve_review_execution_enabled".to_string()));

        let transaction_runner_dry_run =
            ContractRepairApprovalApproveExecutionTransactionRunnerDryRun {
                status: "approve_execution_transaction_runner_ready".to_string(),
                action: "approve".to_string(),
                approval_id: approval_id.to_string(),
                runner_enabled: true,
                admission_ready: true,
                transaction_plan_ready: true,
                commit_barrier_ready: true,
                recovery_marker_ready: true,
                rollback_plan_ready: true,
                commit_ready: true,
                would_start_runner: false,
                would_write_recovery_marker: false,
                would_commit_transaction: false,
                would_rollback_on_error: false,
                phase_order: vec![
                    "write_recovery_marker".to_string(),
                    "transition_review_state".to_string(),
                    "persist_approval_record".to_string(),
                    "emit_lifecycle_event".to_string(),
                    "append_lifecycle_entry".to_string(),
                    "write_contract_source".to_string(),
                    "clear_recovery_marker".to_string(),
                ],
                rollback_order: vec![
                    "restore_contract_source".to_string(),
                    "restore_approval_record".to_string(),
                    "mark_recovery_marker_rolled_back".to_string(),
                ],
                blocked_by: Vec::new(),
            };
        let cleanup_phase_readiness_dry_run =
            ContractRepairApprovalApproveExecutionRunnerRecoveryMarkerCleanupPhaseReadinessDryRun {
                status: "approve_execution_runner_recovery_marker_cleanup_phase_readiness_ready"
                    .to_string(),
                action: "approve".to_string(),
                approval_id: approval_id.to_string(),
                marker_key: approval_id.to_string(),
                file_name: format!("{approval_id}.recovery-marker.json"),
                gate_name: "approve_execution_runner_recovery_marker_cleanup_phase_ready"
                    .to_string(),
                phase_sequence_ready: true,
                source_mutation_phase_enablement_ready: true,
                source_mutation_phase_ready: true,
                cleanup_phase_present: true,
                marker_persistence_plan_ready: true,
                marker_persistence_ready: true,
                cleanup_phase_enabled: true,
                cleanup_phase_enablement_ready: true,
                cleanup_phase_ready: true,
                would_clear_recovery_marker: false,
                would_continue_to_commit: false,
                would_return_success: false,
                would_touch_disk: false,
                would_unblock_transaction_commit: false,
                would_unblock_control_readiness: false,
                inherited_cleanup_phase_blockers: Vec::new(),
                inherited_cleanup_phase_enablement_blocked_gates: Vec::new(),
                required_gates: Vec::new(),
                passed_gates: Vec::new(),
                blocked_gates: Vec::new(),
            };
        let transaction_commit_phase_readiness_dry_run =
            ContractRepairApprovalApproveExecutionRunnerTransactionCommitPhaseReadinessDryRun {
                status: "approve_execution_runner_transaction_commit_phase_readiness_ready"
                    .to_string(),
                action: "approve".to_string(),
                approval_id: approval_id.to_string(),
                gate_name: "approve_execution_runner_transaction_commit_phase_ready".to_string(),
                phase_sequence_ready: true,
                cleanup_phase_enablement_ready: true,
                cleanup_phase_ready: true,
                runner_plan_ready: true,
                runner_enabled: true,
                admission_ready: true,
                commit_barrier_ready: true,
                rollback_plan_ready: true,
                recovery_marker_persistence_plan_ready: true,
                recovery_marker_persistence_ready: true,
                commit_gate_enabled: true,
                commit_ready: true,
                commit_phase_enabled: true,
                commit_phase_enablement_ready: true,
                commit_phase_ready: true,
                would_commit_transaction: false,
                would_return_success: false,
                would_touch_disk: false,
                would_unblock_rollback_execution: false,
                would_unblock_control_readiness: false,
                inherited_commit_phase_blockers: Vec::new(),
                inherited_commit_phase_enablement_blocked_gates: Vec::new(),
                required_gates: Vec::new(),
                passed_gates: Vec::new(),
                blocked_gates: Vec::new(),
            };
        let rollback_execution_phase_readiness_dry_run =
            ContractRepairApprovalApproveExecutionRunnerRollbackExecutionPhaseReadinessDryRun {
                status: "approve_execution_runner_rollback_execution_phase_readiness_ready"
                    .to_string(),
                action: "approve".to_string(),
                approval_id: approval_id.to_string(),
                gate_name: "approve_execution_runner_rollback_execution_phase_ready".to_string(),
                phase_sequence_ready: true,
                rollback_sequence_ready: true,
                commit_phase_enablement_ready: true,
                commit_phase_ready: true,
                rollback_phase_present: true,
                rollback_plan_ready: true,
                rollback_execution_enabled: true,
                rollback_execution_enablement_ready: true,
                rollback_execution_ready: true,
                rollback_order: vec![
                    "restore_contract_source".to_string(),
                    "restore_approval_record".to_string(),
                    "mark_recovery_marker_rolled_back".to_string(),
                ],
                would_restore_contract_source: false,
                would_restore_approval_record: false,
                would_mark_recovery_marker_rolled_back: false,
                would_rollback_on_error: false,
                would_return_success: false,
                would_touch_disk: false,
                would_unblock_runner_activation: false,
                would_unblock_control_readiness: false,
                inherited_rollback_execution_blockers: Vec::new(),
                inherited_rollback_execution_enablement_blocked_gates: Vec::new(),
                required_gates: Vec::new(),
                passed_gates: Vec::new(),
                blocked_gates: Vec::new(),
            };

        let execution_plan =
            contract_repair_approval_approve_execution_final_atomic_execution_plan_dry_run(
                "approve",
                approval_id,
                &transaction_runner_dry_run,
                &cleanup_phase_readiness_dry_run,
                &transaction_commit_phase_readiness_dry_run,
                &rollback_execution_phase_readiness_dry_run,
                &final_atomic_readiness,
                false,
            );

        assert_eq!(
            execution_plan.status,
            "approve_execution_final_atomic_execution_plan_ready_blocked"
        );
        assert!(execution_plan.execution_order_ready);
        assert!(execution_plan.rollback_order_ready);
        assert!(execution_plan.transaction_runner_ready);
        assert!(execution_plan.cleanup_phase_ready);
        assert!(execution_plan.transaction_commit_phase_ready);
        assert!(execution_plan.rollback_execution_ready);
        assert!(!execution_plan.final_atomic_readiness_ready);
        assert!(!execution_plan.formal_review_execution_ready);
        assert!(!execution_plan.review_execution_enabled);
        assert!(!execution_plan.final_atomic_execution_plan_ready);
        assert!(!execution_plan.partial_execution_allowed);
        assert!(execution_plan.recovery_marker_required);
        assert!(execution_plan.commit_barrier_ready);
        assert!(execution_plan.rollback_plan_ready);
        assert!(!execution_plan.would_start_atomic_execution);
        assert!(!execution_plan.would_persist_approval_record);
        assert!(!execution_plan.would_mutate_contract);
        assert!(!execution_plan.would_persist_recovery_marker);
        assert!(!execution_plan.would_clear_recovery_marker);
        assert!(!execution_plan.would_commit_transaction);
        assert!(!execution_plan.would_rollback_on_error);
        assert!(!execution_plan.would_return_http_ok);
        assert!(!execution_plan.would_touch_disk);
        assert!(execution_plan
            .execution_order
            .contains(&"return_review_approve_executed".to_string()));
        assert!(execution_plan
            .rollback_order
            .contains(&"mark_recovery_marker_rolled_back".to_string()));
        assert!(execution_plan
            .blocked_gates
            .contains(&"final_atomic_readiness_ready".to_string()));
        assert!(execution_plan
            .blocked_gates
            .contains(&"formal_review_execution_ready".to_string()));
        assert!(execution_plan.inherited_runner_blockers.is_empty());
        assert!(execution_plan
            .inherited_transaction_commit_phase_blocked_gates
            .is_empty());
        assert!(execution_plan
            .inherited_rollback_execution_blocked_gates
            .is_empty());
        assert!(execution_plan
            .inherited_final_atomic_blocked_gates
            .contains(&"formal_review_execution_ready".to_string()));

        let admission_gate =
            contract_repair_approval_approve_execution_final_atomic_admission_gate_dry_run(
                "approve",
                approval_id,
                &execution_plan,
            );

        assert_eq!(
            admission_gate.status,
            "approve_execution_final_atomic_admission_gate_ready_blocked"
        );
        assert_eq!(
            admission_gate.gate_name,
            "approve_execution_final_atomic_admission_gate"
        );
        assert!(admission_gate.execution_plan_structural_ready);
        assert!(admission_gate.execution_order_ready);
        assert!(admission_gate.rollback_order_ready);
        assert!(!admission_gate.final_atomic_execution_plan_ready);
        assert!(!admission_gate.final_atomic_readiness_ready);
        assert!(!admission_gate.formal_review_execution_ready);
        assert!(!admission_gate.review_execution_enabled);
        assert!(!admission_gate.partial_execution_allowed);
        assert!(admission_gate.recovery_marker_required);
        assert!(admission_gate.commit_barrier_ready);
        assert!(admission_gate.rollback_plan_ready);
        assert!(!admission_gate.admission_ready);
        assert!(!admission_gate.would_enter_final_execution);
        assert!(!admission_gate.would_execute_decision);
        assert!(!admission_gate.would_return_http_ok);
        assert!(!admission_gate.would_touch_disk);
        assert!(admission_gate
            .blocked_gates
            .contains(&"final_atomic_execution_plan_ready".to_string()));
        assert!(admission_gate
            .blocked_gates
            .contains(&"final_atomic_readiness_ready".to_string()));
        assert!(admission_gate
            .blocked_gates
            .contains(&"formal_review_execution_ready".to_string()));
        assert!(admission_gate
            .inherited_execution_plan_blocked_gates
            .contains(&"final_atomic_readiness_ready".to_string()));
        assert!(admission_gate
            .inherited_execution_order
            .contains(&"return_review_approve_executed".to_string()));
        assert!(admission_gate
            .inherited_rollback_order
            .contains(&"restore_contract_source".to_string()));
    }

    fn contract_repair_approval_verified_live_route_gates_for_test(
    ) -> ContractRepairApprovalApproveLiveRouteGates {
        let policy = contract_repair_approval_approve_live_route_activation_policy_from_value(
            Some(CONTRACT_REPAIR_APPROVAL_APPROVE_LIVE_ROUTE_ENABLED_VALUE),
        );
        contract_repair_approval_approve_live_route_gates_for_policy(&policy)
    }

    fn contract_repair_approval_live_route_gate_values_for_test(
        gates: &ContractRepairApprovalApproveLiveRouteGates,
    ) -> [bool; 22] {
        [
            gates.review_transition_enabled,
            gates.lifecycle_effects_enabled,
            gates.contract_source_write_enabled,
            gates.transaction_runner_enabled,
            gates.runner_attempt_enabled,
            gates.runner_execution_enabled,
            gates.route_dispatch_enabled,
            gates.runner_call_enabled,
            gates.runner_body_enabled,
            gates.phase_execution_enabled,
            gates.lifecycle_phase_enabled,
            gates.source_mutation_phase_enabled,
            gates.rollback_execution_enabled,
            gates.runner_activation_enabled,
            gates.activation_switch_write_transaction_enabled,
            gates.recovery_marker_persistence_enabled,
            gates.recovery_marker_cleanup_phase_enabled,
            gates.transaction_commit_enabled,
            gates.transaction_commit_phase_enabled,
            gates.atomic_side_effects_enabled,
            gates.route_success_enabled,
            gates.formal_approve_review_execution_enabled,
        ]
    }

    #[test]
    fn contract_repair_approval_live_route_activation_policy_requires_verified_environment_value() {
        let missing_policy =
            contract_repair_approval_approve_live_route_activation_policy_from_value(None);
        assert_eq!(
            missing_policy.policy_name,
            CONTRACT_REPAIR_APPROVAL_APPROVE_LIVE_ROUTE_POLICY_NAME
        );
        assert_eq!(
            missing_policy.env_name,
            CONTRACT_REPAIR_APPROVAL_APPROVE_LIVE_ROUTE_ENV
        );
        assert_eq!(
            missing_policy.required_value,
            CONTRACT_REPAIR_APPROVAL_APPROVE_LIVE_ROUTE_ENABLED_VALUE
        );
        assert_eq!(missing_policy.configured_value, None);
        assert!(!missing_policy.live_route_enabled);
        assert_eq!(
            missing_policy.blocked_gates,
            vec![CONTRACT_REPAIR_APPROVAL_APPROVE_LIVE_ROUTE_POLICY_GATE.to_string()]
        );
        let missing_gates =
            contract_repair_approval_approve_live_route_gates_for_policy(&missing_policy);
        assert!(
            contract_repair_approval_live_route_gate_values_for_test(&missing_gates)
                .iter()
                .all(|enabled| !enabled)
        );

        let invalid_policy =
            contract_repair_approval_approve_live_route_activation_policy_from_value(Some("true"));
        assert_eq!(invalid_policy.configured_value.as_deref(), Some("true"));
        assert!(!invalid_policy.live_route_enabled);
        assert_eq!(
            invalid_policy.blocked_gates,
            vec![CONTRACT_REPAIR_APPROVAL_APPROVE_LIVE_ROUTE_POLICY_GATE.to_string()]
        );

        let verified_policy =
            contract_repair_approval_approve_live_route_activation_policy_from_value(Some(
                CONTRACT_REPAIR_APPROVAL_APPROVE_LIVE_ROUTE_ENABLED_VALUE,
            ));
        assert_eq!(
            verified_policy.configured_value.as_deref(),
            Some(CONTRACT_REPAIR_APPROVAL_APPROVE_LIVE_ROUTE_ENABLED_VALUE)
        );
        assert!(verified_policy.live_route_enabled);
        assert!(verified_policy.blocked_gates.is_empty());
        let verified_gates =
            contract_repair_approval_approve_live_route_gates_for_policy(&verified_policy);
        assert!(
            contract_repair_approval_live_route_gate_values_for_test(&verified_gates)
                .iter()
                .all(|enabled| *enabled)
        );
    }

    #[test]
    fn contract_repair_approval_route_success_gate_dispatches_when_enabled() {
        let route_success_readiness_dry_run =
            contract_repair_approval_ready_route_success_dry_run_for_test(
                "contract-repair-apr-route-success-test",
            );

        let locked_route_success =
            contract_repair_approval_approve_execution_runner_route_success_with_gate(
                &route_success_readiness_dry_run,
                false,
            );
        assert_eq!(locked_route_success, route_success_readiness_dry_run);
        assert!(
            !contract_repair_approval_approve_runner_success_from_route_success(
                &locked_route_success,
            )
        );

        let dispatched_route_success =
            contract_repair_approval_approve_execution_runner_route_success_with_gate(
                &route_success_readiness_dry_run,
                true,
            );

        assert_eq!(
            dispatched_route_success.status,
            "approve_execution_runner_route_success_dispatched"
        );
        assert!(dispatched_route_success.route_success_ready);
        assert!(dispatched_route_success.would_set_route_success);
        assert!(dispatched_route_success.would_mark_review_approve_executed);
        assert!(dispatched_route_success.would_activate_runner);
        assert!(dispatched_route_success.would_return_success);
        assert!(!dispatched_route_success.would_touch_disk);
        assert!(dispatched_route_success.blocked_gates.is_empty());
        assert!(
            contract_repair_approval_approve_runner_success_from_route_success(
                &dispatched_route_success,
            )
        );
    }

    #[test]
    fn contract_repair_approval_verified_live_gate_map_can_dispatch_route_success() {
        let gates = contract_repair_approval_verified_live_route_gates_for_test();
        assert!(gates.review_transition_enabled);
        assert!(gates.lifecycle_effects_enabled);
        assert!(gates.contract_source_write_enabled);
        assert!(gates.transaction_runner_enabled);
        assert!(gates.runner_attempt_enabled);
        assert!(gates.runner_execution_enabled);
        assert!(gates.route_dispatch_enabled);
        assert!(gates.runner_call_enabled);
        assert!(gates.runner_body_enabled);
        assert!(gates.phase_execution_enabled);
        assert!(gates.lifecycle_phase_enabled);
        assert!(gates.source_mutation_phase_enabled);
        assert!(gates.rollback_execution_enabled);
        assert!(gates.runner_activation_enabled);
        assert!(gates.activation_switch_write_transaction_enabled);
        assert!(gates.recovery_marker_persistence_enabled);
        assert!(gates.recovery_marker_cleanup_phase_enabled);
        assert!(gates.transaction_commit_enabled);
        assert!(gates.transaction_commit_phase_enabled);
        assert!(gates.atomic_side_effects_enabled);
        assert!(gates.route_success_enabled);
        assert!(gates.formal_approve_review_execution_enabled);

        let route_success_readiness_dry_run =
            contract_repair_approval_ready_route_success_dry_run_for_test(
                "contract-repair-apr-verified-live-map-test",
            );
        let dispatched_route_success =
            contract_repair_approval_approve_execution_runner_route_success_with_gate(
                &route_success_readiness_dry_run,
                gates.route_success_enabled,
            );
        let approve_runner_success =
            contract_repair_approval_approve_runner_success_from_route_success(
                &dispatched_route_success,
            );
        let response_status = if approve_runner_success {
            "review_approve_executed"
        } else {
            "review_decision_execution_blocked"
        };

        assert_eq!(
            dispatched_route_success.status,
            "approve_execution_runner_route_success_dispatched"
        );
        assert!(dispatched_route_success.route_success_ready);
        assert!(dispatched_route_success.would_set_route_success);
        assert!(dispatched_route_success.would_mark_review_approve_executed);
        assert!(dispatched_route_success.would_activate_runner);
        assert!(dispatched_route_success.would_return_success);
        assert!(!dispatched_route_success.would_touch_disk);
        assert!(dispatched_route_success.blocked_gates.is_empty());
        assert!(approve_runner_success);
        assert_eq!(response_status, "review_approve_executed");
    }

    #[test]
    fn contract_repair_approval_approve_live_route_gate_map_defaults_to_locked() {
        let gates = contract_repair_approval_approve_live_route_gates();

        assert!(!gates.review_transition_enabled);
        assert!(!gates.lifecycle_effects_enabled);
        assert!(!gates.contract_source_write_enabled);
        assert!(!gates.transaction_runner_enabled);
        assert!(!gates.runner_attempt_enabled);
        assert!(!gates.runner_execution_enabled);
        assert!(!gates.route_dispatch_enabled);
        assert!(!gates.runner_call_enabled);
        assert!(!gates.runner_body_enabled);
        assert!(!gates.phase_execution_enabled);
        assert!(!gates.lifecycle_phase_enabled);
        assert!(!gates.source_mutation_phase_enabled);
        assert!(!gates.rollback_execution_enabled);
        assert!(!gates.runner_activation_enabled);
        assert!(!gates.activation_switch_write_transaction_enabled);
        assert!(!gates.recovery_marker_persistence_enabled);
        assert!(!gates.recovery_marker_cleanup_phase_enabled);
        assert!(!gates.transaction_commit_enabled);
        assert!(!gates.transaction_commit_phase_enabled);
        assert!(!gates.atomic_side_effects_enabled);
        assert!(!gates.route_success_enabled);
        assert!(!gates.formal_approve_review_execution_enabled);
    }

    #[test]
    fn contract_repair_approval_runner_control_gates_follow_live_map_switches() {
        let approval_id = "contract-repair-apr-runner-control-test";
        let gates = ContractRepairApprovalApproveLiveRouteGates {
            runner_attempt_enabled: true,
            runner_execution_enabled: true,
            route_dispatch_enabled: true,
            ..contract_repair_approval_approve_live_route_gates()
        };
        let atomic_side_effects_gate =
            ContractRepairApprovalApproveExecutionAtomicSideEffectsGate {
                status: "approve_execution_atomic_side_effects_ready".to_string(),
                action: "approve".to_string(),
                approval_id: approval_id.to_string(),
                lifecycle_plan_ready: true,
                contract_mutation_plan_ready: true,
                recovery_marker_persistence_plan_ready: true,
                transaction_commit_plan_ready: true,
                atomic_side_effects_plan_ready: true,
                lifecycle_effects_ready: true,
                contract_mutation_ready: true,
                recovery_marker_persistence_ready: true,
                transaction_commit_ready: true,
                atomic_side_effects_enabled: true,
                atomic_side_effects_ready: true,
                would_emit_lifecycle: false,
                would_mutate_contract: false,
                would_persist_recovery_marker: false,
                would_commit_transaction: false,
                would_touch_disk: false,
                required_gates: vec![],
                passed_gates: vec![],
                blocked_gates: vec![],
            };
        let transaction_runner_enablement =
            ContractRepairApprovalApproveExecutionTransactionRunnerEnablementDryRun {
                status: "approve_execution_transaction_runner_enablement_ready".to_string(),
                action: "approve".to_string(),
                approval_id: approval_id.to_string(),
                switch_name: "approve_execution_transaction_runner_enabled".to_string(),
                approve_action: true,
                transaction_plan_ready: true,
                transaction_execution_ready: true,
                atomicity_scope_ready: true,
                lifecycle_emission_enabled: true,
                contract_mutation_api_enabled: true,
                transaction_runner_enabled: true,
                enablement_prerequisites_ready: true,
                runner_enablement_ready: true,
                would_enable_runner: false,
                would_start_runner: false,
                would_touch_disk: false,
                required_gates: vec![],
                passed_gates: vec![],
                blocked_gates: vec![],
            };

        let attempt_enablement =
            contract_repair_approval_approve_execution_runner_attempt_enablement_dry_run(
                "approve",
                approval_id,
                true,
                &atomic_side_effects_gate,
                &transaction_runner_enablement,
                gates.runner_attempt_enabled,
            );
        let attempt = contract_repair_approval_approve_execution_runner_attempt(
            "approve",
            approval_id,
            true,
            &atomic_side_effects_gate,
            gates.runner_attempt_enabled,
        );

        assert!(attempt_enablement.runner_attempt_enabled);
        assert!(attempt_enablement.runner_attempt_enablement_ready);
        assert!(attempt_enablement.blocked_gates.is_empty());
        assert!(attempt.runner_attempt_enabled);
        assert!(attempt.runner_attempt_ready);
        assert!(attempt.blocked_by.is_empty());
        assert!(!attempt.would_start_runner);

        let execution_enablement =
            contract_repair_approval_approve_execution_runner_execution_enablement_dry_run(
                &attempt,
                &attempt_enablement,
                gates.runner_execution_enabled,
            );
        let outcome = contract_repair_approval_approve_execution_runner_outcome(
            &attempt,
            gates.runner_execution_enabled,
        );

        assert!(execution_enablement.runner_execution_enabled);
        assert!(execution_enablement.runner_execution_enablement_ready);
        assert!(execution_enablement.blocked_gates.is_empty());
        assert!(outcome.runner_execution_enabled);
        assert!(outcome.runner_execution_ready);
        assert!(outcome.blocked_by.is_empty());
        assert!(!outcome.would_start_runner);
        assert!(!outcome.would_commit_transaction);

        let route_dispatch_enablement =
            contract_repair_approval_approve_execution_runner_route_dispatch_enablement_dry_run(
                &outcome,
                &execution_enablement,
                gates.route_dispatch_enabled,
            );
        let dispatch_gate = contract_repair_approval_approve_execution_runner_dispatch_gate(
            &outcome,
            gates.route_dispatch_enabled,
        );

        assert!(route_dispatch_enablement.route_dispatch_enabled);
        assert!(route_dispatch_enablement.route_dispatch_enablement_ready);
        assert!(route_dispatch_enablement.blocked_gates.is_empty());
        assert!(dispatch_gate.route_dispatch_enabled);
        assert!(dispatch_gate.dispatch_ready);
        assert!(dispatch_gate.blocked_gates.is_empty());
        assert!(!dispatch_gate.would_return_success);
        assert!(!dispatch_gate.would_touch_disk);
    }

    #[test]
    fn contract_repair_approval_runner_call_gate_follows_live_map_switch() {
        let approval_id = "contract-repair-apr-runner-call-test";
        let gates = ContractRepairApprovalApproveLiveRouteGates {
            runner_call_enabled: true,
            ..contract_repair_approval_approve_live_route_gates()
        };
        let handoff = ContractRepairApprovalApproveExecutionRunnerHandoff {
            status: "approve_execution_runner_handoff_ready".to_string(),
            action: "approve".to_string(),
            approval_id: approval_id.to_string(),
            dispatch_ready: true,
            route_dispatch_enabled: true,
            handoff_ready: true,
            expected_http_status: 200,
            expected_route_status: "review_approve_executed".to_string(),
            would_call_runner: false,
            would_return_success: false,
            would_persist_any_side_effect: false,
            would_touch_disk: false,
            blocked_by: vec![],
        };
        let route_dispatch_enablement =
            ContractRepairApprovalApproveExecutionRunnerRouteDispatchEnablementDryRun {
                status: "approve_execution_runner_route_dispatch_enablement_ready".to_string(),
                action: "approve".to_string(),
                approval_id: approval_id.to_string(),
                switch_name: "approve_execution_runner_route_dispatch_enabled".to_string(),
                branch_selected: true,
                runner_execution_ready: true,
                runner_execution_enablement_ready: true,
                route_dispatch_enabled: true,
                enablement_prerequisites_ready: true,
                route_dispatch_enablement_ready: true,
                would_enable_route_dispatch: false,
                would_enter_runner_branch: false,
                would_return_success: false,
                would_persist_any_side_effect: false,
                would_touch_disk: false,
                required_gates: vec![],
                passed_gates: vec![],
                blocked_gates: vec![],
            };

        let call_enablement =
            contract_repair_approval_approve_execution_runner_call_enablement_dry_run(
                &handoff,
                &route_dispatch_enablement,
                gates.runner_call_enabled,
            );
        let call_dry_run = contract_repair_approval_approve_execution_runner_call_dry_run(
            &handoff,
            gates.runner_call_enabled,
        );
        let call_readiness =
            contract_repair_approval_approve_execution_runner_call_readiness_dry_run(
                &call_dry_run,
                &call_enablement,
            );

        assert!(call_enablement.runner_call_enabled);
        assert!(call_enablement.runner_call_enablement_ready);
        assert!(call_enablement.blocked_gates.is_empty());
        assert!(!call_enablement.would_call_runner);
        assert!(!call_enablement.would_commit_transaction);
        assert!(!call_enablement.would_touch_disk);
        assert!(call_dry_run.runner_call_enabled);
        assert!(call_dry_run.call_ready);
        assert_eq!(
            call_dry_run.expected_runner_result,
            "approve_execution_committed"
        );
        assert!(!call_dry_run.would_call_runner);
        assert!(!call_dry_run.would_return_success);
        assert!(!call_dry_run.would_persist_any_side_effect);
        assert!(!call_dry_run.would_commit_transaction);
        assert!(!call_dry_run.would_rollback_on_error);
        assert!(!call_dry_run.would_touch_disk);
        assert!(call_dry_run.blocked_by.is_empty());
        assert!(call_readiness.runner_call_enabled);
        assert!(call_readiness.runner_call_enablement_ready);
        assert!(call_readiness.call_ready);
        assert!(call_readiness.blocked_gates.is_empty());
        assert!(!call_readiness.would_call_runner);
        assert!(!call_readiness.would_unblock_body);
        assert!(!call_readiness.would_unblock_control_readiness);
    }

    #[test]
    fn contract_repair_approval_runner_body_gate_follows_live_map_switch() {
        let approval_id = "contract-repair-apr-runner-body-test";
        let gates = ContractRepairApprovalApproveLiveRouteGates {
            runner_body_enabled: true,
            ..contract_repair_approval_approve_live_route_gates()
        };
        let call_dry_run = ContractRepairApprovalApproveExecutionRunnerCallDryRun {
            status: "approve_execution_runner_call_ready".to_string(),
            action: "approve".to_string(),
            approval_id: approval_id.to_string(),
            handoff_ready: true,
            runner_call_enabled: true,
            call_ready: true,
            expected_runner_result: "approve_execution_committed".to_string(),
            would_call_runner: false,
            would_return_success: false,
            would_persist_any_side_effect: false,
            would_commit_transaction: false,
            would_rollback_on_error: false,
            would_touch_disk: false,
            blocked_by: vec![],
        };
        let atomic_side_effects_gate =
            ContractRepairApprovalApproveExecutionAtomicSideEffectsGate {
                status: "approve_execution_atomic_side_effects_ready".to_string(),
                action: "approve".to_string(),
                approval_id: approval_id.to_string(),
                lifecycle_plan_ready: true,
                contract_mutation_plan_ready: true,
                recovery_marker_persistence_plan_ready: true,
                transaction_commit_plan_ready: true,
                atomic_side_effects_plan_ready: true,
                lifecycle_effects_ready: true,
                contract_mutation_ready: true,
                recovery_marker_persistence_ready: true,
                transaction_commit_ready: true,
                atomic_side_effects_enabled: true,
                atomic_side_effects_ready: true,
                would_emit_lifecycle: false,
                would_mutate_contract: false,
                would_persist_recovery_marker: false,
                would_commit_transaction: false,
                would_touch_disk: false,
                required_gates: vec![],
                passed_gates: vec![],
                blocked_gates: vec![],
            };

        let body_enablement =
            contract_repair_approval_approve_execution_runner_body_enablement_dry_run(
                &call_dry_run,
                &atomic_side_effects_gate,
                gates.runner_body_enabled,
            );
        let body_dry_run = contract_repair_approval_approve_execution_runner_call_body_dry_run(
            &call_dry_run,
            &atomic_side_effects_gate,
            gates.runner_body_enabled,
        );
        let body_readiness =
            contract_repair_approval_approve_execution_runner_body_readiness_dry_run(
                &body_dry_run,
                &body_enablement,
            );

        assert!(body_enablement.runner_body_enabled);
        assert!(body_enablement.runner_body_enablement_ready);
        assert!(body_enablement.blocked_gates.is_empty());
        assert!(!body_enablement.would_enter_body);
        assert!(!body_enablement.would_emit_lifecycle);
        assert!(!body_enablement.would_mutate_contract);
        assert!(!body_enablement.would_persist_recovery_marker);
        assert!(!body_enablement.would_commit_transaction);
        assert!(!body_enablement.would_touch_disk);
        assert!(body_dry_run.runner_body_enabled);
        assert!(body_dry_run.body_ready);
        assert!(body_dry_run.blocked_by.is_empty());
        assert!(!body_dry_run.would_enter_body);
        assert!(!body_dry_run.would_return_success);
        assert!(!body_dry_run.would_touch_disk);
        assert!(body_readiness.runner_body_enabled);
        assert!(body_readiness.runner_body_enablement_ready);
        assert!(body_readiness.body_ready);
        assert!(body_readiness.blocked_gates.is_empty());
        assert!(!body_readiness.would_unblock_phase_sequence);
        assert!(!body_readiness.would_unblock_control_readiness);
    }

    #[test]
    fn contract_repair_approval_runner_phase_gate_follows_live_map_switch() {
        let approval_id = "contract-repair-apr-runner-phase-test";
        let gates = ContractRepairApprovalApproveLiveRouteGates {
            phase_execution_enabled: true,
            ..contract_repair_approval_approve_live_route_gates()
        };
        let body_dry_run = ContractRepairApprovalApproveExecutionRunnerCallBodyDryRun {
            status: "approve_execution_runner_call_body_ready".to_string(),
            action: "approve".to_string(),
            approval_id: approval_id.to_string(),
            runner_entrypoint: "contract_repair_approval_approve_execution_runner".to_string(),
            call_ready: true,
            side_effect_bundle_ready: true,
            atomic_side_effects_ready: true,
            runner_body_enabled: true,
            body_ready: true,
            would_enter_body: false,
            would_emit_lifecycle: false,
            would_mutate_contract: false,
            would_persist_recovery_marker: false,
            would_commit_transaction: false,
            would_rollback_on_error: false,
            would_return_success: false,
            would_touch_disk: false,
            blocked_by: vec![],
        };
        let runner_dry_run = ContractRepairApprovalApproveExecutionTransactionRunnerDryRun {
            status: "approve_execution_transaction_runner_ready".to_string(),
            action: "approve".to_string(),
            approval_id: approval_id.to_string(),
            runner_enabled: true,
            admission_ready: true,
            transaction_plan_ready: true,
            commit_barrier_ready: true,
            recovery_marker_ready: true,
            rollback_plan_ready: true,
            commit_ready: true,
            would_start_runner: false,
            would_write_recovery_marker: false,
            would_commit_transaction: false,
            would_rollback_on_error: false,
            phase_order: vec![
                "write_recovery_marker".to_string(),
                "emit_lifecycle_event".to_string(),
                "write_contract_source".to_string(),
                "clear_recovery_marker".to_string(),
            ],
            rollback_order: vec![
                "restore_contract_source".to_string(),
                "restore_approval_record".to_string(),
                "mark_recovery_marker_rolled_back".to_string(),
            ],
            blocked_by: vec![],
        };

        let phase_enablement =
            contract_repair_approval_approve_execution_runner_phase_execution_enablement_dry_run(
                &body_dry_run,
                &runner_dry_run,
                gates.phase_execution_enabled,
            );
        let phase_sequence =
            contract_repair_approval_approve_execution_runner_body_phase_sequence_dry_run(
                &body_dry_run,
                &runner_dry_run,
                gates.phase_execution_enabled,
            );
        let phase_readiness =
            contract_repair_approval_approve_execution_runner_phases_readiness_dry_run(
                &phase_sequence,
                &phase_enablement,
            );

        assert!(phase_enablement.phase_execution_enabled);
        assert!(phase_enablement.phase_execution_enablement_ready);
        assert!(phase_enablement.blocked_gates.is_empty());
        assert!(!phase_enablement.would_execute_phase_sequence);
        assert!(!phase_enablement.would_execute_rollback_sequence);
        assert!(!phase_enablement.would_return_success);
        assert!(!phase_enablement.would_touch_disk);
        assert!(phase_sequence.phase_execution_enabled);
        assert!(phase_sequence.phases_ready);
        assert!(phase_sequence.blocked_by.is_empty());
        assert!(!phase_sequence.would_execute_phase_sequence);
        assert!(!phase_sequence.would_execute_rollback_sequence);
        assert!(!phase_sequence.would_return_success);
        assert!(!phase_sequence.would_touch_disk);
        assert!(phase_readiness.phase_execution_enabled);
        assert!(phase_readiness.phase_execution_enablement_ready);
        assert!(phase_readiness.phases_ready);
        assert!(phase_readiness.blocked_gates.is_empty());
        assert!(!phase_readiness.would_unblock_lifecycle_phase);
        assert!(!phase_readiness.would_unblock_control_readiness);
    }

    #[test]
    fn contract_repair_approval_runner_activation_gate_follows_live_map_switch() {
        let approval_id = "contract-repair-apr-runner-activation-test";
        let gates = ContractRepairApprovalApproveLiveRouteGates {
            runner_activation_enabled: true,
            ..contract_repair_approval_approve_live_route_gates()
        };
        let enablement_plan = ContractRepairApprovalApproveExecutionRunnerEnablementPlanDryRun {
            status: "approve_execution_runner_enablement_plan_ready".to_string(),
            action: "approve".to_string(),
            approval_id: approval_id.to_string(),
            structural_plan_ready: true,
            runner_control_ready: true,
            phase_chain_ready: true,
            rollback_chain_ready: true,
            side_effect_enablement_ready: true,
            runner_activation_enabled: gates.runner_activation_enabled,
            runner_activation_ready: true,
            would_activate_runner: false,
            would_return_success: false,
            would_touch_disk: false,
            required_enablements: vec!["approve_execution_runner_activation_enabled".to_string()],
            passed_enablements: vec!["approve_execution_runner_activation_enabled".to_string()],
            blocked_enablements: vec![],
            blocked_by: vec![],
        };
        let rollback_enablement =
            ContractRepairApprovalApproveExecutionRunnerRollbackExecutionPhaseEnablementDryRun {
                status: "approve_execution_runner_rollback_execution_phase_enablement_ready"
                    .to_string(),
                action: "approve".to_string(),
                approval_id: approval_id.to_string(),
                switch_name: "approve_execution_runner_rollback_execution_enabled".to_string(),
                phase_sequence_ready: true,
                rollback_sequence_ready: true,
                commit_phase_enablement_ready: true,
                commit_phase_ready: true,
                rollback_phase_present: true,
                rollback_plan_ready: true,
                rollback_execution_enabled: true,
                enablement_prerequisites_ready: true,
                rollback_execution_enablement_ready: true,
                rollback_order: vec![
                    "restore_contract_source".to_string(),
                    "restore_approval_record".to_string(),
                    "mark_recovery_marker_rolled_back".to_string(),
                ],
                would_enable_rollback_execution: false,
                would_restore_contract_source: false,
                would_restore_approval_record: false,
                would_mark_recovery_marker_rolled_back: false,
                would_rollback_on_error: false,
                would_return_success: false,
                would_touch_disk: false,
                required_gates: vec![],
                passed_gates: vec![],
                blocked_gates: vec![],
            };

        let activation_enablement =
            contract_repair_approval_approve_execution_runner_activation_enablement_dry_run(
                &enablement_plan,
                &rollback_enablement,
            );
        let activation_readiness =
            contract_repair_approval_approve_execution_runner_activation_enablement_readiness_dry_run(
                &activation_enablement,
            );
        let activation_path =
            contract_repair_approval_approve_execution_runner_activation_path_dry_run(
                &activation_enablement,
            );

        assert!(activation_enablement.runner_activation_enabled);
        assert!(activation_enablement.runner_activation_enablement_ready);
        assert!(activation_enablement.blocked_gates.is_empty());
        assert!(activation_enablement.blocked_enablements.is_empty());
        assert!(!activation_enablement.would_enable_runner_activation);
        assert!(!activation_enablement.would_activate_runner);
        assert!(!activation_enablement.would_return_success);
        assert!(!activation_enablement.would_touch_disk);
        assert!(activation_readiness.runner_activation_enabled);
        assert!(activation_readiness.runner_activation_enablement_ready);
        assert!(activation_readiness.blocked_gates.is_empty());
        assert!(!activation_readiness.would_enable_runner_activation);
        assert!(!activation_readiness.would_unblock_activation_path);
        assert!(!activation_readiness.would_touch_disk);
        assert!(activation_path.activation_path_ready);
        assert!(!activation_path.would_activate_runner);
        assert!(!activation_path.would_return_success);
        assert!(!activation_path.would_touch_disk);
    }

    #[test]
    fn contract_repair_approval_admission_gate_enables_runner_when_gate_is_open() {
        let admission_gate = ContractRepairApprovalApproveExecutionAdmissionGate {
            status: "approve_execution_admission_ready_blocked".to_string(),
            action: "approve".to_string(),
            approval_id: "contract-repair-apr-admission-test".to_string(),
            transaction_plan_ready: true,
            transaction_execution_ready: true,
            atomicity_scope_ready: true,
            transaction_runner_enabled: false,
            admission_ready: false,
            partial_execution_allowed: false,
            would_start_transaction: false,
            would_persist_any_side_effect: false,
            required_gates: vec![
                "approve_action_required".to_string(),
                "transaction_plan_ready".to_string(),
                "atomicity_scope_ready".to_string(),
                "lifecycle_event_emission_enabled".to_string(),
                "contract_mutation_api_enabled".to_string(),
                "approve_execution_transaction_runner_enabled".to_string(),
            ],
            passed_gates: vec![
                "approve_action_required".to_string(),
                "transaction_plan_ready".to_string(),
                "atomicity_scope_ready".to_string(),
                "lifecycle_event_emission_enabled".to_string(),
                "contract_mutation_api_enabled".to_string(),
            ],
            blocked_gates: vec!["approve_execution_transaction_runner_enabled".to_string()],
        };

        let locked_gate = contract_repair_approval_approve_execution_admission_gate_with_gate(
            &admission_gate,
            false,
        );
        assert_eq!(locked_gate, admission_gate);

        let open_gate = contract_repair_approval_approve_execution_admission_gate_with_gate(
            &admission_gate,
            true,
        );
        assert_eq!(open_gate.status, "approve_execution_admission_ready");
        assert!(open_gate.transaction_runner_enabled);
        assert!(open_gate.admission_ready);
        assert!(!open_gate.partial_execution_allowed);
        assert!(open_gate.would_start_transaction);
        assert!(!open_gate.would_persist_any_side_effect);
        assert!(open_gate.blocked_gates.is_empty());
        assert!(open_gate
            .passed_gates
            .contains(&"approve_execution_transaction_runner_enabled".to_string()));
    }

    #[test]
    fn contract_repair_request_validation_requires_changed_fields() {
        let mut request = valid_request();
        request.changed_fields.clear();

        let error = validate_contract_repair_approval_request(&request).unwrap_err();

        assert_eq!(error.0, StatusCode::BAD_REQUEST);
        assert!(error.1.contains("contract_repair_changed_fields_missing"));
    }

    #[test]
    fn contract_repair_request_validation_rejects_enabled_mutation() {
        let mut request = valid_request();
        request.mutation_enabled = true;

        let error = validate_contract_repair_approval_request(&request).unwrap_err();

        assert_eq!(error.0, StatusCode::BAD_REQUEST);
        assert!(error
            .1
            .contains("contract_repair_mutation_must_be_disabled"));
    }

    #[test]
    fn contract_repair_approval_preview_is_stable_across_field_order() {
        let mut left = valid_request();
        left.changed_fields = vec!["type_name".to_string(), "nullable".to_string()];
        let mut right = valid_request();
        right.changed_fields = vec!["nullable".to_string(), "type_name".to_string()];

        let left_preview = contract_repair_approval_record_preview(&left).unwrap();
        let right_preview = contract_repair_approval_record_preview(&right).unwrap();

        assert_eq!(left_preview.idempotency_key, right_preview.idempotency_key);
        assert_eq!(left_preview.approval_id, right_preview.approval_id);
        assert_eq!(
            left_preview.status,
            "approval_record_preview_only".to_string()
        );
        assert!(!left_preview.would_persist);
        assert!(!left_preview.persistence_enabled);
    }

    #[test]
    fn contract_repair_review_intent_allows_enabled_review_preflight() {
        let request = ContractRepairApprovalReviewIntentRequest {
            action: "claim".to_string(),
            reviewer_id: "reviewer-a".to_string(),
            reason: "inspect preview".to_string(),
            review_enabled: true,
        };

        assert!(validate_contract_repair_approval_review_intent(&request).is_ok());

        let approve_request = ContractRepairApprovalReviewIntentRequest {
            action: "approve".to_string(),
            reviewer_id: "reviewer-a".to_string(),
            reason: "approve preview".to_string(),
            review_enabled: true,
        };

        assert!(validate_contract_repair_approval_review_intent(&approve_request).is_ok());

        let reject_request = ContractRepairApprovalReviewIntentRequest {
            action: "reject".to_string(),
            reviewer_id: "reviewer-a".to_string(),
            reason: "reject preview".to_string(),
            review_enabled: true,
        };

        assert!(validate_contract_repair_approval_review_intent(&reject_request).is_ok());
    }

    #[test]
    fn transient_review_status_tracks_intent_action() {
        assert_eq!(
            transient_review_status_for_action("claim"),
            "claim_intent_recorded"
        );
        assert_eq!(
            transient_review_status_for_action("approve"),
            "approve_intent_recorded"
        );
        assert_eq!(
            transient_review_status_for_action("reject"),
            "reject_intent_recorded"
        );
        assert_eq!(executed_review_status_for_action("claim"), "claim_executed");
        assert_eq!(
            executed_review_status_for_action("approve"),
            "approve_executed"
        );
        assert_eq!(
            executed_review_status_for_action("reject"),
            "reject_executed"
        );
    }

    #[test]
    fn contract_repair_review_execution_gate_tracks_ready_preconditions_but_stays_fail_closed() {
        let storage_readiness_gate = ContractRepairApprovalStorageReadinessGate {
            status: "blocked".to_string(),
            persistence_enabled: false,
            store_ready: true,
            schema_ready: true,
            idempotency_ready: true,
            snapshot_ready: true,
            ready_gates: vec![
                "record_schema_preview_ready".to_string(),
                "idempotency_key_ready".to_string(),
                "persistence_path_preview_ready".to_string(),
                "record_snapshot_preview_ready".to_string(),
                "contract_repair_approval_store_ready".to_string(),
            ],
            blocked_gates: vec!["approval_persistence_enabled".to_string()],
        };
        let idempotency_precheck = ContractRepairApprovalIdempotencyPrecheck {
            status: "precheck_checked_blocked".to_string(),
            idempotency_key: "sha256:test".to_string(),
            candidate_record_key: "contract-repair-apr-test".to_string(),
            store_lookup_enabled: true,
            existing_record_found: true,
            conflict_detected: false,
            safe_to_write: false,
            blocked_by: vec!["approval_persistence_enabled".to_string()],
        };
        let authorization_precheck = ContractRepairApprovalReviewerAuthorizationPrecheck {
            status: "authorization_precheck_denied".to_string(),
            policy_version: CONTRACT_REPAIR_REVIEWER_POLICY_VERSION.to_string(),
            required_role: CONTRACT_REPAIR_REVIEWER_REQUIRED_ROLE.to_string(),
            grant_source: "not_configured".to_string(),
            reviewer_id: "user:0".to_string(),
            auth_subject: "user:0".to_string(),
            identity_format_valid: true,
            identity_matches_auth_subject: true,
            role_policy_available: true,
            authorized: false,
            blocked_by: vec!["formal_reviewer_role_grant_missing".to_string()],
        };
        let gate = contract_repair_review_execution_gate(
            &auth::UserId(0),
            "claim",
            false,
            true,
            "user:0",
            &storage_readiness_gate,
            &idempotency_precheck,
            &authorization_precheck,
            false,
            false,
        );

        assert_eq!(gate.status, "blocked");
        assert!(gate
            .passed_gates
            .contains(&"transient_preview_exists".to_string()));
        assert!(gate
            .passed_gates
            .contains(&"review_intent_valid".to_string()));
        assert!(gate
            .passed_gates
            .contains(&"reviewer_identity_present".to_string()));
        assert!(gate
            .passed_gates
            .contains(&"reviewer_identity_format_valid".to_string()));
        assert!(gate
            .passed_gates
            .contains(&"reviewer_identity_matches_auth_subject".to_string()));
        assert!(gate
            .passed_gates
            .contains(&"contract_repair_approval_store_ready".to_string()));
        assert!(gate
            .passed_gates
            .contains(&"idempotency_precheck_passed".to_string()));
        assert!(gate
            .blocked_gates
            .contains(&"review_workflow_enabled".to_string()));
        assert!(gate
            .blocked_gates
            .contains(&"approval_persistence_enabled".to_string()));
        assert!(gate
            .blocked_gates
            .contains(&"formal_reviewer_authorized".to_string()));
    }

    #[test]
    fn contract_repair_review_execution_gate_narrows_approve_preflight_blockers() {
        let storage_readiness_gate = ContractRepairApprovalStorageReadinessGate {
            status: "ready_blocked".to_string(),
            persistence_enabled: false,
            store_ready: true,
            schema_ready: true,
            idempotency_ready: true,
            snapshot_ready: true,
            ready_gates: vec![
                "record_schema_preview_ready".to_string(),
                "idempotency_key_ready".to_string(),
                "persistence_path_preview_ready".to_string(),
                "record_snapshot_preview_ready".to_string(),
                "contract_repair_approval_store_ready".to_string(),
            ],
            blocked_gates: vec!["approval_persistence_enabled".to_string()],
        };
        let idempotency_precheck = ContractRepairApprovalIdempotencyPrecheck {
            status: "precheck_checked_blocked".to_string(),
            idempotency_key: "sha256:test".to_string(),
            candidate_record_key: "contract-repair-apr-test".to_string(),
            store_lookup_enabled: true,
            existing_record_found: true,
            conflict_detected: false,
            safe_to_write: false,
            blocked_by: vec!["approval_persistence_enabled".to_string()],
        };
        let authorization_precheck = ContractRepairApprovalReviewerAuthorizationPrecheck {
            status: "authorization_precheck_authorized".to_string(),
            policy_version: CONTRACT_REPAIR_REVIEWER_POLICY_VERSION.to_string(),
            required_role: CONTRACT_REPAIR_REVIEWER_REQUIRED_ROLE.to_string(),
            grant_source: "file:contract-repair-reviewer-grants.json".to_string(),
            reviewer_id: "user:0".to_string(),
            auth_subject: "user:0".to_string(),
            identity_format_valid: true,
            identity_matches_auth_subject: true,
            role_policy_available: true,
            authorized: true,
            blocked_by: Vec::new(),
        };
        let gate = contract_repair_review_execution_gate(
            &auth::UserId(0),
            "approve",
            true,
            true,
            "user:0",
            &storage_readiness_gate,
            &idempotency_precheck,
            &authorization_precheck,
            false,
            false,
        );

        assert_eq!(gate.status, "blocked");
        assert!(gate
            .passed_gates
            .contains(&"review_workflow_enabled".to_string()));
        assert!(gate
            .passed_gates
            .contains(&"approval_persistence_enabled".to_string()));
        assert!(gate
            .passed_gates
            .contains(&"formal_reviewer_authorized".to_string()));
        assert!(!gate
            .blocked_gates
            .contains(&"review_workflow_enabled".to_string()));
        assert!(!gate
            .blocked_gates
            .contains(&"approval_persistence_enabled".to_string()));
        assert!(gate
            .blocked_gates
            .contains(&"lifecycle_event_emission_enabled".to_string()));
        assert!(gate
            .blocked_gates
            .contains(&"contract_mutation_api_enabled".to_string()));
    }

    #[test]
    fn contract_repair_review_execution_gate_allows_approve_live_route_when_policy_gates_open() {
        let storage_readiness_gate = ContractRepairApprovalStorageReadinessGate {
            status: "ready".to_string(),
            persistence_enabled: true,
            store_ready: true,
            schema_ready: true,
            idempotency_ready: true,
            snapshot_ready: true,
            ready_gates: vec![
                "record_schema_preview_ready".to_string(),
                "idempotency_key_ready".to_string(),
                "persistence_path_preview_ready".to_string(),
                "record_snapshot_preview_ready".to_string(),
                "contract_repair_approval_store_ready".to_string(),
            ],
            blocked_gates: Vec::new(),
        };
        let idempotency_precheck = ContractRepairApprovalIdempotencyPrecheck {
            status: "precheck_passed".to_string(),
            idempotency_key: "sha256:test".to_string(),
            candidate_record_key: "contract-repair-apr-test".to_string(),
            store_lookup_enabled: true,
            existing_record_found: false,
            conflict_detected: false,
            safe_to_write: true,
            blocked_by: Vec::new(),
        };
        let authorization_precheck = ContractRepairApprovalReviewerAuthorizationPrecheck {
            status: "authorization_precheck_authorized".to_string(),
            policy_version: CONTRACT_REPAIR_REVIEWER_POLICY_VERSION.to_string(),
            required_role: CONTRACT_REPAIR_REVIEWER_REQUIRED_ROLE.to_string(),
            grant_source: "file:contract-repair-reviewer-grants.json".to_string(),
            reviewer_id: "user:0".to_string(),
            auth_subject: "user:0".to_string(),
            identity_format_valid: true,
            identity_matches_auth_subject: true,
            role_policy_available: true,
            authorized: true,
            blocked_by: Vec::new(),
        };
        let gate = contract_repair_review_execution_gate(
            &auth::UserId(0),
            "approve",
            true,
            true,
            "user:0",
            &storage_readiness_gate,
            &idempotency_precheck,
            &authorization_precheck,
            true,
            true,
        );

        assert_eq!(gate.status, "ready");
        assert!(gate
            .passed_gates
            .contains(&"lifecycle_event_emission_enabled".to_string()));
        assert!(gate
            .passed_gates
            .contains(&"contract_mutation_api_enabled".to_string()));
        assert!(gate.blocked_gates.is_empty());
    }

    #[test]
    fn contract_repair_review_execution_plan_preview_is_non_executing() {
        let gate = ContractRepairApprovalReviewExecutionGate {
            status: "blocked".to_string(),
            required_gates: vec!["review_workflow_enabled".to_string()],
            passed_gates: Vec::new(),
            blocked_gates: vec![
                "review_workflow_enabled".to_string(),
                "contract_mutation_api_enabled".to_string(),
            ],
        };
        let plan = contract_repair_review_execution_plan_preview("approve", &gate, false, false);

        assert_eq!(plan.status, "execution_plan_preview_only");
        assert!(!plan.execution_enabled);
        assert_eq!(
            plan.target_review_state,
            RuntimeApprovalReviewState::Approved
        );
        assert!(!plan.would_persist_approval_record);
        assert!(!plan.would_mutate_contract);
        assert!(!plan.would_emit_lifecycle_event);
        assert!(plan
            .blocked_by
            .contains(&"contract_mutation_api_enabled".to_string()));
    }

    #[test]
    fn contract_repair_approval_approve_execution_admission_gate_blocks_partial_execution() {
        let transaction_dry_run = ContractRepairApprovalApproveExecutionTransactionDryRun {
            status: "approve_execution_transaction_ready_blocked".to_string(),
            action: "approve".to_string(),
            approval_id: "contract-repair-apr-test".to_string(),
            transaction_plan_ready: true,
            execution_ready: false,
            approved_transition_ready: true,
            record_write_ready: true,
            lifecycle_emission_ready: true,
            lifecycle_append_ready: true,
            contract_writeback_ready: true,
            lifecycle_emission_enabled: false,
            mutation_api_enabled: false,
            would_transition_review: false,
            would_write_approval_record: false,
            would_emit_lifecycle_event: false,
            would_append_lifecycle_entry: false,
            would_write_contract_source: false,
            would_execute_transaction: false,
            step_order: vec![
                "transition_review_state".to_string(),
                "persist_approval_record".to_string(),
                "emit_lifecycle_event".to_string(),
                "append_lifecycle_entry".to_string(),
                "write_contract_source".to_string(),
            ],
            atomicity_scope: vec![
                "approval_record".to_string(),
                "lifecycle_entry".to_string(),
                "contract_source".to_string(),
            ],
            blocked_by: vec![
                "lifecycle_event_emission_enabled".to_string(),
                "contract_mutation_api_enabled".to_string(),
            ],
        };
        let gate = contract_repair_approval_approve_execution_admission_gate(
            "approve",
            "contract-repair-apr-test",
            &transaction_dry_run,
        );

        assert_eq!(gate.status, "approve_execution_admission_ready_blocked");
        assert!(gate.transaction_plan_ready);
        assert!(!gate.transaction_execution_ready);
        assert!(gate.atomicity_scope_ready);
        assert!(!gate.transaction_runner_enabled);
        assert!(!gate.admission_ready);
        assert!(!gate.partial_execution_allowed);
        assert!(!gate.would_start_transaction);
        assert!(!gate.would_persist_any_side_effect);
        assert!(gate
            .passed_gates
            .contains(&"transaction_plan_ready".to_string()));
        assert!(gate
            .passed_gates
            .contains(&"atomicity_scope_ready".to_string()));
        assert!(gate
            .blocked_gates
            .contains(&"lifecycle_event_emission_enabled".to_string()));
        assert!(gate
            .blocked_gates
            .contains(&"contract_mutation_api_enabled".to_string()));
        assert!(gate
            .blocked_gates
            .contains(&"approve_execution_transaction_runner_enabled".to_string()));
    }

    #[tokio::test]
    async fn reviewer_authorization_precheck_requires_role_grant_even_when_subject_matches() {
        let grants_path = std::env::temp_dir().join(format!(
            "quantpilot_missing_contract_repair_grants_{}.json",
            test_stamp()
        ));
        let precheck = contract_repair_approval_reviewer_authorization_precheck(
            &grants_path,
            &auth::UserId(0),
            "user:0",
        )
        .await;

        assert_eq!(precheck.status, "authorization_precheck_denied");
        assert_eq!(
            precheck.policy_version,
            "quantpilot/contract-repair-reviewer-role-policy/v1"
        );
        assert_eq!(precheck.required_role, "contract_repair_reviewer");
        assert_eq!(precheck.grant_source, "not_configured");
        assert_eq!(precheck.reviewer_id, "user:0");
        assert_eq!(precheck.auth_subject, "user:0");
        assert!(precheck.identity_format_valid);
        assert!(precheck.identity_matches_auth_subject);
        assert!(precheck.role_policy_available);
        assert!(!precheck.authorized);
        assert!(precheck
            .blocked_by
            .contains(&"formal_reviewer_role_grant_missing".to_string()));
        assert!(!precheck
            .blocked_by
            .contains(&"reviewer_identity_format_valid".to_string()));
        assert!(!precheck
            .blocked_by
            .contains(&"reviewer_identity_matches_auth_subject".to_string()));
    }

    #[tokio::test]
    async fn reviewer_authorization_precheck_accepts_matching_role_grant_file() {
        let grants_path = std::env::temp_dir().join(format!(
            "quantpilot_contract_repair_grants_{}.json",
            test_stamp()
        ));
        tokio::fs::write(
            &grants_path,
            r#"{"policy_version":"quantpilot/contract-repair-reviewer-role-policy/v1","grants":[{"subject":"user:0","role":"contract_repair_reviewer"}]}"#,
        )
        .await
        .expect("grant file should be writable");

        let precheck = contract_repair_approval_reviewer_authorization_precheck(
            &grants_path,
            &auth::UserId(0),
            "user:0",
        )
        .await;

        assert_eq!(precheck.status, "authorization_precheck_authorized");
        assert_eq!(
            precheck.required_role,
            CONTRACT_REPAIR_REVIEWER_REQUIRED_ROLE
        );
        assert!(precheck
            .grant_source
            .starts_with("file:quantpilot_contract_repair_grants_"));
        assert!(precheck.grant_source.ends_with(".json"));
        assert!(precheck.identity_format_valid);
        assert!(precheck.identity_matches_auth_subject);
        assert!(precheck.role_policy_available);
        assert!(precheck.authorized);
        assert!(precheck.blocked_by.is_empty());

        let _ = tokio::fs::remove_file(grants_path).await;
    }

    #[test]
    fn contract_repair_approval_persistence_plan_preview_does_not_write() {
        let preview = contract_repair_approval_record_preview(&valid_request()).unwrap();
        let plan = contract_repair_approval_persistence_plan_preview(&preview);

        assert_eq!(plan.status, "persistence_plan_preview_only");
        assert!(!plan.persistence_enabled);
        assert!(!plan.would_write_record);
        assert_eq!(plan.store_kind, "contract_repair_approval_records");
        assert_eq!(plan.record_kind, "contract_repair_approval");
        assert_eq!(plan.record_key, preview.approval_id);
        assert_eq!(plan.idempotency_key, preview.idempotency_key);
        assert_eq!(plan.record_source_kind, "transient_preview_cache");
        assert_eq!(
            plan.blocked_by,
            vec!["approval_persistence_enabled".to_string()]
        );
    }

    #[test]
    fn contract_repair_approval_persistence_path_preview_sanitizes_record_key() {
        let mut preview = contract_repair_approval_record_preview(&valid_request()).unwrap();
        preview.approval_id = r"..\contract-repair-apr-owned".to_string();
        let plan = contract_repair_approval_persistence_plan_preview(&preview);
        let path = contract_repair_approval_persistence_path_preview(&plan);

        assert_eq!(path.status, "persistence_path_preview_only");
        assert_eq!(path.store_kind, "contract_repair_approval_records");
        assert_eq!(path.record_key, r"..\contract-repair-apr-owned");
        assert_eq!(path.path_segment, "___contract-repair-apr-owned");
        assert_eq!(path.file_name, "___contract-repair-apr-owned.json");
        assert!(path.atomic_write_required);
        assert!(!path.would_touch_disk);
        assert!(path
            .blocked_by
            .contains(&"contract_repair_approval_store_ready".to_string()));
    }

    #[test]
    fn contract_repair_approval_record_snapshot_preview_is_non_persistent() {
        let preview = contract_repair_approval_record_preview(&valid_request()).unwrap();
        let snapshot = contract_repair_approval_record_snapshot_preview(
            &preview,
            "reject",
            "reviewer-a",
            "needs stronger evidence",
        );

        assert_eq!(snapshot.status, "record_snapshot_preview_only");
        assert_eq!(snapshot.approval_id, preview.approval_id);
        assert_eq!(snapshot.record_kind, "contract_repair_approval");
        assert_eq!(
            snapshot.target_path,
            "memory_schema/decision.dual_ma/cross_up/last_signal_at"
        );
        assert_eq!(snapshot.target_kind, "memory_field");
        assert_eq!(snapshot.changed_fields, vec!["type_name".to_string()]);
        assert_eq!(snapshot.review_state, RuntimeApprovalReviewState::Rejected);
        assert_eq!(snapshot.reviewer_id, "reviewer-a");
        assert_eq!(snapshot.review_reason, "needs stronger evidence");
        assert_eq!(snapshot.idempotency_key, preview.idempotency_key);
        assert!(!snapshot.persistence_enabled);
        assert!(!snapshot.would_write_record);
    }

    #[tokio::test]
    async fn contract_repair_approval_storage_readiness_gate_reflects_missing_store() {
        let preview = contract_repair_approval_record_preview(&valid_request()).unwrap();
        let persistence_plan = contract_repair_approval_persistence_plan_preview(&preview);
        let path_preview = contract_repair_approval_persistence_path_preview(&persistence_plan);
        let store_dir = std::env::temp_dir().join(format!(
            "quantpilot-contract-repair-missing-store-{}",
            preview.approval_id
        ));
        let snapshot = contract_repair_approval_record_snapshot_preview(
            &preview,
            "approve",
            "reviewer-a",
            "preview looks safe",
        );
        let gate = contract_repair_approval_storage_readiness_gate(
            &store_dir,
            &persistence_plan,
            &path_preview,
            &snapshot,
        )
        .await;

        assert_eq!(gate.status, "blocked");
        assert!(!gate.persistence_enabled);
        assert!(!gate.store_ready);
        assert!(gate.schema_ready);
        assert!(gate.idempotency_ready);
        assert!(gate.snapshot_ready);
        assert!(gate
            .ready_gates
            .contains(&"persistence_path_preview_ready".to_string()));
        assert!(gate
            .ready_gates
            .contains(&"record_schema_preview_ready".to_string()));
        assert!(gate
            .blocked_gates
            .contains(&"contract_repair_approval_store_ready".to_string()));
    }

    #[tokio::test]
    async fn contract_repair_approval_storage_dry_run_preview_never_writes() {
        let preview = contract_repair_approval_record_preview(&valid_request()).unwrap();
        let persistence_plan = contract_repair_approval_persistence_plan_preview(&preview);
        let path_preview = contract_repair_approval_persistence_path_preview(&persistence_plan);
        let store_dir = std::env::temp_dir().join(format!(
            "quantpilot-contract-repair-ready-store-{}",
            preview.approval_id
        ));
        fs::create_dir_all(&store_dir).await.unwrap();
        let snapshot = contract_repair_approval_record_snapshot_preview(
            &preview,
            "approve",
            "reviewer-a",
            "preview looks safe",
        );
        let readiness_gate = contract_repair_approval_storage_readiness_gate(
            &store_dir,
            &persistence_plan,
            &path_preview,
            &snapshot,
        )
        .await;
        let dry_run =
            contract_repair_approval_storage_dry_run_preview(&persistence_plan, &readiness_gate);

        assert_eq!(dry_run.status, "dry_run_blocked");
        assert!(readiness_gate.store_ready);
        assert!(readiness_gate
            .ready_gates
            .contains(&"contract_repair_approval_store_ready".to_string()));
        assert!(!readiness_gate
            .blocked_gates
            .contains(&"contract_repair_approval_store_ready".to_string()));
        assert_eq!(
            dry_run.adapter_kind,
            "contract_repair_approval_store_adapter"
        );
        assert_eq!(dry_run.store_kind, "contract_repair_approval_records");
        assert_eq!(dry_run.record_key, preview.approval_id);
        assert!(dry_run.accepted_by_adapter);
        assert!(!dry_run.would_write);
        assert_eq!(dry_run.readiness_status, "blocked");
        assert!(dry_run
            .blocked_by
            .contains(&"approval_persistence_enabled".to_string()));
    }

    #[tokio::test]
    async fn contract_repair_approval_idempotency_precheck_checks_store_but_blocks_safe_write() {
        let preview = contract_repair_approval_record_preview(&valid_request()).unwrap();
        let persistence_plan = contract_repair_approval_persistence_plan_preview(&preview);
        let store_dir = std::env::temp_dir().join(format!(
            "quantpilot-contract-repair-precheck-{}",
            preview.approval_id
        ));
        let precheck = contract_repair_approval_idempotency_precheck(&store_dir, &persistence_plan)
            .await
            .unwrap();

        assert_eq!(precheck.status, "precheck_checked_blocked");
        assert_eq!(precheck.idempotency_key, preview.idempotency_key);
        assert_eq!(precheck.candidate_record_key, preview.approval_id);
        assert!(precheck.store_lookup_enabled);
        assert!(!precheck.existing_record_found);
        assert!(!precheck.conflict_detected);
        assert!(!precheck.safe_to_write);
        assert!(precheck
            .blocked_by
            .contains(&"approval_persistence_enabled".to_string()));
    }

    #[test]
    fn contract_repair_approval_lifecycle_event_dry_run_never_emits() {
        let preview = contract_repair_approval_record_preview(&valid_request()).unwrap();
        let snapshot = contract_repair_approval_record_snapshot_preview(
            &preview,
            "approve",
            "reviewer-a",
            "preview looks safe",
        );
        let transition_dry_run = ContractRepairApprovalReviewTransitionDryRun {
            status: "transition_dry_run_ready_blocked".to_string(),
            approval_id: preview.approval_id.clone(),
            from_review_state: RuntimeApprovalReviewState::Pending,
            target_review_state: RuntimeApprovalReviewState::Approved,
            reviewer_id: "reviewer-a".to_string(),
            reason_code: "review_transition_preview_only".to_string(),
            sequence_no_preview: 1,
            transition_ready: true,
            would_transition: false,
            blocked_by: vec![
                "review_workflow_enabled".to_string(),
                "approval_persistence_enabled".to_string(),
            ],
        };
        let dry_run = contract_repair_approval_lifecycle_event_dry_run(
            &snapshot,
            "approve",
            &transition_dry_run,
            false,
        );

        assert_eq!(dry_run.status, "lifecycle_dry_run_emission_ready_blocked");
        assert_eq!(
            dry_run.event_id,
            format!("contract-repair-review-approve:{}", preview.approval_id)
        );
        assert_eq!(dry_run.event_kind, "contract_repair_approval_review");
        assert_eq!(
            dry_run.target_review_state,
            RuntimeApprovalReviewState::Approved
        );
        assert_eq!(dry_run.actor_id, "reviewer-a");
        assert_eq!(dry_run.reason_code, "review_transition_preview_only");
        assert_eq!(dry_run.sequence_no, 1);
        assert!(dry_run.transition_ready);
        assert!(dry_run.event_payload_ready);
        assert!(dry_run.emission_ready);
        assert!(!dry_run.would_emit);
        assert!(dry_run
            .blocked_by
            .contains(&"approval_persistence_enabled".to_string()));
    }

    #[test]
    fn contract_repair_approval_lifecycle_event_dry_run_emits_when_enabled() {
        let preview = contract_repair_approval_record_preview(&valid_request()).unwrap();
        let snapshot = contract_repair_approval_record_snapshot_preview(
            &preview,
            "claim",
            "reviewer-a",
            "claim review",
        );
        let transition_dry_run = ContractRepairApprovalReviewTransitionDryRun {
            status: "transition_executed".to_string(),
            approval_id: preview.approval_id.clone(),
            from_review_state: RuntimeApprovalReviewState::Pending,
            target_review_state: RuntimeApprovalReviewState::UnderReview,
            reviewer_id: "reviewer-a".to_string(),
            reason_code: "review_transition_preview_only".to_string(),
            sequence_no_preview: 1,
            transition_ready: true,
            would_transition: true,
            blocked_by: Vec::new(),
        };
        let dry_run = contract_repair_approval_lifecycle_event_dry_run(
            &snapshot,
            "claim",
            &transition_dry_run,
            true,
        );

        assert_eq!(dry_run.status, "lifecycle_emitted");
        assert_eq!(
            dry_run.event_id,
            format!("contract-repair-review-claim:{}", preview.approval_id)
        );
        assert!(dry_run.emission_ready);
        assert!(dry_run.would_emit);
        assert!(dry_run.blocked_by.is_empty());
    }

    #[test]
    fn contract_repair_approval_lifecycle_entry_append_dry_run_never_appends() {
        let lifecycle_event_dry_run = ContractRepairApprovalLifecycleEventDryRun {
            status: "lifecycle_dry_run_emission_ready_blocked".to_string(),
            event_id: "contract-repair-review-approve:contract-repair-apr-test".to_string(),
            event_kind: "contract_repair_approval_review".to_string(),
            target_review_state: RuntimeApprovalReviewState::Approved,
            actor_id: "reviewer-a".to_string(),
            reason_code: "review_transition_preview_only".to_string(),
            sequence_no: 2,
            transition_ready: true,
            event_payload_ready: true,
            emission_ready: true,
            would_emit: false,
            blocked_by: vec!["lifecycle_event_emission_enabled".to_string()],
        };
        let dry_run = contract_repair_approval_lifecycle_entry_append_dry_run(
            &lifecycle_event_dry_run,
            false,
        );

        assert_eq!(dry_run.status, "lifecycle_entry_append_ready_blocked");
        assert_eq!(dry_run.event_id, lifecycle_event_dry_run.event_id);
        assert_eq!(dry_run.review_state, RuntimeApprovalReviewState::Approved);
        assert_eq!(dry_run.sequence_no, 2);
        assert!(dry_run.entry_ready);
        assert!(dry_run.emission_ready);
        assert!(dry_run.append_ready);
        assert!(!dry_run.would_append);
        assert!(dry_run
            .blocked_by
            .contains(&"lifecycle_event_emission_enabled".to_string()));
    }

    #[test]
    fn contract_repair_approval_lifecycle_entry_append_dry_run_appends_after_emit() {
        let lifecycle_event_dry_run = ContractRepairApprovalLifecycleEventDryRun {
            status: "lifecycle_emitted".to_string(),
            event_id: "contract-repair-review-claim:contract-repair-apr-test".to_string(),
            event_kind: "contract_repair_approval_review".to_string(),
            target_review_state: RuntimeApprovalReviewState::UnderReview,
            actor_id: "reviewer-a".to_string(),
            reason_code: "review_transition_preview_only".to_string(),
            sequence_no: 3,
            transition_ready: true,
            event_payload_ready: true,
            emission_ready: true,
            would_emit: true,
            blocked_by: Vec::new(),
        };
        let dry_run =
            contract_repair_approval_lifecycle_entry_append_dry_run(&lifecycle_event_dry_run, true);

        assert_eq!(dry_run.status, "lifecycle_entry_appended");
        assert_eq!(
            dry_run.review_state,
            RuntimeApprovalReviewState::UnderReview
        );
        assert_eq!(dry_run.sequence_no, 3);
        assert!(dry_run.append_ready);
        assert!(dry_run.would_append);
        assert!(dry_run.blocked_by.is_empty());
    }

    #[test]
    fn contract_repair_approval_lifecycle_effects_gate_emits_and_appends_when_enabled() {
        let preview = contract_repair_approval_record_preview(&valid_request()).unwrap();
        let snapshot = contract_repair_approval_record_snapshot_preview(
            &preview,
            "approve",
            "reviewer-a",
            "preview looks safe",
        );
        let transition_dry_run = ContractRepairApprovalReviewTransitionDryRun {
            status: "transition_executed".to_string(),
            approval_id: preview.approval_id.clone(),
            from_review_state: RuntimeApprovalReviewState::Pending,
            target_review_state: RuntimeApprovalReviewState::Approved,
            reviewer_id: "reviewer-a".to_string(),
            reason_code: "review_transition_preview_only".to_string(),
            sequence_no_preview: 4,
            transition_ready: true,
            would_transition: true,
            blocked_by: Vec::new(),
        };
        let lifecycle_event_dry_run = contract_repair_approval_lifecycle_event_dry_run(
            &snapshot,
            "approve",
            &transition_dry_run,
            false,
        );

        assert_eq!(lifecycle_event_dry_run.status, "lifecycle_emit_blocked");
        assert!(!lifecycle_event_dry_run.would_emit);
        assert_eq!(
            lifecycle_event_dry_run.blocked_by,
            vec!["lifecycle_event_emission_enabled".to_string()]
        );

        let locked_event =
            contract_repair_approval_lifecycle_event_with_gate(&lifecycle_event_dry_run, false);
        assert_eq!(locked_event, lifecycle_event_dry_run);

        let emitted_event =
            contract_repair_approval_lifecycle_event_with_gate(&lifecycle_event_dry_run, true);
        assert_eq!(emitted_event.status, "lifecycle_emitted");
        assert!(emitted_event.would_emit);
        assert!(emitted_event.blocked_by.is_empty());

        let lifecycle_entry_append_dry_run =
            contract_repair_approval_lifecycle_entry_append_dry_run(&emitted_event, false);
        assert_eq!(
            lifecycle_entry_append_dry_run.status,
            "lifecycle_entry_append_ready_blocked"
        );
        assert!(lifecycle_entry_append_dry_run.append_ready);
        assert!(!lifecycle_entry_append_dry_run.would_append);

        let locked_append = contract_repair_approval_lifecycle_entry_append_with_gate(
            &emitted_event,
            &lifecycle_entry_append_dry_run,
            false,
        );
        assert_eq!(locked_append, lifecycle_entry_append_dry_run);

        let appended_entry = contract_repair_approval_lifecycle_entry_append_with_gate(
            &emitted_event,
            &lifecycle_entry_append_dry_run,
            true,
        );
        assert_eq!(appended_entry.status, "lifecycle_entry_appended");
        assert!(appended_entry.would_append);
        assert!(appended_entry.blocked_by.is_empty());

        let lifecycle_enablement_gate = contract_repair_approval_lifecycle_emission_enablement_gate(
            &emitted_event,
            &appended_entry,
        );
        assert_eq!(
            lifecycle_enablement_gate.status,
            "lifecycle_emission_enablement_ready"
        );
        assert!(lifecycle_enablement_gate.lifecycle_effects_ready);
        assert!(lifecycle_enablement_gate.lifecycle_event_emission_enabled);
        assert!(lifecycle_enablement_gate.lifecycle_entry_append_enabled);
        assert!(lifecycle_enablement_gate.would_emit);
        assert!(lifecycle_enablement_gate.would_append);
        assert!(lifecycle_enablement_gate.would_touch_lifecycle_log);
        assert!(lifecycle_enablement_gate.blocked_gates.is_empty());

        let lifecycle_entry_preview =
            contract_repair_approval_lifecycle_entry_preview(&appended_entry);
        assert_eq!(lifecycle_entry_preview.event_id, emitted_event.event_id);
        assert_eq!(
            lifecycle_entry_preview.review_state,
            RuntimeApprovalReviewState::Approved
        );
        assert_eq!(lifecycle_entry_preview.sequence_no, 4);
    }

    #[test]
    fn contract_repair_approval_contract_writeback_dry_run_never_mutates() {
        let preview = contract_repair_approval_record_preview(&valid_request()).unwrap();
        let snapshot = contract_repair_approval_record_snapshot_preview(
            &preview,
            "approve",
            "reviewer-a",
            "preview looks safe",
        );
        let transition_dry_run = ContractRepairApprovalReviewTransitionDryRun {
            status: "transition_dry_run_ready_blocked".to_string(),
            approval_id: preview.approval_id.clone(),
            from_review_state: RuntimeApprovalReviewState::Pending,
            target_review_state: RuntimeApprovalReviewState::Approved,
            reviewer_id: "reviewer-a".to_string(),
            reason_code: "review_transition_preview_only".to_string(),
            sequence_no_preview: 1,
            transition_ready: true,
            would_transition: false,
            blocked_by: vec!["contract_mutation_api_enabled".to_string()],
        };
        let lifecycle_entry_append_dry_run = ContractRepairApprovalLifecycleEntryAppendDryRun {
            status: "lifecycle_entry_append_ready_blocked".to_string(),
            event_id: format!("contract-repair-review-approve:{}", preview.approval_id),
            review_state: RuntimeApprovalReviewState::Approved,
            sequence_no: 1,
            entry_ready: true,
            emission_ready: true,
            append_ready: true,
            would_append: false,
            blocked_by: vec!["lifecycle_event_emission_enabled".to_string()],
        };
        let patch_plan = contract_repair_approval_contract_patch_plan_preview(&snapshot);
        let patch_apply_dry_run = ready_patch_apply_dry_run();
        let source_write_dry_run = ready_source_write_dry_run();
        let dry_run = contract_repair_approval_contract_writeback_dry_run(
            &snapshot,
            "approve",
            &transition_dry_run,
            &lifecycle_entry_append_dry_run,
            &ready_source_resolution(),
            patch_plan,
            &patch_apply_dry_run,
            &source_write_dry_run,
        );

        assert_eq!(dry_run.status, "contract_writeback_dry_run_ready_blocked");
        assert_eq!(dry_run.patch_kind, "memory_schema_patch");
        assert_eq!(
            dry_run.target_path,
            "memory_schema/decision.dual_ma/cross_up/last_signal_at"
        );
        assert_eq!(dry_run.target_kind, "memory_field");
        assert_eq!(dry_run.changed_fields, vec!["type_name".to_string()]);
        assert_eq!(
            dry_run.patch_payload.get("type_name"),
            Some(&serde_json::Value::String("time?".to_string()))
        );
        assert_eq!(dry_run.patch_plan.status, "contract_patch_plan_ready");
        assert_eq!(dry_run.patch_plan.plan_kind, "memory_schema_patch");
        assert!(dry_run.patch_plan.contract_patch_ready);
        assert!(!dry_run.patch_plan.evidence_sample_patch);
        assert_eq!(dry_run.patch_plan.operations.len(), 1);
        assert_eq!(
            dry_run.patch_plan.operations[0].selector,
            "machines[machine_id=decision.dual_ma].memory[name=last_signal_at]"
        );
        assert_eq!(dry_run.patch_plan.operations[0].field_name, "type_name");
        assert_eq!(
            dry_run.patch_apply_dry_run.status,
            "contract_patch_apply_ready_blocked"
        );
        assert!(dry_run.patch_apply_dry_run.apply_ready);
        assert_eq!(dry_run.patch_apply_dry_run.applied_operation_count, 1);
        assert!(!dry_run.patch_apply_dry_run.would_persist_source);
        assert_eq!(
            dry_run.source_write_dry_run.status,
            "contract_source_write_ready_blocked"
        );
        assert!(dry_run.source_write_dry_run.write_ready);
        assert!(dry_run.source_write_dry_run.atomic_write_required);
        assert!(!dry_run.source_write_dry_run.would_write_source);
        assert!(!dry_run.source_write_dry_run.would_touch_disk);
        assert!(dry_run.eligible_after_approval);
        assert!(dry_run.patch_ready);
        assert!(dry_run.patch_payload_ready);
        assert!(dry_run.contract_source_ready);
        assert!(dry_run.transition_ready);
        assert!(dry_run.lifecycle_append_ready);
        assert!(dry_run.writeback_ready);
        assert!(dry_run.missing_patch_fields.is_empty());
        assert!(dry_run.missing_contract_source_fields.is_empty());
        assert!(!dry_run.would_mutate_contract);
        assert!(dry_run
            .blocked_by
            .contains(&"contract_mutation_api_enabled".to_string()));
    }

    #[test]
    fn contract_repair_approval_contract_writeback_dry_run_requires_patch_payload() {
        let mut request = valid_request();
        request.patch_payload.clear();
        let preview = contract_repair_approval_record_preview(&request).unwrap();
        let snapshot = contract_repair_approval_record_snapshot_preview(
            &preview,
            "approve",
            "reviewer-a",
            "preview looks safe",
        );
        let transition_dry_run = ContractRepairApprovalReviewTransitionDryRun {
            status: "transition_dry_run_ready_blocked".to_string(),
            approval_id: preview.approval_id.clone(),
            from_review_state: RuntimeApprovalReviewState::Pending,
            target_review_state: RuntimeApprovalReviewState::Approved,
            reviewer_id: "reviewer-a".to_string(),
            reason_code: "review_transition_preview_only".to_string(),
            sequence_no_preview: 1,
            transition_ready: true,
            would_transition: false,
            blocked_by: vec!["contract_mutation_api_enabled".to_string()],
        };
        let lifecycle_entry_append_dry_run = ContractRepairApprovalLifecycleEntryAppendDryRun {
            status: "lifecycle_entry_append_ready_blocked".to_string(),
            event_id: format!("contract-repair-review-approve:{}", preview.approval_id),
            review_state: RuntimeApprovalReviewState::Approved,
            sequence_no: 1,
            entry_ready: true,
            emission_ready: true,
            append_ready: true,
            would_append: false,
            blocked_by: vec!["lifecycle_event_emission_enabled".to_string()],
        };
        let patch_plan = contract_repair_approval_contract_patch_plan_preview(&snapshot);
        let patch_apply_dry_run = blocked_patch_apply_dry_run();
        let source_write_dry_run = blocked_source_write_dry_run();
        let dry_run = contract_repair_approval_contract_writeback_dry_run(
            &snapshot,
            "approve",
            &transition_dry_run,
            &lifecycle_entry_append_dry_run,
            &ready_source_resolution(),
            patch_plan,
            &patch_apply_dry_run,
            &source_write_dry_run,
        );

        assert_eq!(dry_run.status, "contract_writeback_dry_run_blocked");
        assert!(dry_run.patch_ready);
        assert!(!dry_run.patch_payload_ready);
        assert_eq!(dry_run.missing_patch_fields, vec!["type_name".to_string()]);
        assert!(!dry_run.writeback_ready);
        assert!(dry_run
            .blocked_by
            .contains(&"contract_patch_payload_ready".to_string()));
        assert!(dry_run
            .blocked_by
            .contains(&"contract_patch_apply_ready".to_string()));
        assert!(dry_run
            .blocked_by
            .contains(&"contract_source_write_ready".to_string()));
    }

    #[test]
    fn contract_repair_approval_contract_writeback_dry_run_requires_source_ref() {
        let mut request = valid_request();
        request.contract_source_ref = ContractRepairApprovalContractSourceRef::default();
        let preview = contract_repair_approval_record_preview(&request).unwrap();
        let snapshot = contract_repair_approval_record_snapshot_preview(
            &preview,
            "approve",
            "reviewer-a",
            "preview looks safe",
        );
        let transition_dry_run = ContractRepairApprovalReviewTransitionDryRun {
            status: "transition_dry_run_ready_blocked".to_string(),
            approval_id: preview.approval_id.clone(),
            from_review_state: RuntimeApprovalReviewState::Pending,
            target_review_state: RuntimeApprovalReviewState::Approved,
            reviewer_id: "reviewer-a".to_string(),
            reason_code: "review_transition_preview_only".to_string(),
            sequence_no_preview: 1,
            transition_ready: true,
            would_transition: false,
            blocked_by: vec!["contract_mutation_api_enabled".to_string()],
        };
        let lifecycle_entry_append_dry_run = ContractRepairApprovalLifecycleEntryAppendDryRun {
            status: "lifecycle_entry_append_ready_blocked".to_string(),
            event_id: format!("contract-repair-review-approve:{}", preview.approval_id),
            review_state: RuntimeApprovalReviewState::Approved,
            sequence_no: 1,
            entry_ready: true,
            emission_ready: true,
            append_ready: true,
            would_append: false,
            blocked_by: vec!["lifecycle_event_emission_enabled".to_string()],
        };
        let patch_plan = contract_repair_approval_contract_patch_plan_preview(&snapshot);
        let patch_apply_dry_run = blocked_patch_apply_dry_run();
        let source_write_dry_run = blocked_source_write_dry_run();
        let dry_run = contract_repair_approval_contract_writeback_dry_run(
            &snapshot,
            "approve",
            &transition_dry_run,
            &lifecycle_entry_append_dry_run,
            &blocked_source_resolution(),
            patch_plan,
            &patch_apply_dry_run,
            &source_write_dry_run,
        );

        assert_eq!(dry_run.status, "contract_writeback_dry_run_blocked");
        assert!(dry_run.patch_ready);
        assert!(dry_run.patch_payload_ready);
        assert!(!dry_run.contract_source_ready);
        assert_eq!(
            dry_run.missing_contract_source_fields,
            vec![
                "source_kind".to_string(),
                "source_id".to_string(),
                "version".to_string()
            ]
        );
        assert!(!dry_run.writeback_ready);
        assert!(dry_run
            .blocked_by
            .contains(&"contract_source_ref_ready".to_string()));
        assert!(dry_run
            .blocked_by
            .contains(&"contract_patch_apply_ready".to_string()));
        assert!(dry_run
            .blocked_by
            .contains(&"contract_source_write_ready".to_string()));
    }

    #[test]
    fn contract_repair_approval_patch_plan_blocks_evidence_sample_as_contract_writeback() {
        let mut request = valid_request();
        request.target_path = "event_catalog/observation.bar_closed/fast_ma/bar-002".to_string();
        request.target_kind = "event_instance_payload".to_string();
        request.changed_fields = vec!["payload_value".to_string()];
        request.patch_payload = BTreeMap::from([(
            "payload_value".to_string(),
            serde_json::Value::String("42".to_string()),
        )]);
        let preview = contract_repair_approval_record_preview(&request).unwrap();
        let snapshot = contract_repair_approval_record_snapshot_preview(
            &preview,
            "approve",
            "reviewer-a",
            "preview looks safe",
        );
        let transition_dry_run = ContractRepairApprovalReviewTransitionDryRun {
            status: "transition_dry_run_ready_blocked".to_string(),
            approval_id: preview.approval_id.clone(),
            from_review_state: RuntimeApprovalReviewState::Pending,
            target_review_state: RuntimeApprovalReviewState::Approved,
            reviewer_id: "reviewer-a".to_string(),
            reason_code: "review_transition_preview_only".to_string(),
            sequence_no_preview: 1,
            transition_ready: true,
            would_transition: false,
            blocked_by: vec!["contract_mutation_api_enabled".to_string()],
        };
        let lifecycle_entry_append_dry_run = ContractRepairApprovalLifecycleEntryAppendDryRun {
            status: "lifecycle_entry_append_ready_blocked".to_string(),
            event_id: format!("contract-repair-review-approve:{}", preview.approval_id),
            review_state: RuntimeApprovalReviewState::Approved,
            sequence_no: 1,
            entry_ready: true,
            emission_ready: true,
            append_ready: true,
            would_append: false,
            blocked_by: vec!["lifecycle_event_emission_enabled".to_string()],
        };
        let patch_plan = contract_repair_approval_contract_patch_plan_preview(&snapshot);
        let patch_apply_dry_run = blocked_patch_apply_dry_run();
        let source_write_dry_run = blocked_source_write_dry_run();
        let dry_run = contract_repair_approval_contract_writeback_dry_run(
            &snapshot,
            "approve",
            &transition_dry_run,
            &lifecycle_entry_append_dry_run,
            &ready_source_resolution(),
            patch_plan,
            &patch_apply_dry_run,
            &source_write_dry_run,
        );

        assert_eq!(dry_run.status, "contract_writeback_dry_run_blocked");
        assert_eq!(dry_run.patch_kind, "event_payload_instance_patch");
        assert!(dry_run.patch_ready);
        assert!(dry_run.patch_payload_ready);
        assert!(dry_run.contract_source_ready);
        assert!(!dry_run.patch_plan.contract_patch_ready);
        assert!(dry_run.patch_plan.evidence_sample_patch);
        assert_eq!(dry_run.patch_plan.operations.len(), 1);
        assert_eq!(
            dry_run.patch_plan.operations[0].domain,
            "event_payload_sample"
        );
        assert!(!dry_run.writeback_ready);
        assert!(dry_run
            .blocked_by
            .contains(&"contract_patch_plan_ready".to_string()));
        assert!(dry_run
            .patch_plan
            .blocked_by
            .contains(&"contract_patch_target_kind_supported".to_string()));
    }
}
