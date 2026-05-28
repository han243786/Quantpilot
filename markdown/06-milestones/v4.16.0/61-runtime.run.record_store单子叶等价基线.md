# v4.16.0 runtime.run.record_store 单子叶等价基线

> 版本类型: MINOR architecture / governance。  
> 执行档位: 重型。  
> 批次: BE-001J-01。  
> 基准: `60-runtime.run.session_start单叶closeout.md`。  
> 判定: 建立 `runtime.run.record_store` 单子叶等价基线；本批只冻结 run record list/detail/save/discard、持久化 helper、audit helper、安全路径清洗、父子通信和回归证据，不移动代码。

---

## 选择理由

`runtime.run.session_start` 已完成 closeout，下一步必须回到 `runtime.run` sibling 队列。`runtime.run.record_store` 是最适合承接的下一片:

1. 它覆盖 `list_runs`、`get_run_detail`、`save_run_record`、`discard_run_record`，正好是 run lifecycle 中 session start 之后的 record 查询和保存边界。
2. 它同时接触 `state.runs`、`run_store_dir`、`audit_store_dir` 和 `runtime_persistence`，状态与持久化责任清楚，值得单独做等价基线。
3. 它能被 `api_run` 中 created run contract、governance metadata、legacy persisted record 等代表测试覆盖。
4. 它不需要混入 replay/status/SSE；这些仍可作为后续 sibling 单独处理。

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001J handler sibling 等价基线 | 扩展 |
| 规范矩阵 | record store owner、state/persistence owner、audit 写入、父子通信 | 固化 |
| 引导矩阵 | `runtime.run.record_store` 白箱节点 | 扩展 |
| 模块树 | `runtime.run.record_store` | 建立单子叶基线 |

---

## 引导坐标

| 项 | 指向 |
| --- | --- |
| 引导坐标 | `root.backend.runtime.routes.run.runtime.run.record_store` |
| 全量树节点 | `markdown/10-overview/overview-full-feature-tree.md` 根 2 backend 与根 7 v4.16 |
| 模块树节点 | `markdown/00-matrix-governance/module-tree.md` 的 `runtime.run.record_store` |
| 真实文件 | `src/runtime/run.rs`、`src/runtime/mod.rs`、`src/backend/runtime/routes/run.rs`、`src/runtime_persistence.rs`、`src/runtime_response_mapping.rs`、`src/collaboration.rs` |
| public 方法 | `list_runs`、`get_run_detail`、`save_run_record`、`discard_run_record`、`load_run_record_from_state`、`list_run_records`、`persist_run_record`、`run_list_item_from_record`、`run_detail_response_from_record`、`sanitize_storage_path_segment`、`persist_graph_audit_entry`、`build_graph_audit_entry` |
| 测试/门禁 | `cargo check -p quantpilot`、`cargo test -p quantpilot --test api_run`、`tools/check-matrix-governance.ps1`、`tools/check-full-feature-tree.ps1` |

---

## 白箱输入输出

| 方向 | 内容 | 来源/去向 | 约束 |
| --- | --- | --- | --- |
| 输入 | `AppState` | `backend.app_state_wiring` | 只读取既有 `runs`、`run_store_dir`、`audit_store_dir`，不迁移 owner |
| 输入 | `UserId` + `run_id` | auth middleware、path param | detail/save/discard 必须继续使用 scoped run key 或安全路径段 |
| 输入 | `PaginationQuery` | `/api/runtime/runs` | 不改变分页语义或排序语义 |
| 输出 | `PaginatedResponse<RunListItem>` | frontend、tests | 不改 list schema，不改变 created_at 倒序排序 |
| 输出 | `RunDetailResponse` | frontend、tests | 不改 governance、actor、account、events、evidence 字段映射 |
| 输出 | 持久化 run manifest | `run_store_dir` | 必须继续走 bounded read、atomic write 和 `sanitize_storage_path_segment` |
| 输出 | graph audit entry | `audit_store_dir` | save 时仍使用 `GraphAuditAction::RunCreated` 和现有 actor guard |
| 输出 | discard response | frontend、tests | 已保存记录必须 conflict；仅 transient in-memory record 可丢弃 |

---

## 兼容桥

```text
backend.interface_boundary
  -> backend.runtime::register_routes
  -> backend.runtime.routes::register_routes
  -> backend.runtime.routes.run::register_routes
  -> crate::runtime::{list_runs,get_run_detail,save_run_record,discard_run_record}
  -> AppState.runs / runtime_persistence / runtime_response_mapping / collaboration audit
  -> existing JSON response
```

本批只固定这条链路。下一步若抽离实现，也必须保留父级 `runtime` 受控出口和 `backend.runtime.routes.run` route facade，不允许绕过父模块新增横向调用。

---

## owner 基线

| 子域 | 当前真实 owner | 代表方法/类型 | 当前处理 |
| --- | --- | --- | --- |
| route facade | `src/backend/runtime/routes/run.rs` | `/api/runtime/runs`、`/api/runtime/runs/:run_id`、`/api/runtime/runs/:run_id/save`、`/api/runtime/runs/:run_id/discard` | 不改 path/method |
| record handler | `src/runtime/run.rs` | `list_runs`、`get_run_detail`、`save_run_record`、`discard_run_record` | 本批不移动 |
| state lookup | `src/runtime_persistence.rs` | `load_run_record_from_state` | 不改 `state.runs` fallback 到 manifest 的顺序 |
| persistence | `src/runtime_persistence.rs` | `list_run_records`、`persist_run_record`、`sanitize_storage_path_segment` | 不迁移 persistence owner |
| response projection | `src/runtime_response_mapping.rs` | `run_list_item_from_record`、`run_detail_response_from_record` | 不改 schema |
| audit | `src/collaboration.rs` | `persist_graph_audit_entry`、`build_graph_audit_entry`、`GraphAuditAction::RunCreated` | 不迁移 graph audit owner |

---

## 等价证据

| 证据 | 覆盖范围 | 必须证明 |
| --- | --- | --- |
| `cargo check -p quantpilot` | handler 可见性、schema、helper 调用 | 基线不破坏类型 |
| `cargo test -p quantpilot --test api_run` | list/detail/save/discard、governance metadata、legacy persisted record、replay downstream 代表链路 | record store 行为不漂移 |
| `tools/check-matrix-governance.ps1` | 本基线、模块树、全量树和门禁锚点 | 治理入口不丢 |
| `tools/check-full-feature-tree.ps1` | 文件路径覆盖 | 新基线和真实文件可定位 |

`api_run` 中必须继续覆盖以下代表测试:

| 测试 | 覆盖 |
| --- | --- |
| `run_endpoints_expose_service_level_contract_for_created_run` | created run 的 list/detail/save/discard 合同 |
| `runtime_report_records_persist_governed_run_evidence_metadata` | 保存后 governance evidence metadata |
| `legacy_run_record_without_governance_loads_with_safe_defaults` | 旧 manifest 缺 governance 时的安全默认 |
| `run_replay_endpoint_exposes_paginated_ordered_timeline` | saved/current record 对 downstream replay 的代表兼容 |

---

## 本批次不做

- 不移动 `src/runtime/run.rs` 中的 `list_runs`、`get_run_detail`、`save_run_record`、`discard_run_record`。
- 不新建 record store 实现文件。
- 不改 `/api/runtime/runs`、`/api/runtime/runs/:run_id`、`/api/runtime/runs/:run_id/save` 或 `/api/runtime/runs/:run_id/discard` 的 path、method、payload、response schema 或 error code。
- 不迁移 `state.runs`、`run_store_dir`、`audit_store_dir`、AppState 字段 owner、runtime state owner、lock order 或 persistence owner。
- 不拆 `runtime.run.replay_status`、`runtime.event_stream`、`runtime.run.session_start`、`runtime.run.v4_handoff`、backtest、mutation、report 或 experiment owner。
- 不主动提出发布版本过渡或横向连接。
- 不宣称 `runtime.run.record_store` 已完成抽离。

---

## 后续判断

若继续本子叶，下一步才允许做 `runtime.run.record_store` 实际抽离方案，且必须满足:

1. 只迁移 record list/detail/save/discard handler 边界，不混入 replay/status 或 SSE。
2. 保持 `state.runs`、`run_store_dir`、`audit_store_dir`、bounded read、atomic write 和安全路径清洗语义不变。
3. 保留 `api_run` 代表测试作为等价证据。
4. 若引入新文件，必须同步模块树、全量树和治理门禁。

---

## 验收标准

1. `61-runtime.run.record_store单子叶等价基线.md` 进入 v4.16 里程碑索引。
2. 模块树出现 `runtime.run.record_store` 白箱节点。
3. 全量树能定位本基线和真实 runtime 文件。
4. 治理门禁能发现本基线文件缺失。
5. 后续 record store 抽离必须引用本基线，不得绕过父模块直接迁移 handler。
