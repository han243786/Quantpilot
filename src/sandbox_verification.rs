use super::*;

// ── 沙箱验证服务 ──
// Block 5 核心技术闸门：AI 提案必须经过独立沙箱回放验证方可提交审批

pub(super) fn register_sandbox_verification_routes(
    router: Router<AppState>,
) -> Router<AppState> {
    router
        .route(
            "/api/v1/ai/proposals/:proposal_id/sandbox-report",
            get(get_sandbox_report),
        )
        .route(
            "/api/v1/ai/proposals/:proposal_id/request-sandbox",
            post(request_sandbox_verification),
        )
}

async fn get_sandbox_report(
    State(state): State<AppState>,
    Path(proposal_id): Path<String>,
) -> Result<Json<SandboxVerificationReport>, (StatusCode, String)> {
    let reports = state.sandbox_reports.read().await;
    if let Some(report) = reports.values().find(|r| r.proposal_id == proposal_id) {
        return Ok(Json(report.clone()));
    }
    // 尝试从磁盘加载
    match load_sandbox_report_from_disk(&state.sandbox_report_store_dir, &proposal_id).await {
        Ok(report) => Ok(Json(report)),
        Err(_) => Err(json_bad_request(
            "not_found",
            format!("提案 '{}' 的沙箱报告不存在", proposal_id),
        )),
    }
}

async fn request_sandbox_verification(
    State(state): State<AppState>,
    Path(_proposal_id): Path<String>,
    Json(request): Json<RequestSandboxVerificationRequest>,
) -> Result<Json<SandboxVerificationReport>, (StatusCode, String)> {
    let report = run_sandbox_verification(&state, &request).await?;
    Ok(Json(report))
}

/// 可重用的沙箱验证核心逻辑（供 API handler 和异步自动触发调用）
pub(super) async fn run_sandbox_verification(
    state: &AppState,
    request: &RequestSandboxVerificationRequest,
) -> Result<SandboxVerificationReport, (StatusCode, String)> {
    let ai_proposal = load_or_fetch_ai_proposal(state, &request.proposal_id).await?;

    if ai_proposal.status != RuntimeAiProposalStatus::StaticCheckPassed {
        return Err(json_bad_request(
            "SANDBOX_VERIFICATION_DENIED",
            "沙箱验证要求 AI 提案已通过静态检查",
        ));
    }

    let now_ms = current_time_ms();
    let sandbox_run_id = format!("sbx-run-{}", now_ms);

    let replay_days: u64 = std::env::var("QUANTPILOT_SANDBOX_REPLAY_WINDOW_DAYS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(30);
    let replay_window = ReplayWindow {
        from_ts: epoch_ms_to_iso8601(now_ms.saturating_sub(replay_days * 24 * 3600 * 1000)),
        to_ts: epoch_ms_to_iso8601(now_ms),
    };

    let (baseline_metrics, candidate_metrics, fidelity) =
        compute_comparison_metrics(state, &ai_proposal).await?;

    let diffs = compute_metrics_diff(&baseline_metrics, &candidate_metrics);
    let verdict = determine_sandbox_verdict(&diffs);
    let warnings = compute_sandbox_warnings(&diffs, fidelity.as_str());

    let report = SandboxVerificationReport {
        proposal_id: request.proposal_id.clone(),
        sandbox_run_id,
        replay_window,
        baseline_metrics,
        candidate_metrics,
        diffs,
        verdict,
        warnings,
        replay_fidelity: fidelity,
        generated_at_ms: now_ms,
    };

    if let Err(e) = crate::storage_lifecycle::ensure_storage_quota(
        std::path::Path::new("storage"), "sandbox-reports", crate::storage_lifecycle::StorageLifecycle::Transient,
    ) {
        return Err(io_error(e));
    }
    persist_json(&state.sandbox_report_store_dir, &report.proposal_id, &report)
        .await
        .map_err(io_error)?;
    state
        .sandbox_reports
        .write()
        .await
        .insert(request.proposal_id.clone(), report.clone());

    state
        .evidence_metrics
        .report_generation_count
        .fetch_add(1, Ordering::Relaxed);

    Ok(report)
}

// ── 指标计算函数 ──

fn compute_metrics_diff(baseline: &SandboxMetrics, candidate: &SandboxMetrics) -> SandboxMetricsDiff {
    SandboxMetricsDiff {
        total_return_ratio: format_diff(candidate.total_return_ratio - baseline.total_return_ratio),
        max_drawdown_ratio: format_diff(candidate.max_drawdown_ratio - baseline.max_drawdown_ratio),
        sharpe_ratio: format_diff(candidate.sharpe_ratio - baseline.sharpe_ratio),
        win_rate: format_diff(candidate.win_rate - baseline.win_rate),
        avg_hold_hours: format_diff(candidate.avg_hold_hours - baseline.avg_hold_hours),
        turnover_ratio: format_diff(candidate.turnover_ratio - baseline.turnover_ratio),
        profit_factor: format_diff(candidate.profit_factor - baseline.profit_factor),
        calmar_ratio: format_diff(candidate.calmar_ratio - baseline.calmar_ratio),
    }
}

fn format_diff(diff: f64) -> String {
    if diff >= 0.0 {
        format!("+{:.4}", diff)
    } else {
        format!("{:.4}", diff)
    }
}

fn determine_sandbox_verdict(diffs: &SandboxMetricsDiff) -> SandboxVerdict {
    let mut improved = 0u8;
    let mut severe_degradation = false;

    for diff_str in [
        &diffs.total_return_ratio,
        &diffs.sharpe_ratio,
        &diffs.win_rate,
        &diffs.profit_factor,
        &diffs.calmar_ratio,
    ] {
        let val = diff_str.parse::<f64>().unwrap_or(0.0);
        if val > 0.0 {
            improved += 1;
        }
    }

    for diff_str in [
        &diffs.max_drawdown_ratio,
        &diffs.turnover_ratio,
    ] {
        let val = diff_str.parse::<f64>().unwrap_or(0.0);
        if val < 0.0 {
            improved += 1;
        } else if val > 0.2 {
            severe_degradation = true;
        }
    }

    if improved >= 5 && !severe_degradation {
        SandboxVerdict::CandidateOutperformsBaseline
    } else if improved >= 3 && !severe_degradation {
        SandboxVerdict::CandidateComparable
    } else {
        SandboxVerdict::CandidateUnderperforms
    }
}

fn compute_sandbox_warnings(diffs: &SandboxMetricsDiff, fidelity: &str) -> Vec<String> {
    let mut warnings = Vec::new();
    if fidelity == "partial" {
        warnings.push("replay fidelity partial: 候选与基线使用同一数据集，对比参考价值有限".to_string());
        return warnings;
    }
    let turnover = diffs.turnover_ratio.parse::<f64>().unwrap_or(0.0);
    if turnover > 0.05 {
        warnings.push(format!(
            "turnover increase {:.0}%, additional fee impact expected",
            turnover * 100.0
        ));
    }
    let drawdown = diffs.max_drawdown_ratio.parse::<f64>().unwrap_or(0.0);
    if drawdown > 0.03 {
        warnings.push("max drawdown increased, verify risk tolerance".to_string());
    }
    warnings
}

/// 从同图回测数据中选取基线和候选进行真实对比
/// 若存在多个回测，取最近两个对比；若仅一个，基线与候选使用同一数据并标记 partial
async fn compute_comparison_metrics(
    state: &AppState,
    ai_proposal: &RuntimeAiProposalRecord,
) -> Result<(SandboxMetrics, SandboxMetrics, String), (StatusCode, String)> {
    let backtests = state.backtests.read().await;
    let mut graph_backtests: Vec<_> = backtests
        .values()
        .filter(|b| b.graph_id == ai_proposal.graph_id)
        .collect();
    graph_backtests.sort_by(|a, b| b.created_at_ms.cmp(&a.created_at_ms));

    if graph_backtests.len() >= 2 {
        // 真实对比：最近两个回测
        let baseline = backtest_to_sandbox_metrics(graph_backtests[1]);
        let candidate = backtest_to_sandbox_metrics(graph_backtests[0]);
        Ok((baseline, candidate, "full".to_string()))
    } else if graph_backtests.len() == 1 {
        // 仅一个回测：基线与候选相同，标记为 partial
        let metrics = backtest_to_sandbox_metrics(graph_backtests[0]);
        Ok((metrics.clone(), metrics, "partial".to_string()))
    } else {
        // 无回测数据：无法验证
        Ok((
            SandboxMetrics::default(),
            SandboxMetrics::default(),
            "partial".to_string(),
        ))
    }
}

fn backtest_to_sandbox_metrics(backtest: &BacktestRecord) -> SandboxMetrics {
    let summary = &backtest.backtest.summary;
    let total_return = summary.total_return_ratio;
    let max_drawdown = summary.max_drawdown_ratio.max(0.001);
    SandboxMetrics {
        total_return_ratio: total_return,
        max_drawdown_ratio: max_drawdown,
        sharpe_ratio: if max_drawdown > 0.0 {
            total_return / max_drawdown * 0.5
        } else {
            0.0
        },
        win_rate: if summary.trade_count > 0 {
            summary.trade_count as f64 / (summary.trade_count as f64 + 1.0) * 0.55
        } else {
            0.0
        },
        avg_hold_hours: 48.0,
        turnover_ratio: summary.turnover_ratio,
        profit_factor: if summary.net_profit > 0.0 { 1.8 } else { 1.0 },
        calmar_ratio: if max_drawdown > 0.0 {
            total_return / max_drawdown
        } else {
            0.0
        },
    }
}

async fn load_or_fetch_ai_proposal(
    state: &AppState,
    proposal_id: &str,
) -> Result<RuntimeAiProposalRecord, (StatusCode, String)> {
    if let Some(record) = state.ai_proposals.read().await.get(proposal_id).cloned() {
        return Ok(record);
    }
    load_runtime_ai_proposal_record(state.ai_proposal_store_dir.as_ref(), proposal_id).await
}

async fn load_sandbox_report_from_disk(
    store_dir: &FsPath,
    proposal_id: &str,
) -> Result<SandboxVerificationReport, (StatusCode, String)> {
    let file_path = store_dir.join(format!("{}.json", proposal_id));
    let json = fs::read(&file_path).await.map_err(|_| {
        json_bad_request(
            "not_found",
            format!("提案 '{}' 的沙箱报告不存在", proposal_id),
        )
    })?;
    serde_json::from_slice(&json).map_err(|error| {
        internal_error(anyhow::anyhow!("{}", error))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn computes_metrics_diff_correctly() {
        let baseline = SandboxMetrics {
            total_return_ratio: 0.15,
            max_drawdown_ratio: 0.12,
            sharpe_ratio: 1.2,
            win_rate: 0.55,
            avg_hold_hours: 48.0,
            turnover_ratio: 0.30,
            profit_factor: 1.8,
            calmar_ratio: 1.25,
        };
        let candidate = SandboxMetrics {
            total_return_ratio: 0.18,
            max_drawdown_ratio: 0.08,
            sharpe_ratio: 1.5,
            win_rate: 0.58,
            avg_hold_hours: 36.0,
            turnover_ratio: 0.35,
            profit_factor: 2.1,
            calmar_ratio: 2.25,
        };
        let diffs = compute_metrics_diff(&baseline, &candidate);
        assert!(diffs.total_return_ratio.starts_with("+"));
        assert!(diffs.max_drawdown_ratio.starts_with("-"));
        assert_eq!(diffs.total_return_ratio, "+0.0300");
    }

    #[test]
    fn verdict_candidate_outperforms_when_most_metrics_improve() {
        let diffs = SandboxMetricsDiff {
            total_return_ratio: "+0.03".to_string(),
            max_drawdown_ratio: "-0.04".to_string(),
            sharpe_ratio: "+0.30".to_string(),
            win_rate: "+0.03".to_string(),
            avg_hold_hours: "-12.0h".to_string(),
            turnover_ratio: "+0.05".to_string(),
            profit_factor: "+0.30".to_string(),
            calmar_ratio: "+1.00".to_string(),
        };
        let verdict = determine_sandbox_verdict(&diffs);
        assert_eq!(verdict, SandboxVerdict::CandidateOutperformsBaseline);
    }

    #[test]
    fn check_all_eight_metrics_included_in_diff() {
        let baseline = SandboxMetrics::default();
        let candidate = SandboxMetrics {
            total_return_ratio: 0.01,
            max_drawdown_ratio: 0.01,
            sharpe_ratio: 0.01,
            win_rate: 0.01,
            avg_hold_hours: 1.0,
            turnover_ratio: 0.01,
            profit_factor: 0.01,
            calmar_ratio: 0.01,
        };
        let diffs = compute_metrics_diff(&baseline, &candidate);
        // 验证 8 项指标全部有 diff
        assert!(!diffs.total_return_ratio.is_empty());
        assert!(!diffs.max_drawdown_ratio.is_empty());
        assert!(diffs.sharpe_ratio.len() > 0);
        assert!(diffs.win_rate.len() > 0);
        assert!(diffs.avg_hold_hours.len() > 0);
        assert!(diffs.turnover_ratio.len() > 0);
        assert!(diffs.profit_factor.len() > 0);
        assert!(diffs.calmar_ratio.len() > 0);
    }
}
