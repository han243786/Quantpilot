# 运行时证据合约

## 目的

本文档拥有 v0.2.0 升级 Block 2 中引入的活跃证据面合约。它是时间线、回放、紧凑证据、保留的关键索引和报告生命周期负载的字段级别真实数据源。

证据面不得创建第二套运行时事实模型。它投影运行时治理合约中受治理的运行时/回测事件，并保持事件信封作为序列、阶段、保留和治理标识的源。

## 所有权

| 合约 | 后端负责人 | 前端负责人 | 持久化 | 更新规则 |
|---|---|---|---|---|
| 时间线项目 | `runtime_event_projection` 和 `runtime_response_mapping` | `runtimeTimeline` 读取器和 `GovernedTimelinePanel` | 否，从受治理记录投影 | 增补字段需要 API 合约快照更新和 UI 读取器回退。破坏性重命名需要 v2 合约。 |
| 回放窗口 | `runtime_api` 回放处理器 | `EventReplaySection` 和 `graphStoreRuntimeHistoryApi` | 否，从持久化/当前源事件分页 | 游标、过滤器或序列语义必须更新回放测试和合约快照。 |
| 保留的关键事件索引 | `runtime_response_mapping` 紧凑/关键投影 | `runtimeTimeline` 读取器 | 否，从详情源投影 | 保留策略变更必须保留所有 `retention_class=key` 和系统治理事件。 |
| 紧凑证据 | `runtime_response_mapping` 压缩投影 | `runtimeTimeline`、`runtimeEvidenceSummary`、报告 UI | 否，从详情源投影 | 丢弃策略变更必须更新策略版本和快照 fixture。 |
| 报告生命周期记录 | `runtime_api` 报告存储和源物化 | `RuntimeReportPanel` | 是，报告元数据存储 | 新的生命周期字段必须链接到源、重载安全，并由快照测试覆盖。 |
| 报告工件导出 | `runtime_response_mapping` 报告工件投影 | 导出/揭示链接 | 从报告元数据推导 | 导出负载绝不能复制原始事件日志；它链接到源标识、范围、治理、策略、摘要和加载策略。 |
| 证据健康 | `runtime_api` 证据健康处理器和内存证据指标 | 运维/状态 UI 或冒烟检查 | 否，仅运行时计数器 | 新计数器必须是增补的，不得改变用户可见的报告/时间线行为。 |
| 证据清理 | `runtime_persistence` 清理策略和清理处理器 | 仅手动操作操作 | 仅适用于瞬态生成输出 | 清理绝不能删除持久化的报告 JSON 记录或已保存的运行/回测/实验工件。 |

## 时间线项目

每个详情、回放、紧凑和报告输入路径必须使用相同的时间线项目形态：

- `timeline_item_version`：合约版本，当前为 `1`
- `event_id` 和 `event_type`：从受治理的运行时事件复制
- `sequence_no`：事件信封序列号；主要回放游标
- `occurred_at_ms` 和 `ingested_at_ms`：信封时间字段
- `stage`：类型化信封阶段，如 `system`、`data`、`risk`、`execution`
- `retention_class`：类型化保留类，如 `key`、`summary`、`debug`
- `severity`、`module_key`、`node_id`、`summary`、`reason_code`
- `governance`：`capability_hash`、`deployment_revision`、`strategy_version` 和 `parameter_version`
- `payload_version` 和 `compactability`

时间线读取器仅可通过限制性默认值修复旧缺失值。新代码不得从显示文本推断阶段或治理信息。

## 回放窗口

回放响应按序号而非挂钟时间在时间线源上分页。活跃字段是：

- 源标识：`kind`、`record_id`、`graph_id`
- 源计数：`source_event_count`、`total_events`
- 游标：`cursor`、`sequence_cursor`、`previous_*`、`next_*`、`window_end`
- 过滤器：`stage`、`severity`、`retention_class`、`module_key`、`key_only`
- 证据数组：旧版 `events` 包装器和受治理的 `timeline`
- 摘要上下文：`fill_event_count`、`account`、`checkpoints`

前端回放控件必须消费返回的游标元数据。当后端序列元数据可用时，它们不得从本地切片数组推导页面边界。

## 保留的关键事件索引

保留的关键事件索引是共享时间线上的紧凑索引：

- `index_version`
- `policy_version`
- `source_event_count`
- `retained_event_count`
- `key_event_count`
- `system_event_count`
- `entries`

策略保留每个 `retention_class=key` 项目和系统治理事件，如 `CapabilitySnapshotTaken` 和 `SecurityViolationDetected`。它可以从紧凑路径丢弃摘要/调试事件，但必须保留足够的序列元数据以便稍后重新打开详细回放窗口。

## 紧凑证据

紧凑证据是大型日志审查的首选输入。它暴露：

- `projection_version`
- `policy_version`
- `source_event_count`、`retained_event_count`、`dropped_event_count`
- `dropped_by_retention` 和 `dropped_by_stage`
- `key_event_count` 和 `system_event_count`
- `governance`
- `entries`

UI 摘要卡片和报告生成必须首先读取紧凑证据。如果紧凑条目不可用，它们可回退到当前详情窗口，并标记需要详情窗口。

## 报告生命周期

报告记录是持久化的元数据，而非复制的日志。活跃生命周期字段是：

- `report_id`
- `source_kind`、`source_id`、`graph_id`
- `status`：`requested`、`generating`、`ready`、`failed`、`expired`、`source_changed`
- `source_sequence_range`
- `source_event_count`、`retained_event_count`
- `governance`
- `generation_policy`
- `artifacts`
- `failure_reason` 用于兼容性
- `failure`：结构化的 `reason_code`、`message` 和 `retry_eligible`
- `created_at_ms`、`updated_at_ms`

报告列表、详情和导出路径必须在声称 `ready` 之前针对当前已保存源物化记录。如果源图 ID、序列范围、源/保留计数、治理标识或生成策略不再匹配，报告变为 `source_changed`，并已准备好的工件从返回记录中移除。

## 报告工件导出

报告导出是从报告元数据推导的确定性工件：

- `schema_version`：`quantpilot/evidence-report-artifact/v1`
- 来自报告记录的源标识和生命周期字段
- `evidence_digest`
- `loading_strategy`：`primary_source`、`source_event_count`、`retained_event_count`、`requires_detail_window`
- `sections`

导出负载不得包含原始 `events` 或紧凑 `entries`。摘要是追踪回确切证据链的源元数据。

## 证据健康

证据健康端点是 `GET /api/runtime/evidence/health`。它暴露运维计数器和策略元数据，不改变用户工作流。

活跃字段：

- `status`：当前为 `ok`
- `metrics.report_generation_count`
- `metrics.report_generation_failure_count`
- `metrics.report_source_changed_count`
- `metrics.replay_page_count`
- `metrics.replay_page_latency_total_ms`
- `metrics.replay_page_latency_avg_ms`
- `metrics.compact_projection_source_event_count_total`
- `metrics.compact_projection_retained_event_count_total`
- `metrics.compact_detail_window_required_count`
- `persisted_report_count`
- `report_status_counts`
- `cleanup_policy`

这些指标是内存中的运行时计数器。它们用于健康检查和冒烟诊断，而非用于计费、合规汇总或持久化分析。

## 证据清理

证据清理端点是 `POST /api/runtime/evidence/cleanup`。它仅移除报告存储下名称以清理策略前缀开头的文件或目录的瞬态报告生成输出：

- `report-generation-tmp-`
- `report-generation-partial-`

默认 TTL 为 24 小时。测试可传递 `max_age_ms` 以确定性方式执行策略。

清理必须保留：

- 持久化的报告记录 JSON 文件
- 已保存的运行记录
- 已保存的回测工件目录
- 已保存的实验记录
- 紧凑证据投影，因为它们从源证据推导而非作为独立缓存条目存储

清理响应报告策略、移除的瞬态输出计数和保留的报告记录计数。

## 变更检查清单

更改任何证据合约字段之前：

1. 更新本文档和 v0.2.0 工作清单。
2. 更新后端投影或物化测试。
3. 更新 API 合约快照 fixture。
4. 当显示语义更改时，更新前端读取器/UI 测试。
5. 对中文 UI 标签和文档运行 UTF-8 Markdown 检查。
