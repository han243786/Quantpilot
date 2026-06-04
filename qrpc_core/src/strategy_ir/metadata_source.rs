use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct StrategyMetadata {
    pub strategy_id: String,
    pub name: String,
    pub summary: String,
    pub source: StrategySource,
    #[serde(default)]
    pub authors: Vec<String>,
    #[serde(default)]
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StrategySource {
    pub source_type: StrategySourceType,
    pub paper_title: String,
    pub paper_reference: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum StrategySourceType {
    ManualPaperAnalysis,
    LlmPaperAnalysis,
    HumanAuthored,
    Imported,
}
