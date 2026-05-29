use super::*;

fn normalize_experiment_float_axis(
    values: &[f64],
    base: f64,
    field: &str,
) -> Result<Vec<f64>, (StatusCode, String)> {
    let mut normalized = Vec::new();
    if values.is_empty() {
        normalized.push(base);
        return Ok(normalized);
    }

    for value in values {
        if *value < 0.0 {
            return Err(json_bad_request(
                "bad_request",
                format!("parameter_grid.{field} 必须 >= 0"),
            ));
        }
        if !normalized.contains(value) {
            normalized.push(*value);
        }
    }

    Ok(normalized)
}

fn normalize_experiment_latency_axis(values: &[u64], base: u64) -> Vec<u64> {
    let mut normalized = Vec::new();
    if values.is_empty() {
        normalized.push(base);
        return normalized;
    }

    for value in values {
        if !normalized.contains(value) {
            normalized.push(*value);
        }
    }

    normalized
}

pub(super) fn build_experiment_overrides(
    request: &FrontendExperimentRequest,
    qs_protocol: &RuntimeProtocolCoreConfig,
) -> Result<Vec<FrontendExecutionAssumptionOverrides>, (StatusCode, String)> {
    let provided_values = request.parameter_grid.fee_bps.len()
        + request.parameter_grid.slippage_bps.len()
        + request.parameter_grid.latency_ms.len();
    if provided_values == 0 {
        return Err(json_bad_request(
            "bad_request",
            "参数网格必须至少包含一个执行假设值",
        ));
    }

    let base = resolved_backtest_execution_assumptions(
        qs_protocol,
        request.backtest_options.execution_assumptions.as_ref(),
    );
    let fee_values = normalize_experiment_float_axis(
        &request.parameter_grid.fee_bps,
        base.taker_fee_bps,
        "fee_bps",
    )?;
    let slippage_values = normalize_experiment_float_axis(
        &request.parameter_grid.slippage_bps,
        base.default_slippage_bps,
        "slippage_bps",
    )?;
    let latency_values = normalize_experiment_latency_axis(
        &request.parameter_grid.latency_ms,
        base.latency_assumption_ms.unwrap_or(0),
    );

    let variant_count = fee_values.len() * slippage_values.len() * latency_values.len();
    if variant_count > MAX_EXPERIMENT_VARIANTS {
        return Err(json_bad_request(
            "bad_request",
            format!(
                "参数扫描展开为 {variant_count} 个变体，超出当前限制 {MAX_EXPERIMENT_VARIANTS}"
            ),
        ));
    }

    let mut variants = Vec::with_capacity(variant_count);
    for fee_bps in fee_values {
        for slippage_bps in &slippage_values {
            for latency_ms in &latency_values {
                variants.push(FrontendExecutionAssumptionOverrides {
                    fee_bps: Some(fee_bps),
                    slippage_bps: Some(*slippage_bps),
                    latency_ms: Some(*latency_ms),
                });
            }
        }
    }

    Ok(variants)
}
