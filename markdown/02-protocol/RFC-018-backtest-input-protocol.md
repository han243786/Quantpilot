# RFC-018 回测输入协议

## 状态

当前状态：draft

适用范围：

- `RunSpec`
- `BacktestSpec`
- `DatasetSpec`
- `MarketDataSnapshotSpec`
- `ExecutionAssumptionSpec`

## 目标

本 RFC 定义了运行/回测执行的稳定输入侧模式。

其直接目标是使以下边界显式化：

- 编译了什么内容
- 请求了何种市场数据形态
- 使用了何种执行假设
- 何种回放模式产生了结果

## 核心对象

### DatasetSpec（数据集规范）

```json
{
  "dataset_id": "data_data_1",
  "data_id": "data_data_1",
  "exchange": "Binance",
  "symbol": "BtcUsdt",
  "market_type": "Spot",
  "kind": "KlineSeries",
  "interval": "1d",
  "lookback_days": 200,
  "enabled": true
}
```

目的：

- 以适合运行使用的形态冻结所请求的数据集边界
- 使未来的运行/回测模式与前端节点 JSON 解耦

### ExecutionAssumptionSpec（执行假设规范）

```json
{
  "initial_cash_balance": 100000.0,
  "taker_fee_bps": 10.0,
  "default_slippage_bps": 5.0,
  "total_cost_buffer_bps": 20.0,
  "time_in_force": "Gtc",
  "allow_partial_fills": true,
  "latency_assumption_ms": null
}
```

目的：

- 冻结对可重现性有实质性影响的执行和成本假设

### RunSpec（运行规范）

```json
{
  "schema_version": "quantpilot/run-spec/v1",
  "run_mode": "backtest",
  "graph_id": "graph_test",
  "compile_id": "compile_test",
  "runtime_mode": "paper",
  "protocol_name": "quantpilot/minimal-sim/v1",
  "config_hash": "runtime-spec-...",
  "datasets": [],
  "execution_assumptions": {},
  "core_ir_digest": {
    "algorithm": "sha256_canonical_json",
    "value": "<sha256>"
  }
}
```

目的：

- 提供一个以运行为中心的标识对象，由纸面交易和回测流程共享
- 将后续输出工件锚定到单一的编译与假设边界

### MarketDataSnapshotSpec（市场数据快照规范）

```json
{
  "snapshot_id": "market_snapshot_backtest_123",
  "replay_source": "deterministic_mock",
  "captured_at_ms": 1700000000000,
  "datasets": []
}
```

目的：

- 描述特定回测请求所使用的回放数据集边界
- 将历史回放和确定性模拟回放放在同一类型维度上

### BacktestSpec（回测规范）

```json
{
  "schema_version": "quantpilot/backtest-spec/v1",
  "backtest_id": "backtest_1700000000000",
  "replay_source": "deterministic_mock",
  "requested_at_ms": 1700000000000,
  "run_spec": {},
  "market_data_snapshot": {}
}
```

目的：

- 冻结精确的输入侧回测边界
- 为后续的事件日志和指标投影提供稳定的父级对象

## 回放源语义

当前支持的值：

- `historical_replay`（历史回放）
- `deterministic_mock`（确定性模拟）

规则：

- 两种回放模式仍必须使用相同的编译工件边界
- 回放模式改变数据源实现方式，而非编译语义
- 回放模式必须在 `BacktestSpec` 和 `MarketDataSnapshotSpec` 中显式声明

## 边界规则

- `RunSpec` 是共享的输入边界
- `BacktestSpec` 是 `RunSpec + 回放特定的市场数据上下文`
- `DatasetSpec` 从运行时协议数据源推导而来，而非从前端模块 JSON
- `ExecutionAssumptionSpec` 只能包含可能改变执行结果的语义

## 当前实现

当前代码路径：

- 共享模式类型：`qrpc_core/src/lib.rs`
- 回测规范组装：`src/main.rs`
- 编译/回测 API 暴露：`src/main.rs`

## 与未来工作的关系

本 RFC 有意限定于输入侧合约。

下一阶段应增加：

- 事件日志工件投影
- 交易台账投影
- 权益曲线投影
- 指标投影
- 可重现性清单

这些输出侧对象应引用 `BacktestSpec`，而非发明新的输入摘要。
