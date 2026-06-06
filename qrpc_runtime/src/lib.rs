mod agent_module;
pub mod backtest_metrics;
pub mod circuit_breaker;
mod compat;
mod config_tracker;
mod core_ir_evaluator;
pub(crate) mod data_module;
mod execution_module;
mod fill_engine;
pub mod hotswap;
mod intent_module;
pub mod live_execution;
mod merge;
mod merge_coordinator;
pub mod plugin_market;
mod plugin_runtime_registry;
pub mod plugin_sandbox;
mod reconcile;
mod risk_checker;
pub mod risk_monitor;
mod runtime_state;
mod sandbox;
pub mod slippage;
pub mod v4_runtime;

mod runtime_facade_coordinator;

pub use slippage::{
    compute_fill_price, estimate_spread, spread_from_quote, ExecutionAssumptions,
    ExtendedMarketState, LatencyModel, MarketImpactModel, SlippageModel, SpreadEstimate,
    SpreadEstimateSource,
};

pub use agent_module::{
    AgentEvaluationOutput, AgentEvaluationRequest, AgentModuleProvider, BuiltinAgentModule,
};
pub use core_ir_evaluator::{
    evaluate_indicator_signal, CoreIrIndicatorEvaluation, CoreIrIndicatorEvaluatorError,
};
/// v1.1.x: 设置 Mock 数据生成器的波动率(0=使用默认1.5%)
pub fn set_mock_volatility(vol: f64) {
    data_module::MOCK_VOLATILITY.store(vol.to_bits(), std::sync::atomic::Ordering::Relaxed);
}

pub use compat::{
    CompatibilityChecker, CompatibilityContext, CompatibilityReport, CompatibilityVerdict,
    ModuleSurface,
};
pub use data_module::MOCK_VOLATILITY;
pub use data_module::{
    BuiltinDataModule, DataCollectionOutput, DataCollectionRequest, DataModuleProvider,
};
pub use execution_module::{
    BuiltinExecutionModule, ExecutionModuleProvider, ExecutionPlanner, ExecutionPlanningOutput,
    ExecutionPlanningRequest, ExecutionSubmitter,
};
pub use hotswap::{
    DefaultHotSwapValidator, HotSwapModuleTarget, HotSwapOrchestrator, HotSwapRequest,
    HotSwapResult, HotSwapState, HotSwapStep, HotSwapValidationResult, HotSwapValidator,
};
pub use intent_module::{
    BuiltinIntentModule, IntentEvaluationOutput, IntentEvaluationRequest, IntentModuleProvider,
};
pub use merge::{
    MergeDecisionRecord, MergePolicy, MergedOutput, StrategyInput, StrategyMergeEngine,
};
pub use plugin_market::{MarketMetadata, PluginMarketClient, PluginSummary};
pub use plugin_runtime_registry::{
    PluginLifecycleState, PluginSecurityAction, RuntimePluginLifecycle, RuntimePluginRegistry,
};
pub use reconcile::{
    OrderReconciler, ReconcileStrategy, ReconciliationDiscrepancy, ReconciliationResult,
};
pub use risk_checker::{RiskCheckOutput, RiskCheckRequest, RiskChecker, RiskCheckerProvider};
pub use runtime_facade_coordinator::{ConfigGenerationEntry, RuntimeCoordinator};
pub use sandbox::{
    build_v4_deterministic_replay_bars, runtime_support_boundary,
    sort_v4_replay_ticks_deterministically, DeterministicClockMode, DeterministicEventOrdering,
    DeterministicParallelismPolicy, DeterministicTestMode, FastBacktestSandbox, RealTimeSandbox,
    RuntimeSupportBoundary, Sandbox, SandboxMode, SandboxSnapshot,
    SUPPORTED_RUNTIME_EXECUTION_MODULE_KEYS, SUPPORTED_RUNTIME_MODE_KEYS,
};
pub use v4_runtime::{
    expand_v4_graph_for_symbols, normalize_v4_backtest_symbols, V4BacktestBarInput,
    V4BacktestTickInput, V4CachedMachineOutput, V4ExecutionCapabilityRuntimeEntry,
    V4ExecutionCapabilityRuntimeStatus, V4ExecutionRuntimeDecision, V4ExecutionRuntimeSnapshot,
    V4MachineRuntimeSnapshot, V4MachineRuntimeStatus, V4PaperSimulatedRunOutput,
    V4PaperSimulatedRuntime, V4RiskPlaneRuntimeDecision, V4RiskPlaneRuntimeSnapshot, V4Runtime,
    V4RuntimeEventEnvelope, V4RuntimeEventOrigin, V4RuntimeInputEvent, V4RuntimeMemorySnapshot,
    V4SimulatedAssetPoint, V4SimulatedExecutionConfig, V4SimulatedExecutionSnapshot,
    V4SimulatedFill, V4SimulatedOrder, V4SimulatedOrderRequest, V4SimulatedOrderSide,
    V4SimulatedOrderStatus, V4SimulatedOrderType, V4SimulatedPosition, V4SimulatedPositionAction,
    V4SimulatedTimeInForce, V4VenueAdapterRuntimeBoundary, EVENT_EXECUTION_FEE_CHARGED,
    EVENT_EXECUTION_ORDER_ACKNOWLEDGED, EVENT_EXECUTION_ORDER_AMENDED,
    EVENT_EXECUTION_ORDER_CANCELED, EVENT_EXECUTION_ORDER_EXPIRED, EVENT_EXECUTION_ORDER_FILLED,
    EVENT_EXECUTION_ORDER_PARTIALLY_FILLED, EVENT_EXECUTION_ORDER_REJECTED,
    EVENT_EXECUTION_PORTFOLIO_CHANGED, V4_DEFAULT_MARKET_DATA_SOURCE,
};
