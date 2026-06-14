pub(crate) mod analysis;
pub(crate) mod diagnostics;
mod fetch_lookback_warning_gate;
mod indirect_recursion_gate;
mod lookahead_window_gate;
mod symbol_whitelist_gate;
mod unsupported_construct_gate;
mod warmup_fetch_gate;

pub use analysis::{analyze_script_module, ScriptAnalysis};
pub use diagnostics::{Diagnostic, DiagnosticSeverity, Span, SpanContext};
pub(in crate::analysis_diagnostics) use warmup_fetch_gate::arg_number_named;
