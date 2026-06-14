use super::dead_code_emit_gate::check_dead_code_emit;
use super::fetch_lookback_warning_gate::collect_fetch_lookback_warnings;
use super::index_bounds_gate::{build_data_source_map, check_all_index_bounds};
use super::indirect_recursion_gate::detect_indirect_recursion;
use super::lookahead_window_gate::{
    collect_centered_window_diagnostics, collect_series_index_diagnostics,
};
use super::strategy_presence_gate::{check_strategy_has_emit, check_strategy_has_fetch};
use super::symbol_whitelist_gate::check_fetch_symbol_whitelist;
use super::unsupported_construct_gate::collect_unsupported_construct_diagnostics;
use super::warmup_fetch_gate::{collect_warmup_diagnostics, infer_required_warmup_bars};

use crate::diagnostics::{Diagnostic, DiagnosticSeverity};
use crate::resolve::ResolveResult;
use crate::script::{MatchArmBody, ScriptModule, Stmt};

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ScriptAnalysis {
    pub required_warmup_bars: usize,
    pub diagnostics: Vec<Diagnostic>,
}

impl ScriptAnalysis {
    pub fn has_errors(&self) -> bool {
        self.diagnostics
            .iter()
            .any(|diagnostic| diagnostic.severity == DiagnosticSeverity::Error)
    }
}

pub fn analyze_script_module(module: &ScriptModule, resolved: &ResolveResult) -> ScriptAnalysis {
    let required_warmup_bars = infer_required_warmup_bars(resolved);
    let mut diagnostics = Vec::new();
    diagnostics.extend(collect_unsupported_construct_diagnostics(module, resolved));
    diagnostics.extend(collect_series_index_diagnostics(module));
    diagnostics.extend(collect_centered_window_diagnostics(module));
    diagnostics.extend(collect_warmup_diagnostics(module, required_warmup_bars));
    diagnostics.extend(check_strategy_has_fetch(module));
    diagnostics.extend(check_strategy_has_emit(module));
    diagnostics.extend(collect_fetch_lookback_warnings(module));
    diagnostics.extend(detect_indirect_recursion(module));
    let data_source_map = build_data_source_map(module);
    diagnostics.extend(check_all_index_bounds(module, &data_source_map));
    diagnostics.extend(check_dead_code_emit(module));
    diagnostics.extend(check_fetch_symbol_whitelist(module));

    ScriptAnalysis {
        required_warmup_bars,
        diagnostics,
    }
}

pub(in crate::analysis_diagnostics) fn contains_emit_in_stmts(stmts: &[Stmt]) -> bool {
    stmts.iter().any(|stmt| match stmt {
        Stmt::EmitIntent { .. } => true,
        Stmt::If {
            then_branch,
            else_if_branches,
            else_branch,
            ..
        } => {
            contains_emit_in_stmts(then_branch)
                || else_if_branches
                    .iter()
                    .any(|(_, b)| contains_emit_in_stmts(b))
                || else_branch
                    .as_ref()
                    .is_some_and(|b| contains_emit_in_stmts(b))
        }
        Stmt::For { body, .. } => contains_emit_in_stmts(body),
        Stmt::While { body, .. } => contains_emit_in_stmts(body),
        Stmt::Match { arms, .. } => arms.iter().any(|arm| match &arm.body {
            MatchArmBody::Statement(stmt) => {
                contains_emit_in_stmts(std::slice::from_ref(stmt.as_ref()))
            }
            MatchArmBody::Expr(_) => false,
        }),
        _ => false,
    })
}
