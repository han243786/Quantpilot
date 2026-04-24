use anyhow::{anyhow, Result};
use qrpc_core::{Exchange, MarketType};

use crate::script::{CallArg, Expr};

#[derive(Debug, Clone, Copy)]
pub(crate) enum ArgSelector<'a> {
    Positional(usize),
    Named(&'a str),
    NamedOrPositional(&'a str, usize),
}

pub(crate) fn find_arg<'a>(args: &'a [CallArg], selector: ArgSelector<'_>) -> Option<&'a Expr> {
    match selector {
        ArgSelector::Positional(index) => args
            .iter()
            .filter(|arg| arg.name.is_none())
            .nth(index)
            .map(|arg| &arg.value),
        ArgSelector::Named(name) => args
            .iter()
            .find(|arg| arg.name.as_deref() == Some(name))
            .map(|arg| &arg.value),
        ArgSelector::NamedOrPositional(name, index) => args
            .iter()
            .find(|arg| arg.name.as_deref() == Some(name))
            .map(|arg| &arg.value)
            .or_else(|| {
                args.iter()
                    .filter(|arg| arg.name.is_none())
                    .nth(index)
                    .map(|arg| &arg.value)
            }),
    }
}

pub(crate) fn arg_expr_required<'a>(
    args: &'a [CallArg],
    selector: ArgSelector<'_>,
) -> Result<&'a Expr> {
    find_arg(args, selector).ok_or_else(|| match selector {
        ArgSelector::Positional(index) => {
            anyhow!("missing required function argument at index {index}")
        }
        ArgSelector::Named(name) => anyhow!("missing required function argument: {name}"),
        ArgSelector::NamedOrPositional(name, index) => {
            anyhow!("missing required function argument: {name} or positional index {index}")
        }
    })
}

pub(crate) fn arg_number_optional(args: &[CallArg], selector: ArgSelector<'_>) -> Option<f64> {
    find_arg(args, selector).and_then(expr_number)
}

pub(crate) fn arg_string_optional(args: &[CallArg], selector: ArgSelector<'_>) -> Option<String> {
    find_arg(args, selector).and_then(expr_string)
}

pub(crate) fn expr_number(expr: &Expr) -> Option<f64> {
    match expr {
        Expr::Number(value) => Some(*value),
        Expr::Unary {
            op: crate::script::UnaryOp::Negate,
            expr,
        } => expr_number(expr).map(|value| -value),
        _ => None,
    }
}

pub(crate) fn expr_string(expr: &Expr) -> Option<String> {
    match expr {
        Expr::String(value) => Some(value.clone()),
        Expr::Identifier(value) => Some(value.clone()),
        Expr::Raw(value) => Some(value.clone()),
        _ => None,
    }
}

pub(crate) fn sanitize_id(value: &str) -> String {
    value
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() {
                ch.to_ascii_lowercase()
            } else {
                '_'
            }
        })
        .collect::<String>()
        .trim_matches('_')
        .to_string()
}

pub(crate) fn format_exchange(exchange: &Exchange) -> &'static str {
    match exchange {
        Exchange::Binance => "binance",
        Exchange::Okx => "okx",
    }
}

pub(crate) fn format_market_type(market_type: &MarketType) -> &'static str {
    match market_type {
        MarketType::Spot => "spot",
    }
}
