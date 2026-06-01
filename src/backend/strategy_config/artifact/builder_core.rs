use axum::http::StatusCode;
use serde::Serialize;
use serde_json::{json, Value};

use super::{
    build_config_domains, EvidenceAnchor, EvidenceAnchorInput, ProposalBinding,
    ProposalBindingInput, RuntimeBoundarySummary, StrategyConfigArtifact,
    StrategyConfigArtifactRequest, StrategyConfigCapabilitySummary, StrategyConfigSourceSummary,
};
use crate::{
    build_capability_response, canonical_json_sha256_digest, json_bad_request, GraphVersionEntry,
};

pub(crate) const STRATEGY_CONFIG_ARTIFACT_SCHEMA: &str =
    "quantpilot/v4-strategy-config-artifact/v1";

pub(crate) fn build_strategy_config_artifact(
    request: StrategyConfigArtifactRequest,
    now_ms: u64,
) -> Result<StrategyConfigArtifact, (StatusCode, String)> {
    let strategy_id =
        non_empty(request.strategy_id.clone()).unwrap_or_else(|| "local-strategy".to_string());
    let strategy_version =
        non_empty(request.strategy_version.clone()).unwrap_or_else(|| "local-draft".to_string());
    let source = build_source_summary(&request);
    let capability = build_capability_summary(&request);
    let runtime_boundary = build_runtime_boundary(&request);
    let evidence_anchors = normalize_evidence_anchors(request.evidence_anchors.clone());
    let proposal_bindings = normalize_proposal_bindings(request.proposal_bindings.clone());
    let config_domains = build_config_domains(
        &request,
        &source,
        &capability,
        &runtime_boundary,
        &evidence_anchors,
        &proposal_bindings,
    );
    let artifact_id = format!(
        "strategy_config_{}",
        digest_for_value(&json!({
            "strategy_id": strategy_id,
            "strategy_version": strategy_version,
            "source": source,
            "capability_hash": capability.capability_snapshot_hash,
            "created_at_ms": now_ms,
        }))?
        .trim_start_matches("sha256:")
        .chars()
        .take(16)
        .collect::<String>()
    );

    let mut artifact = StrategyConfigArtifact {
        schema_version: STRATEGY_CONFIG_ARTIFACT_SCHEMA.to_string(),
        artifact_id,
        strategy_id,
        strategy_version,
        created_at_ms: now_ms,
        source,
        capability,
        config_domains,
        runtime_boundary,
        evidence_anchors,
        proposal_bindings,
        artifact_digest: String::new(),
    };
    artifact.artifact_digest = digest_for_value(&artifact_digest_input(&artifact))?;
    Ok(artifact)
}

pub(crate) fn version_artifact_request(
    graph_id: &str,
    version: &GraphVersionEntry,
    graph: &Value,
    qs_source: Option<String>,
) -> StrategyConfigArtifactRequest {
    StrategyConfigArtifactRequest {
        strategy_id: Some(graph_id.to_string()),
        strategy_version: Some(
            version
                .version_label
                .clone()
                .unwrap_or_else(|| version.version_id.clone()),
        ),
        source_mode: Some("graph_version".to_string()),
        graph_json: Some(graph.clone()),
        runtime_config: graph.get("runtime_config").cloned(),
        qs_source: qs_source.and_then(|source| non_empty(Some(source))),
        core_ir: graph
            .get("metadata")
            .and_then(|metadata| metadata.get("artifacts"))
            .and_then(|artifacts| artifacts.get("core_ir"))
            .cloned(),
        v4_graph: graph
            .get("metadata")
            .and_then(|metadata| metadata.get("artifacts"))
            .and_then(|artifacts| artifacts.get("v4_graph"))
            .cloned(),
        capability_snapshot_hash: Some(build_capability_response().schema_hash),
        capability_source: Some("backend_current".to_string()),
        runtime_mode: Some("PaperSimulated".to_string()),
        evidence_anchors: vec![EvidenceAnchorInput {
            anchor_type: "snapshot".to_string(),
            anchor_id: Some(version.version_id.clone()),
            digest: digest_for_value(graph).ok(),
            summary: version
                .version_label
                .clone()
                .or_else(|| version.save_note.clone()),
        }],
        proposal_bindings: Vec::new(),
        required_execution_capability_sources: vec!["runtime_simulated".to_string()],
    }
}

fn build_source_summary(request: &StrategyConfigArtifactRequest) -> StrategyConfigSourceSummary {
    StrategyConfigSourceSummary {
        source_mode: non_empty(request.source_mode.clone())
            .unwrap_or_else(|| infer_source_mode(request)),
        graph_digest: digest_option_value(&request.graph_json),
        runtime_config_digest: digest_option_value(&request.runtime_config),
        qs_digest: request
            .qs_source
            .as_ref()
            .and_then(|source| digest_for_value(&json!({ "qs_source": source })).ok()),
        core_ir_digest: digest_option_value(&request.core_ir),
        v4_graph_digest: digest_option_value(&request.v4_graph),
    }
}

fn build_capability_summary(
    request: &StrategyConfigArtifactRequest,
) -> StrategyConfigCapabilitySummary {
    let response = build_capability_response();
    let capability_snapshot_hash = non_empty(request.capability_snapshot_hash.clone())
        .unwrap_or_else(|| response.schema_hash.clone());
    let capability_snapshot_status = if capability_snapshot_hash == response.schema_hash {
        "current"
    } else if capability_snapshot_hash == "safe-fallback" {
        "safe_fallback"
    } else {
        "stale"
    }
    .to_string();
    StrategyConfigCapabilitySummary {
        capability_snapshot_hash,
        capability_expected_hash: response.schema_hash,
        capability_snapshot_status,
        capability_source: non_empty(request.capability_source.clone())
            .unwrap_or_else(|| "backend_current".to_string()),
        frontend_module_keys: response
            .frontend
            .supported_module_keys
            .into_iter()
            .map(str::to_string)
            .collect(),
        ui_actions: response
            .ui_actions
            .actions
            .into_iter()
            .map(|entry| entry.key.to_string())
            .collect(),
        workspace_surfaces: response
            .workspace
            .surfaces
            .into_iter()
            .map(|entry| entry.key.to_string())
            .collect(),
    }
}

fn build_runtime_boundary(request: &StrategyConfigArtifactRequest) -> RuntimeBoundarySummary {
    let raw_mode = request
        .runtime_mode
        .as_deref()
        .unwrap_or("PaperSimulated")
        .trim()
        .to_ascii_lowercase();
    let legacy_live_label = raw_mode == "live" || raw_mode == "live.okx";
    let mode_label = if matches!(
        raw_mode.as_str(),
        "paperactual" | "paper_actual" | "okx_demo" | "demo" | "live" | "live.okx"
    ) {
        "PaperActual"
    } else {
        "PaperSimulated"
    }
    .to_string();
    let mut execution_capability_sources =
        if request.required_execution_capability_sources.is_empty() {
            vec![if mode_label == "PaperActual" {
                "provider_native".to_string()
            } else {
                "runtime_simulated".to_string()
            }]
        } else {
            request
                .required_execution_capability_sources
                .iter()
                .filter_map(|source| non_empty(Some(source.clone())))
                .collect()
        };
    execution_capability_sources.sort();
    execution_capability_sources.dedup();

    let mut rejection_reasons = Vec::new();
    if legacy_live_label {
        rejection_reasons.push(
            "legacy_live_label_normalized_to_paper_actual_demo; 用户侧不得用 live 描述 OKX demo"
                .to_string(),
        );
    }
    if execution_capability_sources
        .iter()
        .any(|source| source == "unsupported")
    {
        rejection_reasons.push("required execution capability contains unsupported".to_string());
    }

    RuntimeBoundarySummary {
        mode_label: mode_label.clone(),
        provider_order_submission_attached: mode_label == "PaperActual",
        provider_order_submission_allowed: mode_label == "PaperActual"
            && !execution_capability_sources
                .iter()
                .any(|source| source == "unsupported"),
        live_execution_allowed: false,
        execution_capability_sources,
        rejection_reasons,
    }
}

fn normalize_evidence_anchors(inputs: Vec<EvidenceAnchorInput>) -> Vec<EvidenceAnchor> {
    inputs
        .into_iter()
        .enumerate()
        .filter_map(|(index, input)| {
            let anchor_type = input.anchor_type.trim().to_string();
            if anchor_type.is_empty() {
                return None;
            }
            Some(EvidenceAnchor {
                anchor_id: input
                    .anchor_id
                    .and_then(|value| non_empty(Some(value)))
                    .unwrap_or_else(|| format!("{}_{}", anchor_type, index + 1)),
                anchor_type,
                digest: input.digest.and_then(|value| non_empty(Some(value))),
                summary: input.summary.and_then(|value| non_empty(Some(value))),
            })
        })
        .collect()
}

fn normalize_proposal_bindings(inputs: Vec<ProposalBindingInput>) -> Vec<ProposalBinding> {
    inputs
        .into_iter()
        .filter_map(|input| {
            let proposal_id = input.proposal_id.trim().to_string();
            if proposal_id.is_empty() {
                return None;
            }
            Some(ProposalBinding {
                proposal_id,
                target_domain: input.target_domain,
                before_digest: input.before_digest.and_then(|value| non_empty(Some(value))),
                after_digest: input.after_digest.and_then(|value| non_empty(Some(value))),
                evidence_anchor_ids: input.evidence_anchor_ids,
                sandbox_status: input
                    .sandbox_status
                    .and_then(|value| non_empty(Some(value)))
                    .unwrap_or_else(|| "pending".to_string()),
                approval_status: input
                    .approval_status
                    .and_then(|value| non_empty(Some(value)))
                    .unwrap_or_else(|| "pending".to_string()),
            })
        })
        .collect()
}

fn artifact_digest_input(artifact: &StrategyConfigArtifact) -> Value {
    json!({
        "schema_version": artifact.schema_version,
        "artifact_id": artifact.artifact_id,
        "strategy_id": artifact.strategy_id,
        "strategy_version": artifact.strategy_version,
        "created_at_ms": artifact.created_at_ms,
        "source": artifact.source,
        "capability": artifact.capability,
        "config_domains": artifact.config_domains,
        "runtime_boundary": artifact.runtime_boundary,
        "evidence_anchors": artifact.evidence_anchors,
        "proposal_bindings": artifact.proposal_bindings,
    })
}

fn digest_option_value(value: &Option<Value>) -> Option<String> {
    value
        .as_ref()
        .and_then(|value| digest_for_value(value).ok())
}

fn digest_for_value(value: &impl Serialize) -> Result<String, (StatusCode, String)> {
    canonical_json_sha256_digest(value)
        .map(|digest| format!("sha256:{}", digest.value))
        .map_err(|error| {
            json_bad_request(
                "strategy_config_digest_failed",
                format!("策略配置摘要计算失败: {error}"),
            )
        })
}

fn infer_source_mode(request: &StrategyConfigArtifactRequest) -> String {
    if request.v4_graph.is_some() {
        "v4_qs_handoff".to_string()
    } else if request.qs_source.is_some() {
        "formal_qs".to_string()
    } else {
        "strategy_graph".to_string()
    }
}

pub(crate) fn non_empty(value: Option<String>) -> Option<String> {
    value
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}
