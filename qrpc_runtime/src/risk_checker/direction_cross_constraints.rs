use super::*;

/// v2.1.0: 跨Agent方向冲突检测
/// 同一标的有多个Agent同时发出买卖相反方向→DirectionConflict拒绝
pub(super) fn apply_direction_conflict_check(
    mut decisions: Vec<RiskDecision>,
    events: &mut Vec<RuntimeEvent>,
    now_ms: u64,
) -> Vec<RiskDecision> {
    use std::collections::HashMap;
    // 按标的收集所有已批准的决策
    let mut by_symbol: HashMap<String, Vec<usize>> = HashMap::new();
    for (i, d) in decisions.iter().enumerate() {
        if matches!(d.status, DecisionStatus::Approve | DecisionStatus::Clamp) {
            by_symbol
                .entry(format!("{:?}", d.symbol))
                .or_default()
                .push(i);
        }
    }
    // 对每个标的有≥2个不同Agent的决策，检查方向冲突
    for indices in by_symbol.values() {
        if indices.len() < 2 {
            continue;
        }
        let has_buy = indices.iter().any(|&i| {
            decisions[i]
                .adjusted_actions
                .iter()
                .any(|a| matches!(a.side, OrderSide::Buy))
        });
        let has_sell = indices.iter().any(|&i| {
            decisions[i]
                .adjusted_actions
                .iter()
                .any(|a| matches!(a.side, OrderSide::Sell))
        });
        if has_buy && has_sell {
            for &i in indices {
                decisions[i].status = DecisionStatus::Reject;
                decisions[i].reason_codes = vec![RiskReasonCode::DirectionConflict];
                decisions[i].reason_text =
                    "方向冲突: 同一标的上存在多个Agent的相反方向操作，已全部拒绝".to_string();
                decisions[i].adjusted_actions.clear();
                decisions[i].adjusted_portfolio_target_decision = None;
                events.push(RuntimeEvent {
                    event_id: format!(
                        "evt-risk-direction-conflict-{}-{}",
                        decisions[i].risk_decision_id, now_ms
                    ),
                    event_type: RuntimeEventType::RiskDecisionProduced,
                    trace_id: decisions[i].trace_id.clone(),
                    source_id: decisions[i].risk_id.clone(),
                    ts_ms: now_ms,
                    payload: serde_json::json!({
                        "risk_decision_id": decisions[i].risk_decision_id,
                        "symbol": decisions[i].symbol,
                        "reason": "direction_conflict",
                        "explanation": "同一标的多Agent方向冲突，为避免相互踩踏全部拒绝"
                    }),
                });
            }
        }
    }
    decisions
}

/// v1.1.0: Phase 2 跨标的联合约束检查
pub(super) fn apply_cross_symbol_constraints(
    risks: &[qrpc_core_ir::RiskPolicy],
    mut decisions: Vec<RiskDecision>,
    portfolio: &PortfolioState,
    now_ms: u64,
    provider_key: &str,
    events: &mut Vec<RuntimeEvent>,
) -> Vec<RiskDecision> {
    for risk in risks.iter().filter(|r| r.enabled) {
        let cross_leverage_limit = risk.max_cross_symbol_leverage.unwrap_or(f64::MAX);

        if cross_leverage_limit >= f64::MAX {
            continue; // 无跨标的约束配置
        }

        let equity = portfolio_equity(portfolio).max(1.0);

        // 计算所有已批准/clamped 决策的合计敞口
        // v2.0.1: quantity_ratio 已经是 equity 的比例 (如 0.2 = 20%)，
        // 所以名义本金 = equity * quantity_ratio，无需再乘价格。
        let total_notional: f64 = decisions
            .iter()
            .filter(|d| !matches!(d.status, DecisionStatus::Reject))
            .map(|d| {
                d.adjusted_actions
                    .iter()
                    .map(|a| a.quantity_ratio * equity)
                    .sum::<f64>()
            })
            .sum();

        let cross_symbol_leverage = total_notional / equity;

        if cross_symbol_leverage > cross_leverage_limit && cross_leverage_limit.is_finite() {
            // 按比例缩减所有非 Reject 的 decision
            let scale = cross_leverage_limit / cross_symbol_leverage;
            for d in decisions
                .iter_mut()
                .filter(|d| !matches!(d.status, DecisionStatus::Reject))
            {
                for action in &mut d.adjusted_actions {
                    action.quantity_ratio *= scale;
                }
                if let Some(ref mut target) = d.adjusted_portfolio_target_decision {
                    for tw in &mut target.target.target_weights {
                        tw.target_weight =
                            tw.current_weight + (tw.target_weight - tw.current_weight) * scale;
                    }
                }
                // 标记为 Clamp 如果不是 Approve
                if matches!(d.status, DecisionStatus::Approve) {
                    d.status = DecisionStatus::Clamp;
                }
                d.reason_codes
                    .push(RiskReasonCode::ExceedPortfolioNetExposure);
                d.reason_text.push_str(&format!(
                    " 跨标的杠杆 {:.2} 超过限制 {:.2}, 缩减至 {:.0}%",
                    cross_symbol_leverage,
                    cross_leverage_limit,
                    scale * 100.0,
                ));
            }
            events.push(RuntimeEvent {
                event_id: format!("evt-risk-cross-symbol-{now_ms}"),
                event_type: RuntimeEventType::RiskDecisionProduced,
                trace_id: "cross_symbol".to_string(),
                source_id: risk.policy_id.clone(),
                ts_ms: now_ms,
                payload: serde_json::json!({
                    "provider_key": provider_key,
                    "status": "clamped_cross_symbol",
                    "cross_symbol_leverage": cross_symbol_leverage,
                    "limit": cross_leverage_limit,
                    "scale": scale,
                    "reason_text": format!("跨标的合计杠杆 {:.2} 超过限制 {:.2}", cross_symbol_leverage, cross_leverage_limit),
                }),
            });
        }
    }
    decisions
}
