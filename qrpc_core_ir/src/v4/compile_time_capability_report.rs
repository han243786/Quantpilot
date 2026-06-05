use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

use super::{
    v4_first_wave_execution_capabilities, CapabilitySupportSource, ComplexityMetrics,
    ExecutionCapabilityKind, PluginKind, QsTypeRef, RuntimeTradingMode, V4StaticContractBundle,
    VenueCapabilityMatrix, V4_COMPILE_TIME_CAPABILITY_REPORT_VERSION,
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

impl V4StaticContractBundle {
    pub fn build_compile_time_capability_report(
        &self,
        request: V4CompileTimeCapabilityRequest,
    ) -> V4CompileTimeCapabilityReport {
        let mut diagnostics = Vec::new();

        if request.schema_version != V4_COMPILE_TIME_CAPABILITY_REQUEST_VERSION {
            push_capability_diagnostic(
                &mut diagnostics,
                V4CapabilityReportDiagnosticSeverity::Error,
                "V4CAP000",
                "request.schema_version",
                format!(
                    "compile-time capability request schema_version must be `{}`",
                    V4_COMPILE_TIME_CAPABILITY_REQUEST_VERSION
                ),
            );
        }
        if request.graph_id.trim().is_empty() {
            push_capability_diagnostic(
                &mut diagnostics,
                V4CapabilityReportDiagnosticSeverity::Error,
                "V4CAP001",
                "request.graph_id",
                "compile-time capability request requires graph_id",
            );
        }
        if request.venue_id.trim().is_empty() {
            push_capability_diagnostic(
                &mut diagnostics,
                V4CapabilityReportDiagnosticSeverity::Error,
                "V4CAP002",
                "request.venue_id",
                "compile-time capability request requires venue_id",
            );
        }
        if let Err(errors) = self.validate_static_contract() {
            for error in errors {
                push_capability_diagnostic(
                    &mut diagnostics,
                    V4CapabilityReportDiagnosticSeverity::Error,
                    "V4CAP003",
                    "static_contract_bundle",
                    error,
                );
            }
        }

        let graph = self
            .machine_graphs
            .iter()
            .find(|graph| graph.graph_id == request.graph_id);
        let graph_found = graph.is_some();
        if !graph_found && !request.graph_id.trim().is_empty() {
            push_capability_diagnostic(
                &mut diagnostics,
                V4CapabilityReportDiagnosticSeverity::Error,
                "V4CAP004",
                "request.graph_id",
                format!("machine graph `{}` is not declared", request.graph_id),
            );
        }

        let venue = self
            .venue_matrices
            .iter()
            .find(|matrix| matrix.venue_id == request.venue_id);
        let venue_found = venue.is_some();
        if !venue_found && !request.venue_id.trim().is_empty() {
            push_capability_diagnostic(
                &mut diagnostics,
                V4CapabilityReportDiagnosticSeverity::Error,
                "V4CAP005",
                "request.venue_id",
                format!(
                    "venue capability matrix `{}` is not declared",
                    request.venue_id
                ),
            );
        }

        let runtime_mode_found = self.runtime_modes.mode_spec(request.runtime_mode).is_some();
        if !runtime_mode_found {
            push_capability_diagnostic(
                &mut diagnostics,
                V4CapabilityReportDiagnosticSeverity::Error,
                "V4CAP006",
                "request.runtime_mode",
                format!("runtime mode `{:?}` is not declared", request.runtime_mode),
            );
        }

        let complexity_metrics = graph.map(|graph| {
            ComplexityMetrics::from_machine_graph(
                graph,
                self.runtime_modes.modes.len() as u32,
                self.plugin_manifests.len() as u32,
            )
        });
        if let Some(metrics) = &complexity_metrics {
            if let Err(errors) = self.complexity_budget.validate_metrics(metrics) {
                for error in errors {
                    push_capability_diagnostic(
                        &mut diagnostics,
                        V4CapabilityReportDiagnosticSeverity::Error,
                        "V4CAP007",
                        "complexity_metrics",
                        error,
                    );
                }
            }
        }

        let type_entries = self.build_type_capability_entries(&request, &mut diagnostics);
        let execution_entries =
            self.build_execution_capability_entries(&request, venue, &mut diagnostics);
        let plugin_entries = self.build_plugin_capability_entries(&request, &mut diagnostics);

        let verdict = if diagnostics
            .iter()
            .any(|diagnostic| diagnostic.severity == V4CapabilityReportDiagnosticSeverity::Error)
        {
            V4CapabilityReportVerdict::Rejected
        } else {
            V4CapabilityReportVerdict::Accepted
        };

        V4CompileTimeCapabilityReport {
            schema_version: V4_COMPILE_TIME_CAPABILITY_REPORT_VERSION.to_string(),
            request,
            verdict,
            graph_found,
            venue_found,
            runtime_mode_found,
            complexity_metrics,
            type_entries,
            execution_entries,
            plugin_entries,
            diagnostics,
            execution_submission_attached: false,
        }
    }

    fn build_type_capability_entries(
        &self,
        request: &V4CompileTimeCapabilityRequest,
        diagnostics: &mut Vec<V4CapabilityReportDiagnostic>,
    ) -> Vec<V4TypeCapabilityReportEntry> {
        let mut entries = Vec::new();

        for (index, type_ref) in request.required_type_refs.iter().enumerate() {
            match self.type_system.validate_type_ref(type_ref) {
                Ok(()) => entries.push(V4TypeCapabilityReportEntry {
                    type_ref: type_ref.clone(),
                    status: V4TypeCapabilityStatus::Accepted,
                    diagnostics: Vec::new(),
                }),
                Err(errors) => {
                    for error in &errors {
                        push_capability_diagnostic(
                            diagnostics,
                            V4CapabilityReportDiagnosticSeverity::Error,
                            "V4CAP100",
                            format!("request.required_type_refs[{index}]"),
                            error.clone(),
                        );
                    }
                    entries.push(V4TypeCapabilityReportEntry {
                        type_ref: type_ref.clone(),
                        status: V4TypeCapabilityStatus::Rejected,
                        diagnostics: errors,
                    });
                }
            }
        }

        entries
    }

    fn build_execution_capability_entries(
        &self,
        request: &V4CompileTimeCapabilityRequest,
        venue: Option<&VenueCapabilityMatrix>,
        diagnostics: &mut Vec<V4CapabilityReportDiagnostic>,
    ) -> Vec<V4ExecutionCapabilityReportEntry> {
        let required = request
            .required_execution_capabilities
            .iter()
            .copied()
            .collect::<BTreeSet<_>>();
        let mut capabilities = v4_first_wave_execution_capabilities()
            .iter()
            .copied()
            .collect::<BTreeSet<_>>();
        capabilities.extend(required.iter().copied());

        capabilities
            .into_iter()
            .map(|capability| {
                let is_required = required.contains(&capability);
                let Some(matrix) = venue else {
                    if is_required {
                        push_capability_diagnostic(
                            diagnostics,
                            V4CapabilityReportDiagnosticSeverity::Error,
                            "V4CAP200",
                            format!("execution_capability.{:?}", capability),
                            format!(
                                "required execution capability `{:?}` has no venue matrix",
                                capability
                            ),
                        );
                    }
                    return V4ExecutionCapabilityReportEntry {
                        capability,
                        required: is_required,
                        status: V4ExecutionCapabilityStatus::NotDeclared,
                        source: CapabilitySupportSource::Unsupported,
                        supported_modes: Vec::new(),
                        selected_source: None,
                        rejection_reason: Some("venue capability matrix is missing".to_string()),
                    };
                };

                let Some(entry) = matrix.capability_entry(&capability) else {
                    if is_required {
                        push_capability_diagnostic(
                            diagnostics,
                            V4CapabilityReportDiagnosticSeverity::Error,
                            "V4CAP201",
                            format!("execution_capability.{:?}", capability),
                            format!(
                                "required execution capability `{:?}` must be explicitly declared",
                                capability
                            ),
                        );
                    }
                    return V4ExecutionCapabilityReportEntry {
                        capability,
                        required: is_required,
                        status: V4ExecutionCapabilityStatus::NotDeclared,
                        source: CapabilitySupportSource::Unsupported,
                        supported_modes: Vec::new(),
                        selected_source: None,
                        rejection_reason: Some(
                            "capability is not declared in the venue matrix".to_string(),
                        ),
                    };
                };

                match matrix.require_supported_for_mode(
                    &capability,
                    request.runtime_mode,
                    &self.runtime_modes,
                ) {
                    Ok(source) => V4ExecutionCapabilityReportEntry {
                        capability,
                        required: is_required,
                        status: V4ExecutionCapabilityStatus::Supported,
                        source: entry.source,
                        supported_modes: entry.supported_modes.clone(),
                        selected_source: Some(source),
                        rejection_reason: None,
                    },
                    Err(reason) => {
                        let status = if matches!(entry.source, CapabilitySupportSource::Unsupported)
                        {
                            V4ExecutionCapabilityStatus::Unsupported
                        } else {
                            V4ExecutionCapabilityStatus::ModeRejected
                        };
                        if is_required {
                            push_capability_diagnostic(
                                diagnostics,
                                V4CapabilityReportDiagnosticSeverity::Error,
                                "V4CAP202",
                                format!("execution_capability.{:?}", capability),
                                reason.clone(),
                            );
                        }
                        V4ExecutionCapabilityReportEntry {
                            capability,
                            required: is_required,
                            status,
                            source: entry.source,
                            supported_modes: entry.supported_modes.clone(),
                            selected_source: None,
                            rejection_reason: Some(reason),
                        }
                    }
                }
            })
            .collect()
    }

    fn build_plugin_capability_entries(
        &self,
        request: &V4CompileTimeCapabilityRequest,
        diagnostics: &mut Vec<V4CapabilityReportDiagnostic>,
    ) -> Vec<V4PluginCapabilityReportEntry> {
        let required = request
            .required_plugin_ids
            .iter()
            .map(String::as_str)
            .collect::<BTreeSet<_>>();
        let mut reported = BTreeSet::new();
        let mut entries = Vec::new();

        for manifest in &self.plugin_manifests {
            let is_required = required.contains(manifest.plugin_id.as_str());
            reported.insert(manifest.plugin_id.as_str());
            match self.plugin_governance.validate_plugin_manifest(
                manifest,
                &self.type_system,
                &self.runtime_modes,
            ) {
                Ok(()) => entries.push(V4PluginCapabilityReportEntry {
                    plugin_id: manifest.plugin_id.clone(),
                    required: is_required,
                    kind: Some(manifest.kind),
                    status: V4PluginCapabilityStatus::Accepted,
                    diagnostics: Vec::new(),
                }),
                Err(errors) => {
                    for error in &errors {
                        push_capability_diagnostic(
                            diagnostics,
                            V4CapabilityReportDiagnosticSeverity::Error,
                            "V4CAP300",
                            format!("plugin_manifest.{}", manifest.plugin_id),
                            error.clone(),
                        );
                    }
                    entries.push(V4PluginCapabilityReportEntry {
                        plugin_id: manifest.plugin_id.clone(),
                        required: is_required,
                        kind: Some(manifest.kind),
                        status: V4PluginCapabilityStatus::Rejected,
                        diagnostics: errors,
                    });
                }
            }
        }

        for plugin_id in &request.required_plugin_ids {
            if reported.contains(plugin_id.as_str()) {
                continue;
            }
            push_capability_diagnostic(
                diagnostics,
                V4CapabilityReportDiagnosticSeverity::Error,
                "V4CAP301",
                format!("request.required_plugin_ids.{plugin_id}"),
                format!("required plugin `{plugin_id}` is not declared"),
            );
            entries.push(V4PluginCapabilityReportEntry {
                plugin_id: plugin_id.clone(),
                required: true,
                kind: None,
                status: V4PluginCapabilityStatus::Missing,
                diagnostics: vec![format!("required plugin `{plugin_id}` is not declared")],
            });
        }

        entries
    }
}

impl V4CompileTimeCapabilityReport {
    pub fn validate_for_compile(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        if self.schema_version != V4_COMPILE_TIME_CAPABILITY_REPORT_VERSION {
            errors.push(format!(
                "compile-time capability report schema_version must be `{}`",
                V4_COMPILE_TIME_CAPABILITY_REPORT_VERSION
            ));
        }
        if self.verdict != V4CapabilityReportVerdict::Accepted {
            errors.push("compile-time capability report verdict must be accepted".to_string());
        }
        if !self.graph_found {
            errors.push("compile-time capability report must resolve graph_id".to_string());
        }
        if !self.venue_found {
            errors.push("compile-time capability report must resolve venue_id".to_string());
        }
        if !self.runtime_mode_found {
            errors.push("compile-time capability report must resolve runtime_mode".to_string());
        }
        if self.execution_submission_attached {
            errors.push(
                "compile-time capability report must not attach execution submission".to_string(),
            );
        }
        for diagnostic in &self.diagnostics {
            if diagnostic.severity == V4CapabilityReportDiagnosticSeverity::Error {
                errors.push(format!(
                    "{} {}: {}",
                    diagnostic.code, diagnostic.target, diagnostic.message
                ));
            }
        }

        let required_capabilities = self
            .request
            .required_execution_capabilities
            .iter()
            .copied()
            .collect::<BTreeSet<_>>();
        for capability in required_capabilities {
            let supported = self.execution_entries.iter().any(|entry| {
                entry.capability == capability
                    && entry.required
                    && entry.status == V4ExecutionCapabilityStatus::Supported
            });
            if !supported {
                errors.push(format!(
                    "required execution capability `{:?}` is not supported by the report",
                    capability
                ));
            }
        }

        for plugin_id in &self.request.required_plugin_ids {
            let accepted = self.plugin_entries.iter().any(|entry| {
                entry.plugin_id == *plugin_id
                    && entry.required
                    && entry.status == V4PluginCapabilityStatus::Accepted
            });
            if !accepted {
                errors.push(format!(
                    "required plugin `{plugin_id}` is not accepted by the report"
                ));
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

fn default_compile_time_capability_request_version() -> String {
    V4_COMPILE_TIME_CAPABILITY_REQUEST_VERSION.to_string()
}

fn default_compile_time_capability_report_version() -> String {
    V4_COMPILE_TIME_CAPABILITY_REPORT_VERSION.to_string()
}

fn push_capability_diagnostic(
    diagnostics: &mut Vec<V4CapabilityReportDiagnostic>,
    severity: V4CapabilityReportDiagnosticSeverity,
    code: impl Into<String>,
    target: impl Into<String>,
    message: impl Into<String>,
) {
    diagnostics.push(V4CapabilityReportDiagnostic {
        severity,
        code: code.into(),
        target: target.into(),
        message: message.into(),
    });
}
