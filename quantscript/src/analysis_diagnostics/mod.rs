pub(crate) mod analysis;
mod dead_code_emit_gate;
pub(crate) mod diagnostics;
mod fetch_lookback_warning_gate;
mod index_bounds_gate;
mod indirect_recursion_gate;
mod lookahead_window_gate;
mod strategy_presence_gate;
mod symbol_whitelist_gate;
mod unsupported_construct_gate;
mod warmup_fetch_gate;

pub(in crate::analysis_diagnostics) use analysis::contains_emit_in_stmts;
pub use analysis::{analyze_script_module, ScriptAnalysis};
pub use diagnostics::{Diagnostic, DiagnosticSeverity, Span, SpanContext};
pub(in crate::analysis_diagnostics) use warmup_fetch_gate::arg_number_named;
pub(in crate::analysis_diagnostics) use warmup_fetch_gate::fetch_lookback;
