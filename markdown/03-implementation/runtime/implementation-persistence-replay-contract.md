# 持久化 / 回放合约

此文件是 `CL-P1-004` 的活跃措辞边界。

## 目标

将持久化的运行时详情、回测详情、历史卡片和回放视图保持在单一的稳定负载系列上。

## 持久化真实数据源

- `/api/runtime/history/:run_id` 返回的运行详情负载
- `/api/runtime/backtests/:backtest_id` 返回的回测详情负载
- `/api/runtime/history/:run_id/replay` 或 `/api/runtime/backtests/:backtest_id/replay` 返回的回放负载
- 持久化的 `backtest_artifacts.event_log.events`
- 持久化的 `runtime_diagnostics`

前端不得从无关的仅存活性状态重建这些形态。

## 共享前端状态规则

- 已完成的回测选择现在使用 `frontend/src/store/graphStoreRuntimeSelectionState.js` 中的一个共享构建器
- 实时回测完成和持久化回测重载必须投影相同的 `runtime` 选择形态
- 持久化运行详情和持久化回测详情必须清除相反的选择 ID，而非留下混合的 `selectedHistoryRunId` / `selectedBacktestId` 状态
- 回放小部件直接从回放响应读取持久化记录 ID 加序列窗口

## 重载规则

- 详情页应优先使用持久化工件，如 `backtest_artifacts.event_log.events`
- `runtime_diagnostics` 应在重载后直接复用
- 高亮节点 ID 应从持久化事件推导，而非从过时的画布选择记忆
- `selectedNodeId` 应在存在第一个高亮持久化节点时跟随该节点

## 收口规则

- 实时完成和持久化重载必须在描述相同记录时，在 `runId`、`runKind`、`account`、`backtestArtifacts`、`diagnostics`、`events`、选定 ID 和高亮节点 ID 上保持一致
- 历史卡片、详情页和回放必须在完整页面重载后保持可读，无需重建缺失的解释状态
- 不得引入第二个仅重载的 DTO 系列

## 当前实现入口点

- `frontend/src/store/graphStoreRuntimeSelectionState.js`
- `frontend/src/store/graphStoreRuntimeSessionState.js`
- `frontend/src/store/graphStoreRuntimeHistoryState.js`
- `frontend/src/store/graphStoreRuntimeHistoryProjection.js`
- `frontend/src/components/EventStreamPanel.jsx`
- `frontend/src/components/EventReplaySection.jsx`
- `frontend/src/pages/BacktestDetailPage.jsx`
