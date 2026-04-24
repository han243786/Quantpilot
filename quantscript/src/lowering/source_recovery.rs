use crate::resolve::{
    classify_builtin_math_name, classify_member_mutation_name, ResolvedBuiltinMathKind,
    ResolvedMemberMutationKind,
};
use crate::script::{BinaryOp, CallArg, Expr, MatchArmBody, Stmt};
use anyhow::Result;
use qrpc_core::DataSourceConfig;

use super::binding_sources::{
    arg_data_source_optional, comparison_parts, parse_call, resolve_data_source_ref,
    resolved_change_kind,
};
use super::bindings::BindingEnv;
use super::helper_env::hydrate_helper_function_env;
use super::semantic::{is_number_literal, resolve_expr_alias, ChangeKind};
use super::shared::{arg_expr_required, expr_number, ArgSelector};

#[derive(Debug, Clone)]
struct PushSite {
    expr: Expr,
    guards: Vec<Expr>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DeltaOrientation {
    Forward,
    Backward,
}

pub(crate) fn gain_loss_source_binding(
    expr: &Expr,
    env: &BindingEnv,
    data_sources: &[DataSourceConfig],
) -> Result<Option<(DataSourceConfig, ChangeKind)>> {
    let expr = resolve_expr_alias(expr, env).unwrap_or(expr);
    let Some((fn_name, args)) = parse_call(expr) else {
        return Ok(None);
    };
    let Some(kind) = resolved_change_kind(env, &fn_name) else {
        if let Some(binding) =
            helper_function_change_binding(&fn_name, args, ChangeKind::Gain, env, data_sources)?
        {
            return Ok(Some(binding));
        }
        if let Some(binding) =
            helper_function_change_binding(&fn_name, args, ChangeKind::Loss, env, data_sources)?
        {
            return Ok(Some(binding));
        }
        return Ok(None);
    };
    if args.is_empty() {
        return Ok(None);
    }
    let Some(source) =
        arg_data_source_optional(args, ArgSelector::Positional(0), env, data_sources)?
    else {
        return Ok(None);
    };
    Ok(Some((source, kind)))
}

fn helper_function_change_binding(
    name: &str,
    args: &[CallArg],
    kind: ChangeKind,
    env: &BindingEnv,
    data_sources: &[DataSourceConfig],
) -> Result<Option<(DataSourceConfig, ChangeKind)>> {
    let Some(function) = env.functions.get(name) else {
        return Ok(None);
    };
    let Some(helper_env) = hydrate_helper_function_env(function, args, env, data_sources, false)?
    else {
        return Ok(None);
    };
    let Some(return_expr) = function.return_expr.as_ref() else {
        return Ok(None);
    };

    if let Some(binding) =
        direct_change_source_binding(return_expr, kind, &helper_env, data_sources)?
    {
        return Ok(Some(binding));
    }

    let Some(target_list) = function.returned_list_target.as_ref() else {
        return Ok(None);
    };

    let mut push_sites = Vec::new();
    collect_push_sites(
        &function.body,
        target_list,
        &mut Vec::new(),
        &mut push_sites,
    )?;
    infer_change_binding_from_push_sites(&push_sites, kind, &helper_env, data_sources)
}

fn direct_change_source_binding(
    expr: &Expr,
    kind: ChangeKind,
    env: &BindingEnv,
    data_sources: &[DataSourceConfig],
) -> Result<Option<(DataSourceConfig, ChangeKind)>> {
    let expr = resolve_expr_alias(expr, env).unwrap_or(expr);
    let Some((fn_name, args)) = parse_call(expr) else {
        return Ok(None);
    };
    let Some(direct_kind) = resolved_change_kind(env, &fn_name) else {
        return Ok(None);
    };
    if direct_kind != kind || args.is_empty() {
        return Ok(None);
    }
    let Some(source) =
        arg_data_source_optional(args, ArgSelector::Positional(0), env, data_sources)?
    else {
        return Ok(None);
    };
    Ok(Some((source, kind)))
}

fn collect_push_sites(
    stmts: &[Stmt],
    target_list: &str,
    guards: &mut Vec<Expr>,
    push_sites: &mut Vec<PushSite>,
) -> Result<()> {
    for stmt in stmts {
        match stmt {
            Stmt::Expr(expr) => {
                if let Some(push_expr) = push_call_expr(expr, target_list) {
                    push_sites.push(PushSite {
                        expr: push_expr.clone(),
                        guards: guards.clone(),
                    });
                }
            }
            Stmt::If {
                condition,
                then_branch,
                else_if_branches,
                else_branch,
            } => {
                guards.push(condition.clone());
                collect_push_sites(then_branch, target_list, guards, push_sites)?;
                guards.pop();

                let mut prior_negations =
                    vec![invert_condition(condition).unwrap_or_else(|| Expr::Unary {
                        op: crate::script::UnaryOp::Not,
                        expr: Box::new(condition.clone()),
                    })];
                for (else_if_condition, branch) in else_if_branches {
                    for negated in &prior_negations {
                        guards.push(negated.clone());
                    }
                    guards.push(else_if_condition.clone());
                    collect_push_sites(branch, target_list, guards, push_sites)?;
                    for _ in 0..(prior_negations.len() + 1) {
                        guards.pop();
                    }
                    prior_negations.push(invert_condition(else_if_condition).unwrap_or_else(
                        || Expr::Unary {
                            op: crate::script::UnaryOp::Not,
                            expr: Box::new(else_if_condition.clone()),
                        },
                    ));
                }

                if let Some(branch) = else_branch {
                    for negated in &prior_negations {
                        guards.push(negated.clone());
                    }
                    collect_push_sites(branch, target_list, guards, push_sites)?;
                    for _ in 0..prior_negations.len() {
                        guards.pop();
                    }
                }
            }
            Stmt::For { body, .. } | Stmt::While { body, .. } => {
                collect_push_sites(body, target_list, guards, push_sites)?;
            }
            Stmt::Match { arms, .. } => {
                for arm in arms {
                    match &arm.body {
                        MatchArmBody::Statement(stmt) => {
                            collect_push_sites(
                                std::slice::from_ref(stmt.as_ref()),
                                target_list,
                                guards,
                                push_sites,
                            )?;
                        }
                        MatchArmBody::Expr(expr) => {
                            if let Some(push_expr) = push_call_expr(expr, target_list) {
                                push_sites.push(PushSite {
                                    expr: push_expr.clone(),
                                    guards: guards.clone(),
                                });
                            }
                        }
                    }
                }
            }
            Stmt::Let { .. } | Stmt::Return(_) | Stmt::EmitIntent { .. } => {}
        }
    }
    Ok(())
}

fn push_call_expr<'a>(expr: &'a Expr, target_list: &str) -> Option<&'a Expr> {
    let Expr::Call { callee, args } = expr else {
        return None;
    };
    if args.len() != 1 {
        return None;
    }
    let Expr::Member { object, field } = callee.as_ref() else {
        return None;
    };
    if !matches!(
        classify_member_mutation_name(field),
        Some(ResolvedMemberMutationKind::Push)
    ) {
        return None;
    }
    match object.as_ref() {
        Expr::Identifier(name) if name == target_list => Some(&args[0].value),
        Expr::List(items) if items.is_empty() => Some(&args[0].value),
        _ => None,
    }
}

fn infer_change_binding_from_push_sites(
    push_sites: &[PushSite],
    kind: ChangeKind,
    env: &BindingEnv,
    data_sources: &[DataSourceConfig],
) -> Result<Option<(DataSourceConfig, ChangeKind)>> {
    let mut source: Option<DataSourceConfig> = None;
    let mut saw_signal = false;

    for push in push_sites {
        let Some(next_source) =
            recognized_change_push(&push.expr, &push.guards, kind, env, data_sources)?
        else {
            continue;
        };
        saw_signal = true;
        match &source {
            Some(existing) if existing.data_id != next_source.data_id => return Ok(None),
            None => source = Some(next_source),
            _ => {}
        }
    }

    if saw_signal {
        Ok(source.map(|source| (source, kind)))
    } else {
        Ok(None)
    }
}

fn recognized_change_push(
    expr: &Expr,
    guards: &[Expr],
    kind: ChangeKind,
    env: &BindingEnv,
    data_sources: &[DataSourceConfig],
) -> Result<Option<DataSourceConfig>> {
    let expr = resolve_expr_alias(expr, env).unwrap_or(expr);
    if is_number_literal(expr, 0.0) {
        return Ok(None);
    }

    if let Some(source) = clamped_change_source(expr, kind, env, data_sources)? {
        return Ok(Some(source));
    }

    if let Some(source) = guarded_abs_change_source(expr, guards, kind, env, data_sources)? {
        return Ok(Some(source));
    }

    if let Some(source) = guarded_change_source(expr, guards, kind, env, data_sources)? {
        return Ok(Some(source));
    }

    Ok(None)
}

fn guarded_abs_change_source(
    expr: &Expr,
    guards: &[Expr],
    kind: ChangeKind,
    env: &BindingEnv,
    data_sources: &[DataSourceConfig],
) -> Result<Option<DataSourceConfig>> {
    let Some((fn_name, args)) = parse_call(expr) else {
        return Ok(None);
    };
    if !matches!(
        classify_builtin_math_name(&fn_name),
        Some(ResolvedBuiltinMathKind::Abs)
    ) || args.len() != 1
    {
        return Ok(None);
    }
    let Some((source, _)) = delta_source_expr(
        arg_expr_required(args, ArgSelector::Positional(0))?,
        env,
        data_sources,
    )?
    else {
        return Ok(None);
    };
    let expected_positive = matches!(kind, ChangeKind::Gain);
    if guards_imply_change_sign(guards, &source, expected_positive, env, data_sources)? {
        Ok(Some(source))
    } else {
        Ok(None)
    }
}

fn clamped_change_source(
    expr: &Expr,
    kind: ChangeKind,
    env: &BindingEnv,
    data_sources: &[DataSourceConfig],
) -> Result<Option<DataSourceConfig>> {
    let Some((fn_name, args)) = parse_call(expr) else {
        return Ok(None);
    };
    match (fn_name.as_str(), args.len()) {
        ("max", 2) => {
            let left = arg_expr_required(args, ArgSelector::Positional(0))?;
            let right = arg_expr_required(args, ArgSelector::Positional(1))?;
            if is_number_literal(left, 0.0) {
                return oriented_change_source(right, kind, env, data_sources);
            }
            if is_number_literal(right, 0.0) {
                return oriented_change_source(left, kind, env, data_sources);
            }
            Ok(None)
        }
        ("abs", 1) if kind == ChangeKind::Loss => {
            let Some((source, orientation)) = delta_source_expr(
                arg_expr_required(args, ArgSelector::Positional(0))?,
                env,
                data_sources,
            )?
            else {
                return Ok(None);
            };
            if orientation == DeltaOrientation::Backward {
                Ok(Some(source))
            } else {
                Ok(None)
            }
        }
        _ => Ok(None),
    }
}

fn guarded_change_source(
    expr: &Expr,
    guards: &[Expr],
    kind: ChangeKind,
    env: &BindingEnv,
    data_sources: &[DataSourceConfig],
) -> Result<Option<DataSourceConfig>> {
    let Some((source, orientation)) = delta_source_expr(expr, env, data_sources)? else {
        return Ok(None);
    };
    let required = match kind {
        ChangeKind::Gain => DeltaOrientation::Forward,
        ChangeKind::Loss => DeltaOrientation::Backward,
    };
    if orientation != required {
        return Ok(None);
    }

    let expected_positive = matches!(kind, ChangeKind::Gain);
    if guards_imply_change_sign(guards, &source, expected_positive, env, data_sources)? {
        Ok(Some(source))
    } else {
        Ok(None)
    }
}

fn oriented_change_source(
    expr: &Expr,
    kind: ChangeKind,
    env: &BindingEnv,
    data_sources: &[DataSourceConfig],
) -> Result<Option<DataSourceConfig>> {
    let Some((source, orientation)) = delta_source_expr(expr, env, data_sources)? else {
        return Ok(None);
    };
    let expected = match kind {
        ChangeKind::Gain => DeltaOrientation::Forward,
        ChangeKind::Loss => DeltaOrientation::Backward,
    };
    if orientation == expected {
        Ok(Some(source))
    } else {
        Ok(None)
    }
}

fn guards_imply_change_sign(
    guards: &[Expr],
    source: &DataSourceConfig,
    expected_positive: bool,
    env: &BindingEnv,
    data_sources: &[DataSourceConfig],
) -> Result<bool> {
    for guard in guards {
        let guard = resolve_expr_alias(guard, env).unwrap_or(guard);
        let Some((left, op, right)) = comparison_parts(guard) else {
            continue;
        };
        let left_constraint =
            delta_sign_constraint(left, op.clone(), right, source, env, data_sources)?;
        if left_constraint == Some(expected_positive) {
            return Ok(true);
        }
        let right_constraint = delta_sign_constraint(
            right,
            flip_relation(op.clone()),
            left,
            source,
            env,
            data_sources,
        )?;
        if right_constraint == Some(expected_positive) {
            return Ok(true);
        }
    }
    Ok(false)
}

fn delta_sign_constraint(
    delta_expr: &Expr,
    relation: BinaryOp,
    other_side: &Expr,
    source: &DataSourceConfig,
    env: &BindingEnv,
    data_sources: &[DataSourceConfig],
) -> Result<Option<bool>> {
    if !is_number_literal(other_side, 0.0) {
        return Ok(None);
    }
    let Some((guard_source, orientation)) = delta_source_expr(delta_expr, env, data_sources)?
    else {
        return Ok(None);
    };
    if guard_source.data_id != source.data_id {
        return Ok(None);
    }
    let positive_for_expr = match relation {
        BinaryOp::Greater | BinaryOp::GreaterEqual => Some(true),
        BinaryOp::Less | BinaryOp::LessEqual => Some(false),
        _ => None,
    };
    let Some(positive_for_expr) = positive_for_expr else {
        return Ok(None);
    };
    Ok(Some(match orientation {
        DeltaOrientation::Forward => positive_for_expr,
        DeltaOrientation::Backward => !positive_for_expr,
    }))
}

fn flip_relation(op: BinaryOp) -> BinaryOp {
    match op {
        BinaryOp::Greater => BinaryOp::Less,
        BinaryOp::GreaterEqual => BinaryOp::LessEqual,
        BinaryOp::Less => BinaryOp::Greater,
        BinaryOp::LessEqual => BinaryOp::GreaterEqual,
        other => other,
    }
}

fn delta_source_expr(
    expr: &Expr,
    env: &BindingEnv,
    data_sources: &[DataSourceConfig],
) -> Result<Option<(DataSourceConfig, DeltaOrientation)>> {
    let expr = resolve_expr_alias(expr, env).unwrap_or(expr);
    match expr {
        Expr::Unary {
            op: crate::script::UnaryOp::Negate,
            expr,
        } => {
            let Some((source, orientation)) = delta_source_expr(expr, env, data_sources)? else {
                return Ok(None);
            };
            Ok(Some((
                source,
                match orientation {
                    DeltaOrientation::Forward => DeltaOrientation::Backward,
                    DeltaOrientation::Backward => DeltaOrientation::Forward,
                },
            )))
        }
        Expr::Binary {
            left,
            op: BinaryOp::Subtract,
            right,
        } => {
            let Some((left_source, left_var, left_offset)) =
                indexed_loop_source_ref(left, env, data_sources)?
            else {
                return Ok(None);
            };
            let Some((right_source, right_var, right_offset)) =
                indexed_loop_source_ref(right, env, data_sources)?
            else {
                return Ok(None);
            };
            if left_source.data_id != right_source.data_id || left_var != right_var {
                return Ok(None);
            }
            if left_offset == right_offset + 1 {
                Ok(Some((left_source, DeltaOrientation::Forward)))
            } else if right_offset == left_offset + 1 {
                Ok(Some((left_source, DeltaOrientation::Backward)))
            } else {
                Ok(None)
            }
        }
        _ => Ok(None),
    }
}

fn indexed_loop_source_ref(
    expr: &Expr,
    env: &BindingEnv,
    data_sources: &[DataSourceConfig],
) -> Result<Option<(DataSourceConfig, String, isize)>> {
    let Expr::Index { object, index } = expr else {
        return Ok(None);
    };
    let Some(source) = resolve_data_source_ref(object, env, data_sources)? else {
        return Ok(None);
    };
    let Some((var, offset)) = loop_index_offset(index) else {
        return Ok(None);
    };
    Ok(Some((source, var, offset)))
}

fn loop_index_offset(expr: &Expr) -> Option<(String, isize)> {
    match expr {
        Expr::Number(value) => Some(("__absolute_index__".into(), value.round() as isize)),
        Expr::Identifier(name) => Some((name.clone(), 0)),
        Expr::Binary {
            left,
            op: BinaryOp::Add,
            right,
        } => match (left.as_ref(), expr_integer(right)) {
            (Expr::Identifier(name), Some(offset)) => Some((name.clone(), offset)),
            _ => None,
        },
        Expr::Binary {
            left,
            op: BinaryOp::Subtract,
            right,
        } => match (left.as_ref(), expr_integer(right)) {
            (Expr::Identifier(name), Some(offset)) => Some((name.clone(), -offset)),
            _ => None,
        },
        _ => None,
    }
}

fn expr_integer(expr: &Expr) -> Option<isize> {
    expr_number(expr).map(|value| value.round() as isize)
}

fn invert_condition(expr: &Expr) -> Option<Expr> {
    let (left, op, right) = comparison_parts(expr)?;
    let inverted = match op {
        BinaryOp::Greater => BinaryOp::LessEqual,
        BinaryOp::GreaterEqual => BinaryOp::Less,
        BinaryOp::Less => BinaryOp::GreaterEqual,
        BinaryOp::LessEqual => BinaryOp::Greater,
        BinaryOp::Equal => BinaryOp::NotEqual,
        BinaryOp::NotEqual => BinaryOp::Equal,
        _ => return None,
    };
    Some(Expr::Binary {
        left: Box::new(left.clone()),
        op: inverted,
        right: Box::new(right.clone()),
    })
}
