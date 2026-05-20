use qrpc_core::{Exchange, OrderSide, OrderType, SimOrder};
use serde::{Deserialize, Serialize};

/// 滑点模型 — 决定成交价偏离市价的程度
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum SlippageModel {
    /// 固定 bps（v1.0.7 现有行为）
    FixedBps { bps: f64 },
    /// 波动率缩放: slippage = base_bps + vol_coefficient * volatility_bps
    VolatilityScaled {
        base_bps: f64,
        vol_coefficient: f64,
    },
    /// 订单簿深度: 滑点 ∝ order_qty / estimated_depth
    OrderBookDepth {
        tick_size: f64,
        depth_factor: f64,
    },
}

impl Default for SlippageModel {
    fn default() -> Self {
        Self::FixedBps { bps: 5.0 }
    }
}

/// 市场冲击模型 — 决定自身交易对市价的永久影响
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum MarketImpactModel {
    /// 无冲击（v1.0.7 现有行为）
    None,
    /// 平方根模型: σ * sqrt(Q/V) * η
    SquareRoot { eta: f64 },
}

impl Default for MarketImpactModel {
    fn default() -> Self {
        Self::None
    }
}

/// 延迟模型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum LatencyModel {
    /// 固定延迟（v1.0.7 现有行为）
    Fixed { delay_ms: u64 },
    /// 正态分布延迟
    Normal { mean_ms: u64, std_ms: u64 },
    /// 按交易所分别配置
    PerExchange { binance_ms: u64, okx_ms: u64 },
}

impl Default for LatencyModel {
    fn default() -> Self {
        Self::Fixed { delay_ms: 0 }
    }
}

impl LatencyModel {
    pub fn delay_ms(&self, exchange: &Exchange, _seed: u64) -> u64 {
        match self {
            Self::Fixed { delay_ms } => *delay_ms,
            // 随机延迟: 用 seed 作为伪随机源（确定性回测要求）
            Self::Normal { mean_ms, std_ms } => {
                if *std_ms == 0 {
                    *mean_ms
                } else {
                    let r = pseudo_random_normal(_seed);
                    (*mean_ms as f64 + r * *std_ms as f64).max(0.0) as u64
                }
            }
            Self::PerExchange { binance_ms, okx_ms } => match exchange {
                Exchange::Binance => *binance_ms,
                Exchange::Okx => *okx_ms,
            },
        }
    }
}

/// 价差估算来源 — 透明标注价差数据来源
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SpreadEstimateSource {
    /// 来自真实报价数据（Quote）
    RealQuote,
    /// 基于波动率估算
    VolatilityBased,
}

/// 价差估算结果
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SpreadEstimate {
    pub bid: f64,
    pub ask: f64,
    pub mid: f64,
    pub spread_bps: f64,
    pub source: SpreadEstimateSource,
}

/// 执行假设 — 控制 FillEngine 的成交行为
#[derive(Debug, Clone, PartialEq)]
pub struct ExecutionAssumptions {
    pub slippage: SlippageModel,
    pub impact: MarketImpactModel,
    /// 是否尝试使用真实 bid/ask（Quote 数据可用时为 true）
    pub use_bid_ask: bool,
    pub latency: LatencyModel,
    /// v2.1.0: 挂单成交手续费 (bps)，默认 10bps
    pub taker_fee_bps: f64,
}


impl Default for ExecutionAssumptions {
    fn default() -> Self {
        Self {
            slippage: SlippageModel::FixedBps { bps: 5.0 },
            impact: MarketImpactModel::None,
            use_bid_ask: false,
            latency: LatencyModel::Fixed { delay_ms: 0 },
            taker_fee_bps: 10.0,
        }
    }
}

impl ExecutionAssumptions {
    /// v1.0.7 兼容模式: 固定 5bps 滑点，无冲击，无价差，无延迟
    pub fn v1_0_7_compat() -> Self {
        Self {
            slippage: SlippageModel::FixedBps { bps: 5.0 },
            impact: MarketImpactModel::None,
            use_bid_ask: false,
            latency: LatencyModel::Fixed { delay_ms: 0 },
            taker_fee_bps: 10.0,
        }
    }

    /// 标签（用于回测对比页面的执行假设字段）
    pub fn label(&self) -> String {
        let slippage_label = match &self.slippage {
            SlippageModel::FixedBps { bps } => format!("固定滑点 {}bps", bps),
            SlippageModel::VolatilityScaled { base_bps, vol_coefficient } => {
                format!("波动率缩放 {}+{}bps", base_bps, vol_coefficient)
            }
            SlippageModel::OrderBookDepth { tick_size, depth_factor } => {
                format!("盘口深度 tick={} factor={}", tick_size, depth_factor)
            }
        };
        let impact_label = match &self.impact {
            MarketImpactModel::None => "无冲击".to_string(),
            MarketImpactModel::SquareRoot { eta } => format!("平方根 η={}", eta),
        };
        let spread_label = if self.use_bid_ask { "真实价差" } else { "估算价差" };
        let latency_label = match &self.latency {
            LatencyModel::Fixed { delay_ms } => format!("延迟 {}ms", delay_ms),
            LatencyModel::Normal { mean_ms, std_ms: _ } => format!("延迟 ~{}ms", mean_ms),
            LatencyModel::PerExchange { binance_ms, okx_ms } => {
                format!("延迟 B:{}ms O:{}ms", binance_ms, okx_ms)
            }
        };
        format!("{} / {} / {} / {}", slippage_label, impact_label, spread_label, latency_label)
    }
}

/// 从波动率估算买卖价差
///
/// 不使用 OHLC 极值（high/low 是成交价而非报价，波动期会系统性高估价差）。
/// 改用基础价差 + 波动率调整的保守估算。
pub fn estimate_spread(mid_price: f64, volatility: f64, timeframe_minutes: u64) -> SpreadEstimate {
    // v1.2.0: NaN/Inf 守卫，避免价格传播为 NaN
    if !mid_price.is_finite() || !volatility.is_finite() {
        return SpreadEstimate {
            bid: mid_price,
            ask: mid_price,
            mid: mid_price,
            spread_bps: 0.0,
            source: SpreadEstimateSource::VolatilityBased,
        };
    }
    let base_spread_bps = match timeframe_minutes {
        0..=5 => 0.5,       // 分钟线: 极紧
        6..=60 => 1.0,      // 小时线
        61..=240 => 1.5,    // 4 小时线
        _ => 2.0,           // 日线及以上
    };
    // 波动率 2% 为基准，vol_adjustment 限制在 0.5x-3x
    let vol_adjustment = (volatility / 0.02).clamp(0.5, 3.0);
    let spread_bps = base_spread_bps * vol_adjustment;
    let half_spread = if mid_price.is_finite() && mid_price > 0.0 {
        mid_price * spread_bps / 20_000.0
    } else {
        0.0
    };

    SpreadEstimate {
        bid: mid_price - half_spread,
        ask: mid_price + half_spread,
        mid: mid_price,
        spread_bps,
        source: SpreadEstimateSource::VolatilityBased,
    }
}

/// 从真实报价构建价差
pub fn spread_from_quote(bid: f64, ask: f64, mid: f64) -> SpreadEstimate {
    let spread_bps = if mid > 0.0 { (ask - bid) / mid * 10_000.0 } else { 0.0 };
    SpreadEstimate {
        bid,
        ask,
        mid,
        spread_bps,
        source: SpreadEstimateSource::RealQuote,
    }
}

/// 市场快照（扩展原有 MarketState，增加价差信息）
#[derive(Debug, Clone)]
pub struct ExtendedMarketState {
    /// 基准市价（mid price）
    pub price: f64,
    /// 买入方报价（用于计算买单成交价）— 买方支付 ask
    pub ask_price: f64,
    /// 卖出方报价（用于计算卖单成交价）— 卖方收到 bid
    pub bid_price: f64,
    pub buy_liquidity: f64,
    pub sell_liquidity: f64,
    /// 价差估算来源
    pub spread_source: SpreadEstimateSource,
}

impl ExtendedMarketState {
    pub fn from_mid_price(
        price: f64,
        buy_liquidity: f64,
        sell_liquidity: f64,
        volatility: f64,
        timeframe_minutes: u64,
    ) -> Self {
        let spread = estimate_spread(price, volatility, timeframe_minutes);
        Self {
            price,
            ask_price: spread.ask,
            bid_price: spread.bid,
            buy_liquidity,
            sell_liquidity,
            spread_source: spread.source,
        }
    }

    pub fn from_quote(bid: f64, ask: f64, buy_liquidity: f64, sell_liquidity: f64) -> Self {
        let mid = (bid + ask) / 2.0;
        Self {
            price: mid,
            ask_price: ask,
            bid_price: bid,
            buy_liquidity,
            sell_liquidity,
            spread_source: SpreadEstimateSource::RealQuote,
        }
    }
}

/// 根据订单方向返回基准成交价（未施加滑点和冲击）
fn base_fill_price(side: &OrderSide, market: &ExtendedMarketState, use_bid_ask: bool) -> f64 {
    if use_bid_ask {
        match side {
            OrderSide::Buy => market.ask_price,
            OrderSide::Sell => market.bid_price,
        }
    } else {
        market.price
    }
}

/// 计算滑点 bps（正值表示不利方向）
fn compute_slippage_bps(
    order: &SimOrder,
    market: &ExtendedMarketState,
    model: &SlippageModel,
    volatility: f64,
) -> f64 {
    match model {
        SlippageModel::FixedBps { bps } => *bps,
        SlippageModel::VolatilityScaled { base_bps, vol_coefficient } => {
            // 年化波动率 → bps
            let vol_bps = volatility * 10_000.0;
            base_bps + vol_coefficient * vol_bps
        }
        SlippageModel::OrderBookDepth { tick_size, depth_factor } => {
            let depth = match order.side {
                OrderSide::Buy => market.buy_liquidity.max(1e-9),
                OrderSide::Sell => market.sell_liquidity.max(1e-9),
            };
            let ratio = (order.quantity / depth).min(1.0);
            let price_impact = ratio * depth_factor * tick_size;
            if market.price > 0.0 {
                price_impact / market.price * 10_000.0
            } else {
                0.0
            }
        }
    }
}

/// 计算市场冲击对价格的永久影响（返回价格调整值，正数表示价格被推高）
fn compute_market_impact(order: &SimOrder, _market: &ExtendedMarketState, model: &MarketImpactModel, volatility: f64) -> f64 {
    match model {
        MarketImpactModel::None => 0.0,
        MarketImpactModel::SquareRoot { eta } => {
            // 简化 Almgren-Chriss: σ * sqrt(Q/V_daily) * η
            // 此处 V_daily 从 market 估算（若无成交量数据，用订单量本身做保守估计）
            let q = order.quantity;
            let sigma_price = volatility * _market.price;
            // 日成交量估算: 用流动性 * 10 作为日成交量近似
            let daily_volume = match order.side {
                OrderSide::Buy => _market.buy_liquidity * 10.0,
                OrderSide::Sell => _market.sell_liquidity * 10.0,
            }
            .max(q);
            let participation = q / daily_volume;
            let impact_ratio = sigma_price * participation.sqrt() * eta;
            match order.side {
                OrderSide::Buy => impact_ratio,       // 买单推高价格
                OrderSide::Sell => -impact_ratio,     // 卖单压低价格
            }
        }
    }
}

/// 计算最终成交价 = 基准价 + 方向性滑点 + 市场冲击
pub fn compute_fill_price(
    order: &SimOrder,
    market: &ExtendedMarketState,
    assumptions: &ExecutionAssumptions,
    volatility: f64,
) -> f64 {
    // v1.1.1: NaN/Inf 守卫 — 腐败市场数据回退到参考价
    if !market.price.is_finite() {
        return order.reference_price;
    }
    let direction = match order.side {
        OrderSide::Buy => 1.0,
        OrderSide::Sell => -1.0,
    };

    // 1. 基准价: bid/ask 或 mid
    let base = base_fill_price(&order.side, market, assumptions.use_bid_ask);

    // 2. 滑点: 不利方向（买更贵、卖更便宜）
    let slippage_bps = compute_slippage_bps(order, market, &assumptions.slippage, volatility);
    let with_slippage = base * (1.0 + direction * slippage_bps / 10_000.0);

    // 3. 市场冲击: 永久性价格影响
    let impact = compute_market_impact(order, market, &assumptions.impact, volatility);

    // 4. Limit 单约束: 买方不高于 limit，卖方不低于 limit
    let filled = with_slippage + impact;
    if let Some(limit) = order.limit_price {
        if matches!(order.order_type, OrderType::Limit | OrderType::StopLossLimit | OrderType::TakeProfitLimit) {
            return match order.side {
                OrderSide::Buy => filled.min(limit),
                OrderSide::Sell => filled.max(limit),
            };
        }
    }

    filled
}

/// 确定性正态分布采样 (Box-Muller 变换, 用于延迟模拟)
///
/// 使用两个 seed 派生值生成 N(0,1) 分布。
/// 相同的 seed 始终产生相同的输出（确定性回放保证）。
fn pseudo_random_normal(seed: u64) -> f64 {
    // 从 seed 派生两个独立值
    let s1 = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
    let s2 = s1.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
    // 映射到 (0, 1] 避免 ln(0)
    let u1 = (s1 as f64) / (u64::MAX as f64) * 0.999 + 0.0005;
    let u2 = (s2 as f64) / (u64::MAX as f64) * 0.999 + 0.0005;
    // Box-Muller: z = sqrt(-2 * ln(u1)) * cos(2 * pi * u2)
    (-2.0_f64 * u1.ln()).sqrt() * (std::f64::consts::TAU * u2).cos()
}

#[cfg(test)]
mod tests {
    use super::*;
    use qrpc_core::{Exchange, OrderSide, OrderType, SimOrder, Symbol, TimeInForce};

    fn sample_buy_order(qty: f64) -> SimOrder {
        SimOrder {
            order_id: "ord-1".into(),
            exchange: Exchange::Binance,
            symbol: Symbol::BtcUsdt,
            side: OrderSide::Buy,
            order_type: OrderType::Market,
            quantity: qty,
            limit_price: None,
            time_in_force: TimeInForce::Gtc,
            allow_partial: false,
            reference_price: 50_000.0,
            slippage_bps: 5.0,
            fee_bps: 10.0,
            strategy_tag: "test".into(),
        }
    }

    fn sample_market() -> ExtendedMarketState {
        ExtendedMarketState {
            price: 50_000.0,
            ask_price: 50_001.0,
            bid_price: 49_999.0,
            buy_liquidity: 10.0,
            sell_liquidity: 10.0,
            spread_source: SpreadEstimateSource::RealQuote,
        }
    }

    #[test]
    fn fixed_bps_slippage_buy_increases_price() {
        let order = sample_buy_order(1.0);
        let market = sample_market();
        let assumptions = ExecutionAssumptions {
            slippage: SlippageModel::FixedBps { bps: 5.0 },
            use_bid_ask: false,
            ..ExecutionAssumptions::v1_0_7_compat()
        };
        let price = compute_fill_price(&order, &market, &assumptions, 0.02);
        // 买方成交价 > mid price（不利方向）
        assert!(price > market.price, "买方成交价应高于 mid: {price} > {}", market.price);
    }

    #[test]
    fn fixed_bps_slippage_sell_decreases_price() {
        let mut order = sample_buy_order(1.0);
        order.side = OrderSide::Sell;
        let market = sample_market();
        let assumptions = ExecutionAssumptions {
            slippage: SlippageModel::FixedBps { bps: 5.0 },
            use_bid_ask: false,
            ..ExecutionAssumptions::v1_0_7_compat()
        };
        let price = compute_fill_price(&order, &market, &assumptions, 0.02);
        // 卖方成交价 < mid price（不利方向）
        assert!(price < market.price, "卖方成交价应低于 mid: {price} < {}", market.price);
    }

    #[test]
    fn bid_ask_pricing_buy_uses_ask() {
        let order = sample_buy_order(1.0);
        let market = sample_market();
        let assumptions = ExecutionAssumptions {
            slippage: SlippageModel::FixedBps { bps: 0.0 }, // 零滑点，隔离价差效果
            use_bid_ask: true,
            ..ExecutionAssumptions::v1_0_7_compat()
        };
        let price = compute_fill_price(&order, &market, &assumptions, 0.02);
        // 无滑点时，买单价 ≈ ask
        assert!((price - market.ask_price).abs() < 0.01,
            "零滑点买单价应等于 ask: {price} ≈ {}", market.ask_price);
    }

    #[test]
    fn bid_ask_pricing_sell_uses_bid() {
        let mut order = sample_buy_order(1.0);
        order.side = OrderSide::Sell;
        let market = sample_market();
        let assumptions = ExecutionAssumptions {
            slippage: SlippageModel::FixedBps { bps: 0.0 },
            use_bid_ask: true,
            ..ExecutionAssumptions::v1_0_7_compat()
        };
        let price = compute_fill_price(&order, &market, &assumptions, 0.02);
        assert!((price - market.bid_price).abs() < 0.01,
            "零滑点卖单价应等于 bid: {price} ≈ {}", market.bid_price);
    }

    #[test]
    fn volatility_scaled_slippage_higher_when_volatile() {
        let order = sample_buy_order(1.0);
        let market = sample_market();
        let assumptions = ExecutionAssumptions {
            slippage: SlippageModel::VolatilityScaled { base_bps: 5.0, vol_coefficient: 2.0 },
            use_bid_ask: false,
            ..ExecutionAssumptions::v1_0_7_compat()
        };
        let low_vol = compute_fill_price(&order, &market, &assumptions, 0.01);  // 1% vol
        let high_vol = compute_fill_price(&order, &market, &assumptions, 0.05); // 5% vol
        // 高波动时滑点更大 → 买得更高
        assert!(high_vol > low_vol,
            "高波动滑点应更大: high({high_vol}) > low({low_vol})");
    }

    #[test]
    fn limit_order_caps_fill_price() {
        let mut order = sample_buy_order(1.0);
        order.order_type = OrderType::Limit;
        order.limit_price = Some(50_100.0);
        let market = ExtendedMarketState {
            price: 50_500.0, // 市价远高于 limit
            ask_price: 50_501.0,
            bid_price: 50_499.0,
            buy_liquidity: 10.0,
            sell_liquidity: 10.0,
            spread_source: SpreadEstimateSource::RealQuote,
        };
        let assumptions = ExecutionAssumptions::v1_0_7_compat();
        let price = compute_fill_price(&order, &market, &assumptions, 0.02);
        // Limit 买单成交价不高于 limit_price
        assert!(price <= order.limit_price.unwrap() + 1e-9,
            "Limit 买单价 {price} 不应超过 limit {}", order.limit_price.unwrap());
    }

    #[test]
    fn estimate_spread_reasonable_range() {
        let spread = estimate_spread(50_000.0, 0.02, 1440);
        // 日线 BTC 典型价差 1-5 bps
        assert!(spread.spread_bps >= 0.5 && spread.spread_bps <= 15.0,
            "日线价差 {:.1} bps 应在合理范围 0.5-15", spread.spread_bps);
        assert!(spread.bid < spread.ask, "bid < ask");
        assert!(spread.ask > spread.mid, "ask > mid");
        assert_eq!(spread.source, SpreadEstimateSource::VolatilityBased);
    }

    #[test]
    fn estimate_spread_tighter_for_shorter_timeframes() {
        let spread_1m = estimate_spread(50_000.0, 0.02, 1);
        let spread_1d = estimate_spread(50_000.0, 0.02, 1440);
        assert!(spread_1m.spread_bps < spread_1d.spread_bps,
            "分钟线价差应小于日线: {:.1} < {:.1}", spread_1m.spread_bps, spread_1d.spread_bps);
    }

    #[test]
    fn execution_assumptions_label_includes_all_components() {
        let assumptions = ExecutionAssumptions::default();
        let label = assumptions.label();
        assert!(label.contains("固定滑点"), "label 应包含滑点信息: {label}");
        assert!(label.contains("无冲击"), "label 应包含冲击信息: {label}");
        assert!(label.contains("估算价差"), "label 应包含价差信息: {label}");
    }

    #[test]
    fn v1_0_7_compat_matches_legacy_behavior() {
        let order = sample_buy_order(1.0);
        let market = sample_market();
        let assumptions = ExecutionAssumptions::v1_0_7_compat();
        // v1.0.7 行为: mid_price * (1 + direction * slippage_bps / 10000)
        let expected = market.price * (1.0 + 5.0 / 10_000.0);
        let actual = compute_fill_price(&order, &market, &assumptions, 0.0);
        assert!((actual - expected).abs() < 0.01,
            "v1.0.7 兼容模式应与旧行为一致: {actual} ≈ {expected}");
    }
}
