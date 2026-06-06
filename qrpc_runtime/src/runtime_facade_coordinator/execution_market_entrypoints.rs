use super::RuntimeCoordinator;
use anyhow::Result;
use qrpc_core::{ExecutionPlan, NormalizedMarketData};

impl RuntimeCoordinator {
    pub fn submit_execution_plan(
        &mut self,
        plan: &ExecutionPlan,
        normalized_data: &[NormalizedMarketData],
        now_ms: u64,
        trace_id: &str,
    ) -> Result<qrpc_core::FillResult> {
        let result = self
            .execution_module
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .submit_plan(
                plan,
                normalized_data,
                &mut self.state.portfolio,
                now_ms,
                trace_id,
            );
        self.refresh_portfolio_state(normalized_data, now_ms);
        Ok(result)
    }

    pub fn on_market_data(
        &mut self,
        normalized_data: &[NormalizedMarketData],
        now_ms: u64,
        trace_id: &str,
    ) -> Result<qrpc_core::FillResult> {
        let result = self
            .execution_module
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .on_market_update(normalized_data, &mut self.state.portfolio, now_ms, trace_id);
        self.refresh_portfolio_state(normalized_data, now_ms);
        Ok(result)
    }
}
