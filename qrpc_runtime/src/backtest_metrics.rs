use qrpc_core::{
    BacktestBenchmarkComparison, BacktestDrawdownAnalysis, BacktestEquityPoint,
    BacktestRiskAdjusted, BacktestSummary, BacktestTradeAnalysis, SessionOutput,
};
use qrpc_core_ir::v4::V4BacktestMicrostructureMetrics;

pub const QPRT_EQUITY_CURVE_NON_MONOTONIC: &str = "QPRT4101";

#[derive(Debug, Clone, Copy)]
pub struct MicrostructureOrderSample {
    pub requested_quantity: f64,
    pub filled_quantity: f64,
    pub reference_price: f64,
    pub is_open: bool,
}

#[derive(Debug, Clone, Copy)]
pub struct MicrostructureFillSample {
    pub quantity: f64,
    pub price: f64,
    pub reference_price: f64,
}

pub fn compute_microstructure_metrics(
    orders: &[MicrostructureOrderSample],
    fills: &[MicrostructureFillSample],
) -> V4BacktestMicrostructureMetrics {
    let requested_quantity = orders
        .iter()
        .map(|order| order.requested_quantity.max(0.0))
        .sum::<f64>();
    let filled_quantity = orders
        .iter()
        .map(|order| order.filled_quantity.max(0.0))
        .sum::<f64>();
    let fill_rate = if requested_quantity > f64::EPSILON {
        (filled_quantity / requested_quantity).clamp(0.0, 1.0)
    } else {
        0.0
    };

    let mut slippage_weight = 0.0;
    let mut slippage_sum = 0.0;
    let mut fill_notional = 0.0;
    let mut reference_notional = 0.0;
    for fill in fills {
        if fill.quantity <= 0.0 || fill.reference_price <= 0.0 {
            continue;
        }
        let weight = fill.quantity;
        let slippage_bps =
            ((fill.price - fill.reference_price).abs() / fill.reference_price) * 10_000.0;
        slippage_sum += slippage_bps * weight;
        slippage_weight += weight;
        fill_notional += fill.price * weight;
        reference_notional += fill.reference_price * weight;
    }

    let average_slippage_bps = if slippage_weight > f64::EPSILON {
        slippage_sum / slippage_weight
    } else {
        0.0
    };
    let vwap_deviation_bps = if slippage_weight > f64::EPSILON && reference_notional > 0.0 {
        let fill_vwap = fill_notional / slippage_weight;
        let reference_vwap = reference_notional / slippage_weight;
        ((fill_vwap - reference_vwap) / reference_vwap) * 10_000.0
    } else {
        0.0
    };
    let open_ratio = if orders.is_empty() {
        0.0
    } else {
        orders.iter().filter(|order| order.is_open).count() as f64 / orders.len() as f64
    };

    V4BacktestMicrostructureMetrics {
        submitted_order_count: orders.len() as u64,
        filled_order_count: orders
            .iter()
            .filter(|order| order.filled_quantity > f64::EPSILON)
            .count() as u64,
        fill_rate,
        average_slippage_bps,
        queue_position_estimate: open_ratio,
        vwap_deviation_bps,
    }
}

/// 从回测输出计算全部风险调整收益指标
pub fn compute_backtest_metrics(
    summary: &mut BacktestSummary,
    sessions: &[SessionOutput],
    equity_curve: &[BacktestEquityPoint],
    benchmark_curve: &[BacktestEquityPoint],
) {
    let initial_equity = equity_curve
        .first()
        .map(|p| p.equity)
        .unwrap_or(1.0)
        .max(1.0);

    // 每日收益率序列
    let daily_returns = compute_step_returns(equity_curve);
    let _trading_days = daily_returns.len().max(1) as f64;
    let days_covered = if equity_curve.len() >= 2 {
        let start_ms = equity_curve.first().map(|p| p.ts_ms).unwrap_or(0);
        let end_ms = equity_curve.last().map(|p| p.ts_ms).unwrap_or(0);
        (end_ms.saturating_sub(start_ms) as f64 / 86_400_000.0).max(1.0)
    } else {
        1.0
    };

    // 年化收益率（回测小于 30 天时不年化，避免短周期外推爆炸）
    let total_return = summary.total_return_ratio;
    // v2.0.1: 当 total_return < -1.0 时底数为负，powf 返回 NaN。
    // 使用 max(MIN_POSITIVE) 夹紧以防止 NaN 传播到下游指标。
    let base = (1.0 + total_return).max(f64::MIN_POSITIVE);
    let annualized_return = if days_covered >= 30.0 {
        base.powf(365.0 / days_covered) - 1.0
    } else {
        total_return // 短回测直接报告总收益率
    };

    // 年化波动率
    let daily_vol = std_deviation(&daily_returns);
    let annualized_volatility = daily_vol * 252.0_f64.sqrt();

    // 夏普比率
    let sharpe_ratio = if annualized_volatility.is_finite() && annualized_volatility > 0.0 {
        annualized_return / annualized_volatility
    } else {
        0.0
    };

    // v1.1.13: 索提诺比率 — 修正为标准下行偏差公式
    // DD = sqrt(1/N * Σ min(0, R_i)^2)，包含全部观测值
    let n = daily_returns.len().max(1) as f64;
    let downside_var: f64 = daily_returns
        .iter()
        .map(|&r| r.min(0.0).powi(2))
        .sum::<f64>()
        / n;
    let downside_vol = downside_var.sqrt() * 252.0_f64.sqrt();
    let sortino_ratio = if downside_vol.is_finite() && downside_vol > 0.0 {
        annualized_return / downside_vol
    } else {
        0.0
    };

    // 卡尔玛比率
    let max_dd = summary.drawdown_analysis.max_drawdown_ratio;
    let calmar_ratio = if max_dd > f64::EPSILON {
        annualized_return / max_dd
    } else {
        0.0
    };

    summary.annualized_return = annualized_return;
    summary.annualized_volatility = annualized_volatility;
    summary.risk_adjusted = BacktestRiskAdjusted {
        sharpe_ratio,
        sortino_ratio,
        calmar_ratio,
        var_95: 0.0,
        cvar_95: 0.0,
    };

    // 交易分析
    let trade_analysis = compute_trade_analysis(sessions, initial_equity);
    summary.trade_analysis = trade_analysis;

    // 回撤分析
    let drawdown_analysis = compute_drawdown_analysis(equity_curve);
    // 保留已经在 run_backtest 中计算的 max_drawdown_ratio
    summary.drawdown_analysis.max_drawdown_duration_days =
        drawdown_analysis.max_drawdown_duration_days;
    summary.drawdown_analysis.avg_drawdown_duration_days =
        drawdown_analysis.avg_drawdown_duration_days;

    // 基准对比
    if benchmark_curve.len() >= 2 {
        let benchmark_returns = compute_step_returns(benchmark_curve);
        let benchmark_total_return = compute_total_return(benchmark_curve);
        let (alpha, beta) =
            compute_alpha_beta(&daily_returns, &benchmark_returns, annualized_return);
        let ir = information_ratio(&daily_returns, &benchmark_returns);

        summary.benchmark_comparison = Some(BacktestBenchmarkComparison {
            benchmark_total_return,
            alpha,
            beta,
            information_ratio: ir,
        });
    }

    // v1.1.0 P2: VaR / CVaR
    let (var_95, cvar_95) = compute_var_cvar(&daily_returns);
    summary.risk_adjusted.var_95 = var_95;
    summary.risk_adjusted.cvar_95 = cvar_95;

    // v1.1.0 P2: 偏度 / 峰度
    let (skew, kurt) = compute_skewness_kurtosis(&daily_returns);
    summary.skewness = skew;
    summary.kurtosis = kurt;
}

pub fn equity_curve_monotonicity_diagnostics(equity_curve: &[BacktestEquityPoint]) -> Vec<String> {
    equity_curve
        .windows(2)
        .enumerate()
        .filter_map(|(index, window)| {
            let left = &window[0];
            let right = &window[1];
            (right.ts_ms < left.ts_ms).then(|| {
                format!(
                    "{QPRT_EQUITY_CURVE_NON_MONOTONIC}: equity_curve timestamp regressed at index {} ({} -> {})",
                    index + 1,
                    left.ts_ms,
                    right.ts_ms
                )
            })
        })
        .collect()
}

/// 从毫秒时间戳计算年月字符串 (确定性的，无外部库依赖)
fn epoch_ms_to_year_month(ts_ms: u64) -> String {
    let total_days = (ts_ms / 86_400_000).min(i64::MAX as u64) as i64;
    // 从 1970-01-01 逐月推进
    let mut remaining = total_days;
    let mut year: i64 = 1970;
    let mut month: i64 = 1;
    loop {
        let days_in_month = days_in_month(year, month);
        if remaining < days_in_month {
            break;
        }
        remaining -= days_in_month;
        month += 1;
        if month > 12 {
            month = 1;
            year += 1;
        }
    }
    format!("{:04}-{:02}", year, month)
}

fn days_in_month(year: i64, month: i64) -> i64 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if is_leap(year) {
                29
            } else {
                28
            }
        }
        _ => 30,
    }
}

fn is_leap(year: i64) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

/// v1.1.0 P2: 计算月度/季度/年度收益率分解
pub fn compute_period_returns(
    equity_curve: &[BacktestEquityPoint],
) -> Vec<qrpc_core::PeriodReturn> {
    if equity_curve.len() < 2 {
        return Vec::new();
    }

    let mut periods: std::collections::BTreeMap<String, (f64, f64, u32)> =
        std::collections::BTreeMap::new();
    // key: "YYYY-MM", value: (start_equity, end_equity, trade_count)

    for window in equity_curve.windows(2) {
        let ts_ms = window[1].ts_ms;
        let period = epoch_ms_to_year_month(ts_ms);

        let entry = periods
            .entry(period)
            .or_insert((window[0].equity, window[1].equity, 0));
        entry.1 = window[1].equity; // 更新为该月最新的权益
        entry.2 += 1; // 该月内的 step 数
    }

    periods
        .into_iter()
        .map(|(period, (start, end, steps))| qrpc_core::PeriodReturn {
            period,
            return_ratio: if start.is_finite() && start > 0.0 {
                (end - start) / start
            } else {
                0.0
            },
            trade_count: steps,
        })
        .collect()
}

/// 返回 equity_curve 连续点间的收益率。
/// 注意：返回值的单位取决于 equity_curve 的时间间隔（日/小时/周等）。
/// 调用方负责根据实际时间跨度进行年化。
fn compute_step_returns(equity_curve: &[BacktestEquityPoint]) -> Vec<f64> {
    let mut returns = Vec::with_capacity(equity_curve.len().saturating_sub(1));
    for window in equity_curve.windows(2) {
        let prev = window[0].equity.max(f64::MIN_POSITIVE);
        let curr = window[1].equity;
        // v2.4.0 P1-C2: 拒绝 NaN/Inf 毒化下游指标
        if !curr.is_finite() {
            return Vec::new();
        }
        returns.push((curr - prev) / prev);
    }
    returns
}

fn compute_total_return(equity_curve: &[BacktestEquityPoint]) -> f64 {
    let first = equity_curve
        .first()
        .map(|p| p.equity)
        .unwrap_or(1.0)
        .max(f64::MIN_POSITIVE);
    let last = equity_curve.last().map(|p| p.equity).unwrap_or(first);
    (last - first) / first
}

fn std_deviation(values: &[f64]) -> f64 {
    let n = values.len() as f64;
    if n < 2.0 {
        return 0.0;
    }
    let mean = values.iter().sum::<f64>() / n;
    let variance = values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / (n - 1.0);
    variance.sqrt()
}

/// 从成交记录计算交易分析指标
fn compute_trade_analysis(
    sessions: &[SessionOutput],
    _initial_equity: f64,
) -> BacktestTradeAnalysis {
    // 从所有 session 的 fill_reports 提取逐笔 PnL
    struct Trade {
        pnl: f64,
    }

    // v2.3.0: 不再使用仅手续费的伪 PnL 回退。若无持仓级已实现盈亏, 返回默认空分析
    let trades: Vec<Trade> = Vec::new();

    // v1.1.13: 使用最终session的持仓级PnL，避免跨session重复累计
    let mut realized_pnls = Vec::new();
    if let Some(last_session) = sessions.last() {
        for position in &last_session.final_portfolio.positions {
            let pnl = position.realized_pnl;
            if pnl.abs() > 1e-9 {
                realized_pnls.push(pnl);
            }
        }
    }

    if realized_pnls.is_empty() && trades.is_empty() {
        return BacktestTradeAnalysis::default();
    }

    let pnls: Vec<f64> = if !realized_pnls.is_empty() {
        realized_pnls
    } else {
        trades.iter().map(|t| t.pnl).collect()
    };

    let wins: Vec<f64> = pnls.iter().filter(|&&p| p > 0.0).copied().collect();
    let losses: Vec<f64> = pnls.iter().filter(|&&p| p < 0.0).copied().collect();

    let total_profit: f64 = wins.iter().sum();
    let total_loss: f64 = losses.iter().map(|l| l.abs()).sum();

    let profit_factor = if total_loss.is_finite() && total_loss > 0.0 {
        total_profit / total_loss
    } else if total_profit > 0.0 {
        f64::INFINITY
    } else {
        0.0
    };

    let avg_win = if !wins.is_empty() {
        total_profit / wins.len() as f64
    } else {
        0.0
    };
    let avg_loss = if !losses.is_empty() {
        total_loss / losses.len() as f64
    } else {
        0.0
    };

    let mut max_consecutive_wins: u32 = 0;
    let mut max_consecutive_losses: u32 = 0;
    let mut current_streak: u32 = 0;
    let mut current_is_win: Option<bool> = None;
    for &pnl in &pnls {
        // v1.1.1: 零 PnL 不打断连胜/连亏
        if pnl.abs() < 1e-9 {
            continue;
        }
        let is_win = pnl > 0.0;
        match current_is_win {
            Some(prev) if prev == is_win => current_streak += 1,
            _ => {
                match current_is_win {
                    Some(true) => max_consecutive_wins = max_consecutive_wins.max(current_streak),
                    Some(false) => {
                        max_consecutive_losses = max_consecutive_losses.max(current_streak)
                    }
                    None => {}
                }
                current_streak = 1;
                current_is_win = Some(is_win);
            }
        }
    }
    match current_is_win {
        Some(true) => max_consecutive_wins = max_consecutive_wins.max(current_streak),
        Some(false) => max_consecutive_losses = max_consecutive_losses.max(current_streak),
        None => {}
    }

    BacktestTradeAnalysis {
        profit_factor,
        avg_win,
        avg_loss,
        max_consecutive_wins,
        max_consecutive_losses,
    }
}

/// 计算回撤持续天数
fn compute_drawdown_analysis(equity_curve: &[BacktestEquityPoint]) -> BacktestDrawdownAnalysis {
    if equity_curve.len() < 2 {
        return BacktestDrawdownAnalysis::default();
    }

    let mut max_drawdown_ratio = 0.0f64;
    let mut max_drawdown_duration_days = 0.0f64;
    let mut peak_equity = equity_curve[0].equity;
    let mut drawdown_start_ms: Option<u64> = None;
    let mut drawdown_durations = Vec::new();

    for point in equity_curve.iter().skip(1) {
        if point.equity > peak_equity {
            // 创新高：结束当前回撤
            if let Some(start_ms) = drawdown_start_ms {
                let duration_days = (point.ts_ms.saturating_sub(start_ms)) as f64 / 86_400_000.0;
                drawdown_durations.push(duration_days);
                max_drawdown_duration_days = max_drawdown_duration_days.max(duration_days);
                drawdown_start_ms = None;
            }
            peak_equity = point.equity;
        } else {
            let dd = (peak_equity - point.equity) / peak_equity;
            max_drawdown_ratio = max_drawdown_ratio.max(dd);
            if drawdown_start_ms.is_none() && dd > 0.001 {
                drawdown_start_ms = Some(point.ts_ms);
            }
        }
    }

    // v1.1.0 fix: 回测结束时仍在回撤中，用最后一个点计算持续时间
    if let Some(start_ms) = drawdown_start_ms {
        if let Some(last_point) = equity_curve.last() {
            let duration_days = (last_point.ts_ms.saturating_sub(start_ms)) as f64 / 86_400_000.0;
            drawdown_durations.push(duration_days);
            max_drawdown_duration_days = max_drawdown_duration_days.max(duration_days);
        }
    }

    let avg_drawdown_duration_days = if !drawdown_durations.is_empty() {
        drawdown_durations.iter().sum::<f64>() / drawdown_durations.len() as f64
    } else {
        0.0
    };

    BacktestDrawdownAnalysis {
        max_drawdown_ratio,
        max_drawdown_duration_days,
        avg_drawdown_duration_days,
    }
}

/// 计算 Alpha 和 Beta（线性回归: R_strategy = alpha + beta * R_benchmark）
fn compute_alpha_beta(
    strategy_returns: &[f64],
    benchmark_returns: &[f64],
    annualized_return: f64,
) -> (f64, f64) {
    let n = strategy_returns.len().min(benchmark_returns.len());
    if n < 2 {
        return (0.0, 1.0);
    }

    let sr: Vec<f64> = strategy_returns[..n].to_vec();
    let br: Vec<f64> = benchmark_returns[..n].to_vec();

    let sr_mean = sr.iter().sum::<f64>() / n as f64;
    let br_mean = br.iter().sum::<f64>() / n as f64;

    let covariance: f64 = sr
        .iter()
        .zip(br.iter())
        .map(|(s, b)| (s - sr_mean) * (b - br_mean))
        .sum::<f64>()
        / (n - 1) as f64;
    let bench_variance: f64 =
        br.iter().map(|b| (b - br_mean).powi(2)).sum::<f64>() / (n - 1) as f64;

    let beta = if bench_variance.is_finite() && bench_variance > 0.0 {
        covariance / bench_variance
    } else {
        1.0
    };
    // 年化 Alpha
    let alpha = annualized_return - beta * (compute_annualized_from_daily(&br));

    (alpha, beta)
}

fn compute_annualized_from_daily(daily_returns: &[f64]) -> f64 {
    let daily_mean = daily_returns.iter().sum::<f64>() / daily_returns.len().max(1) as f64;
    // 简单的年化：日收益率均值 × 365（不复合）
    daily_mean * 365.0
}

/// v1.1.0 P2: VaR 和 CVaR (历史模拟法, 95% 置信度)
fn compute_var_cvar(daily_returns: &[f64]) -> (f64, f64) {
    if daily_returns.len() < 20 {
        return (0.0, 0.0);
    }
    let mut sorted: Vec<f64> = daily_returns.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    // 5% 分位数 = 第 5 个百分位
    let idx = ((sorted.len() as f64 * 0.05).ceil() as usize).saturating_sub(1);
    let var_95 = sorted[idx.min(sorted.len().saturating_sub(1))]; // 负值表示损失
                                                                  // CVaR = 低于 VaR 的所有收益的平均值
    let tail: Vec<f64> = sorted.iter().take(idx + 1).copied().collect();
    let cvar_95 = if tail.is_empty() {
        var_95
    } else {
        tail.iter().sum::<f64>() / tail.len() as f64
    };
    (var_95, cvar_95)
}

/// v1.1.0 P2: 偏度与峰度
fn compute_skewness_kurtosis(daily_returns: &[f64]) -> (f64, f64) {
    let n = daily_returns.len();
    if n < 12 {
        return (0.0, 0.0);
    }
    let mean = daily_returns.iter().sum::<f64>() / n as f64;
    let variance: f64 = daily_returns
        .iter()
        .map(|r| (r - mean).powi(2))
        .sum::<f64>()
        / (n - 1) as f64;
    let std = variance.sqrt();
    if std < 1e-12 {
        return (0.0, 0.0);
    }
    // 偏度: E[(X-μ)³] / σ³
    let skewness = daily_returns
        .iter()
        .map(|r| ((r - mean) / std).powi(3))
        .sum::<f64>()
        * n as f64
        / ((n - 1) * (n - 2)) as f64;
    // 峰度(超额): E[(X-μ)⁴] / σ⁴ - 3
    let kurtosis = daily_returns
        .iter()
        .map(|r| ((r - mean) / std).powi(4))
        .sum::<f64>()
        * n as f64
        * (n as f64 + 1.0)
        / ((n - 1) * (n - 2) * (n - 3)) as f64
        - 3.0 * (n - 1) as f64 * (n - 1) as f64 / ((n - 2) * (n - 3)) as f64;
    (skewness, kurtosis.max(-3.0))
}

/// 信息比率 = mean(R_s - R_b) / std(R_s - R_b) 年化
fn information_ratio(strategy_returns: &[f64], benchmark_returns: &[f64]) -> f64 {
    let n = strategy_returns.len().min(benchmark_returns.len());
    if n < 2 {
        return 0.0;
    }
    let diffs: Vec<f64> = strategy_returns[..n]
        .iter()
        .zip(benchmark_returns[..n].iter())
        .map(|(s, b)| s - b)
        .collect();
    let mean_diff = diffs.iter().sum::<f64>() / n as f64;
    let std_diff = std_deviation(&diffs);
    if std_diff.is_finite() && std_diff > 0.0 {
        mean_diff * 252.0_f64.sqrt() / std_diff
    } else {
        0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use qrpc_core::BacktestSummary;

    #[test]
    fn compute_metrics_populates_risk_adjusted() {
        // 100 天稳步上涨: 权益从 100k 到 120k
        let day_ms = 86_400_000;
        let mut equity = Vec::with_capacity(101);
        let mut total_return = 0.0;
        for i in 0..=100 {
            let eq = 100_000.0 * (1.0 + 0.00182_f64).powf(i as f64);
            if i == 100 {
                total_return = (eq - 100_000.0) / 100_000.0;
            }
            equity.push(BacktestEquityPoint {
                ts_ms: i as u64 * day_ms,
                equity: eq,
                cash_balance: eq,
                net_notional: 0.0,
            });
        }
        let mut summary = BacktestSummary {
            step_count: 101,
            trade_count: 20,
            total_return_ratio: total_return,
            final_equity: equity.last().unwrap().equity,
            net_profit: equity.last().unwrap().equity - 100_000.0,
            win_rate: 0.55,
            annualized_return: 0.0,
            annualized_volatility: 0.0,
            risk_adjusted: Default::default(),
            trade_analysis: Default::default(),
            drawdown_analysis: BacktestDrawdownAnalysis {
                max_drawdown_ratio: 0.0,
                ..Default::default()
            },
            benchmark_comparison: None,
            skewness: 0.0,
            kurtosis: 0.0,
        };

        compute_backtest_metrics(&mut summary, &[], &equity, &[]);

        assert!(
            summary.risk_adjusted.sharpe_ratio > 0.0,
            "夏普比率应为正: {}",
            summary.risk_adjusted.sharpe_ratio
        );
        // sortino 在单调上涨时可能为 0（无下行波动），这是正确的
        assert!(summary.annualized_return > 0.0);
        assert!(summary.annualized_volatility > 0.0);
    }

    #[test]
    fn benchmark_comparison_computes_alpha_beta() {
        let day_ms = 86_400_000;
        let mut strategy_equity = Vec::with_capacity(31);
        let mut benchmark_equity = Vec::with_capacity(31);
        for i in 0..=30 {
            let ts = i as u64 * day_ms;
            let seq = 100_000.0 * (1.0 + 0.0046_f64).powf(i as f64);
            strategy_equity.push(BacktestEquityPoint {
                ts_ms: ts,
                equity: seq,
                cash_balance: seq,
                net_notional: 0.0,
            });
            let beq = 100_000.0 * (1.0 + 0.0019_f64).powf(i as f64);
            benchmark_equity.push(BacktestEquityPoint {
                ts_ms: ts,
                equity: beq,
                cash_balance: beq,
                net_notional: 0.0,
            });
        }
        let total_return = (strategy_equity.last().unwrap().equity - 100_000.0) / 100_000.0;
        let mut summary = BacktestSummary {
            step_count: 31,
            trade_count: 15,
            total_return_ratio: total_return,
            final_equity: strategy_equity.last().unwrap().equity,
            net_profit: strategy_equity.last().unwrap().equity - 100_000.0,
            win_rate: 0.6,
            annualized_return: 0.0,
            annualized_volatility: 0.0,
            risk_adjusted: Default::default(),
            trade_analysis: Default::default(),
            drawdown_analysis: BacktestDrawdownAnalysis {
                max_drawdown_ratio: 0.01,
                ..Default::default()
            },
            benchmark_comparison: None,
            skewness: 0.0,
            kurtosis: 0.0,
        };

        compute_backtest_metrics(&mut summary, &[], &strategy_equity, &benchmark_equity);

        let bc = summary.benchmark_comparison.expect("应有基准对比数据");
        assert!(
            bc.benchmark_total_return > 0.0,
            "基准收益应为正: {}",
            bc.benchmark_total_return
        );
        // 策略跑赢基准，Alpha 应为正
        assert!(bc.alpha > 0.0, "跑赢基准时 Alpha 应为正: {}", bc.alpha);
        assert!(bc.information_ratio > 0.0, "跑赢基准时 IR 应为正");
    }

    #[test]
    fn drawdown_analysis_detects_recovery() {
        let equity: Vec<BacktestEquityPoint> = [
            (0, 100_000.0),
            (86_400_000, 95_000.0),
            (172_800_000, 92_000.0),
            (259_200_000, 98_000.0),
            (345_600_000, 102_000.0),
        ]
        .iter()
        .map(|(ts, eq)| BacktestEquityPoint {
            ts_ms: *ts,
            equity: *eq,
            cash_balance: *eq,
            net_notional: 0.0,
        })
        .collect();
        let result = compute_drawdown_analysis(&equity);
        assert!(result.max_drawdown_ratio >= 0.07);
        // 回撤持续 ≈ 3 天 (从 day 1 到 day 3 开始恢复)
        assert!(result.max_drawdown_duration_days > 0.0);
    }

    #[test]
    fn empty_input_does_not_panic() {
        let mut summary = BacktestSummary {
            step_count: 0,
            trade_count: 0,
            total_return_ratio: 0.0,
            final_equity: 0.0,
            net_profit: 0.0,
            win_rate: 0.0,
            annualized_return: 0.0,
            annualized_volatility: 0.0,
            risk_adjusted: Default::default(),
            trade_analysis: Default::default(),
            drawdown_analysis: Default::default(),
            benchmark_comparison: None,
            skewness: 0.0,
            kurtosis: 0.0,
        };
        compute_backtest_metrics(&mut summary, &[], &[], &[]);
        // 不 panic 即通过
        assert_eq!(summary.risk_adjusted.sharpe_ratio, 0.0);
    }

    #[test]
    fn non_monotonic_equity_curve_reports_diagnostic_and_does_not_panic() {
        let equity: Vec<BacktestEquityPoint> =
            [(2000, 100_000.0), (1000, 99_000.0), (3000, 101_000.0)]
                .iter()
                .map(|(ts, eq)| BacktestEquityPoint {
                    ts_ms: *ts,
                    equity: *eq,
                    cash_balance: *eq,
                    net_notional: 0.0,
                })
                .collect();
        let mut summary = BacktestSummary {
            step_count: 3,
            trade_count: 0,
            total_return_ratio: 0.01,
            final_equity: 101_000.0,
            net_profit: 1_000.0,
            win_rate: 0.0,
            annualized_return: 0.0,
            annualized_volatility: 0.0,
            risk_adjusted: Default::default(),
            trade_analysis: Default::default(),
            drawdown_analysis: Default::default(),
            benchmark_comparison: None,
            skewness: 0.0,
            kurtosis: 0.0,
        };

        compute_backtest_metrics(&mut summary, &[], &equity, &[]);
        let diagnostics = equity_curve_monotonicity_diagnostics(&equity);
        assert_eq!(diagnostics.len(), 1);
        assert!(diagnostics[0].contains(QPRT_EQUITY_CURVE_NON_MONOTONIC));
    }
}
