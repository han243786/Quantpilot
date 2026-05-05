# 运行时 / 回测解释合约

此文件是 `CL-P1-003` 的活跃措辞边界。

## 目标

将运行时详情、回测详情、事件流和回放面保持在同一个解释协议系列上。

## 解释真实数据源

- `runtime_diagnostics.node_details[*]`
- 持久化的 `event_log.events[*]` 负载事实
- 按顺序返回相同事件事实的持久化回放负载

任何页面都不应在此类事实之外发明第二套解释 DTO 系列。

## 共享前端投影规则

- `RuntimeDiagnosticsPanel` 通过 `buildRuntimeDiagnosticsProjection(...)` 渲染所选节点。
- `EventStreamPanel` 历史卡片和 `BacktestDetailPage` 解释卡片从相同的 `runtime_diagnostics.node_details[*]` 行聚合。
- `EventReplaySection` 仅读取事件级别的 `payload.explanation_summary` 或 `payload.reason_text`；它不构建第二套详情协议。

## 允许的详情系列

- `explanation_rows`（解释行）
- `data_quality_rows`（数据质量行）
- `risk_detail_rows`（风险详情行）
- `order_detail_rows`（订单详情行）

如果详情页需要更多解释，后端必须扩展这些结构化系列之一，或添加现有投影可读取的运行时事实。前端不应添加临时的仅解释 DTO。

## 收口规则

- 事件流、诊断、回放和回测详情在引用相同持久化事实时，必须在相同的节点名称、行标签和解释摘要上保持一致
- 持久化历史重载不得从无关的瞬态 UI 状态重建解释行
- 仅当 `runtime_diagnostics` 不存在时允许事件级别回退，且它仍必须来自持久化的事件负载事实

## 当前实现入口点

- `frontend/src/utils/runtimeDiagnosticsProjection.js`
- `frontend/src/utils/runtimeExplanation.js`
- `frontend/src/components/RuntimeDiagnosticsPanel.jsx`
- `frontend/src/components/EventStreamPanel.jsx`
- `frontend/src/components/EventReplaySection.jsx`
- `frontend/src/pages/BacktestDetailPage.jsx`
