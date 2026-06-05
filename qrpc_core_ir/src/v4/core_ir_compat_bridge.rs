mod compat_graph_builder;
mod core_ir_validation;

use compat_graph_builder::build_core_ir_compat_machine_graph;
use core_ir_validation::validate_core_ir_for_v4_bridge;
use serde::{Deserialize, Serialize};

use super::{V4MachineGraphContract, V4_CORE_IR_COMPAT_BRIDGE_VERSION};
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CoreIrV4CompatibilityReport {
    #[serde(default = "default_core_ir_compat_bridge_version")]
    pub schema_version: String,
    pub verdict: CoreIrV4BridgeVerdict,
    pub core_ir_version: String,
    pub strategy_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub graph: Option<V4MachineGraphContract>,
    #[serde(default)]
    pub diagnostics: Vec<CoreIrV4BridgeDiagnostic>,
    #[serde(default)]
    pub lowering_attached: bool,
    #[serde(default)]
    pub runtime_attached: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CoreIrV4BridgeVerdict {
    Accepted,
    Rejected,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CoreIrV4BridgeDiagnosticSeverity {
    Info,
    Warning,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CoreIrV4BridgeDiagnostic {
    pub severity: CoreIrV4BridgeDiagnosticSeverity,
    pub code: String,
    pub target: String,
    pub message: String,
}

impl CoreIrV4CompatibilityReport {
    pub fn validate_for_phase4(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        if self.schema_version != V4_CORE_IR_COMPAT_BRIDGE_VERSION {
            errors.push(format!(
                "core ir compatibility report schema_version must be `{}`",
                V4_CORE_IR_COMPAT_BRIDGE_VERSION
            ));
        }
        if self.verdict != CoreIrV4BridgeVerdict::Accepted {
            errors.push("core ir compatibility report verdict must be accepted".to_string());
        }
        if self.lowering_attached {
            errors.push(
                "core ir compatibility bridge must not attach v4 lowering in Phase 4".to_string(),
            );
        }
        if self.runtime_attached {
            errors.push(
                "core ir compatibility bridge must not attach runtime in Phase 4".to_string(),
            );
        }
        for diagnostic in &self.diagnostics {
            if diagnostic.severity == CoreIrV4BridgeDiagnosticSeverity::Error {
                errors.push(format!(
                    "{} {}: {}",
                    diagnostic.code, diagnostic.target, diagnostic.message
                ));
            }
        }
        match &self.graph {
            Some(graph) => {
                errors.extend(graph.validate_static_contract().err().unwrap_or_default())
            }
            None => errors.push(
                "core ir compatibility bridge must produce a machine graph when accepted"
                    .to_string(),
            ),
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

pub fn bridge_core_ir_to_v4_machine_graph(
    core_ir: &crate::CoreStrategyIr,
) -> CoreIrV4CompatibilityReport {
    let mut diagnostics = Vec::new();

    validate_core_ir_for_v4_bridge(core_ir, &mut diagnostics);

    let mut graph = if diagnostics
        .iter()
        .any(|diagnostic| diagnostic.severity == CoreIrV4BridgeDiagnosticSeverity::Error)
    {
        None
    } else {
        Some(build_core_ir_compat_machine_graph(core_ir))
    };

    if let Some(candidate_graph) = &graph {
        if let Err(errors) = candidate_graph.validate_static_contract() {
            for error in errors {
                push_core_ir_v4_bridge_diagnostic(
                    &mut diagnostics,
                    CoreIrV4BridgeDiagnosticSeverity::Error,
                    "V4BRIDGE900",
                    "machine_graph",
                    error,
                );
            }
        }
    }

    if diagnostics
        .iter()
        .any(|diagnostic| diagnostic.severity == CoreIrV4BridgeDiagnosticSeverity::Error)
    {
        graph = None;
    }

    let verdict = if diagnostics
        .iter()
        .any(|diagnostic| diagnostic.severity == CoreIrV4BridgeDiagnosticSeverity::Error)
    {
        CoreIrV4BridgeVerdict::Rejected
    } else {
        CoreIrV4BridgeVerdict::Accepted
    };

    CoreIrV4CompatibilityReport {
        schema_version: V4_CORE_IR_COMPAT_BRIDGE_VERSION.to_string(),
        verdict,
        core_ir_version: core_ir.ir_version.clone(),
        strategy_id: core_ir.metadata.strategy_id.clone(),
        graph,
        diagnostics,
        lowering_attached: false,
        runtime_attached: false,
    }
}

fn default_core_ir_compat_bridge_version() -> String {
    V4_CORE_IR_COMPAT_BRIDGE_VERSION.to_string()
}

fn push_core_ir_v4_bridge_diagnostic(
    diagnostics: &mut Vec<CoreIrV4BridgeDiagnostic>,
    severity: CoreIrV4BridgeDiagnosticSeverity,
    code: impl Into<String>,
    target: impl Into<String>,
    message: impl Into<String>,
) {
    diagnostics.push(CoreIrV4BridgeDiagnostic {
        severity,
        code: code.into(),
        target: target.into(),
        message: message.into(),
    });
}
