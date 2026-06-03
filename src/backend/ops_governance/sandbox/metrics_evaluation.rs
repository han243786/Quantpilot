use crate::*;

// ── 指标计算函数 ──

pub(super) fn compute_metrics_diff(
    baseline: &SandboxMetrics,
    candidate: &SandboxMetrics,
) -> SandboxMetricsDiff {
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

pub(super) fn determine_sandbox_verdict(diffs: &SandboxMetricsDiff) -> SandboxVerdict {
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

    for diff_str in [&diffs.max_drawdown_ratio, &diffs.turnover_ratio] {
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

pub(super) fn compute_sandbox_warnings(diffs: &SandboxMetricsDiff, fidelity: &str) -> Vec<String> {
    let mut warnings = Vec::new();
    if fidelity == "partial" {
        warnings.push("回放忠实度部分覆盖: 候选与基线使用同一数据集，对比参考价值有限。建议使用独立数据集重新验证。".to_string());
    }
    let turnover = diffs.turnover_ratio.parse::<f64>().unwrap_or(0.0);
    if turnover > 0.05 {
        warnings.push(format!(
            "换手率增加 {:.0}%，可能产生额外手续费影响",
            turnover * 100.0
        ));
    }
    let drawdown = diffs.max_drawdown_ratio.parse::<f64>().unwrap_or(0.0);
    if drawdown > 0.03 {
        warnings.push("最大回撤增加，请确认策略风险在可接受范围内".to_string());
    }
    warnings
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
