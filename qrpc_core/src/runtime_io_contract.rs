mod decision_flow;
mod execution_io;
mod market_data_io;
mod portfolio_state;
mod runtime_output;

pub use decision_flow::*;
pub use execution_io::*;
pub use market_data_io::*;
pub use portfolio_state::*;
pub use runtime_output::*;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BacktestEquityPoint {
    pub ts_ms: u64,
    pub equity: f64,
    pub cash_balance: f64,
    pub net_notional: f64,
}

/// v1.1.0: 风险调整收益指标组
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BacktestRiskAdjusted {
    #[serde(default)]
    pub sharpe_ratio: f64,
    #[serde(default)]
    pub sortino_ratio: f64,
    #[serde(default)]
    pub calmar_ratio: f64,
    /// v1.1.0 P2: 95% VaR (日收益率, 历史模拟法)
    #[serde(default)]
    pub var_95: f64,
    /// v1.1.0 P2: 95% CVaR / Expected Shortfall
    #[serde(default)]
    pub cvar_95: f64,
}

/// v1.1.0: 交易分析指标组
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BacktestTradeAnalysis {
    #[serde(default)]
    pub profit_factor: f64,
    #[serde(default)]
    pub avg_win: f64,
    #[serde(default)]
    pub avg_loss: f64,
    #[serde(default)]
    pub max_consecutive_wins: u32,
    #[serde(default)]
    pub max_consecutive_losses: u32,
}

/// v1.1.0: 回撤分析指标组
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BacktestDrawdownAnalysis {
    #[serde(default)]
    pub max_drawdown_ratio: f64,
    #[serde(default)]
    pub max_drawdown_duration_days: f64,
    #[serde(default)]
    pub avg_drawdown_duration_days: f64,
}

/// v1.1.0: 基准比较指标组 — None 表示基准未启用
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BacktestBenchmarkComparison {
    #[serde(default)]
    pub benchmark_total_return: f64,
    #[serde(default)]
    pub alpha: f64,
    #[serde(default)]
    pub beta: f64,
    #[serde(default)]
    pub information_ratio: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BacktestSummary {
    pub step_count: usize,
    pub trade_count: usize,
    pub total_return_ratio: f64,
    pub final_equity: f64,
    #[serde(default)]
    pub net_profit: f64,
    #[serde(default)]
    pub win_rate: f64,
    #[serde(default)]
    pub annualized_return: f64,
    #[serde(default)]
    pub annualized_volatility: f64,
    #[serde(default)]
    pub risk_adjusted: BacktestRiskAdjusted,
    #[serde(default)]
    pub trade_analysis: BacktestTradeAnalysis,
    #[serde(default)]
    pub drawdown_analysis: BacktestDrawdownAnalysis,
    #[serde(default)]
    pub benchmark_comparison: Option<BacktestBenchmarkComparison>,
    /// v1.1.0 P2: 日收益率偏度 (正偏 = 多小亏少大赚)
    #[serde(default)]
    pub skewness: f64,
    /// v1.1.0 P2: 日收益率峰度 (高值 = 肥尾风险)
    #[serde(default)]
    pub kurtosis: f64,
}

/// v1.1.0 P2: 月度/季度收益率分解
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeriodReturn {
    pub period: String, // e.g. "2025-01", "2025-Q1", "2025"
    pub return_ratio: f64,
    pub trade_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BacktestOutput {
    pub mode: String,
    pub started_at_ms: u64,
    pub ended_at_ms: u64,
    /// v1.3.4: 回测实际墙钟耗时（毫秒）
    #[serde(default)]
    pub elapsed_ms: Option<u64>,
    pub sessions: Vec<SessionOutput>,
    pub equity_curve: Vec<BacktestEquityPoint>,
    /// v1.1.0: 买入持有基准权益曲线（等权重持有全部交易标的）
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub benchmark_equity_curve: Vec<BacktestEquityPoint>,
    /// v1.1.0 P2: 月度/季度/年度收益率分解
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub period_returns: Vec<PeriodReturn>,
    pub summary: BacktestSummary,
    pub final_portfolio: PortfolioState,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub v4_artifact: Option<qrpc_core_ir::v4::V4BacktestArtifact>,
    #[serde(default)]
    pub debug_values: Option<Vec<std::collections::BTreeMap<String, f64>>>,
}
