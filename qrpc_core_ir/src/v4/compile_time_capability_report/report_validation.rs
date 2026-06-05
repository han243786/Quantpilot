use std::collections::BTreeSet;

use super::{
    V4CapabilityReportDiagnosticSeverity, V4CapabilityReportVerdict, V4CompileTimeCapabilityReport,
    V4ExecutionCapabilityStatus, V4PluginCapabilityStatus,
};
use crate::v4::V4_COMPILE_TIME_CAPABILITY_REPORT_VERSION;

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
