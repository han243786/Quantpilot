use anyhow::{bail, Result};
use qrpc_core::{DataSourceConfig, IntentConfig};
use std::collections::BTreeMap;

use crate::script::{Expr, FunctionDecl, MatchArmBody, Stmt};

use super::super::bindings::BindingEnv;
use super::{
    emit_action, intents_from_condition, invert_condition, legacy_intent_from_emit, merge_intent,
    ERR_NO_EXECUTABLE_INTENTS, ERR_UNSUPPORTED_CONDITIONAL_EMIT,
};

pub(super) fn infer_intents(
    strategy: &FunctionDecl,
    bindings: &BindingEnv,
    data_sources: &[DataSourceConfig],
) -> Result<Vec<IntentConfig>> {
    let mut intents = BTreeMap::<String, IntentConfig>::new();
    collect_intents_from_stmts(&strategy.body, None, bindings, data_sources, &mut intents)?;
    if intents.is_empty() {
        bail!(ERR_NO_EXECUTABLE_INTENTS);
    }
    Ok(intents.into_values().collect())
}

fn collect_intents_from_stmts(
    stmts: &[Stmt],
    active_condition: Option<&Expr>,
    bindings: &BindingEnv,
    data_sources: &[DataSourceConfig],
    intents: &mut BTreeMap<String, IntentConfig>,
) -> Result<()> {
    for stmt in stmts {
        match stmt {
            Stmt::EmitIntent { args } => {
                let action = emit_action(args)?;
                let inferred = if let Some(condition) = active_condition {
                    intents_from_condition(condition, &action, bindings)?
                } else {
                    Vec::new()
                };
                if inferred.is_empty() {
                    if active_condition.is_some() {
                        bail!(ERR_UNSUPPORTED_CONDITIONAL_EMIT);
                    }
                    merge_intent(intents, legacy_intent_from_emit(args, data_sources)?);
                } else {
                    for intent in inferred {
                        merge_intent(intents, intent);
                    }
                }
            }
            Stmt::If {
                condition,
                then_branch,
                else_if_branches,
                else_branch,
            } => {
                collect_intents_from_stmts(
                    then_branch,
                    Some(condition),
                    bindings,
                    data_sources,
                    intents,
                )?;
                for (condition, branch) in else_if_branches {
                    collect_intents_from_stmts(
                        branch,
                        Some(condition),
                        bindings,
                        data_sources,
                        intents,
                    )?;
                }
                if let Some(branch) = else_branch {
                    let else_condition = if else_if_branches.is_empty() {
                        invert_condition(condition)
                    } else {
                        None
                    };
                    collect_intents_from_stmts(
                        branch,
                        else_condition.as_ref(),
                        bindings,
                        data_sources,
                        intents,
                    )?;
                }
            }
            Stmt::For { body, .. } | Stmt::While { body, .. } => {
                collect_intents_from_stmts(
                    body,
                    active_condition,
                    bindings,
                    data_sources,
                    intents,
                )?;
            }
            Stmt::Match { arms, .. } => {
                for arm in arms {
                    if let MatchArmBody::Statement(stmt) = &arm.body {
                        collect_intents_from_stmts(
                            std::slice::from_ref(stmt.as_ref()),
                            active_condition,
                            bindings,
                            data_sources,
                            intents,
                        )?;
                    }
                }
            }
            _ => {}
        }
    }
    Ok(())
}
