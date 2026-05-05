# RFC-019 回测输出工件协议

## 状态

当前状态：draft

适用范围：

- `EventLogArtifact`（事件日志工件）
- `TradeLedgerArtifact`（交易台账工件）
- `EquityCurveArtifact`（权益曲线工件）
- `MetricsArtifact`（指标工件）
- `ReproducibilityManifest`（可重现性清单）
- `BacktestArtifactViews`（回测工件视图）

## 目标

本 RFC 定义了回测执行的稳定输出侧工件模式。

其直接目标是用显式的、版本化的工件替代临时的输出 JSON，这些工件可同时服务于三个角色：

- 详情页渲染
- 存储和重新加载
- 可重现性及后续比较

## 工件集

回测输出作为一个工件视图包暴露：

1. `EventLogArtifact`（事件日志工件）
2. `TradeLedgerArtifact`（交易台账工件）
3. `EquityCurveArtifact`（权益曲线工件）
4. `MetricsArtifact`（指标工件）
5. `ReproducibilityManifest`（可重现性清单）

这些以 `BacktestArtifactViews` 的形式统一呈现于 `backtest_artifacts` 之下。

## 版本化对象

### EventLogArtifact（事件日志工件）

```json
{
  "schema_version": "quantpilot/event-log-artifact/v1",
  "artifact_id": "event_log_artifact_<digest-prefix>",
  "backtest_id": "backtest_test",
  "event_count": 3,
  "digest": {
    "algorithm": "sha256_canonical_json",
    "value": "<sha256>"
  },
  "events": []
}
```

目的：

- 提供已完成回测的事实源事件序列
- 为 UI 和审计工具提供稳定的事件负载，而非隐式的顶层数组
- 锚定后续的确定性投影
- 在每个回测事件负载中携带足够的稳定投影上下文，以推导交易、权益和指标视图，而无需重新读取 `BacktestOutput`

### TradeLedgerArtifact（交易台账工件）

```json
{
  "schema_version": "quantpilot/trade-ledger-artifact/v1",
  "artifact_id": "trade_ledger_artifact_<digest-prefix>",
  "backtest_id": "backtest_test",
  "trade_count": 1,
  "digest": {
    "algorithm": "sha256_canonical_json",
    "value": "<sha256>"
  },
  "trades": [
    {
      "fill_id": "fill_1",
      "plan_id": "plan_1",
      "exchange": "Binance",
      "symbol": "BtcUsdt",
      "side": "buy",
      "filled_qty": 0.2,
      "filled_price": 50250.0,
      "fee_paid": 1.5,
      "filled_at_ms": 1700000060000,
      "status": "filled",
      "trace_id": "trace_1",
      "session_index": 0,
      "cycle_name": "slow"
    }
  ]
}
```

目的：

- 以分析友好的投影方式展示成交记录
- 支持详情页交易表和未来的运行比较
- 保留每次成交的审计标识，无需强制 UI 解析原始运行时事件

### EquityCurveArtifact（权益曲线工件）

```json
{
  "schema_version": "quantpilot/equity-curve-artifact/v1",
  "artifact_id": "equity_curve_artifact_<digest-prefix>",
  "backtest_id": "backtest_test",
  "point_count": 2,
  "digest": {
    "algorithm": "sha256_canonical_json",
    "value": "<sha256>"
  },
  "points": []
}
```

目的：

- 暴露详情页和后续比较所使用的持久化权益曲线投影
- 使图表渲染与原始 `BacktestOutput` 对象解耦

### MetricsArtifact（指标工件）

```json
{
  "schema_version": "quantpilot/metrics-artifact/v1",
  "artifact_id": "metrics_artifact_<digest-prefix>",
  "backtest_id": "backtest_test",
  "digest": {
    "algorithm": "sha256_canonical_json",
    "value": "<sha256>"
  },
  "summary": {},
  "event_count": 3,
  "session_count": 1,
  "started_at_ms": 1700000000000,
  "ended_at_ms": 1700000060000,
  "final_account": {}
}
```

目的：

- 暴露列表页和详情头部所使用的摘要块
- 捕获比较和可重现性检查所需的最小结果指标
- 为结果级别缓存和 UI 引用提供稳定的工件 ID

### ReproducibilityManifest（可重现性清单）

```json
{
  "schema_version": "quantpilot/reproducibility-manifest/v1",
  "manifest_id": "manifest_backtest_test",
  "backtest_id": "backtest_test",
  "graph_id": "graph_test",
  "compile_id": "compile_test",
  "created_at_ms": 1700000060000,
  "protocol_name": "quantpilot/runtime-config/v1",
  "config_hash": "runtime-spec-...",
  "account": {},
  "summary": {},
  "backtest_spec": {},
  "compile_artifacts": {},
  "output_artifacts": [],
  "backtest_output_digest": {
    "algorithm": "sha256_canonical_json",
    "value": "<sha256>"
  }
}
```

目的：

- 将输入侧和输出侧工件连接到一个可重现性边界中
- 列出持久化的输出文件以供存储重载
- 将编译标识、回测标识、摘要和摘要锚点保持在一个对象上

## 投影规则

- `EventLogArtifact` 是回测详情消费者的稳定事件边界。
- `TradeLedgerArtifact`、`EquityCurveArtifact` 和 `MetricsArtifact` 是从 `EventLogArtifact` 推导的确定性投影。
- `ReproducibilityManifest` 引用 `BacktestSpec`、`CompileArtifactBundle` 和每个持久化的输出工件文件。
- 回测 API 详情消费者应直接读取 `backtest_artifacts`；顶层 `events` 和 `backtest` 字段不属于输出工件合约。

实现说明：

- 当前 beta 实现以事件日志优先的方式持久化和重载这些工件。
- 投影演化必须保留此合约：`TradeLedgerArtifact`、`EquityCurveArtifact` 和 `MetricsArtifact` 从事件日志边界重建，而非通过临时侧信道。

## API 边界

当前响应形态：

- `POST /api/runtime/backtest`
- `GET /api/runtime/backtests/{backtest_id}`

两个响应都可能包含：

```json
{
  "backtest_artifacts": {
    "event_log": {},
    "trade_ledger": {},
    "equity_curve": {},
    "metrics": {},
    "manifest": {}
  }
}
```

详情页读者应使用：

- `backtest_artifacts.event_log.events` 用于事件回放和节点高亮
- `backtest_artifacts.metrics` 用于摘要和事件/会话计数
- `backtest_artifacts.trade_ledger` 用于成交表
- `backtest_artifacts.equity_curve` 用于图表输入
- `backtest_artifacts.manifest` 用于可重现性元数据

## 存储布局

持久化的目录布局：

```text
storage/backtests/<backtest_id>/
  event_log.json
  trade_ledger.json
  equity_curve.json
  metrics.json
  backtest_output.json
  manifest.json
```

规则：

- `manifest.json` 是存储入口点
- 重载可以从目录中重建完整的 `BacktestRecord`，而无需依赖传统的单文件回测 JSON
- `manifest.output_artifacts` 中列出的输出文件引用必须与持久化的工件文件匹配

## 摘要规则

所有输出工件摘要使用：

- 算法：`sha256_canonical_json`
- 规范形式：对工件负载执行 `serde_json::to_vec(...)`
- 输出格式：小写十六进制字符串

说明：

- `artifact_id` 是从摘要前缀派生的可读标识符
- `backtest_output_digest` 与工件摘要不同；它对完整的 `BacktestOutput` 负载进行哈希
- 输出工件不得发明第二个编译标识；它们通过 `compile_artifacts` 和 `backtest_spec` 继承编译标识

## 当前实现

当前代码路径：

- 输出工件结构和持久化：`src/backtest_artifacts.rs`
- 回测 API 组装和重载：`src/main.rs`
- 前端详情渲染：`frontend/src/pages/BacktestDetailPage.jsx`
- 前端事件流和摘要面板：`frontend/src/components/EventStreamPanel.jsx`

## 范围外

本 RFC 尚未定义：

- 多运行比较工件模式
- 分页工件查询端点
- 持久化回测的增量事件日志流式传输
- 跨多个回测的实验集清单
