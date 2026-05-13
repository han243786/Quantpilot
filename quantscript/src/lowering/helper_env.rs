use crate::diagnostics::Diagnostic;
use crate::resolve::{ResolvedCallable, ResolvedExprSemantic, ResolvedFunction};
use crate::script::{CallArg, MatchArmBody, Stmt};
use anyhow::Result;
use qrpc_core::DataSourceConfig;
use std::cell::RefCell;
use std::collections::BTreeMap;

use super::binding_sources::resolve_data_source_ref;
use super::bindings::{resolve_indicator_binding, BindingEnv};

pub(crate) fn hydrate_helper_function_env(
    function: &ResolvedFunction,
    args: &[CallArg],
    env: &BindingEnv,
    data_sources: &[DataSourceConfig],
    seed_indicator_bindings: bool,
) -> Result<Option<BindingEnv>> {
    let mut helper_env = BindingEnv {
        data_by_name: Default::default(),
        indicator_by_name: Default::default(),
        expr_by_name: Default::default(),
        expr_semantics: env.expr_semantics.clone(),
        callables: env.callables.clone(),
        functions: env.functions.clone(),
        diagnostics: RefCell::new(Vec::new()),
    };

    for (index, param_name) in function.param_names.iter().enumerate() {
        let Some(value) = args.get(index).map(|arg| arg.value.clone()) else {
            return Ok(None);
        };
        helper_env
            .expr_by_name
            .insert(param_name.clone(), value.clone());
        if let Some(source) = resolve_data_source_ref(&value, env, data_sources)? {
            helper_env.data_by_name.insert(param_name.clone(), source);
        }
        if seed_indicator_bindings {
            if let Some(binding) = resolve_indicator_binding(&value, env, data_sources)? {
                helper_env
                    .indicator_by_name
                    .insert(param_name.clone(), binding);
            }
        }
    }

    collect_bindings_from_stmts(&function.body, &mut helper_env, data_sources)?;
    Ok(Some(helper_env))
}

pub(crate) fn empty_binding_env(
    functions: BTreeMap<String, ResolvedFunction>,
    expr_semantics: BTreeMap<String, ResolvedExprSemantic>,
    callables: BTreeMap<String, ResolvedCallable>,
) -> BindingEnv {
    BindingEnv {
        data_by_name: BTreeMap::new(),
        indicator_by_name: BTreeMap::new(),
        expr_by_name: BTreeMap::new(),
        expr_semantics,
        callables,
        functions,
        diagnostics: RefCell::new(Vec::new()),
    }
}

pub(crate) fn collect_bindings_from_stmts(
    stmts: &[Stmt],
    env: &mut BindingEnv,
    data_sources: &[DataSourceConfig],
) -> Result<()> {
    for stmt in stmts {
        match stmt {
            Stmt::Let { pattern, value, .. } => {
                env.expr_by_name.insert(pattern.clone(), value.clone());
                if let Some(source) = resolve_data_source_ref(value, env, data_sources)? {
                    let runtime_id = data_runtime_id_from_binding(pattern);
                    env.data_by_name
                        .insert(pattern.clone(), alias_data_source(source, &runtime_id));
                }
                if let Some(binding) = resolve_indicator_binding(value, env, data_sources)? {
                    env.indicator_by_name.insert(pattern.clone(), binding);
                }
            }
            Stmt::If {
                then_branch,
                else_if_branches,
                else_branch,
                ..
            } => {
                collect_bindings_from_stmts(then_branch, env, data_sources)?;
                for (_, branch) in else_if_branches {
                    collect_bindings_from_stmts(branch, env, data_sources)?;
                }
                if let Some(branch) = else_branch {
                    collect_bindings_from_stmts(branch, env, data_sources)?;
                }
            }
            Stmt::For { body, .. } | Stmt::While { body, .. } => {
                collect_bindings_from_stmts(body, env, data_sources)?;
            }
            Stmt::Match { arms, .. } => {
                for arm in arms {
                    if let MatchArmBody::Statement(stmt) = &arm.body {
                        collect_bindings_from_stmts(
                            std::slice::from_ref(stmt.as_ref()),
                            env,
                            data_sources,
                        )?;
                    }
                }
            }
            _ => {}
        }
    }
    Ok(())
}

fn alias_data_source(mut source: DataSourceConfig, runtime_id: &str) -> DataSourceConfig {
    if !runtime_id.is_empty() {
        source.data_id = runtime_id.to_string();
    }
    source
}

fn data_runtime_id_from_binding(pattern: &str) -> String {
    let sanitized = super::shared::sanitize_id(pattern);
    if sanitized.starts_with("data_") && sanitized.ends_with("_series") {
        sanitized
            .strip_suffix("_series")
            .unwrap_or(&sanitized)
            .to_string()
    } else {
        String::new()
    }
}
