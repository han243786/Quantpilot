use serde::{Deserialize, Serialize};

mod report_builder;
mod report_validation;

use super::{
    CapabilitySupportSource, ComplexityMetrics, ExecutionCapabilityKind, PluginKind, QsTypeRef,
    RuntimeTradingMode, V4_COMPILE_TIME_CAPABILITY_REPORT_VERSION,
    V4_COMPILE_TIME_CAPABILITY_REQUEST_VERSION,
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct V4CompileTimeCapabilityRequest {
    #[serde(default = "default_compile_time_capability_request_version")]
    pub schema_version: String,
    pub graph_id: String,
    pub venue_id: String,
    pub runtime_mode: RuntimeTradingMode,
    #[serde(default)]
    pub required_execution_capabilities: Vec<ExecutionCapabilityKind>,
    #[serde(default)]
    pub required_type_refs: Vec<QsTypeRef>,
    #[serde(default)]
    pub required_plugin_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct V4CompileTimeCapabilityReport {
    #[serde(default = "default_compile_time_capability_report_version")]
    pub schema_version: String,
    pub request: V4CompileTimeCapabilityRequest,
    pub verdict: V4CapabilityReportVerdict,
    pub graph_found: bool,
    pub venue_found: bool,
    pub runtime_mode_found: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub complexity_metrics: Option<ComplexityMetrics>,
    #[serde(default)]
    pub type_entries: Vec<V4TypeCapabilityReportEntry>,
    #[serde(default)]
    pub execution_entries: Vec<V4ExecutionCapabilityReportEntry>,
    #[serde(default)]
    pub plugin_entries: Vec<V4PluginCapabilityReportEntry>,
    #[serde(default)]
    pub diagnostics: Vec<V4CapabilityReportDiagnostic>,
    #[serde(default)]
    pub execution_submission_attached: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum V4CapabilityReportVerdict {
    Accepted,
    Rejected,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum V4CapabilityReportDiagnosticSeverity {
    Info,
    Warning,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct V4CapabilityReportDiagnostic {
    pub severity: V4CapabilityReportDiagnosticSeverity,
    pub code: String,
    pub target: String,
    pub message: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum V4TypeCapabilityStatus {
    Accepted,
    Rejected,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct V4TypeCapabilityReportEntry {
    pub type_ref: QsTypeRef,
    pub status: V4TypeCapabilityStatus,
    #[serde(default)]
    pub diagnostics: Vec<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum V4ExecutionCapabilityStatus {
    Supported,
    Unsupported,
    ModeRejected,
    NotDeclared,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct V4ExecutionCapabilityReportEntry {
    pub capability: ExecutionCapabilityKind,
    pub required: bool,
    pub status: V4ExecutionCapabilityStatus,
    pub source: CapabilitySupportSource,
    #[serde(default)]
    pub supported_modes: Vec<RuntimeTradingMode>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub selected_source: Option<CapabilitySupportSource>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rejection_reason: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum V4PluginCapabilityStatus {
    Accepted,
    Rejected,
    Missing,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct V4PluginCapabilityReportEntry {
    pub plugin_id: String,
    pub required: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kind: Option<PluginKind>,
    pub status: V4PluginCapabilityStatus,
    #[serde(default)]
    pub diagnostics: Vec<String>,
}

fn default_compile_time_capability_request_version() -> String {
    V4_COMPILE_TIME_CAPABILITY_REQUEST_VERSION.to_string()
}

fn default_compile_time_capability_report_version() -> String {
    V4_COMPILE_TIME_CAPABILITY_REPORT_VERSION.to_string()
}
