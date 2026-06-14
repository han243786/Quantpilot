use crate::Diagnostic;
use qrpc_core_ir::v4::{
    ExecutionCapabilityKind, MachineEventCatalog, MachineGraphEdge, MachineGraphRiskPlane,
    MachineMemoryField, MachineState, MachineTemplateKind, MachineTransition, QsTypeRef,
    RuntimeTradingMode, StateGroup, TransitionConflictPolicy, V4CompileTimeCapabilityReport,
    V4CompileTimeCapabilityRequest, V4MachineGraphContract, V4StaticContractBundle,
};

mod audit_entrypoint;
mod capability_type_parser;
mod edge_parser;
mod event_catalog_derivation;
mod machine_block_parser;
mod memory_parser;
mod parser_utilities_diagnostics;
mod runtime_handoff_builder;
mod state_block_parser;
mod static_document_parser;
mod transition_parser;

use machine_block_parser::ParsedMachine;
use parser_utilities_diagnostics::{
    diag, prepare_lines, split_csv_words, split_words, PreparedLine,
};
use static_document_parser::ParsedV4QsStaticDocument;

pub const V4_QS_STATIC_AUDIT_REPORT_VERSION: &str = "quantpilot/qs-v4-static-audit-report/v1";
pub const V4_QS_RUNTIME_HANDOFF_REPORT_VERSION: &str = "quantpilot/qs-v4-runtime-handoff-report/v1";
const V4_DEFAULT_MARKET_DATA_SOURCE: &str = "market.data";

#[derive(Debug, Clone, PartialEq)]
pub struct V4QsStaticAuditReport {
    pub schema_version: String,
    pub verdict: V4QsStaticAuditVerdict,
    pub graph_id: Option<String>,
    pub parsed_graph: Option<V4MachineGraphContract>,
    pub capability_request: Option<V4CompileTimeCapabilityRequest>,
    pub capability_report: Option<V4CompileTimeCapabilityReport>,
    pub diagnostics: Vec<Diagnostic>,
    pub runtime_attached: bool,
    pub lowering_attached: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct V4QsRuntimeHandoffReport {
    pub schema_version: String,
    pub accepted_for_runtime_handoff: bool,
    pub graph_id: Option<String>,
    pub venue_id: Option<String>,
    pub runtime_mode: Option<RuntimeTradingMode>,
    pub paper_simulated_start_allowed: bool,
    pub provider_order_submission_attached: bool,
    pub runtime_attached: bool,
    pub lowering_attached: bool,
    pub diagnostics: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum V4QsStaticAuditVerdict {
    Accepted,
    Rejected,
}

pub fn audit_v4_quant_script_static(
    input: &str,
    bundle: &V4StaticContractBundle,
) -> V4QsStaticAuditReport {
    audit_entrypoint::audit_v4_quant_script_static(input, bundle)
}

pub fn build_v4_qs_runtime_handoff(report: &V4QsStaticAuditReport) -> V4QsRuntimeHandoffReport {
    runtime_handoff_builder::build_v4_qs_runtime_handoff(report)
}

fn parse_v4_static_document(input: &str) -> Result<ParsedV4QsStaticDocument, Vec<Diagnostic>> {
    static_document_parser::parse_v4_static_document(input)
}

fn derive_event_catalog(graph: &V4MachineGraphContract) -> MachineEventCatalog {
    event_catalog_derivation::derive_event_catalog(graph)
}
fn parse_machine_block(
    lines: &[PreparedLine],
    start_index: usize,
    machine_depth: u32,
    parent_template: Option<MachineTemplateKind>,
) -> Result<(ParsedMachine, usize), (Vec<Diagnostic>, usize)> {
    machine_block_parser::parse_machine_block(lines, start_index, machine_depth, parent_template)
}
fn parse_state(input: &str, line_number: usize) -> Result<MachineState, Diagnostic> {
    state_block_parser::parse_state(input, line_number)
}

fn parse_state_block(
    lines: &[PreparedLine],
    start_index: usize,
    machine_depth: u32,
    parent_template: MachineTemplateKind,
) -> Result<(MachineState, usize), (Vec<Diagnostic>, usize)> {
    state_block_parser::parse_state_block(lines, start_index, machine_depth, parent_template)
}
fn parse_state_group(input: &str, line_number: usize) -> Result<StateGroup, Diagnostic> {
    if input.contains('{') {
        return Err(diag(
            "QSV4111",
            "state_group 在 Phase 3 只能是扁平分组，不能打开嵌套块",
            line_number,
        ));
    }
    let parts = split_words(input);
    if parts.len() < 2 {
        return Err(diag(
            "QSV4112",
            "state_group 必须声明 group id 和至少一个 state",
            line_number,
        ));
    }
    Ok(StateGroup {
        group_id: parts[0].to_string(),
        state_ids: parts[1..]
            .iter()
            .map(|value| (*value).to_string())
            .collect(),
        conflict_policy: TransitionConflictPolicy::Error,
        timeout_ms: None,
    })
}

fn parse_memory(input: &str, line_number: usize) -> Result<MachineMemoryField, Diagnostic> {
    memory_parser::parse_memory(input, line_number)
}
fn parse_transition(
    input: &str,
    machine_id: &str,
    transition_index: usize,
    line_number: usize,
) -> Result<MachineTransition, Diagnostic> {
    transition_parser::parse_transition(input, machine_id, transition_index, line_number)
}
fn parse_edge(
    input: &str,
    edge_index: usize,
    line_number: usize,
) -> Result<MachineGraphEdge, Diagnostic> {
    edge_parser::parse_edge(input, edge_index, line_number)
}
fn parse_risk_plane(input: &str, line_number: usize) -> Result<MachineGraphRiskPlane, Diagnostic> {
    let parts = split_words(input);
    if parts.is_empty() {
        return Err(diag(
            "QSV4121",
            "risk_plane 必须声明至少一个 machine id",
            line_number,
        ));
    }
    let mut machine_ids = Vec::new();
    let mut min_priority = 9_000;
    let mut cursor = 0;
    while cursor < parts.len() {
        if parts[cursor] == "priority" {
            cursor += 1;
            let Some(value) = parts.get(cursor) else {
                return Err(diag("QSV4122", "risk_plane priority 缺少数值", line_number));
            };
            min_priority = value
                .parse::<i32>()
                .map_err(|_| diag("QSV4123", "risk_plane priority 必须是整数", line_number))?;
            cursor += 1;
        } else {
            machine_ids.extend(split_csv_words(parts[cursor]));
            cursor += 1;
        }
    }
    Ok(MachineGraphRiskPlane {
        required: true,
        machine_ids,
        min_priority,
    })
}

fn parse_machine_template(input: &str) -> Result<MachineTemplateKind, String> {
    match input {
        "observation" => Ok(MachineTemplateKind::Observation),
        "decision" => Ok(MachineTemplateKind::Decision),
        "execution" => Ok(MachineTemplateKind::Execution),
        other => Err(format!("未知 machine template: {other}")),
    }
}

fn parse_runtime_mode(input: &str) -> Result<RuntimeTradingMode, String> {
    capability_type_parser::parse_runtime_mode(input)
}

fn parse_execution_capability(input: &str) -> Result<ExecutionCapabilityKind, String> {
    capability_type_parser::parse_execution_capability(input)
}

fn parse_qs_type_ref(input: &str) -> Result<QsTypeRef, String> {
    capability_type_parser::parse_qs_type_ref(input)
}

#[cfg(test)]
mod tests {
    use super::*;
    use qrpc_core_ir::v4::{
        CapabilitySupportSource, PluginKind, PluginManifestSpec, PluginNetworkPermission,
        PluginRuntimePermission, PluginSideEffect, VenueCapabilityMatrix,
    };
    use qrpc_core_ir::v4::{QsScalarTypeKind, V4CapabilityReportVerdict};
    use std::collections::BTreeSet;

    const SAMPLE_V4_QS: &str = r#"
v4_strategy strategy.v4.sample {
  venue paper-local
  mode paper_simulated
  require capability market
  require type fresh<list<price,max=256>>
  require plugin pure.indicator.zscore

  machine data.market observation priority 8000 {
    state idle initial
    state ready
    state_group active idle ready
    memory last_signal_at: time nullable
    on market.tick from idle to ready emit bar_closed write last_signal_at
  }

  machine intent.trend decision priority 5200 {
    state idle initial
    state ready
    state_group active idle ready
    memory last_signal_at: time nullable
    on bar_closed from idle to ready emit intent.long write last_signal_at
  }

  machine risk.guard decision priority 9500 {
    state idle initial
    state ready
    state_group active idle ready
    memory last_signal_at: time nullable
    on intent.long from idle to ready emit risk.approved write last_signal_at
  }

  machine execution.router execution priority 4000 {
    state idle initial
    state ready
    state_group active idle ready
    memory last_signal_at: time nullable
    on risk.approved from idle to ready write last_signal_at
  }

  edge data.market -> intent.trend on bar_closed
  edge intent.trend -> risk.guard on intent.long
  edge risk.guard -> execution.router on risk.approved
  risk_plane risk.guard priority 9000
}
"#;

    fn bundle_with_market_support() -> V4StaticContractBundle {
        V4StaticContractBundle {
            venue_matrices: vec![market_supported_matrix()],
            plugin_manifests: vec![sample_pure_plugin_manifest()],
            ..V4StaticContractBundle::default()
        }
    }

    fn market_supported_matrix() -> VenueCapabilityMatrix {
        let mut matrix = qrpc_core_ir::v4::unsupported_v4_first_wave_matrix("paper-local");
        let market = matrix
            .capabilities
            .iter_mut()
            .find(|entry| entry.capability == ExecutionCapabilityKind::Market)
            .expect("first-wave matrix should include market capability");
        market.source = CapabilitySupportSource::RuntimeSimulated;
        market.supported_modes = vec![RuntimeTradingMode::PaperSimulated];
        matrix
    }

    fn sample_pure_plugin_manifest() -> PluginManifestSpec {
        PluginManifestSpec {
            plugin_id: "pure.indicator.zscore".to_string(),
            name: "ZScore".to_string(),
            version: "0.1.0".to_string(),
            kind: PluginKind::Pure,
            input_schema: Some(QsTypeRef::List {
                item: Box::new(QsTypeRef::Scalar {
                    scalar: QsScalarTypeKind::Price,
                }),
                max_items: 256,
            }),
            output_schema: Some(QsTypeRef::Scalar {
                scalar: QsScalarTypeKind::Decimal,
            }),
            deterministic: true,
            side_effect: PluginSideEffect::None,
            runtime_permission: PluginRuntimePermission::None,
            network_permission: PluginNetworkPermission::None,
            capability_matrix: None,
            test_fixture_id: "fixture.zscore.basic".to_string(),
        }
    }

    fn diagnostic_codes(source: &str, bundle: &V4StaticContractBundle) -> BTreeSet<&'static str> {
        audit_v4_quant_script_static(source, bundle)
            .diagnostics
            .iter()
            .map(|diagnostic| diagnostic.code)
            .collect()
    }

    struct DiagnosticMatrixCase<'a> {
        name: &'a str,
        source: String,
        bundle: &'a V4StaticContractBundle,
        expected_codes: &'a [&'static str],
    }

    #[test]
    fn v4_static_audit_diagnostic_matrix_covers_declared_qsv_codes() {
        let supported_bundle = bundle_with_market_support();
        let unsupported_market_bundle = V4StaticContractBundle {
            venue_matrices: vec![qrpc_core_ir::v4::unsupported_v4_first_wave_matrix(
                "paper-local",
            )],
            plugin_manifests: vec![sample_pure_plugin_manifest()],
            ..V4StaticContractBundle::default()
        };
        let cases = vec![
            DiagnosticMatrixCase {
                name: "empty document",
                source: "".to_string(),
                bundle: &supported_bundle,
                expected_codes: &["QSV4000"],
            },
            DiagnosticMatrixCase {
                name: "invalid strategy header",
                source: "strategy bad {".to_string(),
                bundle: &supported_bundle,
                expected_codes: &["QSV4001"],
            },
            DiagnosticMatrixCase {
                name: "unknown runtime mode",
                source: SAMPLE_V4_QS.replace("mode paper_simulated", "mode paper_unknown"),
                bundle: &supported_bundle,
                expected_codes: &["QSV4002"],
            },
            DiagnosticMatrixCase {
                name: "unknown execution capability",
                source: SAMPLE_V4_QS.replace("require capability market", "require capability pegged"),
                bundle: &supported_bundle,
                expected_codes: &["QSV4003"],
            },
            DiagnosticMatrixCase {
                name: "unknown qs type",
                source: SAMPLE_V4_QS.replace(
                    "require type fresh<list<price,max=256>>",
                    "require type fresh<unknown_type>",
                ),
                bundle: &supported_bundle,
                expected_codes: &["QSV4004"],
            },
            DiagnosticMatrixCase {
                name: "unsupported top-level statement",
                source: SAMPLE_V4_QS.replace(
                    "  risk_plane risk.guard priority 9000",
                    "  unsupported top level\n  risk_plane risk.guard priority 9000",
                ),
                bundle: &supported_bundle,
                expected_codes: &["QSV4006"],
            },
            DiagnosticMatrixCase {
                name: "content after graph close",
                source: format!("{SAMPLE_V4_QS}\nextra_after_close"),
                bundle: &supported_bundle,
                expected_codes: &["QSV4007"],
            },
            DiagnosticMatrixCase {
                name: "missing venue",
                source: SAMPLE_V4_QS.replace("  venue paper-local\n", ""),
                bundle: &supported_bundle,
                expected_codes: &["QSV4008"],
            },
            DiagnosticMatrixCase {
                name: "missing runtime mode",
                source: SAMPLE_V4_QS.replace("  mode paper_simulated\n", ""),
                bundle: &supported_bundle,
                expected_codes: &["QSV4009"],
            },
            DiagnosticMatrixCase {
                name: "invalid machine header",
                source: SAMPLE_V4_QS.replace(
                    "machine data.market observation priority 8000 {",
                    "machine data.market {",
                ),
                bundle: &supported_bundle,
                expected_codes: &["QSV4100"],
            },
            DiagnosticMatrixCase {
                name: "unknown machine template",
                source: SAMPLE_V4_QS.replace(
                    "machine data.market observation priority 8000 {",
                    "machine data.market sensor priority 8000 {",
                ),
                bundle: &supported_bundle,
                expected_codes: &["QSV4101"],
            },
            DiagnosticMatrixCase {
                name: "invalid machine header modifier",
                source: SAMPLE_V4_QS.replace(
                    "machine data.market observation priority 8000 {",
                    "machine data.market observation rank 8000 {",
                ),
                bundle: &supported_bundle,
                expected_codes: &["QSV4102"],
            },
            DiagnosticMatrixCase {
                name: "non-integer machine priority",
                source: SAMPLE_V4_QS.replace(
                    "machine data.market observation priority 8000 {",
                    "machine data.market observation priority high {",
                ),
                bundle: &supported_bundle,
                expected_codes: &["QSV4103"],
            },
            DiagnosticMatrixCase {
                name: "machine nested outside state block",
                source: SAMPLE_V4_QS.replacen(
                    "    state idle initial",
                    "    machine data.market.child observation priority 7000 {\n      state idle initial\n    }\n    state idle initial",
                    1,
                ),
                bundle: &supported_bundle,
                expected_codes: &["QSV4104"],
            },
            DiagnosticMatrixCase {
                name: "unsupported machine statement",
                source: SAMPLE_V4_QS.replacen(
                    "    on risk.approved from idle to ready write last_signal_at",
                    "    transition idle -> ready",
                    1,
                ),
                bundle: &supported_bundle,
                expected_codes: &["QSV4105"],
            },
            DiagnosticMatrixCase {
                name: "invalid state block header",
                source: SAMPLE_V4_QS.replacen("    state idle initial", "    state {", 1),
                bundle: &supported_bundle,
                expected_codes: &["QSV4110"],
            },
            DiagnosticMatrixCase {
                name: "nested state group block",
                source: SAMPLE_V4_QS.replace(
                    "state_group active idle ready",
                    "state_group active {",
                ),
                bundle: &supported_bundle,
                expected_codes: &["QSV4111"],
            },
            DiagnosticMatrixCase {
                name: "state group without members",
                source: SAMPLE_V4_QS.replace(
                    "state_group active idle ready",
                    "state_group active",
                ),
                bundle: &supported_bundle,
                expected_codes: &["QSV4112"],
            },
            DiagnosticMatrixCase {
                name: "memory without colon",
                source: SAMPLE_V4_QS.replace(
                    "memory last_signal_at: time nullable",
                    "memory last_signal_at time nullable",
                ),
                bundle: &supported_bundle,
                expected_codes: &["QSV4113"],
            },
            DiagnosticMatrixCase {
                name: "memory without type",
                source: SAMPLE_V4_QS.replace(
                    "memory last_signal_at: time nullable",
                    "memory last_signal_at:",
                ),
                bundle: &supported_bundle,
                expected_codes: &["QSV4114"],
            },
            DiagnosticMatrixCase {
                name: "transition without from-to syntax",
                source: SAMPLE_V4_QS.replacen(
                    "on market.tick from idle to ready emit bar_closed write last_signal_at",
                    "on market.tick idle ready",
                    1,
                ),
                bundle: &supported_bundle,
                expected_codes: &["QSV4115"],
            },
            DiagnosticMatrixCase {
                name: "transition with unsupported modifier",
                source: SAMPLE_V4_QS.replacen(
                    "on market.tick from idle to ready emit bar_closed write last_signal_at",
                    "on market.tick from idle to ready using custom",
                    1,
                ),
                bundle: &supported_bundle,
                expected_codes: &["QSV4116"],
            },
            DiagnosticMatrixCase {
                name: "unknown memory type",
                source: SAMPLE_V4_QS.replace(
                    "memory last_signal_at: time nullable",
                    "memory last_signal_at: made_up nullable",
                ),
                bundle: &supported_bundle,
                expected_codes: &["QSV4117"],
            },
            DiagnosticMatrixCase {
                name: "third-level nested machine",
                source: SAMPLE_V4_QS.replacen(
                    "state ready\n    state_group active idle ready",
                    "state ready {\n      machine data.market.child observation priority 7000 {\n        state idle {\n          machine data.market.grandchild observation priority 6000 {\n            state idle initial\n          }\n        }\n      }\n    }\n    state_group active idle ready",
                    1,
                ),
                bundle: &supported_bundle,
                expected_codes: &["QSV4118"],
            },
            DiagnosticMatrixCase {
                name: "nested machine template mismatch",
                source: SAMPLE_V4_QS.replacen(
                    "state ready\n    state_group active idle ready",
                    "state ready {\n      machine data.market.child decision priority 7000 {\n        state idle initial\n      }\n    }\n    state_group active idle ready",
                    1,
                ),
                bundle: &supported_bundle,
                expected_codes: &["QSV4119"],
            },
            DiagnosticMatrixCase {
                name: "invalid edge syntax",
                source: SAMPLE_V4_QS.replace(
                    "edge data.market -> intent.trend on bar_closed",
                    "edge data.market intent.trend bar_closed",
                ),
                bundle: &supported_bundle,
                expected_codes: &["QSV4120"],
            },
            DiagnosticMatrixCase {
                name: "unsupported state block content",
                source: SAMPLE_V4_QS.replacen(
                    "state ready\n    state_group active idle ready",
                    "state ready {\n      unsupported line\n    }\n    state_group active idle ready",
                    1,
                ),
                bundle: &supported_bundle,
                expected_codes: &["QSV4121"],
            },
            DiagnosticMatrixCase {
                name: "risk plane priority without value",
                source: SAMPLE_V4_QS.replace(
                    "risk_plane risk.guard priority 9000",
                    "risk_plane risk.guard priority",
                ),
                bundle: &supported_bundle,
                expected_codes: &["QSV4122"],
            },
            DiagnosticMatrixCase {
                name: "risk plane priority not integer",
                source: SAMPLE_V4_QS.replace(
                    "risk_plane risk.guard priority 9000",
                    "risk_plane risk.guard priority high",
                ),
                bundle: &supported_bundle,
                expected_codes: &["QSV4123"],
            },
            DiagnosticMatrixCase {
                name: "static graph validation failure",
                source: SAMPLE_V4_QS.replacen(
                    "on risk.approved from idle to ready write last_signal_at",
                    "on risk.approved from idle to ready write unknown_memory",
                    1,
                ),
                bundle: &supported_bundle,
                expected_codes: &["QSV4200"],
            },
            DiagnosticMatrixCase {
                name: "compile-time capability rejection",
                source: SAMPLE_V4_QS.to_string(),
                bundle: &unsupported_market_bundle,
                expected_codes: &["QSV4300"],
            },
        ];

        let mut covered_codes = BTreeSet::new();
        for case in cases {
            let codes = diagnostic_codes(&case.source, case.bundle);
            for expected_code in case.expected_codes {
                assert!(
                    codes.contains(expected_code),
                    "case `{}` expected `{}` in {:?}",
                    case.name,
                    expected_code,
                    codes
                );
                covered_codes.insert(*expected_code);
            }
        }

        assert!(
            covered_codes.len() >= 30,
            "expected at least 30 QSV codes covered, got {}: {:?}",
            covered_codes.len(),
            covered_codes
        );
    }

    #[test]
    fn v4_static_audit_accepts_supported_state_machine_script_without_runtime() {
        let report = audit_v4_quant_script_static(SAMPLE_V4_QS, &bundle_with_market_support());

        assert_eq!(
            report.verdict,
            V4QsStaticAuditVerdict::Accepted,
            "{:?}",
            report.diagnostics
        );
        assert_eq!(report.graph_id.as_deref(), Some("strategy.v4.sample"));
        assert!(!report.runtime_attached);
        assert!(!report.lowering_attached);
        assert_eq!(
            report
                .capability_report
                .as_ref()
                .expect("accepted report should include capability report")
                .verdict,
            V4CapabilityReportVerdict::Accepted
        );
        let graph = report
            .parsed_graph
            .as_ref()
            .expect("accepted report should include parsed graph");
        assert_eq!(graph.machines.len(), 4);
        assert_eq!(
            graph
                .metadata
                .get("default_venue_id")
                .and_then(|value| value.as_str()),
            Some("paper-local")
        );
        assert_eq!(
            graph
                .metadata
                .get("market_event_source")
                .and_then(|value| value.as_str()),
            Some(V4_DEFAULT_MARKET_DATA_SOURCE)
        );
    }

    #[test]
    fn v4_static_audit_builds_safe_paper_simulated_runtime_handoff() {
        let report = audit_v4_quant_script_static(SAMPLE_V4_QS, &bundle_with_market_support());
        let handoff = build_v4_qs_runtime_handoff(&report);

        assert!(handoff.accepted_for_runtime_handoff);
        assert!(handoff.paper_simulated_start_allowed);
        assert_eq!(handoff.graph_id.as_deref(), Some("strategy.v4.sample"));
        assert_eq!(handoff.venue_id.as_deref(), Some("paper-local"));
        assert_eq!(
            handoff.runtime_mode,
            Some(RuntimeTradingMode::PaperSimulated)
        );
        assert!(!handoff.provider_order_submission_attached);
        assert!(!handoff.runtime_attached);
        assert!(!handoff.lowering_attached);
        assert!(handoff.diagnostics.is_empty());
    }

    #[test]
    fn v4_static_audit_rejects_unsupported_required_capability() {
        let bundle = V4StaticContractBundle {
            venue_matrices: vec![qrpc_core_ir::v4::unsupported_v4_first_wave_matrix(
                "paper-local",
            )],
            plugin_manifests: vec![sample_pure_plugin_manifest()],
            ..V4StaticContractBundle::default()
        };

        let report = audit_v4_quant_script_static(SAMPLE_V4_QS, &bundle);

        assert_eq!(report.verdict, V4QsStaticAuditVerdict::Rejected);
        assert!(report
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "QSV4300"));
        let handoff = build_v4_qs_runtime_handoff(&report);
        assert!(!handoff.accepted_for_runtime_handoff);
        assert!(handoff
            .diagnostics
            .iter()
            .any(|message| message.contains("static audit must be accepted")));
    }

    #[test]
    fn v4_static_audit_accepts_nested_machine_blocks() {
        let source = SAMPLE_V4_QS.replacen(
            "state ready\n    state_group active idle ready",
            "state ready {\n      machine data.market.child observation priority 7000 {\n        state idle initial\n        state ready\n        memory last_signal_at: time nullable\n        on market.tick from idle to ready emit child.ready write last_signal_at\n      }\n    }\n    state_group active idle ready",
            1,
        );

        let report = audit_v4_quant_script_static(&source, &bundle_with_market_support());

        assert_eq!(
            report.verdict,
            V4QsStaticAuditVerdict::Accepted,
            "{:?}",
            report.diagnostics
        );
        let child_machine = report
            .parsed_graph
            .as_ref()
            .expect("accepted report should include parsed graph")
            .machines[0]
            .states[1]
            .child_machine
            .as_ref()
            .expect("nested ready state should include child machine");
        assert_eq!(child_machine.machine_id, "data.market.child");
    }

    #[test]
    fn v4_static_audit_rejects_depth_three_nested_machine_blocks() {
        let source = SAMPLE_V4_QS.replacen(
            "state ready\n    state_group active idle ready",
            "state ready {\n      machine data.market.child observation priority 7000 {\n        state idle {\n          machine data.market.grandchild observation priority 6000 {\n            state idle initial\n          }\n        }\n      }\n    }\n    state_group active idle ready",
            1,
        );

        let report = audit_v4_quant_script_static(&source, &bundle_with_market_support());

        assert_eq!(report.verdict, V4QsStaticAuditVerdict::Rejected);
        assert!(report
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "QSV4118"));
    }

    #[test]
    fn v4_static_audit_rejects_transition_without_event_syntax() {
        let source = SAMPLE_V4_QS.replace(
            "on risk.approved from idle to ready write last_signal_at",
            "transition idle -> ready",
        );

        let report = audit_v4_quant_script_static(&source, &bundle_with_market_support());

        assert_eq!(report.verdict, V4QsStaticAuditVerdict::Rejected);
        assert!(report
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "QSV4105"));
    }

    #[test]
    fn v4_static_audit_rejects_unknown_memory_type() {
        let source = SAMPLE_V4_QS.replace(
            "memory last_signal_at: time nullable",
            "memory last_signal_at: made_up nullable",
        );

        let report = audit_v4_quant_script_static(&source, &bundle_with_market_support());

        assert_eq!(report.verdict, V4QsStaticAuditVerdict::Rejected);
        assert!(report
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "QSV4117"));
    }

    #[test]
    fn v4_static_audit_rejects_memory_write_to_undeclared_field() {
        let source = SAMPLE_V4_QS.replace(
            "on risk.approved from idle to ready write last_signal_at",
            "on risk.approved from idle to ready write unknown_memory",
        );

        let report = audit_v4_quant_script_static(&source, &bundle_with_market_support());

        assert_eq!(report.verdict, V4QsStaticAuditVerdict::Rejected);
        assert!(report
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.message.contains("unknown_memory")));
    }

    #[test]
    fn v4_static_audit_rejects_mode_capability_source_mismatch() {
        let mut matrix = market_supported_matrix();
        let market = matrix
            .capabilities
            .iter_mut()
            .find(|entry| entry.capability == ExecutionCapabilityKind::Market)
            .expect("first-wave matrix should include market capability");
        market.source = CapabilitySupportSource::ProviderNative;
        let bundle = V4StaticContractBundle {
            venue_matrices: vec![matrix],
            plugin_manifests: vec![sample_pure_plugin_manifest()],
            ..V4StaticContractBundle::default()
        };

        let report = audit_v4_quant_script_static(SAMPLE_V4_QS, &bundle);

        assert_eq!(report.verdict, V4QsStaticAuditVerdict::Rejected);
        assert!(report
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.message.contains("requires runtime_simulated")));
    }
}
