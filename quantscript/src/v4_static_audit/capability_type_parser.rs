use qrpc_core_ir::v4::{ExecutionCapabilityKind, QsScalarTypeKind, QsTypeRef, RuntimeTradingMode};

pub(super) fn parse_runtime_mode(input: &str) -> Result<RuntimeTradingMode, String> {
    match input {
        "paper_actual" => Ok(RuntimeTradingMode::PaperActual),
        "paper_simulated" => Ok(RuntimeTradingMode::PaperSimulated),
        "live_actual" => Ok(RuntimeTradingMode::LiveActual),
        "live_simulated" => Ok(RuntimeTradingMode::LiveSimulated),
        other => Err(format!("未知 runtime mode: {other}")),
    }
}

pub(super) fn parse_execution_capability(input: &str) -> Result<ExecutionCapabilityKind, String> {
    match input {
        "market" => Ok(ExecutionCapabilityKind::Market),
        "limit" => Ok(ExecutionCapabilityKind::Limit),
        "post_only" | "limit_maker" => Ok(ExecutionCapabilityKind::PostOnly),
        "stop_market" => Ok(ExecutionCapabilityKind::StopMarket),
        "stop_limit" => Ok(ExecutionCapabilityKind::StopLimit),
        "take_profit_market" => Ok(ExecutionCapabilityKind::TakeProfitMarket),
        "take_profit_limit" => Ok(ExecutionCapabilityKind::TakeProfitLimit),
        "ioc" => Ok(ExecutionCapabilityKind::Ioc),
        "fok" => Ok(ExecutionCapabilityKind::Fok),
        "oco_bracket" | "bracket_tp_sl" | "oco" => Ok(ExecutionCapabilityKind::OcoBracket),
        "trailing_stop" => Ok(ExecutionCapabilityKind::TrailingStop),
        "reduce_only" => Ok(ExecutionCapabilityKind::ReduceOnly),
        "close_only" => Ok(ExecutionCapabilityKind::CloseOnly),
        "open_long" => Ok(ExecutionCapabilityKind::OpenLong),
        "close_long" => Ok(ExecutionCapabilityKind::CloseLong),
        "open_short" => Ok(ExecutionCapabilityKind::OpenShort),
        "close_short" => Ok(ExecutionCapabilityKind::CloseShort),
        "one_way_position_mode" | "one_way" => Ok(ExecutionCapabilityKind::OneWayPositionMode),
        "hedge_position_mode" | "hedge" => Ok(ExecutionCapabilityKind::HedgePositionMode),
        "gtc" => Ok(ExecutionCapabilityKind::Gtc),
        "day" => Ok(ExecutionCapabilityKind::Day),
        "gtd" => Ok(ExecutionCapabilityKind::Gtd),
        "client_order_id" => Ok(ExecutionCapabilityKind::ClientOrderId),
        "cancel_replace_amend" | "cancel" | "replace" | "amend" => {
            Ok(ExecutionCapabilityKind::CancelReplaceAmend)
        }
        other => Err(format!("未知 execution capability: {other}")),
    }
}

pub(super) fn parse_qs_type_ref(input: &str) -> Result<QsTypeRef, String> {
    let input = input.trim();
    if let Some(inner) = strip_wrapper(input, "optional") {
        return Ok(QsTypeRef::Optional {
            inner: Box::new(parse_qs_type_ref(inner)?),
        });
    }
    if let Some(inner) = strip_wrapper(input, "fresh") {
        return Ok(QsTypeRef::Fresh {
            inner: Box::new(parse_qs_type_ref(inner)?),
        });
    }
    if let Some(inner) = strip_wrapper(input, "stale") {
        return Ok(QsTypeRef::Stale {
            inner: Box::new(parse_qs_type_ref(inner)?),
        });
    }
    if let Some(inner) = strip_wrapper(input, "list") {
        let (item, max_items) = parse_items_type_args(inner)?;
        return Ok(QsTypeRef::List {
            item: Box::new(parse_qs_type_ref(item)?),
            max_items,
        });
    }
    if let Some(inner) = strip_wrapper(input, "map") {
        let args = split_top_level_args(inner);
        if args.len() != 3 {
            return Err("map 类型必须写成 map<key,value,max=N>".to_string());
        }
        let key = parse_qs_scalar_type(args[0])?;
        let max_items = parse_max_arg(args[2])?;
        return Ok(QsTypeRef::Map {
            key,
            value: Box::new(parse_qs_type_ref(args[1])?),
            max_items,
        });
    }
    Ok(QsTypeRef::Scalar {
        scalar: parse_qs_scalar_type(input)?,
    })
}

fn strip_wrapper<'a>(input: &'a str, wrapper: &str) -> Option<&'a str> {
    input
        .strip_prefix(wrapper)?
        .strip_prefix('<')?
        .strip_suffix('>')
        .map(str::trim)
}

fn parse_items_type_args(input: &str) -> Result<(&str, u32), String> {
    let args = split_top_level_args(input);
    if args.len() != 2 {
        return Err("list 类型必须写成 list<T,max=N>".to_string());
    }
    Ok((args[0], parse_max_arg(args[1])?))
}

fn parse_max_arg(input: &str) -> Result<u32, String> {
    let Some(value) = input.trim().strip_prefix("max=") else {
        return Err("容量参数必须写成 max=N".to_string());
    };
    value
        .parse::<u32>()
        .map_err(|_| "容量参数 max 必须是正整数".to_string())
}

fn split_top_level_args(input: &str) -> Vec<&str> {
    let mut args = Vec::new();
    let mut depth = 0usize;
    let mut start = 0usize;
    for (index, ch) in input.char_indices() {
        match ch {
            '<' => depth += 1,
            '>' => depth = depth.saturating_sub(1),
            ',' if depth == 0 => {
                args.push(input[start..index].trim());
                start = index + 1;
            }
            _ => {}
        }
    }
    args.push(input[start..].trim());
    args
}

fn parse_qs_scalar_type(input: &str) -> Result<QsScalarTypeKind, String> {
    match input {
        "bool" => Ok(QsScalarTypeKind::Bool),
        "int" => Ok(QsScalarTypeKind::Int),
        "decimal" => Ok(QsScalarTypeKind::Decimal),
        "time" => Ok(QsScalarTypeKind::Time),
        "duration" => Ok(QsScalarTypeKind::Duration),
        "price" => Ok(QsScalarTypeKind::Price),
        "quantity" => Ok(QsScalarTypeKind::Quantity),
        "notional" => Ok(QsScalarTypeKind::Notional),
        "percent" => Ok(QsScalarTypeKind::Percent),
        "ratio" => Ok(QsScalarTypeKind::Ratio),
        "fee" => Ok(QsScalarTypeKind::Fee),
        "slippage" => Ok(QsScalarTypeKind::Slippage),
        "leverage" => Ok(QsScalarTypeKind::Leverage),
        "symbol" => Ok(QsScalarTypeKind::Symbol),
        "venue" => Ok(QsScalarTypeKind::Venue),
        "account" => Ok(QsScalarTypeKind::Account),
        "side" => Ok(QsScalarTypeKind::Side),
        "position_side" => Ok(QsScalarTypeKind::PositionSide),
        "order_type" => Ok(QsScalarTypeKind::OrderType),
        "time_in_force" => Ok(QsScalarTypeKind::TimeInForce),
        "freshness" => Ok(QsScalarTypeKind::Freshness),
        "runtime_mode" => Ok(QsScalarTypeKind::RuntimeMode),
        "order_permission" => Ok(QsScalarTypeKind::OrderPermission),
        other => Err(format!("未知 QS 类型: {other}")),
    }
}
