use super::*;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(rename_all = "snake_case")]
pub(super) enum QuantScriptAuthoringSectionKind {
    Risk,
    Execution,
    Data,
    Intent,
    Agent,
    Mixed,
    Unknown,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(super) enum QuantScriptAuthoringSectionOrigin {
    Authored,
    Hybrid,
    Derived,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(super) enum QuantScriptAuthoringSectionStatus {
    Ok,
    Partial,
    Mismatch,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(super) struct QuantScriptAuthoringSection {
    pub(super) id: String,
    pub(super) declared_kind: QuantScriptAuthoringSectionKind,
    pub(super) effective_kind: QuantScriptAuthoringSectionKind,
    pub(super) origin: QuantScriptAuthoringSectionOrigin,
    pub(super) status: QuantScriptAuthoringSectionStatus,
    pub(super) start_line: usize,
    pub(super) end_line: usize,
    pub(super) snippet: String,
    #[serde(default)]
    pub(super) symbols_defined: Vec<String>,
    #[serde(default)]
    pub(super) symbols_used: Vec<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(super) enum QuantScriptAuthoringEdgeRelation {
    Dataflow,
    DecisionFlow,
    PolicyAttachment,
    ExecutionAttachment,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(super) struct QuantScriptAuthoringEdge {
    pub(super) from: String,
    pub(super) to: String,
    pub(super) relation: QuantScriptAuthoringEdgeRelation,
    pub(super) reason: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(rename_all = "snake_case")]
pub(super) enum QuantScriptAuthoringPoolStageKind {
    Source,
    Eligibility,
    Features,
    Selection,
    Weighting,
    Rebalance,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(super) enum QuantScriptAuthoringPoolStageStatus {
    Present,
    Empty,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(super) struct QuantScriptAuthoringPoolStage {
    pub(super) kind: QuantScriptAuthoringPoolStageKind,
    pub(super) status: QuantScriptAuthoringPoolStageStatus,
    pub(super) summary: String,
    #[serde(default)]
    pub(super) details: Vec<String>,
    #[serde(default)]
    pub(super) related_section_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(super) struct QuantScriptAuthoringPoolPipeline {
    pub(super) order: Vec<QuantScriptAuthoringPoolStageKind>,
    pub(super) stages: Vec<QuantScriptAuthoringPoolStage>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(super) struct QuantScriptAuthoringView {
    pub(super) kind: String,
    pub(super) source_hash: String,
    pub(super) source_order: Vec<QuantScriptAuthoringSectionKind>,
    pub(super) pipeline_order: Vec<QuantScriptAuthoringSectionKind>,
    pub(super) sections: Vec<QuantScriptAuthoringSection>,
    pub(super) edges: Vec<QuantScriptAuthoringEdge>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) pool_pipeline: Option<QuantScriptAuthoringPoolPipeline>,
}
