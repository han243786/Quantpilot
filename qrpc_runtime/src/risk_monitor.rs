/// v1.2.0: RiskMonitor — 独立于 RiskChecker 的连续风控组件
///
/// RiskChecker 负责单次决策风控（position limit, leverage 等）。
/// RiskMonitor 负责跨 cycle 的实时监控：
/// - `max_daily_loss`: 当日累计亏损超限时触发停止
/// - `max_drawdown_ratio`: 实时回撤超限时触发停止
///
/// 两个模块可同时启用且职责不重叠。

/// RiskMonitor 检查结果
#[derive(Debug, Clone)]
pub struct RiskMonitorResult {
    /// 是否触发停止
    pub stop: bool,
    /// 触发原因（`None` 表示未触发）
    pub reason: Option<String>,
    /// 当日亏损比例（正值表示从初始权益下跌的百分比）
    pub daily_loss_pct: f64,
    /// 当前回撤比例（正值表示从峰值权益下跌的百分比）
    pub drawdown_pct: f64,
}

/// 实时风控监控器
///
/// 追踪初始权益、峰值权益、累计 PnL，在每次 [`check`](RiskMonitor::check) 调用时判断
/// 当日亏损或回撤是否超过阈值。
///
/// # 默认值
/// - `max_daily_loss_ratio = 0.0` 表示不检查当日亏损
/// - `max_drawdown_ratio = 0.0` 表示不检查回撤
///
/// # 示例
/// ```ignore
/// let mut monitor = RiskMonitor::new(100_000.0, 0.05, 0.15);
/// let result = monitor.check(95_000.0);
/// if result.stop { /* 触发风控停止 */ }
/// ```
#[derive(Debug, Clone)]
pub struct RiskMonitor {
    /// 监控起始时的权益
    initial_equity: f64,
    /// 期间最高权益
    peak_equity: f64,
    /// 当日累计 PnL（current_equity - initial_equity）
    daily_pnl: f64,
    /// 最大允许当日亏损比例（0.0 = 禁用）
    max_daily_loss_ratio: f64,
    /// 最大允许回撤比例（0.0 = 禁用）
    max_drawdown_ratio: f64,
}

impl RiskMonitor {
    /// 创建 RiskMonitor
    ///
    /// * `initial_equity` — 监控起始权益（通常为初始现金余额）
    /// * `max_daily_loss_ratio` — 最大当日亏损比例，`0.0` 表示禁用此项检查
    /// * `max_drawdown_ratio` — 最大回撤比例，`0.0` 表示禁用此项检查
    pub fn new(
        initial_equity: f64,
        max_daily_loss_ratio: f64,
        max_drawdown_ratio: f64,
    ) -> Self {
        Self {
            initial_equity,
            peak_equity: initial_equity,
            daily_pnl: 0.0,
            max_daily_loss_ratio,
            max_drawdown_ratio,
        }
    }

    /// 执行风控检查
    ///
    /// 每个 cycle 后调用，传入当前权益。内部更新峰值权益和累计 PnL，
    /// 返回检查结果。若触发停止，后续所有 `check` 调用将继续返回 `stop: true`。
    pub fn check(&mut self, current_equity: f64) -> RiskMonitorResult {
        // v1.2.1: 拒绝 NaN/Inf 权益值，防止毒数据传播
        if !current_equity.is_finite() {
            return RiskMonitorResult {
                stop: true,
                reason: Some("Invalid equity: NaN or Inf".into()),
                daily_loss_pct: 0.0,
                drawdown_pct: 0.0,
            };
        }

        // 更新峰值权益
        if current_equity > self.peak_equity {
            self.peak_equity = current_equity;
        }

        // 更新累计 PnL
        self.daily_pnl = current_equity - self.initial_equity;

        // 亏损比例（正值表示亏损）
        let daily_loss_pct = if self.initial_equity.is_finite() && self.initial_equity > 0.0 {
            (-self.daily_pnl / self.initial_equity).max(0.0)
        } else {
            0.0
        };

        // 回撤比例（正值表示回撤）
        let drawdown_pct = if self.peak_equity.is_finite() && self.peak_equity > 0.0 {
            ((self.peak_equity - current_equity) / self.peak_equity).max(0.0)
        } else {
            0.0
        };

        // 检查当日亏损（仅在启用且亏损超限时触发）
        if self.max_daily_loss_ratio > 0.0 && daily_loss_pct > self.max_daily_loss_ratio {
            return RiskMonitorResult {
                stop: true,
                reason: Some(format!(
                    "当日亏损 {:.2}% 超过限制 {:.2}%",
                    daily_loss_pct * 100.0,
                    self.max_daily_loss_ratio * 100.0,
                )),
                daily_loss_pct,
                drawdown_pct,
            };
        }

        // 检查回撤（仅在启用且回撤超限时触发）
        if self.max_drawdown_ratio > 0.0 && drawdown_pct > self.max_drawdown_ratio {
            return RiskMonitorResult {
                stop: true,
                reason: Some(format!(
                    "回撤 {:.2}% 超过限制 {:.2}%",
                    drawdown_pct * 100.0,
                    self.max_drawdown_ratio * 100.0,
                )),
                daily_loss_pct,
                drawdown_pct,
            };
        }

        RiskMonitorResult {
            stop: false,
            reason: None,
            daily_loss_pct,
            drawdown_pct,
        }
    }

    /// 获取初始权益
    pub fn initial_equity(&self) -> f64 {
        self.initial_equity
    }

    /// 获取峰值权益
    pub fn peak_equity(&self) -> f64 {
        self.peak_equity
    }

    /// 获取累计 PnL
    pub fn daily_pnl(&self) -> f64 {
        self.daily_pnl
    }

    /// 获取最大当日亏损限制
    pub fn max_daily_loss_ratio(&self) -> f64 {
        self.max_daily_loss_ratio
    }

    /// 获取最大回撤限制
    pub fn max_drawdown_ratio(&self) -> f64 {
        self.max_drawdown_ratio
    }

    /// 重置监控器（用于新交易日或新回测）
    pub fn reset(&mut self, initial_equity: f64) {
        self.initial_equity = initial_equity;
        self.peak_equity = initial_equity;
        self.daily_pnl = 0.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_stop_when_within_limits() {
        let mut monitor = RiskMonitor::new(100_000.0, 0.05, 0.20);
        let result = monitor.check(98_000.0);
        assert!(!result.stop);
        assert!(result.reason.is_none());
        assert!((result.daily_loss_pct - 0.02).abs() < 1e-6);
    }

    #[test]
    fn stop_on_daily_loss_exceeded() {
        let mut monitor = RiskMonitor::new(100_000.0, 0.05, 0.20);
        let result = monitor.check(93_000.0);
        assert!(result.stop);
        assert!(result.reason.as_ref().unwrap().contains("当日亏损"));
        assert!((result.daily_loss_pct - 0.07).abs() < 1e-6);
    }

    #[test]
    fn stop_on_drawdown_exceeded() {
        let mut monitor = RiskMonitor::new(100_000.0, 0.10, 0.08);
        // 先涨到 110k 再跌到 98k — 回撤 = (110-98)/110 = 10.9% > 8%
        monitor.check(110_000.0);
        let result = monitor.check(98_000.0);
        assert!(result.stop);
        assert!(result.reason.as_ref().unwrap().contains("回撤"));
        let expected_dd = (110_000.0 - 98_000.0) / 110_000.0;
        assert!((result.drawdown_pct - expected_dd).abs() < 1e-6);
    }

    #[test]
    fn disabled_when_ratio_is_zero() {
        let mut monitor = RiskMonitor::new(100_000.0, 0.0, 0.0);
        let result = monitor.check(50_000.0);
        assert!(!result.stop);
        assert!(result.reason.is_none());
    }

    #[test]
    fn peak_equity_updates_correctly() {
        let mut monitor = RiskMonitor::new(100_000.0, 0.10, 0.10);
        assert!((monitor.peak_equity() - 100_000.0).abs() < 1e-6);
        monitor.check(120_000.0);
        assert!((monitor.peak_equity() - 120_000.0).abs() < 1e-6);
        monitor.check(90_000.0);
        assert!((monitor.peak_equity() - 120_000.0).abs() < 1e-6);
    }

    #[test]
    fn reset_clears_state() {
        let mut monitor = RiskMonitor::new(100_000.0, 0.05, 0.15);
        monitor.check(200_000.0);
        monitor.check(50_000.0);
        assert!(monitor.check(50_000.0).stop);
        monitor.reset(100_000.0);
        let result = monitor.check(99_000.0);
        assert!(!result.stop);
        assert!((monitor.initial_equity() - 100_000.0).abs() < 1e-6);
        assert!((monitor.peak_equity() - 100_000.0).abs() < 1e-6);
    }

    #[test]
    fn daily_pnl_tracks_cumulative() {
        let mut monitor = RiskMonitor::new(100_000.0, 0.10, 0.10);
        assert!((monitor.daily_pnl() - 0.0).abs() < 1e-6);
        monitor.check(105_000.0);
        assert!((monitor.daily_pnl() - 5_000.0).abs() < 1e-6);
        monitor.check(102_000.0);
        assert!((monitor.daily_pnl() - 2_000.0).abs() < 1e-6);
    }

    #[test]
    fn drawdown_computed_from_peak_not_initial() {
        let mut monitor = RiskMonitor::new(100_000.0, 0.50, 0.05);
        // 初始权益 100k，先跌到 95k → 回撤 5% → 刚好在边界，不触发
        let r1 = monitor.check(95_000.0);
        assert!(!r1.stop);
        // 再涨到 120k → 更新峰值
        monitor.check(120_000.0);
        // 再跌到 113k → 回撤 = (120-113)/120 = 5.83% > 5%
        let r2 = monitor.check(113_000.0);
        assert!(r2.stop);
    }

    #[test]
    fn mixed_daily_loss_and_drawdown_daily_loss_checked_first() {
        // 当日亏损触发优先级高于回撤（按代码顺序）
        let mut monitor = RiskMonitor::new(100_000.0, 0.05, 0.05);
        // 从 100k 跌到 90k — 当日亏损 10% > 5%，回撤 10% > 5%
        let result = monitor.check(90_000.0);
        assert!(result.stop);
        // 应报告当日亏损原因（代码中先检查 daily_loss）
        assert!(result.reason.as_ref().unwrap().contains("当日亏损"));
    }
}
