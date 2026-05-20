use super::*;

fn event_label(event_type: &str) -> &'static str {
    match event_type {
        "DataUpdated" => "数据更新",
        "IntentTriggered" => "意图触发",
        "IntentEvaluated" => "意图评估",
        "AgentDecisionProduced" => "代理决策",
        "RiskDecisionProduced" => "风控决策",
        "ExecutionPlanned" => "执行计划",
        "ExecutionFilled" => "执行成交",
        "PortfolioUpdated" => "组合更新",
        "RuntimeNotice" => "运行提示",
        "RuntimeWarning" => "运行告警",
        "RuntimeError" => "运行错误",
        _ => "运行事件",
    }
}

fn field_label(key: &str) -> String {
    match key {
        "ask_price" => "卖一价".to_string(),
        "bid_price" => "买一价".to_string(),
        "confidence" => "置信度".to_string(),
        "endpoint" => "数据端点".to_string(),
        "error" => "错误".to_string(),
        "exec_status" => "执行状态".to_string(),
        "freshness_ms" => "新鲜度（毫秒）".to_string(),
        "gap_count" => "缺口数量".to_string(),
        "kind" => "意图类型".to_string(),
        "latest_bar_time" => "最新 K 线时间".to_string(),
        "latest_price" => "最新价格".to_string(),
        "limit_price" => "限价".to_string(),
        "limit_triggered" => "触发限制".to_string(),
        "lifecycle_stage" => "生命周期".to_string(),
        "net_side" => "决策方向".to_string(),
        "net_strength" => "决策强度".to_string(),
        "order_count" => "订单数量".to_string(),
        "order_id" => "订单 ID".to_string(),
        "order_type" => "订单类型".to_string(),
        "order_type_decision_reason" => "下单语义".to_string(),
        "portfolio_net_exposure_ratio" => "组合净敞口".to_string(),
        "concentration_ratio" => "集中度".to_string(),
        "ping_error" => "Ping 错误".to_string(),
        "ping_latency_ms" => "探测延迟（毫秒）".to_string(),
        "price" => "成交价格".to_string(),
        "qty" => "数量".to_string(),
        "quality_flags" => "质量标记".to_string(),
        "reason_text" => "原因".to_string(),
        "remaining_qty" => "剩余数量".to_string(),
        "risk_score" => "风控评分".to_string(),
        "score" => "评分".to_string(),
        "side" => "方向".to_string(),
        "signal_direction" => "信号方向".to_string(),
        "signal_strength" => "信号强度".to_string(),
        "sizing_mode" => "定量模式".to_string(),
        "sizing_source" => "定量来源".to_string(),
        "source_health" => "源健康度".to_string(),
        "source_latency_ms" => "源延迟（毫秒）".to_string(),
        "source_status" => "源状态".to_string(),
        "symbol_net_exposure_ratio" => "单标的净敞口".to_string(),
        "stale_after_ms" => "过期阈值（毫秒）".to_string(),
        "status" => "状态".to_string(),
        "strength" => "强度".to_string(),
        "time_in_force" => "有效期".to_string(),
        "fallback" => "回退路径".to_string(),
        _ => key.to_string(),
    }
}

fn input_fields(event_type: &str) -> &'static [&'static str] {
    match event_type {
        "DataUpdated" => &[
            "latest_price",
            "latest_bar_time",
            "source_status",
            "source_health",
            "source_latency_ms",
            "freshness_ms",
            "stale_after_ms",
            "gap_count",
        ],
        "IntentTriggered" => &["signal_direction", "signal_strength", "confidence"],
        "IntentEvaluated" => &["kind", "strength", "confidence"],
        "AgentDecisionProduced" => &["net_side", "net_strength", "score"],
        "RiskDecisionProduced" => &["status", "risk_score"],
        "ExecutionPlanned" => &["side", "qty", "limit_price", "remaining_qty"],
        "ExecutionFilled" => &["side", "qty", "price", "exec_status"],
        "RuntimeWarning" | "RuntimeError" => &[
            "source_health",
            "source_status",
            "freshness_ms",
            "source_latency_ms",
            "gap_count",
            "quality_flags",
        ],
        _ => &[],
    }
}

fn output_fields(event_type: &str) -> &'static [&'static str] {
    match event_type {
        "DataUpdated" => &[
            "latest_price",
            "source_health",
            "source_status",
            "gap_count",
        ],
        "IntentTriggered" => &["signal_direction", "signal_strength"],
        "IntentEvaluated" => &["kind", "strength"],
        "AgentDecisionProduced" => &["net_side", "score"],
        "RiskDecisionProduced" => &["status", "risk_score"],
        "ExecutionPlanned" => &["exec_status", "order_id", "remaining_qty"],
        "ExecutionFilled" => &["exec_status", "order_id", "price", "qty"],
        "RuntimeWarning" | "RuntimeError" => &[
            "source_health",
            "source_status",
            "freshness_ms",
            "gap_count",
        ],
        _ => &[],
    }
}

fn severity_tone(severity: &str) -> &'static str {
    match severity {
        "Error" | "error" => "danger",
        "Warn" | "Warning" | "warning" => "warning",
        _ => "info",
    }
}

fn format_value(value: &Value) -> String {
    match value {
        Value::Null => "-".to_string(),
        Value::Bool(flag) => {
            if *flag {
                "是".to_string()
            } else {
                "否".to_string()
            }
        }
        Value::Number(number) => number.to_string(),
        Value::String(text) if text.is_empty() => "-".to_string(),
        Value::String(text) => text.clone(),
        Value::Array(items) => items
            .iter()
            .map(format_value)
            .collect::<Vec<_>>()
            .join(", "),
        _ => value.to_string(),
    }
}

fn diagnostics_row(key: &str, value: &Value) -> Option<RuntimeDiagnosticsFieldRow> {
    if value.is_null() || *value == Value::String(String::new()) {
        return None;
    }

    Some(RuntimeDiagnosticsFieldRow {
        key: key.to_string(),
        label: field_label(key),
        value: format_value(value),
    })
}

fn nested_diagnostics_row(
    parent_key: &str,
    child_key: &str,
    label: &str,
    payload: &Value,
) -> Option<RuntimeDiagnosticsFieldRow> {
    let value = payload.get(parent_key)?.get(child_key)?;
    if value.is_null() || *value == Value::String(String::new()) {
        return None;
    }

    Some(RuntimeDiagnosticsFieldRow {
        key: format!("{parent_key}.{child_key}"),
        label: label.to_string(),
        value: format_value(value),
    })
}

fn build_payload_rows(payload: &Value, keys: &[&str]) -> Vec<RuntimeDiagnosticsFieldRow> {
    let Some(object) = payload.as_object() else {
        return Vec::new();
    };

    let preferred = keys
        .iter()
        .filter_map(|key| {
            object
                .get(*key)
                .filter(|value| !value.is_null() && *value != &Value::String(String::new()))
                .map(|value| RuntimeDiagnosticsFieldRow {
                    key: (*key).to_string(),
                    label: field_label(key),
                    value: format_value(value),
                })
        })
        .collect::<Vec<_>>();

    if !preferred.is_empty() {
        return preferred;
    }

    object
        .iter()
        .filter(|(_, value)| !value.is_null() && *value != &Value::String(String::new()))
        .take(4)
        .map(|(key, value)| RuntimeDiagnosticsFieldRow {
            key: key.clone(),
            label: field_label(key),
            value: format_value(value),
        })
        .collect()
}

fn build_explanation_rows(event_type: &str, payload: &Value) -> Vec<RuntimeDiagnosticsFieldRow> {
    let mut rows = Vec::new();

    if let Some(summary) = payload.get("explanation_summary").and_then(Value::as_str) {
        if !summary.is_empty() {
            rows.push(RuntimeDiagnosticsFieldRow {
                key: "explanation_summary".to_string(),
                label: "解释摘要".to_string(),
                value: summary.to_string(),
            });
        }
    }

    match event_type {
        "RiskDecisionProduced" => {
            for key in ["reason_text", "limit_triggered", "sizing_mode"] {
                if let Some(row) = payload
                    .get(key)
                    .and_then(|value| diagnostics_row(key, value))
                {
                    rows.push(row);
                }
            }
        }
        "ExecutionPlanned" | "ExecutionFilled" => {
            for key in [
                "reason_text",
                "lifecycle_stage",
                "sizing_source",
                "order_type_decision_reason",
                "time_in_force",
            ] {
                if let Some(row) = payload
                    .get(key)
                    .and_then(|value| diagnostics_row(key, value))
                {
                    rows.push(row);
                }
            }
        }
        "DataUpdated" | "RuntimeWarning" | "RuntimeError" => {
            for key in ["source_health", "quality_flags", "fallback", "error"] {
                if let Some(row) = payload
                    .get(key)
                    .and_then(|value| diagnostics_row(key, value))
                {
                    rows.push(row);
                }
            }
        }
        _ => {
            safe_eprintln!("[diagnostics] 未知运行时事件类型，无法映射到前端格式");
        }
    }

    rows
}

fn build_data_quality_rows(payload: &Value) -> Vec<RuntimeDiagnosticsFieldRow> {
    let mut rows = Vec::new();
    for key in [
        "source_health",
        "source_status",
        "freshness_ms",
        "stale_after_ms",
        "source_latency_ms",
        "ping_latency_ms",
        "gap_count",
        "quality_flags",
        "fallback",
        "error",
        "ping_error",
    ] {
        if let Some(row) = payload
            .get(key)
            .and_then(|value| diagnostics_row(key, value))
        {
            rows.push(row);
        }
    }
    rows
}

fn build_risk_detail_rows(payload: &Value) -> Vec<RuntimeDiagnosticsFieldRow> {
    let mut rows = Vec::new();
    for key in ["status", "limit_triggered", "sizing_mode", "reason_text"] {
        if let Some(row) = payload
            .get(key)
            .and_then(|value| diagnostics_row(key, value))
        {
            rows.push(row);
        }
    }

    rows.extend(
        [
            ("pre_risk", "max_target_weight", "风控前最大目标权重"),
            ("post_risk", "max_target_weight", "风控后最大目标权重"),
            ("pre_risk", "concentration_ratio", "风控前集中度"),
            ("post_risk", "concentration_ratio", "风控后集中度"),
            (
                "pre_risk",
                "max_symbol_net_exposure_ratio",
                "风控前单标的净敞口",
            ),
            (
                "post_risk",
                "max_symbol_net_exposure_ratio",
                "风控后单标的净敞口",
            ),
            (
                "pre_risk",
                "portfolio_net_exposure_ratio",
                "风控前组合净敞口",
            ),
            (
                "post_risk",
                "portfolio_net_exposure_ratio",
                "风控后组合净敞口",
            ),
            ("pre_risk", "turnover_ratio", "风控前换手比"),
            ("post_risk", "turnover_ratio", "风控后换手比"),
            ("pre_risk", "basket_members", "风控前持仓数"),
            ("post_risk", "basket_members", "风控后持仓数"),
            ("pre_risk", "action_count", "风控前动作数"),
            ("post_risk", "action_count", "风控后动作数"),
        ]
        .into_iter()
        .filter_map(|(parent, child, label)| nested_diagnostics_row(parent, child, label, payload)),
    );

    rows
}

fn build_order_detail_rows(event_type: &str, payload: &Value) -> Vec<RuntimeDiagnosticsFieldRow> {
    let mut rows = Vec::new();
    for key in [
        "order_id",
        "side",
        "qty",
        "remaining_qty",
        "limit_price",
        "exec_status",
        "lifecycle_stage",
        "sizing_source",
        "order_type_decision_reason",
        "time_in_force",
        "reason_text",
    ] {
        if let Some(row) = payload
            .get(key)
            .and_then(|value| diagnostics_row(key, value))
        {
            rows.push(row);
        }
    }

    if event_type == "ExecutionPlanned" {
        if let Some(previews) = payload.get("order_previews").and_then(Value::as_array) {
            rows.push(RuntimeDiagnosticsFieldRow {
                key: "order_count".to_string(),
                label: field_label("order_count"),
                value: previews.len().to_string(),
            });
            if let Some(first) = previews.first() {
                if let Some(value) = first.get("side") {
                    rows.push(RuntimeDiagnosticsFieldRow {
                        key: "preview_side".to_string(),
                        label: "首个订单方向".to_string(),
                        value: format_value(value),
                    });
                }
                if let Some(value) = first.get("qty") {
                    rows.push(RuntimeDiagnosticsFieldRow {
                        key: "preview_qty".to_string(),
                        label: "首个订单数量".to_string(),
                        value: format_value(value),
                    });
                }
                if let Some(value) = first.get("order_type") {
                    rows.push(RuntimeDiagnosticsFieldRow {
                        key: "preview_order_type".to_string(),
                        label: "首个订单类型".to_string(),
                        value: format_value(value),
                    });
                }
                if let Some(value) = first.get("order_type_decision_reason") {
                    rows.push(RuntimeDiagnosticsFieldRow {
                        key: "preview_order_type_decision_reason".to_string(),
                        label: "首个订单下单语义".to_string(),
                        value: format_value(value),
                    });
                }
            }
        }
    }

    rows
}

fn event_summary_from_event(event: &FrontendRuntimeEvent) -> RuntimeDiagnosticsEventSummary {
    RuntimeDiagnosticsEventSummary {
        event_id: event.event_id.clone(),
        event_type: event.event_type.clone(),
        label: event_label(&event.event_type).to_string(),
        summary: if event.summary.is_empty() {
            "运行事件".to_string()
        } else {
            event.summary.clone()
        },
        tone: severity_tone(&event.severity).to_string(),
        severity: event.severity.clone(),
        event_time_ms: Some(event.event_time_ms),
    }
}

fn diagnostics_node_key(event: &FrontendRuntimeEvent) -> Option<String> {
    if !event.node_id.is_empty() {
        return Some(event.node_id.clone());
    }

    if matches!(
        event.event_type.as_str(),
        "DataUpdated" | "RuntimeWarning" | "RuntimeError"
    ) && !event.source_id.is_empty()
    {
        return Some(event.source_id.clone());
    }

    None
}

pub(super) fn build_runtime_diagnostics_from_events(
    events: &[FrontendRuntimeEvent],
    source: &str,
) -> RuntimeDiagnosticsPayload {
    let mut sorted = events.to_vec();
    sorted.sort_by(|left, right| right.event_time_ms.cmp(&left.event_time_ms));

    let mut active_node_ids = Vec::<String>::new();
    for event in &sorted {
        if let Some(node_key) = diagnostics_node_key(event) {
            if !active_node_ids.contains(&node_key) {
                active_node_ids.push(node_key);
            }
        }
    }

    let mut node_details = BTreeMap::<String, RuntimeDiagnosticsNodeDetail>::new();
    let mut active_nodes = Vec::<RuntimeDiagnosticsNodeSummary>::new();

    for node_id in &active_node_ids {
        let node_events = sorted
            .iter()
            .filter(|event| diagnostics_node_key(event).as_deref() == Some(node_id.as_str()))
            .collect::<Vec<_>>();
        let node_event_count = node_events.len();
        let latest_event = node_events.first().copied();
        let latest_data_event = node_events.iter().copied().find(|event| {
            event.payload.get("source_health").is_some()
                || event.payload.get("source_status").is_some()
        });
        let latest_risk_event = node_events
            .iter()
            .copied()
            .find(|event| event.event_type == "RiskDecisionProduced");
        let latest_order_event = node_events.iter().copied().find(|event| {
            event.event_type == "ExecutionPlanned" || event.event_type == "ExecutionFilled"
        });
        let latest_notice = node_events
            .iter()
            .find(|event| severity_tone(&event.severity) != "info")
            .copied();

        active_nodes.push(RuntimeDiagnosticsNodeSummary {
            node_id: node_id.clone(),
            latest_event_type: latest_event.map(|event| event.event_type.clone()),
            latest_event_label: latest_event
                .map(|event| event_label(&event.event_type).to_string())
                .unwrap_or_else(|| "运行事件".to_string()),
            latest_event_time_ms: latest_event.map(|event| event.event_time_ms),
            event_count: node_event_count,
        });

        node_details.insert(
            node_id.clone(),
            RuntimeDiagnosticsNodeDetail {
                node_id: node_id.clone(),
                latest_event: latest_event.map(event_summary_from_event),
                explanation_summary: latest_event
                    .and_then(|event| event.payload.get("explanation_summary"))
                    .and_then(Value::as_str)
                    .map(str::to_string),
                latest_input_rows: latest_event
                    .map(|event| {
                        build_payload_rows(&event.payload, input_fields(&event.event_type))
                    })
                    .unwrap_or_default(),
                latest_output_rows: latest_event
                    .map(|event| {
                        build_payload_rows(&event.payload, output_fields(&event.event_type))
                    })
                    .unwrap_or_default(),
                explanation_rows: latest_event
                    .map(|event| build_explanation_rows(&event.event_type, &event.payload))
                    .unwrap_or_default(),
                data_quality_rows: latest_data_event
                    .map(|event| build_data_quality_rows(&event.payload))
                    .unwrap_or_default(),
                risk_detail_rows: latest_risk_event
                    .map(|event| build_risk_detail_rows(&event.payload))
                    .unwrap_or_default(),
                order_detail_rows: latest_order_event
                    .map(|event| build_order_detail_rows(&event.event_type, &event.payload))
                    .unwrap_or_default(),
                latest_notice: latest_notice.map(event_summary_from_event),
                recent_events: node_events
                    .into_iter()
                    .take(5)
                    .map(event_summary_from_event)
                    .collect(),
                event_count: node_event_count,
            },
        );
    }

    RuntimeDiagnosticsPayload {
        source: source.to_string(),
        default_selected_node_id: active_node_ids.first().cloned(),
        active_nodes,
        node_details,
    }
}
