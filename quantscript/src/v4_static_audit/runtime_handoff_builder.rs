use qrpc_core_ir::v4::{RuntimeTradingMode, V4CapabilityReportVerdict};

use super::{
    V4QsRuntimeHandoffReport, V4QsStaticAuditReport, V4QsStaticAuditVerdict,
    V4_QS_RUNTIME_HANDOFF_REPORT_VERSION,
};

pub(super) fn build_v4_qs_runtime_handoff(
    report: &V4QsStaticAuditReport,
) -> V4QsRuntimeHandoffReport {
    let mut diagnostics = Vec::new();
    if report.verdict != V4QsStaticAuditVerdict::Accepted {
        diagnostics.push("v4 QS static audit must be accepted before runtime handoff".to_string());
    }
    if report.parsed_graph.is_none() {
        diagnostics.push("runtime handoff requires a parsed v4 machine graph".to_string());
    }
    let capability_report = report.capability_report.as_ref();
    if capability_report
        .map(|item| item.verdict != V4CapabilityReportVerdict::Accepted)
        .unwrap_or(true)
    {
        diagnostics.push(
            "runtime handoff requires an accepted compile-time capability report".to_string(),
        );
    }
    if capability_report
        .map(|item| item.execution_submission_attached)
        .unwrap_or(false)
    {
        diagnostics
            .push("runtime handoff must not carry execution submission attachment".to_string());
    }

    let request = report.capability_request.as_ref();
    let runtime_mode = request.map(|item| item.runtime_mode);
    if runtime_mode != Some(RuntimeTradingMode::PaperSimulated) {
        diagnostics.push("current v4 runtime handoff only allows PaperSimulated start".to_string());
    }

    let accepted = diagnostics.is_empty();
    V4QsRuntimeHandoffReport {
        schema_version: V4_QS_RUNTIME_HANDOFF_REPORT_VERSION.to_string(),
        accepted_for_runtime_handoff: accepted,
        graph_id: report.graph_id.clone(),
        venue_id: request.map(|item| item.venue_id.clone()),
        runtime_mode,
        paper_simulated_start_allowed: accepted,
        provider_order_submission_attached: false,
        runtime_attached: false,
        lowering_attached: false,
        diagnostics,
    }
}
