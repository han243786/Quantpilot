use super::*;

pub(super) fn validate_runtime_parameter_mutation_boundary(
    boundary: &RuntimeParameterMutationBoundary,
) -> Result<(), (StatusCode, String)> {
    let requested = boundary.requested.trim();
    if requested.is_empty() {
        return Err(json_bad_request(
            "bad_request",
            "activation_boundary.requested 是必填字段",
        ));
    }
    if requested == "immediate" {
        return Err(json_bad_request(
            "parameter_mutation_boundary_violation",
            "不支持立即激活的参数变更；请使用 next_cycle_start、manual_pause 或 sequence_cursor",
        ));
    }
    if requested == "next_cycle_start" || requested == "manual_pause" {
        return Ok(());
    }
    if requested == "sequence_cursor" && boundary.resolved_sequence_no.is_some() {
        return Ok(());
    }
    if requested
        .strip_prefix("sequence_cursor:")
        .and_then(|value| value.parse::<u64>().ok())
        .is_some()
    {
        return Ok(());
    }
    Err(json_bad_request(
        "parameter_mutation_boundary_violation",
        "不支持的激活边界；请使用 next_cycle_start、manual_pause 或 sequence_cursor",
    ))
}

pub(super) fn resolve_runtime_parameter_mutation_boundary(
    boundary: &RuntimeParameterMutationBoundary,
    current_sequence_no: u64,
) -> Result<RuntimeParameterMutationBoundary, (StatusCode, String)> {
    validate_runtime_parameter_mutation_boundary(boundary)?;
    let requested = boundary.requested.trim();
    if requested == "next_cycle_start" {
        return Ok(RuntimeParameterMutationBoundary {
            requested: "next_cycle_start".to_string(),
            resolved_sequence_no: Some(current_sequence_no + 2),
        });
    }
    if requested == "manual_pause" {
        return Ok(RuntimeParameterMutationBoundary {
            requested: "manual_pause".to_string(),
            resolved_sequence_no: None,
        });
    }
    let sequence_no = boundary.resolved_sequence_no.or_else(|| {
        requested
            .strip_prefix("sequence_cursor:")
            .and_then(|value| value.parse::<u64>().ok())
    });
    let Some(sequence_no) = sequence_no else {
        return Err(json_bad_request(
            "parameter_mutation_boundary_violation",
            "序列游标激活边界需要 resolved_sequence_no",
        ));
    };
    Ok(RuntimeParameterMutationBoundary {
        requested: "sequence_cursor".to_string(),
        resolved_sequence_no: Some(sequence_no),
    })
}

pub(super) fn evaluate_runtime_parameter_mutation_safe_window(
    snapshot: Option<RuntimeParameterMutationSafeWindowSnapshot>,
) -> RuntimeParameterMutationSafeWindowState {
    let snapshot = snapshot.unwrap_or_default();
    let mut reason_code = "SAFE_WINDOW_OPEN";
    let mut message = "安全窗口已开启，允许运行时参数变更".to_string();
    let mut retryable = false;
    let mut retry_after_ms = None;

    if !matches!(
        snapshot.runtime_status.as_str(),
        "paused" | "idle" | "stopped" | "ready"
    ) {
        reason_code = "SAFE_WINDOW_RUNTIME_ACTIVE";
        message = format!(
            "运行时状态 `{}` 不符合参数变更条件",
            snapshot.runtime_status
        );
        retryable = true;
    } else if snapshot.open_order_count > 0 {
        reason_code = "SAFE_WINDOW_OPEN_ORDERS";
        message = format!(
            "{} 笔未结订单必须结算后才可变更参数",
            snapshot.open_order_count
        );
        retryable = true;
    } else if snapshot.outstanding_risk_violation {
        reason_code = "SAFE_WINDOW_RISK_VIOLATION";
        message = "存在未解决的风控违规，阻止参数变更".to_string();
        retryable = true;
    } else if snapshot.data_freshness_ms > 60_000 {
        reason_code = "SAFE_WINDOW_STALE_DATA";
        message = format!(
            "数据新鲜度 {}ms 超出 60000ms 安全窗口限制",
            snapshot.data_freshness_ms
        );
        retryable = true;
    } else if snapshot.portfolio_exposure_bps.abs() > 10_000 {
        reason_code = "SAFE_WINDOW_EXPOSURE_LIMIT";
        message = format!(
            "组合敞口 {}bps 超出安全窗口限制",
            snapshot.portfolio_exposure_bps
        );
        retryable = true;
    } else if snapshot.cooldown_remaining_ms > 0 {
        reason_code = "SAFE_WINDOW_COOLDOWN";
        message = format!("变更冷却还剩 {}ms", snapshot.cooldown_remaining_ms);
        retryable = true;
        retry_after_ms = Some(snapshot.cooldown_remaining_ms);
    }

    let allowed = reason_code == "SAFE_WINDOW_OPEN";
    RuntimeParameterMutationSafeWindowState {
        status: if allowed { "allowed" } else { "denied" }.to_string(),
        policy_version: snapshot.policy_version.clone(),
        allowed,
        reason_code: reason_code.to_string(),
        message,
        retryable,
        retry_after_ms,
        snapshot,
    }
}
