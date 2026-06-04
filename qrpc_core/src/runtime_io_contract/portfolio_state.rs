use serde::{Deserialize, Serialize};

use super::OpenOrder;
use crate::{Exchange, Symbol};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub exchange: Exchange,
    pub symbol: Symbol,
    pub net_qty: f64,
    pub frozen_qty: f64,
    pub avg_entry_price: f64,
    pub mark_price: f64,
    pub unrealized_pnl: f64,
    pub realized_pnl: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExchangeExposure {
    pub exchange: Exchange,
    pub gross_notional: f64,
    pub net_notional: f64,
    pub leverage: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PortfolioState {
    pub cash_balance: f64,
    pub available_cash_balance: f64,
    pub frozen_cash_balance: f64,
    pub open_orders: Vec<OpenOrder>,
    pub positions: Vec<Position>,
    pub exchange_exposures: Vec<ExchangeExposure>,
    pub total_gross_notional: f64,
    pub total_net_notional: f64,
    pub total_leverage: f64,
    pub updated_at_ms: u64,
}

impl PortfolioState {
    pub fn new(initial_cash_balance: f64, ts_ms: u64) -> Self {
        Self {
            cash_balance: initial_cash_balance,
            available_cash_balance: initial_cash_balance,
            frozen_cash_balance: 0.0,
            open_orders: Vec::new(),
            positions: Vec::new(),
            exchange_exposures: Vec::new(),
            total_gross_notional: 0.0,
            total_net_notional: 0.0,
            total_leverage: 0.0,
            updated_at_ms: ts_ms,
        }
    }

    /// v2.5.0: debug 模式下验证跨字段一致性
    pub fn debug_assert_invariants(&self) {
        if cfg!(debug_assertions) {
            // 可用现金 + 冻结现金 = 总现金
            debug_assert!(
                (self.available_cash_balance + self.frozen_cash_balance - self.cash_balance).abs()
                    < 0.01,
                "PortfolioState: available({}) + frozen({}) != cash({})",
                self.available_cash_balance,
                self.frozen_cash_balance,
                self.cash_balance
            );
            // 杠杆为非负
            debug_assert!(self.total_leverage >= 0.0, "total_leverage 不能为负");
            // 余额非负
            debug_assert!(self.cash_balance >= 0.0, "cash_balance 不能为负");
        }
    }
}
