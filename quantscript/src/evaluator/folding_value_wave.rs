use crate::script::{BinaryOp, CallArg, Expr, UnaryOp};
use anyhow::{anyhow, Result};

pub(super) fn expr_iterable_items(expr: &Expr) -> Option<Vec<Expr>> {
    match expr {
        Expr::List(items) => Some(items.clone()),
        Expr::Range { start, end } => {
            let start = expr_number(start)?.round() as i64;
            let end = expr_number(end)?.round() as i64;
            Some(
                (start..end)
                    .map(|value| Expr::Number(value as f64))
                    .collect(),
            )
        }
        _ => None,
    }
}

pub(super) fn fold_unary(op: UnaryOp, expr: Expr) -> Result<Expr> {
    match (op.clone(), expr.clone()) {
        (UnaryOp::Negate, Expr::Number(value)) => Ok(Expr::Number(-value)),
        (UnaryOp::Not, Expr::Bool(value)) => Ok(Expr::Bool(!value)),
        _ => Ok(Expr::Unary {
            op,
            expr: Box::new(expr),
        }),
    }
}

pub(super) fn fold_index(object: Expr, index: Expr) -> Result<Expr> {
    if let Some(position) = expr_integer(&index) {
        if let Expr::List(items) = &object {
            if let Some(item) = list_index(items, position) {
                return Ok(item.clone());
            }
        }
    }

    Ok(Expr::Index {
        object: Box::new(object),
        index: Box::new(index),
    })
}

pub(super) fn fold_slice(
    object: Expr,
    start: Option<Box<Expr>>,
    end: Option<Box<Expr>>,
) -> Result<Expr> {
    if let Expr::List(items) = &object {
        let slice = list_slice(
            items,
            start.as_deref().and_then(expr_integer),
            end.as_deref().and_then(expr_integer),
        );
        if let Some(slice) = slice {
            return Ok(Expr::List(slice.to_vec()));
        }
    }

    Ok(Expr::Slice {
        object: Box::new(object),
        start,
        end,
    })
}

pub(super) fn fold_binary(left: Expr, op: BinaryOp, right: Expr) -> Result<Expr> {
    if let (Some(lhs), Some(rhs)) = (expr_number(&left), expr_number(&right)) {
        // B1-3: 编译期除零/模零检测
        if matches!(op, BinaryOp::Divide | BinaryOp::Modulo) && rhs.abs() < f64::EPSILON {
            return Err(anyhow!("QS0403 编译期除零/模零"));
        }
        return Ok(match op {
            BinaryOp::Add => Expr::Number(lhs + rhs),
            BinaryOp::Subtract => Expr::Number(lhs - rhs),
            BinaryOp::Multiply => Expr::Number(lhs * rhs),
            BinaryOp::Divide => Expr::Number(lhs / rhs),
            BinaryOp::Modulo => Expr::Number(lhs % rhs),
            BinaryOp::Greater => Expr::Bool(lhs > rhs),
            BinaryOp::GreaterEqual => Expr::Bool(lhs >= rhs),
            BinaryOp::Less => Expr::Bool(lhs < rhs),
            BinaryOp::LessEqual => Expr::Bool(lhs <= rhs),
            BinaryOp::Equal => Expr::Bool((lhs - rhs).abs() <= f64::EPSILON),
            BinaryOp::NotEqual => Expr::Bool((lhs - rhs).abs() > f64::EPSILON),
            _ => Expr::Binary {
                left: Box::new(left),
                op,
                right: Box::new(right),
            },
        });
    }

    if let (Some(lhs), Some(rhs)) = (expr_bool(&left), expr_bool(&right)) {
        return Ok(match op {
            BinaryOp::And => Expr::Bool(lhs && rhs),
            BinaryOp::Or => Expr::Bool(lhs || rhs),
            BinaryOp::Equal => Expr::Bool(lhs == rhs),
            BinaryOp::NotEqual => Expr::Bool(lhs != rhs),
            _ => Expr::Binary {
                left: Box::new(left),
                op,
                right: Box::new(right),
            },
        });
    }

    if let (Some(lhs), Some(rhs)) = (expr_string(&left), expr_string(&right)) {
        return Ok(match op {
            BinaryOp::Add => Expr::String(format!("{lhs}{rhs}")),
            BinaryOp::Equal => Expr::Bool(lhs == rhs),
            BinaryOp::NotEqual => Expr::Bool(lhs != rhs),
            _ => Expr::Binary {
                left: Box::new(left),
                op,
                right: Box::new(right),
            },
        });
    }

    Ok(Expr::Binary {
        left: Box::new(left),
        op,
        right: Box::new(right),
    })
}

pub(super) fn fold_builtin_call(callee: Expr, args: Vec<CallArg>) -> Result<Expr> {
    if let Expr::Member { object, field } = &callee {
        if let Some(value) = fold_member_builtin(object, field, &args)? {
            return Ok(value);
        }
    }

    if let Expr::Identifier(name) = &callee {
        if let Some(value) = fold_identifier_builtin(name, &args)? {
            return Ok(value);
        }
    }

    Ok(Expr::Call {
        callee: Box::new(callee),
        args,
    })
}

fn fold_member_builtin(object: &Expr, field: &str, args: &[CallArg]) -> Result<Option<Expr>> {
    if args.is_empty() {
        match field {
            "len" => {
                if let Some(length) = expr_length(object) {
                    return Ok(Some(Expr::Number(length as f64)));
                }
            }
            "sum" => {
                if let Some(sum) = expr_sum(object) {
                    return Ok(Some(Expr::Number(sum)));
                }
            }
            "mean" | "avg" => {
                if let Some(mean) = expr_mean(object) {
                    return Ok(Some(Expr::Number(mean)));
                }
            }
            "min" => {
                if let Some(value) = expr_min(object) {
                    return Ok(Some(Expr::Number(value)));
                }
            }
            "max" => {
                if let Some(value) = expr_max(object) {
                    return Ok(Some(Expr::Number(value)));
                }
            }
            "std" | "stddev" => {
                if let Some(value) = expr_stddev(object) {
                    return Ok(Some(Expr::Number(value)));
                }
            }
            "variance" => {
                if let Some(value) = expr_variance(object) {
                    return Ok(Some(Expr::Number(value)));
                }
            }
            "first" => {
                if let Some(value) = expr_first(object) {
                    return Ok(Some(value));
                }
            }
            "last" => {
                if let Some(value) = expr_last(object) {
                    return Ok(Some(value));
                }
            }
            "ok" => {
                if is_fetch_expr(object) {
                    return Ok(Some(Expr::Bool(true)));
                }
            }
            "retryable" => {
                if is_fetch_expr(object) {
                    return Ok(Some(Expr::Bool(false)));
                }
            }
            _ => {}
        }
    }

    Ok(None)
}

fn fold_identifier_builtin(name: &str, args: &[CallArg]) -> Result<Option<Expr>> {
    let positional = args.iter().map(|arg| &arg.value).collect::<Vec<_>>();
    let result = match (name, positional.as_slice()) {
        ("abs", [value]) => expr_number(value).map(|item| Expr::Number(item.abs())),
        ("sqrt", [value]) => expr_number(value).and_then(|item| {
            if item < 0.0 {
                None
            } else {
                Some(Expr::Number(item.sqrt()))
            }
        }),
        ("sum", [value]) => expr_sum(value).map(Expr::Number),
        ("mean" | "avg", [value]) => expr_mean(value).map(Expr::Number),
        ("min", [value]) => expr_min(value).map(Expr::Number),
        ("max", [value]) => expr_max(value).map(Expr::Number),
        ("std" | "stddev", [value]) => expr_stddev(value).map(Expr::Number),
        ("variance", [value]) => expr_variance(value).map(Expr::Number),
        ("first", [value]) => expr_first(value),
        ("last", [value]) => expr_last(value),
        ("pow", [left, right]) => match (expr_number(left), expr_number(right)) {
            // v2.1.x: 拒绝负底数+非整数指数, 防止编译时 NaN 注入
            (Some(base), Some(exp)) if base >= 0.0 || exp.fract() == 0.0 => {
                Some(Expr::Number(base.powf(exp)))
            }
            _ => None,
        },
        _ => None,
    };
    Ok(result)
}

fn is_fetch_expr(expr: &Expr) -> bool {
    match expr {
        Expr::Call { callee, .. } => matches!(
            callee.as_ref(),
            Expr::Identifier(name) if name == "fetch" || name == "get_data"
        ),
        Expr::Try(inner) | Expr::Await(inner) => is_fetch_expr(inner),
        _ => false,
    }
}

fn expr_length(expr: &Expr) -> Option<usize> {
    match expr {
        Expr::List(items) => Some(items.len()),
        Expr::Call { args, .. } if is_fetch_expr(expr) => args
            .iter()
            .find(|arg| arg.name.as_deref() == Some("lookback"))
            .and_then(|arg| expr_number(&arg.value))
            .map(|value| value.max(1.0) as usize),
        Expr::Try(inner) | Expr::Await(inner) => expr_length(inner),
        _ => None,
    }
}

fn expr_sum(expr: &Expr) -> Option<f64> {
    expr_numbers(expr).map(|values| values.into_iter().sum())
}

fn expr_mean(expr: &Expr) -> Option<f64> {
    let values = expr_numbers(expr)?;
    if values.is_empty() {
        return None;
    }
    Some(values.iter().sum::<f64>() / values.len() as f64)
}

fn expr_min(expr: &Expr) -> Option<f64> {
    expr_numbers(expr)?.into_iter().reduce(f64::min)
}

fn expr_max(expr: &Expr) -> Option<f64> {
    expr_numbers(expr)?.into_iter().reduce(f64::max)
}

fn expr_variance(expr: &Expr) -> Option<f64> {
    let values = expr_numbers(expr)?;
    if values.is_empty() {
        return None;
    }
    let mean = values.iter().sum::<f64>() / values.len() as f64;
    Some(
        values
            .iter()
            .map(|value| {
                let delta = *value - mean;
                delta * delta
            })
            .sum::<f64>()
            / values.len() as f64,
    )
}

fn expr_stddev(expr: &Expr) -> Option<f64> {
    expr_variance(expr).map(f64::sqrt)
}

fn expr_first(expr: &Expr) -> Option<Expr> {
    match expr {
        Expr::List(items) => items.first().cloned(),
        _ => None,
    }
}

fn expr_last(expr: &Expr) -> Option<Expr> {
    match expr {
        Expr::List(items) => items.last().cloned(),
        _ => None,
    }
}

fn expr_numbers(expr: &Expr) -> Option<Vec<f64>> {
    match expr {
        Expr::List(items) => items.iter().map(expr_number).collect::<Option<Vec<_>>>(),
        _ => None,
    }
}

pub(super) fn expr_number(expr: &Expr) -> Option<f64> {
    match expr {
        Expr::Number(value) => Some(*value),
        Expr::Unary {
            op: UnaryOp::Negate,
            expr,
        } => expr_number(expr).map(|value| -value),
        _ => None,
    }
}

fn expr_integer(expr: &Expr) -> Option<isize> {
    expr_number(expr).map(|value| value.round() as isize)
}

pub(super) fn expr_bool(expr: &Expr) -> Option<bool> {
    match expr {
        Expr::Bool(value) => Some(*value),
        _ => None,
    }
}

fn expr_string(expr: &Expr) -> Option<String> {
    match expr {
        Expr::String(value) => Some(value.clone()),
        Expr::Identifier(value) => Some(value.clone()),
        _ => None,
    }
}

fn list_index(items: &[Expr], index: isize) -> Option<&Expr> {
    let len = items.len() as isize;
    let normalized = if index < 0 { len + index } else { index };
    if normalized < 0 || normalized >= len {
        return None;
    }
    items.get(normalized as usize)
}

fn list_slice(items: &[Expr], start: Option<isize>, end: Option<isize>) -> Option<&[Expr]> {
    let len = items.len() as isize;
    let normalized_start = normalize_slice_bound(start.unwrap_or(0), len)?;
    let normalized_end = normalize_slice_bound(end.unwrap_or(len), len)?;
    if normalized_start > normalized_end {
        return Some(&items[0..0]);
    }
    items.get(normalized_start..normalized_end)
}

fn normalize_slice_bound(bound: isize, len: isize) -> Option<usize> {
    let normalized = if bound < 0 { len + bound } else { bound };
    let clamped = normalized.clamp(0, len);
    usize::try_from(clamped).ok()
}
