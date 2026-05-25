# v4.5.0 closeout — 高级订单类型 + Tick 级回放

状态: 已并入 v4.7.0 集成批次。

## 已落实

- `qrpc_runtime/src/v4_runtime.rs`: OCO bracket、trailing stop、GTD expiry、cancel-replace-amend 均有本地模拟事件证据。
- `qrpc_runtime/src/v4_runtime.rs`: 新增 `run_backtest_ticks` 与 `replay_mode = "tick_replay"`。
- `qrpc_runtime/src/backtest_metrics.rs`: 新增 microstructure metrics 计算。
- `qrpc_core_ir/src/v4.rs`: `V4BacktestArtifact` 新增可选 `input_tick_count` 与 `microstructure_metrics`。
- `frontend/src/pages/BacktestDetailPage.jsx`: 展示 tick 数和微结构指标。

## 验证

- `cargo check`
- `cargo test -p qrpc-runtime v4_runtime_ -- --nocapture`
- `cargo test -p qrpc-runtime v4_backtest_tick_replay -- --nocapture`

## 遗留

- 暂未接入真实逐笔行情源；tick replay 当前使用确定性输入或 deterministic mock 派生 tick。
