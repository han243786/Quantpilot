use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct StrategyConfigArtifactRequest {
    #[serde(default)]
    pub(crate) strategy_id: Option<String>,
    #[serde(default)]
    pub(crate) strategy_version: Option<String>,
    #[serde(default)]
    pub(crate) source_mode: Option<String>,
    #[serde(default)]
    pub(crate) graph_json: Option<Value>,
    #[serde(default)]
    pub(crate) runtime_config: Option<Value>,
    #[serde(default)]
    pub(crate) qs_source: Option<String>,
    #[serde(default)]
    pub(crate) core_ir: Option<Value>,
    #[serde(default)]
    pub(crate) v4_graph: Option<Value>,
    #[serde(default)]
    pub(crate) capability_snapshot_hash: Option<String>,
    #[serde(default)]
    pub(crate) capability_source: Option<String>,
    #[serde(default)]
    pub(crate) runtime_mode: Option<String>,
    #[serde(default)]
    pub(crate) evidence_anchors: Vec<EvidenceAnchorInput>,
    #[serde(default)]
    pub(crate) proposal_bindings: Vec<ProposalBindingInput>,
    #[serde(default)]
    pub(crate) required_execution_capability_sources: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct StrategyConfigArtifact {
    pub(crate) schema_version: String,
    pub(crate) artifact_id: String,
    pub(crate) strategy_id: String,
    pub(crate) strategy_version: String,
    pub(crate) created_at_ms: u64,
    pub(crate) source: StrategyConfigSourceSummary,
    pub(crate) capability: StrategyConfigCapabilitySummary,
    pub(crate) config_domains: Vec<ConfigDomainStatus>,
    pub(crate) runtime_boundary: RuntimeBoundarySummary,
    pub(crate) evidence_anchors: Vec<EvidenceAnchor>,
    pub(crate) proposal_bindings: Vec<ProposalBinding>,
    pub(crate) artifact_digest: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct StrategyConfigSourceSummary {
    pub(crate) source_mode: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) graph_digest: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) runtime_config_digest: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) qs_digest: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) core_ir_digest: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) v4_graph_digest: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct StrategyConfigCapabilitySummary {
    pub(crate) capability_snapshot_hash: String,
    pub(crate) capability_expected_hash: String,
    pub(crate) capability_snapshot_status: String,
    pub(crate) capability_source: String,
    pub(crate) frontend_module_keys: Vec<String>,
    pub(crate) ui_actions: Vec<String>,
    pub(crate) workspace_surfaces: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct ConfigDomainStatus {
    pub(crate) domain_id: ConfigDomainId,
    pub(crate) lifecycle: ConfigDomainLifecycle,
    pub(crate) readiness: ConfigDomainReadiness,
    pub(crate) source_refs: Vec<ConfigSourceRef>,
    pub(crate) findings: Vec<StrategyConfigFinding>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) primary_action: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ConfigDomainId {
    Market,
    Observation,
    StateMachine,
    Risk,
    Execution,
    Evidence,
    AiGovernance,
    Snapshot,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ConfigDomainLifecycle {
    Implemented,
    Documentable,
    Milestone,
    Unsupported,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ConfigDomainReadiness {
    Ready,
    Incomplete,
    Restricted,
    Stale,
    Blocked,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct ConfigSourceRef {
    pub(crate) source_kind: String,
    pub(crate) source_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) digest: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct StrategyConfigFinding {
    pub(crate) severity: String,
    pub(crate) code: String,
    pub(crate) message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct RuntimeBoundarySummary {
    pub(crate) mode_label: String,
    pub(crate) provider_order_submission_attached: bool,
    pub(crate) provider_order_submission_allowed: bool,
    pub(crate) live_execution_allowed: bool,
    pub(crate) execution_capability_sources: Vec<String>,
    pub(crate) rejection_reasons: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct EvidenceAnchorInput {
    pub(crate) anchor_type: String,
    #[serde(default)]
    pub(crate) anchor_id: Option<String>,
    #[serde(default)]
    pub(crate) digest: Option<String>,
    #[serde(default)]
    pub(crate) summary: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct EvidenceAnchor {
    pub(crate) anchor_type: String,
    pub(crate) anchor_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) digest: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) summary: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct ProposalBindingInput {
    pub(crate) proposal_id: String,
    pub(crate) target_domain: ConfigDomainId,
    #[serde(default)]
    pub(crate) before_digest: Option<String>,
    #[serde(default)]
    pub(crate) after_digest: Option<String>,
    #[serde(default)]
    pub(crate) evidence_anchor_ids: Vec<String>,
    #[serde(default)]
    pub(crate) sandbox_status: Option<String>,
    #[serde(default)]
    pub(crate) approval_status: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct ProposalBinding {
    pub(crate) proposal_id: String,
    pub(crate) target_domain: ConfigDomainId,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) before_digest: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) after_digest: Option<String>,
    pub(crate) evidence_anchor_ids: Vec<String>,
    pub(crate) sandbox_status: String,
    pub(crate) approval_status: String,
}
