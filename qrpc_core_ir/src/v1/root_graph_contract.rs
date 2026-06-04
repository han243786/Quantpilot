use serde::{Deserialize, Serialize};

use super::{AgentPolicy, DataBinding, ExecutionRule, IndicatorNode, RiskPolicy, SignalRule};

pub const CORE_IR_V1_VERSION: &str = "quantpilot/core-ir/v1";
pub const CUSTOM_EXPR_V1_VERSION: &str = "quantpilot/custom-expr/v1";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CoreStrategyIr {
    pub ir_version: String,
    pub metadata: CoreMetadata,
    #[serde(default)]
    pub data_bindings: Vec<DataBinding>,
    #[serde(default)]
    pub indicators: Vec<IndicatorNode>,
    #[serde(default)]
    pub signal_rules: Vec<SignalRule>,
    #[serde(default)]
    pub agent_policies: Vec<AgentPolicy>,
    #[serde(default)]
    pub risk_policies: Vec<RiskPolicy>,
    pub execution: ExecutionRule,
    /// v1.0.0 DAG 边: 显式声明 data→indicator→signal→agent→risk→exec 的连接
    /// 为空时退化为线性 pipeline (向后兼容)
    #[serde(default)]
    pub edges: Vec<CoreIREdge>,
}

/// v1.0.0 DAG 边 — 连接两个节点的有向边
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CoreIREdge {
    pub source: String,
    pub target: String,
    #[serde(default)]
    pub port: Option<String>,
}

impl CoreStrategyIr {
    pub fn new(metadata: CoreMetadata, execution: ExecutionRule) -> Self {
        Self {
            ir_version: CORE_IR_V1_VERSION.to_string(),
            metadata,
            data_bindings: Vec::new(),
            indicators: Vec::new(),
            signal_rules: Vec::new(),
            agent_policies: Vec::new(),
            risk_policies: Vec::new(),
            execution,
            edges: Vec::new(),
        }
    }

    /// v1.0.0 DAG 环检测: DFS 拓扑验证，有环返回环路径
    pub fn validate_dag(&self) -> Result<(), Vec<String>> {
        if self.edges.is_empty() {
            return Ok(()); // 无显式边 = 线性 pipeline, 无 DAG 约束
        }

        use std::collections::{BTreeMap, BTreeSet};

        let mut adjacency: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
        for edge in &self.edges {
            adjacency
                .entry(&edge.source)
                .or_default()
                .push(&edge.target);
        }

        // DFS 环检测
        let mut visited = BTreeSet::new();
        let mut in_stack = BTreeSet::new();
        let mut cycle_path: Vec<String> = Vec::new();

        fn dfs<'a>(
            node: &'a str,
            adjacency: &BTreeMap<&'a str, Vec<&'a str>>,
            visited: &mut BTreeSet<&'a str>,
            in_stack: &mut BTreeSet<&'a str>,
            cycle_path: &mut Vec<String>,
        ) -> bool {
            visited.insert(node);
            in_stack.insert(node);
            if let Some(neighbors) = adjacency.get(node) {
                for &next in neighbors {
                    if !visited.contains(next) {
                        if dfs(next, adjacency, visited, in_stack, cycle_path) {
                            cycle_path.push(node.to_string());
                            return true;
                        }
                    } else if in_stack.contains(next) {
                        cycle_path.push(next.to_string());
                        cycle_path.push(node.to_string());
                        return true;
                    }
                }
            }
            in_stack.remove(node);
            false
        }

        for edge in &self.edges {
            if !visited.contains(edge.source.as_str())
                && dfs(
                    edge.source.as_str(),
                    &adjacency,
                    &mut visited,
                    &mut in_stack,
                    &mut cycle_path,
                )
            {
                cycle_path.reverse();
                return Err(vec![format!("DAG 环检测失败: {}", cycle_path.join(" → "))]);
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CoreMetadata {
    pub strategy_id: String,
    pub name: String,
    pub source_kind: CoreSourceKind,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CoreSourceKind {
    StrategyIr,
    FormalQuantScript,
    RuntimeProtocol,
    FrontendGraph,
}
