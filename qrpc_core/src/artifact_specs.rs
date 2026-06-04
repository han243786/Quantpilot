use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

mod canonical_digest;
mod run_backtest_specs;

pub use canonical_digest::*;
pub use run_backtest_specs::*;

use crate::{CoreStrategyIr, RuntimeProtocolCoreConfig};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum StrategyArtifactSourceKind {
    FrontendGraph,
    RuntimeProtocol,
    StrategyIr,
    FormalQuantScript,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyArtifact {
    pub schema_version: String,
    pub artifact_id: String,
    pub graph_id: String,
    pub compile_id: String,
    pub strategy_id: String,
    pub name: String,
    pub source_kind: StrategyArtifactSourceKind,
    pub source_ref: String,
    #[serde(default)]
    pub metadata: BTreeMap<String, Value>,
    pub digest: ArtifactDigest,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoreIrArtifact {
    pub schema_version: String,
    pub artifact_id: String,
    pub graph_id: String,
    pub compile_id: String,
    pub ir_version: String,
    pub digest: ArtifactDigest,
    pub core_ir: CoreStrategyIr,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompileArtifact {
    pub schema_version: String,
    pub artifact_id: String,
    pub graph_id: String,
    pub compile_id: String,
    pub protocol_name: String,
    pub config_hash: String,
    pub strategy_artifact_id: String,
    pub core_ir_artifact_id: String,
    pub digest: ArtifactDigest,
    pub runtime_config: RuntimeProtocolCoreConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompileArtifactBundle {
    pub strategy: StrategyArtifact,
    pub compile: CompileArtifact,
    pub core_ir: CoreIrArtifact,
}
