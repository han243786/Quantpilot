pub(crate) mod analysis;
pub(crate) mod diagnostics;

pub use analysis::{analyze_script_module, ScriptAnalysis};
pub use diagnostics::{Diagnostic, DiagnosticSeverity, Span, SpanContext};
