use crate::script::{CallArg, Expr, FunctionDecl, Stmt};
use anyhow::{bail, Result};
use qrpc_core::{
    GLOBAL_RISK_PROFILE_DEFAULT_MAX_EXCHANGE_LEVERAGE, GLOBAL_RISK_PROFILE_DEFAULT_MAX_POSITION,
    GLOBAL_RISK_PROFILE_DEFAULT_MAX_TOTAL_LEVERAGE,
    GLOBAL_RISK_PROFILE_DEFAULT_MIN_ACTION_INTERVAL_MS, GLOBAL_RISK_PROFILE_ID,
    PAPER_EXECUTION_PROFILE_DEFAULT_FEE_BPS, PAPER_EXECUTION_PROFILE_DEFAULT_SLIPPAGE_BPS,
    PAPER_EXECUTION_PROFILE_ID,
};

#[derive(Debug, Clone)]
pub(super) struct GlobalRiskProfileSpec {
    pub(super) max_position_ratio: f64,
    pub(super) max_total_leverage: f64,
    pub(super) max_exchange_leverage: f64,
    pub(super) min_action_interval_ms: u64,
}

#[derive(Debug, Clone)]
pub(super) struct PaperExecutionProfileSpec {
    pub(super) fee_bps: f64,
    pub(super) slippage_bps: f64,
}

pub(super) fn detect_global_risk_profile(
    strategy: &FunctionDecl,
) -> Result<Option<GlobalRiskProfileSpec>> {
    let mut detected = None;
    for stmt in &strategy.body {
        if let Some(call_args) = risk_profile_call_args(stmt) {
            if detected.is_some() {
                bail!("QuantScript 当前最多支持一个 risk.profile(...) 声明");
            }
            detected = Some(parse_global_risk_profile_args(call_args)?);
            continue;
        }
        if nested_risk_profile_call(stmt) {
            bail!("risk.profile(...) 必须作为 fn strategy() 中的顶级语句出现");
        }
    }
    Ok(detected)
}

fn risk_profile_call_args(stmt: &Stmt) -> Option<&[CallArg]> {
    let Stmt::Expr(Expr::Call { callee, args }) = stmt else {
        return None;
    };
    let Expr::Member { object, field } = callee.as_ref() else {
        return None;
    };
    if !matches!(object.as_ref(), Expr::Identifier(name) if name == "risk") || field != "profile" {
        return None;
    }
    Some(args.as_slice())
}

fn parse_global_risk_profile_args(args: &[CallArg]) -> Result<GlobalRiskProfileSpec> {
    let Some(first_arg) = args.first() else {
        bail!("risk.profile(...) 需要 `profile_id` 位置参数");
    };
    if first_arg.name.is_some() {
        bail!("risk.profile(...) 要求第一个参数为位置参数 profile id");
    }
    let Expr::String(profile_id) = &first_arg.value else {
        bail!("risk.profile(...) profile_id 必须是字符串字面量");
    };
    if profile_id != GLOBAL_RISK_PROFILE_ID {
        bail!(
            "risk.profile(...) 当前只支持 profile_id=\"{}\"",
            GLOBAL_RISK_PROFILE_ID
        );
    }

    let mut spec = GlobalRiskProfileSpec {
        max_position_ratio: GLOBAL_RISK_PROFILE_DEFAULT_MAX_POSITION,
        max_total_leverage: GLOBAL_RISK_PROFILE_DEFAULT_MAX_TOTAL_LEVERAGE,
        max_exchange_leverage: GLOBAL_RISK_PROFILE_DEFAULT_MAX_EXCHANGE_LEVERAGE,
        min_action_interval_ms: GLOBAL_RISK_PROFILE_DEFAULT_MIN_ACTION_INTERVAL_MS,
    };

    for arg in &args[1..] {
        let Some(name) = arg.name.as_deref() else {
            bail!("risk.profile(...) 在 profile_id 之后只支持具名关键字字段");
        };
        match name {
            "max_position" => {
                spec.max_position_ratio = risk_profile_number_field(name, &arg.value)?;
                if !spec.max_position_ratio.is_finite() || spec.max_position_ratio <= 0.0 {
                    bail!("risk.profile(..., max_position=...) 必须大于 0");
                }
            }
            "max_total_leverage" => {
                spec.max_total_leverage = risk_profile_number_field(name, &arg.value)?;
                if spec.max_total_leverage < 1.0 {
                    bail!("risk.profile(..., max_total_leverage=...) 必须大于等于 1");
                }
            }
            "max_exchange_leverage" => {
                spec.max_exchange_leverage = risk_profile_number_field(name, &arg.value)?;
                if spec.max_exchange_leverage < 1.0 {
                    bail!("risk.profile(..., max_exchange_leverage=...) 必须大于等于 1");
                }
            }
            "min_action_interval_ms" => {
                let value = risk_profile_number_field(name, &arg.value)?;
                if value < 0.0 || value.fract().abs() > f64::EPSILON {
                    bail!("risk.profile(..., min_action_interval_ms=...) 必须是非负整数");
                }
                spec.min_action_interval_ms = value as u64;
            }
            other => bail!(
                "risk.profile(...) 不支持关键字字段 `{}` 在当前运行时中",
                other
            ),
        }
    }

    Ok(spec)
}

pub(super) fn detect_paper_execution_profile(
    strategy: &FunctionDecl,
) -> Result<Option<PaperExecutionProfileSpec>> {
    let mut detected = None;
    for stmt in &strategy.body {
        if let Some(call_args) = execution_profile_call_args(stmt) {
            if detected.is_some() {
                bail!("QuantScript 当前最多支持一个 execution.profile(...) 声明");
            }
            detected = Some(parse_paper_execution_profile_args(call_args)?);
            continue;
        }
        if nested_execution_profile_call(stmt) {
            bail!("execution.profile(...) 必须作为 fn strategy() 中的顶级语句出现");
        }
    }
    Ok(detected)
}

fn execution_profile_call_args(stmt: &Stmt) -> Option<&[CallArg]> {
    let Stmt::Expr(Expr::Call { callee, args }) = stmt else {
        return None;
    };
    let Expr::Member { object, field } = callee.as_ref() else {
        return None;
    };
    if !matches!(object.as_ref(), Expr::Identifier(name) if name == "execution")
        || field != "profile"
    {
        return None;
    }
    Some(args.as_slice())
}

fn parse_paper_execution_profile_args(args: &[CallArg]) -> Result<PaperExecutionProfileSpec> {
    let Some(first_arg) = args.first() else {
        bail!("execution.profile(...) 需要 `profile_id` 位置参数");
    };
    if first_arg.name.is_some() {
        bail!("execution.profile(...) 要求第一个参数为位置参数 profile id");
    }
    let Expr::String(profile_id) = &first_arg.value else {
        bail!("execution.profile(...) profile_id 必须是字符串字面量");
    };
    if profile_id != PAPER_EXECUTION_PROFILE_ID {
        bail!(
            "execution.profile(...) 当前只支持 profile_id=\"{}\"",
            PAPER_EXECUTION_PROFILE_ID
        );
    }

    let mut spec = PaperExecutionProfileSpec {
        fee_bps: PAPER_EXECUTION_PROFILE_DEFAULT_FEE_BPS,
        slippage_bps: PAPER_EXECUTION_PROFILE_DEFAULT_SLIPPAGE_BPS,
    };

    for arg in &args[1..] {
        let Some(name) = arg.name.as_deref() else {
            bail!("execution.profile(...) 在 profile_id 之后只支持具名关键字字段");
        };
        match name {
            "fee_bps" => {
                spec.fee_bps = execution_profile_number_field(name, &arg.value)?;
                if spec.fee_bps < 0.0 {
                    bail!("execution.profile(..., fee_bps=...) 必须大于等于 0");
                }
            }
            "slippage_bps" => {
                spec.slippage_bps = execution_profile_number_field(name, &arg.value)?;
                if spec.slippage_bps < 0.0 {
                    bail!("execution.profile(..., slippage_bps=...) 必须大于等于 0");
                }
            }
            other => bail!(
                "execution.profile(...) 不支持关键字字段 `{}` 在当前运行时中",
                other
            ),
        }
    }

    Ok(spec)
}

fn execution_profile_number_field(name: &str, expr: &Expr) -> Result<f64> {
    let Expr::Number(value) = expr else {
        bail!("execution.profile(..., {}=...) 必须是数值字面量", name);
    };
    if !value.is_finite() {
        bail!("execution.profile(..., {}=...) 必须是有限数", name);
    }
    Ok(*value)
}

fn risk_profile_number_field(name: &str, expr: &Expr) -> Result<f64> {
    let Expr::Number(value) = expr else {
        bail!("risk.profile(..., {}=...) 必须是数值字面量", name);
    };
    Ok(*value)
}

fn nested_risk_profile_call(stmt: &Stmt) -> bool {
    match stmt {
        Stmt::Let { value, .. } | Stmt::Expr(value) | Stmt::Return(Some(value)) => {
            expr_contains_risk_profile(value)
        }
        Stmt::Return(None) | Stmt::EmitIntent { .. } => false,
        Stmt::If {
            condition,
            then_branch,
            else_if_branches,
            else_branch,
        } => {
            expr_contains_risk_profile(condition)
                || then_branch.iter().any(nested_risk_profile_call)
                || else_if_branches.iter().any(|(expr, body)| {
                    expr_contains_risk_profile(expr) || body.iter().any(nested_risk_profile_call)
                })
                || else_branch
                    .as_ref()
                    .map(|branch| branch.iter().any(nested_risk_profile_call))
                    .unwrap_or(false)
        }
        Stmt::For { iterable, body, .. } => {
            expr_contains_risk_profile(iterable) || body.iter().any(nested_risk_profile_call)
        }
        Stmt::While { condition, body } => {
            expr_contains_risk_profile(condition) || body.iter().any(nested_risk_profile_call)
        }
        Stmt::Match { expr, arms } => {
            expr_contains_risk_profile(expr)
                || arms.iter().any(|arm| match &arm.body {
                    crate::script::MatchArmBody::Statement(stmt) => nested_risk_profile_call(stmt),
                    crate::script::MatchArmBody::Expr(expr) => expr_contains_risk_profile(expr),
                })
        }
    }
}

fn nested_execution_profile_call(stmt: &Stmt) -> bool {
    match stmt {
        Stmt::Let { value, .. } | Stmt::Expr(value) | Stmt::Return(Some(value)) => {
            expr_contains_execution_profile(value)
        }
        Stmt::Return(None) | Stmt::EmitIntent { .. } => false,
        Stmt::If {
            condition,
            then_branch,
            else_if_branches,
            else_branch,
        } => {
            expr_contains_execution_profile(condition)
                || then_branch.iter().any(nested_execution_profile_call)
                || else_if_branches.iter().any(|(expr, body)| {
                    expr_contains_execution_profile(expr)
                        || body.iter().any(nested_execution_profile_call)
                })
                || else_branch
                    .as_ref()
                    .map(|branch| branch.iter().any(nested_execution_profile_call))
                    .unwrap_or(false)
        }
        Stmt::For { iterable, body, .. } => {
            expr_contains_execution_profile(iterable)
                || body.iter().any(nested_execution_profile_call)
        }
        Stmt::While { condition, body } => {
            expr_contains_execution_profile(condition)
                || body.iter().any(nested_execution_profile_call)
        }
        Stmt::Match { expr, arms } => {
            expr_contains_execution_profile(expr)
                || arms.iter().any(|arm| match &arm.body {
                    crate::script::MatchArmBody::Statement(stmt) => {
                        nested_execution_profile_call(stmt)
                    }
                    crate::script::MatchArmBody::Expr(expr) => {
                        expr_contains_execution_profile(expr)
                    }
                })
        }
    }
}

fn expr_contains_risk_profile(expr: &Expr) -> bool {
    match expr {
        Expr::Call { callee, args } => {
            if matches!(
                callee.as_ref(),
                Expr::Member { object, field }
                    if matches!(object.as_ref(), Expr::Identifier(name) if name == "risk") && field == "profile"
            ) {
                return true;
            }
            expr_contains_risk_profile(callee)
                || args
                    .iter()
                    .any(|arg| expr_contains_risk_profile(&arg.value))
        }
        Expr::Member { object, .. }
        | Expr::Index { object, .. }
        | Expr::Unary { expr: object, .. }
        | Expr::Try(object)
        | Expr::Await(object) => expr_contains_risk_profile(object),
        Expr::Slice { object, start, end } => {
            expr_contains_risk_profile(object)
                || start
                    .as_ref()
                    .map(|expr| expr_contains_risk_profile(expr))
                    .unwrap_or(false)
                || end
                    .as_ref()
                    .map(|expr| expr_contains_risk_profile(expr))
                    .unwrap_or(false)
        }
        Expr::Binary { left, right, .. } => {
            expr_contains_risk_profile(left) || expr_contains_risk_profile(right)
        }
        Expr::Range { start, end } => {
            expr_contains_risk_profile(start) || expr_contains_risk_profile(end)
        }
        Expr::List(items) => items.iter().any(expr_contains_risk_profile),
        Expr::Raw(_) | Expr::Identifier(_) | Expr::String(_) | Expr::Number(_) | Expr::Bool(_) => {
            false
        }
    }
}

fn expr_contains_execution_profile(expr: &Expr) -> bool {
    match expr {
        Expr::Call { callee, args } => {
            if matches!(
                callee.as_ref(),
                Expr::Member { object, field }
                    if matches!(object.as_ref(), Expr::Identifier(name) if name == "execution") && field == "profile"
            ) {
                return true;
            }
            expr_contains_execution_profile(callee)
                || args
                    .iter()
                    .any(|arg| expr_contains_execution_profile(&arg.value))
        }
        Expr::Member { object, .. }
        | Expr::Index { object, .. }
        | Expr::Unary { expr: object, .. }
        | Expr::Try(object)
        | Expr::Await(object) => expr_contains_execution_profile(object),
        Expr::Slice { object, start, end } => {
            expr_contains_execution_profile(object)
                || start
                    .as_ref()
                    .map(|expr| expr_contains_execution_profile(expr))
                    .unwrap_or(false)
                || end
                    .as_ref()
                    .map(|expr| expr_contains_execution_profile(expr))
                    .unwrap_or(false)
        }
        Expr::Binary { left, right, .. } => {
            expr_contains_execution_profile(left) || expr_contains_execution_profile(right)
        }
        Expr::Range { start, end } => {
            expr_contains_execution_profile(start) || expr_contains_execution_profile(end)
        }
        Expr::List(items) => items.iter().any(expr_contains_execution_profile),
        Expr::Raw(_) | Expr::Identifier(_) | Expr::String(_) | Expr::Number(_) | Expr::Bool(_) => {
            false
        }
    }
}
