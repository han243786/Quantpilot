use crate::{
    json_bad_request, CreateRuntimeAiProposalRequest, RuntimeAiModelIdentity,
    RuntimeAiProposalStaticCheckDetail, RuntimeAiProposalStaticCheckResult,
    RuntimeAiProposalStatus, RuntimeEvidenceSourceKind, RuntimeParameterMutationTarget,
    StrategyConfigProposalDomain,
};
use axum::http::StatusCode;
use serde_json::{json, Value};

#[cfg(test)]
use crate::RuntimeAiProposalConfigDomainBinding;

pub(super) fn validate_hash_identity(
    value: &str,
    target: &'static str,
    label: &'static str,
) -> Result<(), (StatusCode, String)> {
    let valid = is_valid_hash_identity(value);
    if valid {
        Ok(())
    } else {
        Err(json_bad_request(
            "bad_request",
            format!("{target} 必须为 sha256:<64位小写十六进制> 格式 ({label})"),
        ))
    }
}

fn is_valid_hash_identity(value: &str) -> bool {
    value.trim().strip_prefix("sha256:").is_some_and(|digest| {
        digest.len() == 64
            && digest
                .chars()
                .all(|ch| ch.is_ascii_digit() || matches!(ch, 'a'..='f'))
    })
}

pub(super) fn validate_ai_model_identity(
    model: &RuntimeAiModelIdentity,
) -> Result<(), (StatusCode, String)> {
    if model.provider.trim().is_empty() {
        return Err(json_bad_request(
            "bad_request",
            "AI 提案候选必须指定 model.provider",
        ));
    }
    if model.model.trim().is_empty() {
        return Err(json_bad_request(
            "bad_request",
            "AI 提案候选必须指定 model.model",
        ));
    }
    if model.model_version.trim().is_empty() {
        return Err(json_bad_request(
            "bad_request",
            "AI 提案候选必须指定 model.model_version",
        ));
    }
    Ok(())
}

pub(super) fn ai_proposal_static_check_result(
    request: &CreateRuntimeAiProposalRequest,
    old_parameter_version: &str,
    proposed_parameter_version: &str,
    source_event_count: usize,
    checked_at_ms: u64,
) -> RuntimeAiProposalStaticCheckResult {
    let mut details = Vec::new();
    if source_event_count == 0 {
        details.push(RuntimeAiProposalStaticCheckDetail {
            code: "missing_source_evidence".to_string(),
            target: "source_id".to_string(),
            message: "AI proposals require at least one source evidence event".to_string(),
        });
    }
    if old_parameter_version == proposed_parameter_version {
        details.push(RuntimeAiProposalStaticCheckDetail {
            code: "noop_parameter_version".to_string(),
            target: "new_value".to_string(),
            message: "旧值和新值解析为相同的规范参数版本".to_string(),
        });
    }
    if request.reason.trim().is_empty() {
        details.push(RuntimeAiProposalStaticCheckDetail {
            code: "missing_reason".to_string(),
            target: "reason".to_string(),
            message: "AI 提案候选需要说明原因".to_string(),
        });
    }
    details.extend(validate_ai_proposal_config_domain_binding(
        request,
        old_parameter_version,
        proposed_parameter_version,
    ));

    if is_v4_ai_proposal_target(&request.target)
        && request.source_kind != RuntimeEvidenceSourceKind::Backtest
    {
        details.push(RuntimeAiProposalStaticCheckDetail {
            code: "v4_proposal_requires_backtest_artifact".to_string(),
            target: "source_kind".to_string(),
            message:
                "v4 AI proposals must be anchored to a v4 backtest artifact and machine trajectory."
                    .to_string(),
        });
    }
    if !is_v4_ai_proposal_target(&request.target)
        && request.source_kind != RuntimeEvidenceSourceKind::Run
    {
        details.push(RuntimeAiProposalStaticCheckDetail {
            code: "non_v4_proposal_requires_run_source".to_string(),
            target: "source_kind".to_string(),
            message: "非 v4 AI proposal 必须绑定 runtime run 证据。".to_string(),
        });
    }

    if details.is_empty() {
        RuntimeAiProposalStaticCheckResult {
            status: RuntimeAiProposalStatus::StaticCheckPassed,
            reason_code: "AI_PROPOSAL_STATIC_CHECK_PASSED".to_string(),
            message: "AI 提案候选通过静态校验".to_string(),
            checked_at_ms,
            details,
        }
    } else {
        RuntimeAiProposalStaticCheckResult {
            status: RuntimeAiProposalStatus::StaticCheckFailed,
            reason_code: "AI_PROPOSAL_STATIC_CHECK_FAILED".to_string(),
            message: "AI proposal candidate failed static validation".to_string(),
            checked_at_ms,
            details,
        }
    }
}

fn is_v4_ai_proposal_target(target: &RuntimeParameterMutationTarget) -> bool {
    target.module_key.starts_with("v4.") || target.parameter_path.starts_with("v4.")
}

fn is_v4_guard_proposal_target(target: &RuntimeParameterMutationTarget) -> bool {
    target.module_key == "v4.transition.guard"
}

fn expected_config_domain_for_target(
    target: &RuntimeParameterMutationTarget,
) -> StrategyConfigProposalDomain {
    match target.module_key.as_str() {
        "builtin.data.kline" | "builtin.data.quote" => StrategyConfigProposalDomain::Market,
        "builtin.risk.global" => StrategyConfigProposalDomain::Risk,
        "builtin.execution.paper" | "builtin.runtime.control" => {
            StrategyConfigProposalDomain::Execution
        }
        "v4.machine.param" | "v4.transition.guard" => StrategyConfigProposalDomain::StateMachine,
        module_key if module_key.starts_with("builtin.intent.") => {
            StrategyConfigProposalDomain::Observation
        }
        module_key if module_key.starts_with("builtin.agent.") => {
            StrategyConfigProposalDomain::Observation
        }
        _ => StrategyConfigProposalDomain::AiGovernance,
    }
}

fn validate_ai_proposal_config_domain_binding(
    request: &CreateRuntimeAiProposalRequest,
    old_parameter_version: &str,
    proposed_parameter_version: &str,
) -> Vec<RuntimeAiProposalStaticCheckDetail> {
    let Some(binding) = request.config_domain_binding.as_ref() else {
        return vec![RuntimeAiProposalStaticCheckDetail {
            code: "strategy_config_ai_binding_required".to_string(),
            target: "config_domain_binding".to_string(),
            message: "AI proposal 必须绑定目标策略配置域、修改前后 digest 和证据锚点。".to_string(),
        }];
    };

    let mut details = Vec::new();
    let expected_domain = expected_config_domain_for_target(&request.target);
    if binding.target_domain != expected_domain {
        details.push(RuntimeAiProposalStaticCheckDetail {
            code: "strategy_config_ai_binding_domain_mismatch".to_string(),
            target: "config_domain_binding.target_domain".to_string(),
            message: format!(
                "AI proposal 目标域与模块不一致: expected={:?}, actual={:?}",
                expected_domain, binding.target_domain
            ),
        });
    }
    if !is_valid_hash_identity(&binding.before_digest) {
        details.push(RuntimeAiProposalStaticCheckDetail {
            code: "strategy_config_ai_binding_before_digest_invalid".to_string(),
            target: "config_domain_binding.before_digest".to_string(),
            message: "AI proposal 修改前 digest 必须为 sha256:<64位小写十六进制>。".to_string(),
        });
    }
    if !is_valid_hash_identity(&binding.after_digest) {
        details.push(RuntimeAiProposalStaticCheckDetail {
            code: "strategy_config_ai_binding_after_digest_invalid".to_string(),
            target: "config_domain_binding.after_digest".to_string(),
            message: "AI proposal 修改后 digest 必须为 sha256:<64位小写十六进制>。".to_string(),
        });
    }
    if binding.before_digest != old_parameter_version {
        details.push(RuntimeAiProposalStaticCheckDetail {
            code: "strategy_config_ai_binding_before_digest_mismatch".to_string(),
            target: "config_domain_binding.before_digest".to_string(),
            message: "AI proposal 修改前 digest 与当前策略参数版本不一致。".to_string(),
        });
    }
    if binding.after_digest != proposed_parameter_version {
        details.push(RuntimeAiProposalStaticCheckDetail {
            code: "strategy_config_ai_binding_after_digest_mismatch".to_string(),
            target: "config_domain_binding.after_digest".to_string(),
            message: "AI proposal 修改后 digest 与候选策略参数版本不一致。".to_string(),
        });
    }
    if binding
        .evidence_anchor_ids
        .iter()
        .all(|anchor| anchor.trim().is_empty())
    {
        details.push(RuntimeAiProposalStaticCheckDetail {
            code: "strategy_config_ai_binding_evidence_required".to_string(),
            target: "config_domain_binding.evidence_anchor_ids".to_string(),
            message: "AI proposal 必须声明至少一个配置证据锚点。".to_string(),
        });
    }
    if is_v4_guard_proposal_target(&request.target)
        && !qrpc_core_ir::v4::machine_guard_parameter_path_allowed(&request.target.parameter_path)
    {
        details.push(RuntimeAiProposalStaticCheckDetail {
            code: "v4_guard_parameter_path_not_proposal_only".to_string(),
            target: "target.parameter_path".to_string(),
            message:
                "V4 transition guard proposals may only target guard, cooldown, threshold, or guard risk-limit parameters."
                    .to_string(),
        });
    }
    if request.target.module_key.starts_with("v4.") {
        if !binding
            .evidence_anchor_ids
            .iter()
            .any(|anchor| is_productization_replay_anchor(anchor))
        {
            details.push(RuntimeAiProposalStaticCheckDetail {
                code: "strategy_config_ai_binding_productization_replay_diff_required"
                    .to_string(),
                target: "config_domain_binding.evidence_anchor_ids".to_string(),
                message:
                    "V4 AI proposal candidates require a productization replay diff evidence anchor."
                        .to_string(),
            });
        } else if request.source_kind == RuntimeEvidenceSourceKind::Backtest
            && !binding.evidence_anchor_ids.iter().any(|anchor| {
                is_productization_replay_anchor_for_source(anchor, &request.source_id)
            })
        {
            details.push(RuntimeAiProposalStaticCheckDetail {
                code: "strategy_config_ai_binding_productization_replay_diff_source_mismatch"
                    .to_string(),
                target: "config_domain_binding.evidence_anchor_ids".to_string(),
                message:
                    "V4 AI proposal productization replay diff anchor must name the source backtest."
                        .to_string(),
            });
        }
    }
    if request.target.module_key.starts_with("v4.") {
        if !binding
            .evidence_anchor_ids
            .iter()
            .any(|anchor| is_runtime_runner_verified_anchor(anchor))
        {
            details.push(RuntimeAiProposalStaticCheckDetail {
                code: "strategy_config_ai_binding_runtime_runner_verified_required".to_string(),
                target: "config_domain_binding.evidence_anchor_ids".to_string(),
                message:
                    "V4 AI proposal candidates require a verified runtime replay runner evidence anchor."
                        .to_string(),
            });
        } else if request.source_kind == RuntimeEvidenceSourceKind::Backtest
            && !binding.evidence_anchor_ids.iter().any(|anchor| {
                is_runtime_runner_verified_anchor_for_source(anchor, &request.source_id)
            })
        {
            details.push(RuntimeAiProposalStaticCheckDetail {
                code: "strategy_config_ai_binding_runtime_runner_verified_source_mismatch"
                    .to_string(),
                target: "config_domain_binding.evidence_anchor_ids".to_string(),
                message:
                    "V4 AI proposal runtime runner verified anchor must name the source backtest."
                        .to_string(),
            });
        }
    }
    details
}

fn is_productization_replay_anchor(anchor: &str) -> bool {
    let anchor = anchor.to_ascii_lowercase();
    anchor.contains("productization") || anchor.contains("replay_diff")
}

fn is_productization_replay_anchor_for_source(anchor: &str, source_id: &str) -> bool {
    let anchor = anchor.to_ascii_lowercase();
    is_productization_replay_anchor(&anchor) && anchor_has_source_segment(&anchor, source_id)
}

fn is_runtime_runner_verified_anchor(anchor: &str) -> bool {
    let anchor = anchor.to_ascii_lowercase();
    (anchor.contains("runtime_replay_runner") || anchor.contains("runtime_runner"))
        && anchor.contains("verified")
}

fn is_runtime_runner_verified_anchor_for_source(anchor: &str, source_id: &str) -> bool {
    let anchor = anchor.to_ascii_lowercase();
    is_runtime_runner_verified_anchor(&anchor) && anchor_has_source_segment(&anchor, source_id)
}

fn anchor_has_source_segment(anchor: &str, source_id: &str) -> bool {
    let source_id = source_id.to_ascii_lowercase();
    anchor
        .split(':')
        .any(|segment| segment.eq_ignore_ascii_case(&source_id))
}

#[allow(dead_code)]
fn analyze_v4_backtest_artifact_for_ai(artifact: &qrpc_core_ir::v4::V4BacktestArtifact) -> Value {
    let mut state_counts = std::collections::BTreeMap::<String, u64>::new();
    let mut machine_counts = std::collections::BTreeMap::<String, u64>::new();
    for point in &artifact.machine_trajectory {
        *state_counts
            .entry(format!("{}:{}", point.machine_id, point.state_id))
            .or_default() += 1;
        *machine_counts.entry(point.machine_id.clone()).or_default() += 1;
    }
    let risk_decision_count = artifact.risk_plane_decisions.len() as u64;
    let risk_rejected_count = artifact
        .risk_plane_decisions
        .iter()
        .filter(|decision| !decision.approved)
        .count() as u64;
    let risk_reject_ratio = if risk_decision_count == 0 {
        0.0
    } else {
        risk_rejected_count as f64 / risk_decision_count as f64
    };
    let fill_rate = artifact
        .microstructure_metrics
        .as_ref()
        .map(|metrics| metrics.fill_rate)
        .unwrap_or(0.0);

    json!({
        "analysis_version": "quantpilot/v4-ai-trajectory-analysis/v1",
        "graph_id": artifact.graph_id,
        "replay_mode": artifact.replay_mode,
        "machine_count": machine_counts.len(),
        "trajectory_point_count": artifact.machine_trajectory.len(),
        "state_visit_counts": state_counts,
        "machine_visit_counts": machine_counts,
        "risk_decision_count": risk_decision_count,
        "risk_rejected_count": risk_rejected_count,
        "risk_reject_ratio": risk_reject_ratio,
        "execution_fill_rate": fill_rate,
    })
}

#[cfg(test)]
mod v4_ai_proposal_static_check_tests {
    use super::*;

    fn hash(ch: char) -> String {
        format!("sha256:{}", ch.to_string().repeat(64))
    }

    fn v4_target() -> RuntimeParameterMutationTarget {
        RuntimeParameterMutationTarget {
            node_id: "compat.execution".to_string(),
            module_key: "v4.machine.param".to_string(),
            parameter_path: "v4.machine.timeout_ms".to_string(),
        }
    }

    fn v4_guard_target(parameter_path: &str) -> RuntimeParameterMutationTarget {
        RuntimeParameterMutationTarget {
            node_id: "risk.guard".to_string(),
            module_key: "v4.transition.guard".to_string(),
            parameter_path: parameter_path.to_string(),
        }
    }

    fn v4_binding(
        before_digest: String,
        after_digest: String,
    ) -> RuntimeAiProposalConfigDomainBinding {
        RuntimeAiProposalConfigDomainBinding {
            target_domain: StrategyConfigProposalDomain::StateMachine,
            before_digest,
            after_digest,
            evidence_anchor_ids: vec![
                "backtest:bt1".to_string(),
                "productization_replay_diff:bt1".to_string(),
                "runtime_replay_runner:verified:bt1".to_string(),
            ],
        }
    }

    #[test]
    fn v4_ai_proposal_static_check_requires_backtest_source() {
        let request = CreateRuntimeAiProposalRequest {
            source_kind: RuntimeEvidenceSourceKind::Run,
            source_id: "run-1".to_string(),
            target: v4_target(),
            old_value: json!(1),
            new_value: json!(2),
            model: RuntimeAiModelIdentity {
                provider: "test".to_string(),
                model: "local".to_string(),
                model_version: "v1".to_string(),
            },
            prompt_hash: "sha256:prompt".to_string(),
            evidence_hash: "sha256:evidence".to_string(),
            actor: None,
            reason: "Tune v4 machine timeout from trajectory evidence".to_string(),
            capability_context: None,
            config_domain_binding: Some(v4_binding("old".to_string(), "new".to_string())),
        };

        let result = ai_proposal_static_check_result(&request, "old", "new", 1, 1);

        assert_eq!(result.status, RuntimeAiProposalStatus::StaticCheckFailed);
        assert!(result
            .details
            .iter()
            .any(|detail| detail.code == "v4_proposal_requires_backtest_artifact"));
    }

    #[test]
    fn ai_proposal_static_check_requires_config_domain_binding() {
        let old = hash('b');
        let new = hash('c');
        let request = CreateRuntimeAiProposalRequest {
            source_kind: RuntimeEvidenceSourceKind::Backtest,
            source_id: "bt1".to_string(),
            target: v4_target(),
            old_value: json!(1),
            new_value: json!(2),
            model: RuntimeAiModelIdentity {
                provider: "test".to_string(),
                model: "local".to_string(),
                model_version: "v1".to_string(),
            },
            prompt_hash: hash('d'),
            evidence_hash: hash('a'),
            actor: None,
            reason: "Tune v4 machine timeout from trajectory evidence".to_string(),
            capability_context: None,
            config_domain_binding: None,
        };

        let result = ai_proposal_static_check_result(&request, &old, &new, 1, 1);

        assert_eq!(result.status, RuntimeAiProposalStatus::StaticCheckFailed);
        assert!(result
            .details
            .iter()
            .any(|detail| detail.code == "strategy_config_ai_binding_required"));
    }

    #[test]
    fn ai_proposal_static_check_accepts_matching_config_domain_binding() {
        let old = hash('b');
        let new = hash('c');
        let request = CreateRuntimeAiProposalRequest {
            source_kind: RuntimeEvidenceSourceKind::Backtest,
            source_id: "bt1".to_string(),
            target: v4_target(),
            old_value: json!(1),
            new_value: json!(2),
            model: RuntimeAiModelIdentity {
                provider: "test".to_string(),
                model: "local".to_string(),
                model_version: "v1".to_string(),
            },
            prompt_hash: hash('d'),
            evidence_hash: hash('a'),
            actor: None,
            reason: "Tune v4 machine timeout from trajectory evidence".to_string(),
            capability_context: None,
            config_domain_binding: Some(v4_binding(old.clone(), new.clone())),
        };

        let result = ai_proposal_static_check_result(&request, &old, &new, 1, 1);

        assert_eq!(result.status, RuntimeAiProposalStatus::StaticCheckPassed);
        assert!(result.details.is_empty());
    }

    #[test]
    fn v4_ai_proposal_static_check_accepts_guard_parameter_diff_path() {
        let old = hash('b');
        let new = hash('c');
        let request = CreateRuntimeAiProposalRequest {
            source_kind: RuntimeEvidenceSourceKind::Backtest,
            source_id: "bt1".to_string(),
            target: v4_guard_target("risk.max_notional"),
            old_value: json!(1),
            new_value: json!(2),
            model: RuntimeAiModelIdentity {
                provider: "test".to_string(),
                model: "local".to_string(),
                model_version: "v1".to_string(),
            },
            prompt_hash: hash('d'),
            evidence_hash: hash('a'),
            actor: None,
            reason: "Tune v4 guard risk limit from trajectory evidence".to_string(),
            capability_context: None,
            config_domain_binding: Some(v4_binding(old.clone(), new.clone())),
        };

        let result = ai_proposal_static_check_result(&request, &old, &new, 1, 1);

        assert_eq!(result.status, RuntimeAiProposalStatus::StaticCheckPassed);
        assert!(result.details.is_empty());
    }

    #[test]
    fn v4_ai_proposal_static_check_rejects_guard_topology_path() {
        let old = hash('b');
        let new = hash('c');
        let request = CreateRuntimeAiProposalRequest {
            source_kind: RuntimeEvidenceSourceKind::Backtest,
            source_id: "bt1".to_string(),
            target: v4_guard_target("graph.edges"),
            old_value: json!(1),
            new_value: json!(2),
            model: RuntimeAiModelIdentity {
                provider: "test".to_string(),
                model: "local".to_string(),
                model_version: "v1".to_string(),
            },
            prompt_hash: hash('d'),
            evidence_hash: hash('a'),
            actor: None,
            reason: "Attempt topology edit through guard proposal".to_string(),
            capability_context: None,
            config_domain_binding: Some(v4_binding(old.clone(), new.clone())),
        };

        let result = ai_proposal_static_check_result(&request, &old, &new, 1, 1);

        assert_eq!(result.status, RuntimeAiProposalStatus::StaticCheckFailed);
        assert!(result.details.iter().any(|detail| {
            detail.code == "v4_guard_parameter_path_not_proposal_only"
                && detail.target == "target.parameter_path"
        }));
    }

    #[test]
    fn v4_ai_proposal_static_check_rejects_guard_capability_source_path() {
        let old = hash('b');
        let new = hash('c');
        let request = CreateRuntimeAiProposalRequest {
            source_kind: RuntimeEvidenceSourceKind::Backtest,
            source_id: "bt1".to_string(),
            target: v4_guard_target("execution.capability_source"),
            old_value: json!(1),
            new_value: json!(2),
            model: RuntimeAiModelIdentity {
                provider: "test".to_string(),
                model: "local".to_string(),
                model_version: "v1".to_string(),
            },
            prompt_hash: hash('d'),
            evidence_hash: hash('a'),
            actor: None,
            reason: "Attempt capability source edit through guard proposal".to_string(),
            capability_context: None,
            config_domain_binding: Some(v4_binding(old.clone(), new.clone())),
        };

        let result = ai_proposal_static_check_result(&request, &old, &new, 1, 1);

        assert_eq!(result.status, RuntimeAiProposalStatus::StaticCheckFailed);
        assert!(result.details.iter().any(|detail| {
            detail.code == "v4_guard_parameter_path_not_proposal_only"
                && detail.target == "target.parameter_path"
        }));
    }

    #[test]
    fn v4_ai_proposal_static_check_requires_productization_replay_anchor() {
        let old = hash('b');
        let new = hash('c');
        let request = CreateRuntimeAiProposalRequest {
            source_kind: RuntimeEvidenceSourceKind::Backtest,
            source_id: "bt1".to_string(),
            target: v4_target(),
            old_value: json!(1),
            new_value: json!(2),
            model: RuntimeAiModelIdentity {
                provider: "test".to_string(),
                model: "local".to_string(),
                model_version: "v1".to_string(),
            },
            prompt_hash: hash('d'),
            evidence_hash: hash('a'),
            actor: None,
            reason: "Tune v4 machine timeout from trajectory evidence".to_string(),
            capability_context: None,
            config_domain_binding: Some(RuntimeAiProposalConfigDomainBinding {
                target_domain: StrategyConfigProposalDomain::StateMachine,
                before_digest: old.clone(),
                after_digest: new.clone(),
                evidence_anchor_ids: vec!["backtest:bt1".to_string()],
            }),
        };

        let result = ai_proposal_static_check_result(&request, &old, &new, 1, 1);

        assert_eq!(result.status, RuntimeAiProposalStatus::StaticCheckFailed);
        assert!(result.details.iter().any(|detail| {
            detail.code == "strategy_config_ai_binding_productization_replay_diff_required"
        }));
    }

    #[test]
    fn v4_ai_proposal_static_check_requires_runtime_runner_verified_anchor() {
        let old = hash('b');
        let new = hash('c');
        let request = CreateRuntimeAiProposalRequest {
            source_kind: RuntimeEvidenceSourceKind::Backtest,
            source_id: "bt1".to_string(),
            target: v4_target(),
            old_value: json!(1),
            new_value: json!(2),
            model: RuntimeAiModelIdentity {
                provider: "test".to_string(),
                model: "local".to_string(),
                model_version: "v1".to_string(),
            },
            prompt_hash: hash('d'),
            evidence_hash: hash('a'),
            actor: None,
            reason: "Tune v4 machine timeout from trajectory evidence".to_string(),
            capability_context: None,
            config_domain_binding: Some(RuntimeAiProposalConfigDomainBinding {
                target_domain: StrategyConfigProposalDomain::StateMachine,
                before_digest: old.clone(),
                after_digest: new.clone(),
                evidence_anchor_ids: vec![
                    "backtest:bt1".to_string(),
                    "productization_replay_diff:bt1".to_string(),
                ],
            }),
        };

        let result = ai_proposal_static_check_result(&request, &old, &new, 1, 1);

        assert_eq!(result.status, RuntimeAiProposalStatus::StaticCheckFailed);
        assert!(result.details.iter().any(|detail| {
            detail.code == "strategy_config_ai_binding_runtime_runner_verified_required"
        }));
    }

    #[test]
    fn v4_ai_proposal_static_check_requires_productization_anchor_for_source_backtest() {
        let old = hash('b');
        let new = hash('c');
        let request = CreateRuntimeAiProposalRequest {
            source_kind: RuntimeEvidenceSourceKind::Backtest,
            source_id: "bt1".to_string(),
            target: v4_target(),
            old_value: json!(1),
            new_value: json!(2),
            model: RuntimeAiModelIdentity {
                provider: "test".to_string(),
                model: "local".to_string(),
                model_version: "v1".to_string(),
            },
            prompt_hash: hash('d'),
            evidence_hash: hash('a'),
            actor: None,
            reason: "Tune v4 machine timeout from trajectory evidence".to_string(),
            capability_context: None,
            config_domain_binding: Some(RuntimeAiProposalConfigDomainBinding {
                target_domain: StrategyConfigProposalDomain::StateMachine,
                before_digest: old.clone(),
                after_digest: new.clone(),
                evidence_anchor_ids: vec![
                    "backtest:bt1".to_string(),
                    "productization_replay_diff:other-bt".to_string(),
                    "runtime_replay_runner:verified:bt1".to_string(),
                ],
            }),
        };

        let result = ai_proposal_static_check_result(&request, &old, &new, 1, 1);

        assert_eq!(result.status, RuntimeAiProposalStatus::StaticCheckFailed);
        assert!(result.details.iter().any(|detail| {
            detail.code == "strategy_config_ai_binding_productization_replay_diff_source_mismatch"
        }));
    }

    #[test]
    fn v4_ai_proposal_static_check_requires_runtime_runner_anchor_for_source_backtest() {
        let old = hash('b');
        let new = hash('c');
        let request = CreateRuntimeAiProposalRequest {
            source_kind: RuntimeEvidenceSourceKind::Backtest,
            source_id: "bt1".to_string(),
            target: v4_target(),
            old_value: json!(1),
            new_value: json!(2),
            model: RuntimeAiModelIdentity {
                provider: "test".to_string(),
                model: "local".to_string(),
                model_version: "v1".to_string(),
            },
            prompt_hash: hash('d'),
            evidence_hash: hash('a'),
            actor: None,
            reason: "Tune v4 machine timeout from trajectory evidence".to_string(),
            capability_context: None,
            config_domain_binding: Some(RuntimeAiProposalConfigDomainBinding {
                target_domain: StrategyConfigProposalDomain::StateMachine,
                before_digest: old.clone(),
                after_digest: new.clone(),
                evidence_anchor_ids: vec![
                    "backtest:bt1".to_string(),
                    "productization_replay_diff:bt1".to_string(),
                    "runtime_replay_runner:verified:other-bt".to_string(),
                ],
            }),
        };

        let result = ai_proposal_static_check_result(&request, &old, &new, 1, 1);

        assert_eq!(result.status, RuntimeAiProposalStatus::StaticCheckFailed);
        assert!(result.details.iter().any(|detail| {
            detail.code == "strategy_config_ai_binding_runtime_runner_verified_source_mismatch"
        }));
    }

    #[test]
    fn v4_ai_proposal_static_check_rejects_prefix_source_anchor_match() {
        let old = hash('b');
        let new = hash('c');
        let request = CreateRuntimeAiProposalRequest {
            source_kind: RuntimeEvidenceSourceKind::Backtest,
            source_id: "bt1".to_string(),
            target: v4_target(),
            old_value: json!(1),
            new_value: json!(2),
            model: RuntimeAiModelIdentity {
                provider: "test".to_string(),
                model: "local".to_string(),
                model_version: "v1".to_string(),
            },
            prompt_hash: hash('d'),
            evidence_hash: hash('a'),
            actor: None,
            reason: "Tune v4 machine timeout from trajectory evidence".to_string(),
            capability_context: None,
            config_domain_binding: Some(RuntimeAiProposalConfigDomainBinding {
                target_domain: StrategyConfigProposalDomain::StateMachine,
                before_digest: old.clone(),
                after_digest: new.clone(),
                evidence_anchor_ids: vec![
                    "backtest:bt1".to_string(),
                    "productization_replay_diff:bt10".to_string(),
                    "runtime_replay_runner:verified:bt10".to_string(),
                ],
            }),
        };

        let result = ai_proposal_static_check_result(&request, &old, &new, 1, 1);

        assert_eq!(result.status, RuntimeAiProposalStatus::StaticCheckFailed);
        assert!(result.details.iter().any(|detail| {
            detail.code == "strategy_config_ai_binding_productization_replay_diff_source_mismatch"
        }));
        assert!(result.details.iter().any(|detail| {
            detail.code == "strategy_config_ai_binding_runtime_runner_verified_source_mismatch"
        }));
    }

    #[test]
    fn v4_artifact_analysis_summarizes_trajectory_and_fill_rate() {
        let artifact = qrpc_core_ir::v4::V4BacktestArtifact {
            schema_version: qrpc_core_ir::v4::V4_BACKTEST_ARTIFACT_VERSION.to_string(),
            graph_id: "graph-v4".to_string(),
            started_at_ms: 1,
            ended_at_ms: 2,
            replay_mode: "tick_replay".to_string(),
            input_bar_count: 0,
            input_tick_count: Some(2),
            symbols: vec!["BTCUSDT".to_string()],
            machine_trajectory: vec![qrpc_core_ir::v4::V4BacktestMachineTrajectoryPoint {
                ts_ms: 1,
                event_sequence: 1,
                machine_id: "compat.execution".to_string(),
                template: qrpc_core_ir::v4::MachineTemplateKind::Execution,
                state_id: "ready".to_string(),
                status: "active".to_string(),
                symbol: Some("BTCUSDT".to_string()),
            }],
            risk_plane_decisions: Vec::new(),
            execution_capability_sources: Vec::new(),
            microstructure_metrics: Some(qrpc_core_ir::v4::V4BacktestMicrostructureMetrics {
                submitted_order_count: 1,
                filled_order_count: 1,
                fill_rate: 1.0,
                average_slippage_bps: 2.0,
                queue_position_estimate: 0.0,
                vwap_deviation_bps: 2.0,
            }),
            final_snapshot: None,
        };

        let analysis = analyze_v4_backtest_artifact_for_ai(&artifact);

        assert_eq!(
            analysis["analysis_version"],
            "quantpilot/v4-ai-trajectory-analysis/v1"
        );
        assert_eq!(analysis["machine_count"], 1);
        assert_eq!(analysis["execution_fill_rate"], 1.0);
    }
}
