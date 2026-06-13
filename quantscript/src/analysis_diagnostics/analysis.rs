use super::lookahead_window_gate::{
    collect_centered_window_diagnostics, collect_series_index_diagnostics,
};
use super::unsupported_construct_gate::collect_unsupported_construct_diagnostics;
use super::warmup_fetch_gate::{
    arg_number_named, collect_warmup_diagnostics, fetch_lookback, infer_required_warmup_bars,
};

use crate::diagnostics::{Diagnostic, DiagnosticSeverity, Span};
use crate::resolve::ResolveResult;
use crate::script::{Expr, FunctionDecl, Item, MatchArmBody, ScriptModule, Stmt};
use std::collections::{BTreeMap, BTreeSet};

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

// B1-1: 空 strategy 提前诊断
fn check_strategy_has_fetch(module: &ScriptModule) -> Vec<Diagnostic> {
    for item in &module.items {
        if let Item::Function(function) = item {
            if function.name == "strategy" {
                if !contains_fetch_like_call_in_stmts(&function.body) {
                    return vec![Diagnostic::error(
                        "QS0610",
                        "策略函数必须包含至少一个 fetch() 调用来获取市场数据",
                        Some(Span::function("strategy")),
                    )];
                }
                return vec![];
            }
        }
    }
    vec![]
}

fn contains_fetch_like_call_in_stmts(stmts: &[Stmt]) -> bool {
    stmts.iter().any(|stmt| match stmt {
        Stmt::Let { value, .. } | Stmt::Expr(value) | Stmt::Return(Some(value)) => {
            contains_fetch_like_call_in_expr(value)
        }
        Stmt::Return(None) => false,
        Stmt::EmitIntent { args } => args
            .iter()
            .any(|arg| contains_fetch_like_call_in_expr(&arg.value)),
        Stmt::If {
            condition,
            then_branch,
            else_if_branches,
            else_branch,
        } => {
            contains_fetch_like_call_in_expr(condition)
                || contains_fetch_like_call_in_stmts(then_branch)
                || else_if_branches.iter().any(|(c, b)| {
                    contains_fetch_like_call_in_expr(c) || contains_fetch_like_call_in_stmts(b)
                })
                || else_branch
                    .as_ref()
                    .is_some_and(|b| contains_fetch_like_call_in_stmts(b))
        }
        Stmt::For { iterable, body, .. } => {
            contains_fetch_like_call_in_expr(iterable) || contains_fetch_like_call_in_stmts(body)
        }
        Stmt::While { condition, body } => {
            contains_fetch_like_call_in_expr(condition) || contains_fetch_like_call_in_stmts(body)
        }
        Stmt::Match { expr, arms } => {
            contains_fetch_like_call_in_expr(expr)
                || arms.iter().any(|arm| match &arm.body {
                    MatchArmBody::Statement(stmt) => {
                        contains_fetch_like_call_in_stmts(std::slice::from_ref(stmt.as_ref()))
                    }
                    MatchArmBody::Expr(expr) => contains_fetch_like_call_in_expr(expr),
                })
        }
    })
}

fn contains_fetch_like_call_in_expr(expr: &Expr) -> bool {
    match expr {
        Expr::Call { callee, args } => {
            if matches!(
                callee.as_ref(),
                Expr::Identifier(name) if name == "fetch" || name == "get_data"
            ) {
                return true;
            }
            contains_fetch_like_call_in_expr(callee)
                || args
                    .iter()
                    .any(|arg| contains_fetch_like_call_in_expr(&arg.value))
        }
        Expr::List(items) => items.iter().any(contains_fetch_like_call_in_expr),
        Expr::Member { object, .. }
        | Expr::Await(object)
        | Expr::Try(object)
        | Expr::Unary { expr: object, .. } => contains_fetch_like_call_in_expr(object),
        Expr::Index { object, index } => {
            contains_fetch_like_call_in_expr(object) || contains_fetch_like_call_in_expr(index)
        }
        Expr::Slice { object, start, end } => {
            contains_fetch_like_call_in_expr(object)
                || start
                    .as_ref()
                    .is_some_and(|s| contains_fetch_like_call_in_expr(s))
                || end
                    .as_ref()
                    .is_some_and(|e| contains_fetch_like_call_in_expr(e))
        }
        Expr::Binary { left, right, .. }
        | Expr::Range {
            start: left,
            end: right,
        } => contains_fetch_like_call_in_expr(left) || contains_fetch_like_call_in_expr(right),
        Expr::Raw(_) | Expr::Identifier(_) | Expr::Number(_) | Expr::String(_) | Expr::Bool(_) => {
            false
        }
    }
}

// B1-5: 无 emit 诊断
fn check_strategy_has_emit(module: &ScriptModule) -> Vec<Diagnostic> {
    for item in &module.items {
        if let Item::Function(function) = item {
            if function.name == "strategy" {
                if !contains_emit_in_stmts(&function.body) {
                    return vec![Diagnostic::error(
                        "QS0611",
                        "策略函数必须包含至少一个 emit Intent() 调用来输出交易信号",
                        Some(Span::function("strategy")),
                    )];
                }
                return vec![];
            }
        }
    }
    vec![]
}

fn contains_emit_in_stmts(stmts: &[Stmt]) -> bool {
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

// B1-8: lookback=0 警告
fn collect_fetch_lookback_warnings(module: &ScriptModule) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    for item in &module.items {
        if let Item::Function(function) = item {
            collect_fetch_lookback_warnings_from_stmts(&function.body, &mut diagnostics);
        }
    }
    diagnostics
}

fn collect_fetch_lookback_warnings_from_stmts(stmts: &[Stmt], diagnostics: &mut Vec<Diagnostic>) {
    for stmt in stmts {
        match stmt {
            Stmt::Let { value, .. } | Stmt::Expr(value) | Stmt::Return(Some(value)) => {
                collect_fetch_lookback_warnings_from_expr(value, diagnostics);
            }
            Stmt::Return(None) => {}
            Stmt::EmitIntent { args } => {
                for arg in args {
                    collect_fetch_lookback_warnings_from_expr(&arg.value, diagnostics);
                }
            }
            Stmt::If {
                condition,
                then_branch,
                else_if_branches,
                else_branch,
            } => {
                collect_fetch_lookback_warnings_from_expr(condition, diagnostics);
                collect_fetch_lookback_warnings_from_stmts(then_branch, diagnostics);
                for (cond, branch) in else_if_branches {
                    collect_fetch_lookback_warnings_from_expr(cond, diagnostics);
                    collect_fetch_lookback_warnings_from_stmts(branch, diagnostics);
                }
                if let Some(branch) = else_branch {
                    collect_fetch_lookback_warnings_from_stmts(branch, diagnostics);
                }
            }
            Stmt::For { iterable, body, .. } => {
                collect_fetch_lookback_warnings_from_expr(iterable, diagnostics);
                collect_fetch_lookback_warnings_from_stmts(body, diagnostics);
            }
            Stmt::While { condition, body } => {
                collect_fetch_lookback_warnings_from_expr(condition, diagnostics);
                collect_fetch_lookback_warnings_from_stmts(body, diagnostics);
            }
            Stmt::Match { expr, arms } => {
                collect_fetch_lookback_warnings_from_expr(expr, diagnostics);
                for arm in arms {
                    match &arm.body {
                        MatchArmBody::Statement(stmt) => {
                            collect_fetch_lookback_warnings_from_stmts(
                                std::slice::from_ref(stmt.as_ref()),
                                diagnostics,
                            );
                        }
                        MatchArmBody::Expr(expr) => {
                            collect_fetch_lookback_warnings_from_expr(expr, diagnostics);
                        }
                    }
                }
            }
        }
    }
}

fn collect_fetch_lookback_warnings_from_expr(expr: &Expr, diagnostics: &mut Vec<Diagnostic>) {
    if let Expr::Call { callee, args } = expr {
        let callee_name = match callee.as_ref() {
            Expr::Identifier(name) => name.as_str(),
            _ => return,
        };
        if matches!(callee_name, "fetch" | "get_data") {
            if let Some(value) = arg_number_named(args, "lookback") {
                if value < 1.0 {
                    diagnostics.push(Diagnostic::warning(
                        "QS0503",
                        format!("fetch lookback={} 小于 1, 已自动设为 1", value),
                        Some(Span::expr("fetch.lookback")),
                    ));
                }
            }
        }
    }

    match expr {
        Expr::List(items) => {
            for item in items {
                collect_fetch_lookback_warnings_from_expr(item, diagnostics);
            }
        }
        Expr::Call { callee, args } => {
            collect_fetch_lookback_warnings_from_expr(callee, diagnostics);
            for arg in args {
                collect_fetch_lookback_warnings_from_expr(&arg.value, diagnostics);
            }
        }
        Expr::Member { object, .. }
        | Expr::Await(object)
        | Expr::Try(object)
        | Expr::Unary { expr: object, .. } => {
            collect_fetch_lookback_warnings_from_expr(object, diagnostics);
        }
        Expr::Index { object, index } => {
            collect_fetch_lookback_warnings_from_expr(object, diagnostics);
            collect_fetch_lookback_warnings_from_expr(index, diagnostics);
        }
        Expr::Slice { object, start, end } => {
            collect_fetch_lookback_warnings_from_expr(object, diagnostics);
            if let Some(start) = start {
                collect_fetch_lookback_warnings_from_expr(start, diagnostics);
            }
            if let Some(end) = end {
                collect_fetch_lookback_warnings_from_expr(end, diagnostics);
            }
        }
        Expr::Binary { left, right, .. }
        | Expr::Range {
            start: left,
            end: right,
        } => {
            collect_fetch_lookback_warnings_from_expr(left, diagnostics);
            collect_fetch_lookback_warnings_from_expr(right, diagnostics);
        }
        Expr::Raw(_) | Expr::Identifier(_) | Expr::Number(_) | Expr::String(_) | Expr::Bool(_) => {}
    }
}

// B1-11: 间接递归检测
fn detect_indirect_recursion(module: &ScriptModule) -> Vec<Diagnostic> {
    let functions: Vec<&FunctionDecl> = module
        .items
        .iter()
        .filter_map(|item| match item {
            Item::Function(f) => Some(f),
            _ => None,
        })
        .collect();

    if functions.is_empty() {
        return Vec::new();
    }

    let mut call_graph: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for f in &functions {
        let mut callees = BTreeSet::new();
        collect_callee_names_from_stmts(&f.body, &mut callees);
        call_graph.insert(f.name.clone(), callees);
    }

    let mut diagnostics = Vec::new();
    for start in call_graph.keys() {
        let mut visited = BTreeSet::new();
        let mut stack = vec![start.clone()];
        while let Some(current) = stack.pop() {
            if !visited.insert(current.clone()) {
                continue;
            }
            if let Some(callees) = call_graph.get(&current) {
                for callee in callees {
                    if callee == start && start != &current {
                        diagnostics.push(Diagnostic::error(
                            "QS0605",
                            format!("检测到递归调用循环: {} 间接调用自身", start),
                            Some(Span::function(start.clone())),
                        ));
                    }
                    stack.push(callee.clone());
                }
            }
        }
    }
    diagnostics
}

fn collect_callee_names_from_stmts(stmts: &[Stmt], out: &mut BTreeSet<String>) {
    for stmt in stmts {
        match stmt {
            Stmt::Let { value, .. } | Stmt::Expr(value) | Stmt::Return(Some(value)) => {
                collect_callee_names_from_expr(value, out);
            }
            Stmt::Return(None) => {}
            Stmt::EmitIntent { args } => {
                for arg in args {
                    collect_callee_names_from_expr(&arg.value, out);
                }
            }
            Stmt::If {
                condition,
                then_branch,
                else_if_branches,
                else_branch,
            } => {
                collect_callee_names_from_expr(condition, out);
                collect_callee_names_from_stmts(then_branch, out);
                for (cond, branch) in else_if_branches {
                    collect_callee_names_from_expr(cond, out);
                    collect_callee_names_from_stmts(branch, out);
                }
                if let Some(branch) = else_branch {
                    collect_callee_names_from_stmts(branch, out);
                }
            }
            Stmt::For { iterable, body, .. } => {
                collect_callee_names_from_expr(iterable, out);
                collect_callee_names_from_stmts(body, out);
            }
            Stmt::While { condition, body } => {
                collect_callee_names_from_expr(condition, out);
                collect_callee_names_from_stmts(body, out);
            }
            Stmt::Match { expr, arms } => {
                collect_callee_names_from_expr(expr, out);
                for arm in arms {
                    match &arm.body {
                        MatchArmBody::Statement(stmt) => collect_callee_names_from_stmts(
                            std::slice::from_ref(stmt.as_ref()),
                            out,
                        ),
                        MatchArmBody::Expr(expr) => collect_callee_names_from_expr(expr, out),
                    }
                }
            }
        }
    }
}

fn collect_callee_names_from_expr(expr: &Expr, out: &mut BTreeSet<String>) {
    match expr {
        Expr::Call { callee, args } => {
            if let Expr::Identifier(name) = callee.as_ref() {
                out.insert(name.clone());
            }
            collect_callee_names_from_expr(callee, out);
            for arg in args {
                collect_callee_names_from_expr(&arg.value, out);
            }
        }
        Expr::List(items) => {
            for item in items {
                collect_callee_names_from_expr(item, out);
            }
        }
        Expr::Member { object, .. }
        | Expr::Await(object)
        | Expr::Try(object)
        | Expr::Unary { expr: object, .. } => {
            collect_callee_names_from_expr(object, out);
        }
        Expr::Index { object, index } => {
            collect_callee_names_from_expr(object, out);
            collect_callee_names_from_expr(index, out);
        }
        Expr::Slice { object, start, end } => {
            collect_callee_names_from_expr(object, out);
            if let Some(start) = start {
                collect_callee_names_from_expr(start, out);
            }
            if let Some(end) = end {
                collect_callee_names_from_expr(end, out);
            }
        }
        Expr::Binary { left, right, .. }
        | Expr::Range {
            start: left,
            end: right,
        } => {
            collect_callee_names_from_expr(left, out);
            collect_callee_names_from_expr(right, out);
        }
        Expr::Raw(_) | Expr::Identifier(_) | Expr::Number(_) | Expr::String(_) | Expr::Bool(_) => {}
    }
}

// B1-12: 数组越界索引警告
fn build_data_source_map(module: &ScriptModule) -> BTreeMap<String, usize> {
    let mut map = BTreeMap::new();
    for item in &module.items {
        if let Item::Function(function) = item {
            for stmt in &function.body {
                if let Stmt::Let { pattern, value, .. } = stmt {
                    if let Some(lookback) = fetch_lookback(value) {
                        map.insert(pattern.clone(), lookback);
                    }
                }
            }
        }
    }
    map
}

fn check_all_index_bounds(
    module: &ScriptModule,
    data_sources: &BTreeMap<String, usize>,
) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    for item in &module.items {
        if let Item::Function(function) = item {
            check_index_bounds_from_stmts(&function.body, data_sources, &mut diagnostics);
        }
    }
    diagnostics
}

fn check_index_bounds_from_stmts(
    stmts: &[Stmt],
    data_sources: &BTreeMap<String, usize>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    for stmt in stmts {
        match stmt {
            Stmt::Let { value, .. } | Stmt::Expr(value) | Stmt::Return(Some(value)) => {
                check_index_bounds_from_expr(value, data_sources, diagnostics);
            }
            Stmt::Return(None) => {}
            Stmt::EmitIntent { args } => {
                for arg in args {
                    check_index_bounds_from_expr(&arg.value, data_sources, diagnostics);
                }
            }
            Stmt::If {
                condition,
                then_branch,
                else_if_branches,
                else_branch,
            } => {
                check_index_bounds_from_expr(condition, data_sources, diagnostics);
                check_index_bounds_from_stmts(then_branch, data_sources, diagnostics);
                for (cond, branch) in else_if_branches {
                    check_index_bounds_from_expr(cond, data_sources, diagnostics);
                    check_index_bounds_from_stmts(branch, data_sources, diagnostics);
                }
                if let Some(branch) = else_branch {
                    check_index_bounds_from_stmts(branch, data_sources, diagnostics);
                }
            }
            Stmt::For { iterable, body, .. } => {
                check_index_bounds_from_expr(iterable, data_sources, diagnostics);
                check_index_bounds_from_stmts(body, data_sources, diagnostics);
            }
            Stmt::While { condition, body } => {
                check_index_bounds_from_expr(condition, data_sources, diagnostics);
                check_index_bounds_from_stmts(body, data_sources, diagnostics);
            }
            Stmt::Match { expr, arms } => {
                check_index_bounds_from_expr(expr, data_sources, diagnostics);
                for arm in arms {
                    match &arm.body {
                        MatchArmBody::Statement(stmt) => check_index_bounds_from_stmts(
                            std::slice::from_ref(stmt.as_ref()),
                            data_sources,
                            diagnostics,
                        ),
                        MatchArmBody::Expr(expr) => {
                            check_index_bounds_from_expr(expr, data_sources, diagnostics);
                        }
                    }
                }
            }
        }
    }
}

fn check_index_bounds_from_expr(
    expr: &Expr,
    data_sources: &BTreeMap<String, usize>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    if let Expr::Index { object, index } = expr {
        if let (Expr::Identifier(var), Expr::Number(idx)) = (object.as_ref(), index.as_ref()) {
            if let Some(&lookback) = data_sources.get(var) {
                if *idx as usize >= lookback {
                    diagnostics.push(Diagnostic::warning(
                        "QS0404",
                        format!(
                            "索引 {} 可能超出变量 '{}' 的回看窗口 {}",
                            idx, var, lookback
                        ),
                        Some(Span::binding(var.clone())),
                    ));
                }
            }
        }
    }

    match expr {
        Expr::List(items) => {
            for item in items {
                check_index_bounds_from_expr(item, data_sources, diagnostics);
            }
        }
        Expr::Call { callee, args } => {
            check_index_bounds_from_expr(callee, data_sources, diagnostics);
            for arg in args {
                check_index_bounds_from_expr(&arg.value, data_sources, diagnostics);
            }
        }
        Expr::Member { object, .. }
        | Expr::Await(object)
        | Expr::Try(object)
        | Expr::Unary { expr: object, .. } => {
            check_index_bounds_from_expr(object, data_sources, diagnostics);
        }
        Expr::Index { object, index } => {
            check_index_bounds_from_expr(object, data_sources, diagnostics);
            check_index_bounds_from_expr(index, data_sources, diagnostics);
        }
        Expr::Slice { object, start, end } => {
            check_index_bounds_from_expr(object, data_sources, diagnostics);
            if let Some(start) = start {
                check_index_bounds_from_expr(start, data_sources, diagnostics);
            }
            if let Some(end) = end {
                check_index_bounds_from_expr(end, data_sources, diagnostics);
            }
        }
        Expr::Binary { left, right, .. }
        | Expr::Range {
            start: left,
            end: right,
        } => {
            check_index_bounds_from_expr(left, data_sources, diagnostics);
            check_index_bounds_from_expr(right, data_sources, diagnostics);
        }
        Expr::Raw(_) | Expr::Identifier(_) | Expr::Number(_) | Expr::String(_) | Expr::Bool(_) => {}
    }
}

// B1-14: dead code emit 警告
fn check_dead_code_emit(module: &ScriptModule) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    for item in &module.items {
        if let Item::Function(function) = item {
            check_dead_code_emit_from_stmts(&function.body, &mut diagnostics);
        }
    }
    diagnostics
}

fn check_dead_code_emit_from_stmts(stmts: &[Stmt], diagnostics: &mut Vec<Diagnostic>) {
    for stmt in stmts {
        match stmt {
            Stmt::If {
                condition,
                then_branch,
                else_if_branches,
                else_branch,
            } => {
                if is_constant_false(condition) && contains_emit_in_stmts(then_branch) {
                    diagnostics.push(Diagnostic::warning(
                        "QS0612",
                        "条件为 false 的 if 分支中的 emit 语句永远不会执行",
                        Some(Span::expr("emit")),
                    ));
                }
                check_dead_code_emit_from_stmts(then_branch, diagnostics);
                for (_, branch) in else_if_branches {
                    check_dead_code_emit_from_stmts(branch, diagnostics);
                }
                if let Some(branch) = else_branch {
                    check_dead_code_emit_from_stmts(branch, diagnostics);
                }
            }
            Stmt::For { body, .. } => {
                check_dead_code_emit_from_stmts(body, diagnostics);
            }
            Stmt::While { body, .. } => {
                check_dead_code_emit_from_stmts(body, diagnostics);
            }
            Stmt::Match { expr, arms } => {
                for arm in arms {
                    match &arm.body {
                        MatchArmBody::Statement(stmt) => check_dead_code_emit_from_stmts(
                            std::slice::from_ref(stmt.as_ref()),
                            diagnostics,
                        ),
                        MatchArmBody::Expr(_) => {}
                    }
                }
                let _ = expr;
            }
            _ => {}
        }
    }
}

fn is_constant_false(expr: &Expr) -> bool {
    matches!(expr, Expr::Bool(false))
}

// B1-15: 交易对白名单校验
const KNOWN_SYMBOLS: &[&str] = &[
    "BTCUSDT", "ETHUSDT", "BNBUSDT", "SOLUSDT", "ADAUSDT", "DOGEUSDT", "XRPUSDT",
];

fn check_fetch_symbol_whitelist(module: &ScriptModule) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    for item in &module.items {
        if let Item::Function(function) = item {
            check_fetch_symbol_whitelist_from_stmts(&function.body, &mut diagnostics);
        }
    }
    diagnostics
}

fn check_fetch_symbol_whitelist_from_stmts(stmts: &[Stmt], diagnostics: &mut Vec<Diagnostic>) {
    for stmt in stmts {
        match stmt {
            Stmt::Let { value, .. } | Stmt::Expr(value) | Stmt::Return(Some(value)) => {
                check_fetch_symbol_whitelist_from_expr(value, diagnostics);
            }
            Stmt::Return(None) => {}
            Stmt::EmitIntent { args } => {
                for arg in args {
                    check_fetch_symbol_whitelist_from_expr(&arg.value, diagnostics);
                }
            }
            Stmt::If {
                condition,
                then_branch,
                else_if_branches,
                else_branch,
            } => {
                check_fetch_symbol_whitelist_from_expr(condition, diagnostics);
                check_fetch_symbol_whitelist_from_stmts(then_branch, diagnostics);
                for (cond, branch) in else_if_branches {
                    check_fetch_symbol_whitelist_from_expr(cond, diagnostics);
                    check_fetch_symbol_whitelist_from_stmts(branch, diagnostics);
                }
                if let Some(branch) = else_branch {
                    check_fetch_symbol_whitelist_from_stmts(branch, diagnostics);
                }
            }
            Stmt::For { iterable, body, .. } => {
                check_fetch_symbol_whitelist_from_expr(iterable, diagnostics);
                check_fetch_symbol_whitelist_from_stmts(body, diagnostics);
            }
            Stmt::While { condition, body } => {
                check_fetch_symbol_whitelist_from_expr(condition, diagnostics);
                check_fetch_symbol_whitelist_from_stmts(body, diagnostics);
            }
            Stmt::Match { expr, arms } => {
                check_fetch_symbol_whitelist_from_expr(expr, diagnostics);
                for arm in arms {
                    match &arm.body {
                        MatchArmBody::Statement(stmt) => {
                            check_fetch_symbol_whitelist_from_stmts(
                                std::slice::from_ref(stmt.as_ref()),
                                diagnostics,
                            );
                        }
                        MatchArmBody::Expr(expr) => {
                            check_fetch_symbol_whitelist_from_expr(expr, diagnostics);
                        }
                    }
                }
            }
        }
    }
}

fn check_fetch_symbol_whitelist_from_expr(expr: &Expr, diagnostics: &mut Vec<Diagnostic>) {
    if let Expr::Call { callee, args } = expr {
        let callee_name = match callee.as_ref() {
            Expr::Identifier(name) => name.as_str(),
            _ => return,
        };
        if matches!(callee_name, "fetch" | "get_data") {
            if let Some(symbol_str) = args.first().and_then(|arg| match &arg.value {
                Expr::String(s) => Some(s.as_str()),
                _ => None,
            }) {
                if !KNOWN_SYMBOLS.contains(&symbol_str.to_uppercase().as_str()) {
                    diagnostics.push(Diagnostic::warning(
                        "QS0505",
                        format!(
                            "未知交易对 '{}' 不在已验证列表中。已验证: BTCUSDT, ETHUSDT, SOLUSDT",
                            symbol_str
                        ),
                        Some(Span::expr("instrument")),
                    ));
                }
            }
        }
    }

    match expr {
        Expr::List(items) => {
            for item in items {
                check_fetch_symbol_whitelist_from_expr(item, diagnostics);
            }
        }
        Expr::Call { callee, args } => {
            check_fetch_symbol_whitelist_from_expr(callee, diagnostics);
            for arg in args {
                check_fetch_symbol_whitelist_from_expr(&arg.value, diagnostics);
            }
        }
        Expr::Member { object, .. }
        | Expr::Await(object)
        | Expr::Try(object)
        | Expr::Unary { expr: object, .. } => {
            check_fetch_symbol_whitelist_from_expr(object, diagnostics);
        }
        Expr::Index { object, index } => {
            check_fetch_symbol_whitelist_from_expr(object, diagnostics);
            check_fetch_symbol_whitelist_from_expr(index, diagnostics);
        }
        Expr::Slice { object, start, end } => {
            check_fetch_symbol_whitelist_from_expr(object, diagnostics);
            if let Some(start) = start {
                check_fetch_symbol_whitelist_from_expr(start, diagnostics);
            }
            if let Some(end) = end {
                check_fetch_symbol_whitelist_from_expr(end, diagnostics);
            }
        }
        Expr::Binary { left, right, .. }
        | Expr::Range {
            start: left,
            end: right,
        } => {
            check_fetch_symbol_whitelist_from_expr(left, diagnostics);
            check_fetch_symbol_whitelist_from_expr(right, diagnostics);
        }
        Expr::Raw(_) | Expr::Identifier(_) | Expr::Number(_) | Expr::String(_) | Expr::Bool(_) => {}
    }
}
