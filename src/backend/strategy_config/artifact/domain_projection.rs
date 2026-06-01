use super::{
    ConfigDomainId, ConfigDomainLifecycle, ConfigDomainReadiness, ConfigDomainStatus,
    ConfigSourceRef, EvidenceAnchor, ProposalBinding, RuntimeBoundarySummary,
    StrategyConfigArtifactRequest, StrategyConfigCapabilitySummary, StrategyConfigFinding,
    StrategyConfigSourceSummary,
};

pub(crate) fn build_config_domains(
    request: &StrategyConfigArtifactRequest,
    source: &StrategyConfigSourceSummary,
    capability: &StrategyConfigCapabilitySummary,
    runtime_boundary: &RuntimeBoundarySummary,
    evidence_anchors: &[EvidenceAnchor],
    proposal_bindings: &[ProposalBinding],
) -> Vec<ConfigDomainStatus> {
    vec![
        market_domain(request, source),
        observation_domain(request, source),
        state_machine_domain(request, source),
        risk_domain(request, source),
        execution_domain(runtime_boundary, capability),
        evidence_domain(evidence_anchors),
        ai_governance_domain(proposal_bindings),
        snapshot_domain(evidence_anchors),
    ]
}

fn market_domain(
    request: &StrategyConfigArtifactRequest,
    source: &StrategyConfigSourceSummary,
) -> ConfigDomainStatus {
    let ready = request.graph_json.is_some() || request.runtime_config.is_some();
    ConfigDomainStatus {
        domain_id: ConfigDomainId::Market,
        lifecycle: ConfigDomainLifecycle::Implemented,
        readiness: if ready {
            ConfigDomainReadiness::Ready
        } else {
            ConfigDomainReadiness::Incomplete
        },
        source_refs: refs_from_pairs([
            ("graph_json", source.graph_digest.clone()),
            ("runtime_config", source.runtime_config_digest.clone()),
        ]),
        findings: if ready {
            Vec::new()
        } else {
            vec![finding(
                "warning",
                "strategy_config_market_incomplete",
                "市场与数据配置缺少 graph_json 或 runtime_config 证据。",
            )]
        },
        primary_action: Some("compile".to_string()),
    }
}

fn observation_domain(
    request: &StrategyConfigArtifactRequest,
    source: &StrategyConfigSourceSummary,
) -> ConfigDomainStatus {
    let ready = request.qs_source.is_some()
        || request.core_ir.is_some()
        || request.v4_graph.is_some()
        || request.graph_json.is_some();
    ConfigDomainStatus {
        domain_id: ConfigDomainId::Observation,
        lifecycle: ConfigDomainLifecycle::Implemented,
        readiness: if ready {
            ConfigDomainReadiness::Ready
        } else {
            ConfigDomainReadiness::Incomplete
        },
        source_refs: refs_from_pairs([
            ("qs_source", source.qs_digest.clone()),
            ("core_ir", source.core_ir_digest.clone()),
            ("v4_graph", source.v4_graph_digest.clone()),
            ("graph_json", source.graph_digest.clone()),
        ]),
        findings: if ready {
            Vec::new()
        } else {
            vec![finding(
                "warning",
                "strategy_config_observation_incomplete",
                "观察与信号配置缺少 QS、Core IR、v4 graph 或策略图证据。",
            )]
        },
        primary_action: Some("compile".to_string()),
    }
}

fn state_machine_domain(
    request: &StrategyConfigArtifactRequest,
    source: &StrategyConfigSourceSummary,
) -> ConfigDomainStatus {
    let source_has_machine = request
        .qs_source
        .as_deref()
        .map(|source| source.contains("machine"))
        .unwrap_or(false);
    let ready = request.v4_graph.is_some() || source_has_machine;
    let mut findings = Vec::new();
    if !ready {
        findings.push(finding(
            "info",
            "strategy_config_state_machine_documentable",
            "当前可说明 v4 状态机模型，但该 artifact 未携带 v4 graph 或 machine QS 证据。",
        ));
    }
    if let Some(v4_graph) = &request.v4_graph {
        if serde_json::from_value::<qrpc_core_ir::v4::V4MachineGraphContract>(v4_graph.clone())
            .is_err()
        {
            findings.push(finding(
                "warning",
                "strategy_config_v4_graph_shape_unverified",
                "v4_graph 未能解析为 V4MachineGraphContract，preflight 将保持受限。",
            ));
        }
    }
    ConfigDomainStatus {
        domain_id: ConfigDomainId::StateMachine,
        lifecycle: if ready {
            ConfigDomainLifecycle::Implemented
        } else {
            ConfigDomainLifecycle::Documentable
        },
        readiness: if ready {
            ConfigDomainReadiness::Ready
        } else {
            ConfigDomainReadiness::Incomplete
        },
        source_refs: refs_from_pairs([
            ("qs_source", source.qs_digest.clone()),
            ("v4_graph", source.v4_graph_digest.clone()),
        ]),
        findings,
        primary_action: Some("start_v4_simulation".to_string()),
    }
}

fn risk_domain(
    request: &StrategyConfigArtifactRequest,
    source: &StrategyConfigSourceSummary,
) -> ConfigDomainStatus {
    let mut findings = Vec::new();
    let mut risk_plane_ready = false;
    if let Some(v4_graph) = &request.v4_graph {
        match serde_json::from_value::<qrpc_core_ir::v4::V4MachineGraphContract>(v4_graph.clone()) {
            Ok(contract) => {
                risk_plane_ready = contract
                    .risk_plane
                    .as_ref()
                    .map(|risk_plane| risk_plane.required && !risk_plane.machine_ids.is_empty())
                    .unwrap_or(false);
                if let Err(errors) = contract.validate_static_contract() {
                    findings.extend(
                        errors
                            .into_iter()
                            .filter(|error| {
                                error.contains("risk_plane")
                                    || error.contains("execution machine")
                                    || error.contains("Risk Plane")
                            })
                            .take(5)
                            .map(|error| {
                                finding(
                                    "warning",
                                    "strategy_config_risk_plane_contract_invalid",
                                    error,
                                )
                            }),
                    );
                    risk_plane_ready = false;
                }
            }
            Err(_) => findings.push(finding(
                "warning",
                "strategy_config_risk_plane_shape_unverified",
                "v4_graph 未能解析为 V4MachineGraphContract，Risk Plane 只能保持受限。",
            )),
        }
    }
    if !risk_plane_ready {
        findings.push(finding(
            "warning",
            "strategy_config_risk_plane_not_attached",
            "未在 artifact 中发现已通过静态契约校验的 risk_plane 证据；执行前必须由后端 preflight 继续核验。",
        ));
    }
    ConfigDomainStatus {
        domain_id: ConfigDomainId::Risk,
        lifecycle: ConfigDomainLifecycle::Implemented,
        readiness: if risk_plane_ready {
            ConfigDomainReadiness::Ready
        } else {
            ConfigDomainReadiness::Restricted
        },
        source_refs: refs_from_pairs([("v4_graph", source.v4_graph_digest.clone())]),
        findings,
        primary_action: Some("preflight".to_string()),
    }
}

fn execution_domain(
    runtime_boundary: &RuntimeBoundarySummary,
    capability: &StrategyConfigCapabilitySummary,
) -> ConfigDomainStatus {
    let blocked = runtime_boundary
        .execution_capability_sources
        .iter()
        .any(|source| source == "unsupported");
    let mut findings = Vec::new();
    if runtime_boundary.mode_label == "PaperActual" {
        findings.push(finding(
            "info",
            "strategy_config_paper_actual_demo_boundary",
            "PaperActual 仅代表 OKX demo / testnet 边界，不代表真实资金自动交易。",
        ));
    }
    findings.extend(runtime_boundary.rejection_reasons.iter().map(|reason| {
        finding(
            if blocked { "error" } else { "warning" },
            "strategy_config_execution_boundary",
            reason.clone(),
        )
    }));
    if capability.capability_snapshot_status != "current" {
        findings.push(finding(
            "warning",
            "strategy_config_stale_capability",
            format!(
                "当前配置使用的能力快照为 {}，后端当前能力快照为 {}，需要重新核验。",
                capability.capability_snapshot_hash, capability.capability_expected_hash
            ),
        ));
    }
    let mut source_refs: Vec<ConfigSourceRef> = runtime_boundary
        .execution_capability_sources
        .iter()
        .map(|source| ConfigSourceRef {
            source_kind: "execution_capability_source".to_string(),
            source_id: source.clone(),
            digest: None,
        })
        .collect();
    source_refs.push(ConfigSourceRef {
        source_kind: "capability_snapshot".to_string(),
        source_id: capability.capability_snapshot_status.clone(),
        digest: Some(capability.capability_snapshot_hash.clone()),
    });
    ConfigDomainStatus {
        domain_id: ConfigDomainId::Execution,
        lifecycle: ConfigDomainLifecycle::Implemented,
        readiness: if blocked {
            ConfigDomainReadiness::Blocked
        } else if capability.capability_snapshot_status != "current" {
            ConfigDomainReadiness::Stale
        } else {
            ConfigDomainReadiness::Restricted
        },
        source_refs,
        findings,
        primary_action: Some("preflight".to_string()),
    }
}

fn evidence_domain(evidence_anchors: &[EvidenceAnchor]) -> ConfigDomainStatus {
    ConfigDomainStatus {
        domain_id: ConfigDomainId::Evidence,
        lifecycle: ConfigDomainLifecycle::Implemented,
        readiness: if evidence_anchors.is_empty() {
            ConfigDomainReadiness::Incomplete
        } else {
            ConfigDomainReadiness::Ready
        },
        source_refs: evidence_anchors
            .iter()
            .map(|anchor| ConfigSourceRef {
                source_kind: "evidence_anchor".to_string(),
                source_id: anchor.anchor_id.clone(),
                digest: anchor.digest.clone(),
            })
            .collect(),
        findings: if evidence_anchors.is_empty() {
            vec![finding(
                "info",
                "strategy_config_evidence_empty",
                "当前 artifact 尚未绑定运行、回测、提案或快照证据。",
            )]
        } else {
            Vec::new()
        },
        primary_action: Some("run_backtest".to_string()),
    }
}

fn ai_governance_domain(proposal_bindings: &[ProposalBinding]) -> ConfigDomainStatus {
    let unbound = proposal_bindings
        .iter()
        .any(|binding| binding.before_digest.is_none() || binding.after_digest.is_none());
    ConfigDomainStatus {
        domain_id: ConfigDomainId::AiGovernance,
        lifecycle: ConfigDomainLifecycle::Implemented,
        readiness: if unbound {
            ConfigDomainReadiness::Restricted
        } else {
            ConfigDomainReadiness::Ready
        },
        source_refs: proposal_bindings
            .iter()
            .map(|binding| ConfigSourceRef {
                source_kind: "proposal_binding".to_string(),
                source_id: binding.proposal_id.clone(),
                digest: binding.after_digest.clone(),
            })
            .collect(),
        findings: if unbound {
            vec![finding(
                "warning",
                "strategy_config_ai_binding_digest_missing",
                "AI 提案必须绑定修改前后 digest，缺失时不能激活。",
            )]
        } else if proposal_bindings.is_empty() {
            vec![finding(
                "info",
                "strategy_config_ai_proposal_only",
                "当前无 AI 提案；治理边界保持 proposal_only。",
            )]
        } else {
            Vec::new()
        },
        primary_action: Some("review_proposals".to_string()),
    }
}

fn snapshot_domain(evidence_anchors: &[EvidenceAnchor]) -> ConfigDomainStatus {
    let snapshots: Vec<_> = evidence_anchors
        .iter()
        .filter(|anchor| anchor.anchor_type == "snapshot")
        .collect();
    ConfigDomainStatus {
        domain_id: ConfigDomainId::Snapshot,
        lifecycle: ConfigDomainLifecycle::Documentable,
        readiness: if snapshots.is_empty() {
            ConfigDomainReadiness::Incomplete
        } else {
            ConfigDomainReadiness::Ready
        },
        source_refs: snapshots
            .into_iter()
            .map(|anchor| ConfigSourceRef {
                source_kind: "snapshot".to_string(),
                source_id: anchor.anchor_id.clone(),
                digest: anchor.digest.clone(),
            })
            .collect(),
        findings: if evidence_anchors
            .iter()
            .any(|anchor| anchor.anchor_type == "snapshot")
        {
            Vec::new()
        } else {
            vec![finding(
                "info",
                "strategy_config_snapshot_not_attached",
                "当前 artifact 尚未绑定快照；现有快照完整性仍使用 canonical JSON SHA-256 摘要校验。",
            )]
        },
        primary_action: Some("create_snapshot".to_string()),
    }
}

fn refs_from_pairs<const N: usize>(pairs: [(&str, Option<String>); N]) -> Vec<ConfigSourceRef> {
    pairs
        .into_iter()
        .filter_map(|(source_kind, digest)| {
            digest.map(|value| ConfigSourceRef {
                source_kind: source_kind.to_string(),
                source_id: source_kind.to_string(),
                digest: Some(value),
            })
        })
        .collect()
}

pub(crate) fn finding(
    severity: impl Into<String>,
    code: impl Into<String>,
    message: impl Into<String>,
) -> StrategyConfigFinding {
    StrategyConfigFinding {
        severity: severity.into(),
        code: code.into(),
        message: message.into(),
    }
}
