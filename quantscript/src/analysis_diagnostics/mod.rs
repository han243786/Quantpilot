pub(crate) mod analysis;
pub(crate) mod diagnostics;
mod lookahead_window_gate;
mod unsupported_construct_gate;

pub use analysis::{analyze_script_module, ScriptAnalysis};
pub use diagnostics::{Diagnostic, DiagnosticSeverity, Span, SpanContext};
