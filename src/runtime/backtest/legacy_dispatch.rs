use crate::{
    apply_backtest_execution_assumption_overrides, build_compile_artifact_bundle,
    compile_runtime_protocol_config, compile_runtime_protocol_via_qs, internal_error,
    resolved_backtest_execution_assumptions, resolved_execution_assumption_sources,
    CompileArtifactBundle, DeterministicTestMode, ExecutionAssumptionSourceSummary,
    ExecutionAssumptionSpec, FastBacktestSandbox, FrontendBacktestReplaySource, FrontendRunRequest,
    StrategyArtifactSourceKind, BACKTEST_DETERMINISTIC_SEED,
};
use axum::http::StatusCode;
use qrpc_core::BacktestOutput;
use qrpc_runtime::Sandbox;
use serde_json::Value;
use std::collections::BTreeMap;
use tokio::task;

pub(super) struct LegacyBacktestDispatchOutput {
    pub compiled: qrpc_core::CompiledRuntimeProtocol,
    pub artifacts: CompileArtifactBundle,
    pub replay_source: FrontendBacktestReplaySource,
    pub resolved_execution_assumptions: ExecutionAssumptionSpec,
    pub resolved_execution_assumption_sources: ExecutionAssumptionSourceSummary,
    pub backtest: BacktestOutput,
}

pub(super) struct LegacyBacktestDispatchPlan {
    pub compiled: qrpc_core::CompiledRuntimeProtocol,
    pub resolved_execution_assumptions: ExecutionAssumptionSpec,
    pub resolved_execution_assumption_sources: ExecutionAssumptionSourceSummary,
}

pub(super) fn prepare_legacy_backtest_dispatch(
    graph_json: &Value,
    request: &FrontendRunRequest,
) -> Result<LegacyBacktestDispatchPlan, (StatusCode, String)> {
    let qs_protocol = compile_runtime_protocol_via_qs(graph_json)?;
    let runtime_protocol = apply_backtest_execution_assumption_overrides(
        &qs_protocol,
        request.backtest_options.execution_assumptions.as_ref(),
    );
    let compiled = compile_runtime_protocol_config(&runtime_protocol).map_err(internal_error)?;
    let resolved_execution_assumptions = resolved_backtest_execution_assumptions(
        &compiled.config,
        request.backtest_options.execution_assumptions.as_ref(),
    );
    let resolved_execution_assumption_sources = resolved_execution_assumption_sources(request);

    Ok(LegacyBacktestDispatchPlan {
        compiled,
        resolved_execution_assumptions,
        resolved_execution_assumption_sources,
    })
}

pub(super) async fn run_legacy_backtest_dispatch(
    plan: LegacyBacktestDispatchPlan,
    request: &FrontendRunRequest,
    now_ms: u64,
) -> Result<LegacyBacktestDispatchOutput, (StatusCode, String)> {
    let LegacyBacktestDispatchPlan {
        compiled,
        resolved_execution_assumptions,
        resolved_execution_assumption_sources,
    } = plan;
    let artifacts = build_compile_artifact_bundle(
        &request.runtime_config.metadata.graph_id,
        &request.runtime_config.metadata.compile_id,
        &request.runtime_config.metadata.name,
        &request.runtime_config.metadata.mode,
        StrategyArtifactSourceKind::FrontendGraph,
        &request.runtime_config.metadata.graph_id,
        BTreeMap::new(),
        &compiled,
    )
    .map_err(internal_error)?;
    let replay_source = request.backtest_replay_source();
    let core_ir = compiled.core_ir.clone();
    let latency_override = resolved_execution_assumptions.latency_assumption_ms;

    let backtest = task::spawn_blocking(move || {
        let mut sandbox = match replay_source {
            FrontendBacktestReplaySource::HistoricalReplay => {
                FastBacktestSandbox::with_replay_from_core_ir(core_ir.clone(), now_ms)
                    .map_err(|error| {
                        anyhow::anyhow!(
                            "{}. 历史重放需要本地市场数据文件 (位于 data cache 目录中)。\
                             离线测试请设置 backtest_options.replay_source = \"deterministic_mock\"",
                            error,
                        )
                    })
            }
            FrontendBacktestReplaySource::DeterministicMock => {
                FastBacktestSandbox::with_mock_replay_from_core_ir_and_test_mode(
                    core_ir,
                    now_ms,
                    DeterministicTestMode::replay_defaults(now_ms, BACKTEST_DETERMINISTIC_SEED),
                )
            }
        }
        .map_err(|e| internal_error(anyhow::anyhow!(e)))?;
        if let Some(latency_ms) = latency_override {
            sandbox.set_execution_assumptions(qrpc_runtime::slippage::ExecutionAssumptions {
                latency: qrpc_runtime::slippage::LatencyModel::Fixed { delay_ms: latency_ms },
                ..qrpc_runtime::slippage::ExecutionAssumptions::v1_0_7_compat()
            });
        }
        sandbox
            .start()
            .map_err(|e| internal_error(anyhow::anyhow!(e)))?;
        sandbox
            .run_backtest()
            .map_err(|e| internal_error(anyhow::anyhow!(e)))
    })
    .await
    .map_err(|e| internal_error(anyhow::anyhow!("回测任务被取消: {}", e)))??;

    Ok(LegacyBacktestDispatchOutput {
        compiled,
        artifacts,
        replay_source,
        resolved_execution_assumptions,
        resolved_execution_assumption_sources,
        backtest,
    })
}
