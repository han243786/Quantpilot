use crate::{Diagnostic, DiagnosticSeverity, Span};
use qrpc_core_ir::v4::{
    V4CapabilityReportDiagnosticSeverity, V4CapabilityReportVerdict, V4StaticContractBundle,
};

use super::{
    parse_v4_static_document, V4QsStaticAuditReport, V4QsStaticAuditVerdict,
    V4_QS_STATIC_AUDIT_REPORT_VERSION,
};

pub(super) fn audit_v4_quant_script_static(
    input: &str,
    bundle: &V4StaticContractBundle,
) -> V4QsStaticAuditReport {
    let mut diagnostics = Vec::new();
    let parsed = parse_v4_static_document(input);

    let Some(parsed) = parsed.map_err(|errors| diagnostics.extend(errors)).ok() else {
        return V4QsStaticAuditReport {
            schema_version: V4_QS_STATIC_AUDIT_REPORT_VERSION.to_string(),
            verdict: V4QsStaticAuditVerdict::Rejected,
            graph_id: None,
            parsed_graph: None,
            capability_request: None,
            capability_report: None,
            diagnostics,
            runtime_attached: false,
            lowering_attached: false,
        };
    };

    diagnostics.extend(
        parsed
            .graph
            .validate_static_contract()
            .err()
            .unwrap_or_default()
            .into_iter()
            .map(|message| {
                Diagnostic::error(
                    "QSV4200",
                    format!("v4 状态机图静态校验失败: {message}"),
                    Some(Span::module(parsed.graph.graph_id.clone())),
                )
            }),
    );

    let mut report_bundle = bundle.clone();
    report_bundle
        .machine_graphs
        .retain(|graph| graph.graph_id != parsed.graph.graph_id);
    report_bundle.machine_graphs.push(parsed.graph.clone());
    let capability_report =
        report_bundle.build_compile_time_capability_report(parsed.request.clone());
    diagnostics.extend(
        capability_report
            .diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.severity == V4CapabilityReportDiagnosticSeverity::Error)
            .map(|diagnostic| {
                Diagnostic::error(
                    "QSV4300",
                    format!(
                        "v4 编译期能力报告拒绝: {} {}: {}",
                        diagnostic.code, diagnostic.target, diagnostic.message
                    ),
                    Some(Span::module(parsed.graph.graph_id.clone())),
                )
            }),
    );

    let rejected = diagnostics
        .iter()
        .any(|diagnostic| diagnostic.severity == DiagnosticSeverity::Error)
        || capability_report.verdict != V4CapabilityReportVerdict::Accepted;

    V4QsStaticAuditReport {
        schema_version: V4_QS_STATIC_AUDIT_REPORT_VERSION.to_string(),
        verdict: if rejected {
            V4QsStaticAuditVerdict::Rejected
        } else {
            V4QsStaticAuditVerdict::Accepted
        },
        graph_id: Some(parsed.graph.graph_id.clone()),
        parsed_graph: Some(parsed.graph),
        capability_request: Some(parsed.request),
        capability_report: Some(capability_report),
        diagnostics,
        runtime_attached: false,
        lowering_attached: false,
    }
}
