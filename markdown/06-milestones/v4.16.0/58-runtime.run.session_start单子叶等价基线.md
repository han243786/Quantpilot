# v4.16.0 runtime.run.session_start 单子叶等价基线
> 版本类型: MINOR architecture / governance。  
> 执行档位: 重型。  
> 批次: BE-001I-01。  
> 基准: `57-runtime.run.v4_handoff单叶closeout.md`。  
> 判定: 建立 `runtime.run.session_start` handler 层单子叶等价基线；本批只固定 legacy `/api/runtime/test-run` 的输入、输出、状态 owner、兼容桥、等价证据和禁止迁移边界，不移动 `src/runtime/run.rs` 中的 `start_test_run` 或相关 helper。

---

## 选择理由

`runtime.run.session_start` 是 `runtime.run.v4_handoff` closeout 后最适合作为下一片 run handler 递归的 sibling:

1. 它聚焦 legacy `/api/runtime/test-run`，覆盖一次 Paper run 的启动、编译、session 执行与 in-memory run record 写入。
2. 它比 record store、replay/status 和 event stream 更靠近 run handler 起点，适合作为后续 record/replay 拆分前的上游基线。
3. 它已有 `api_run` 代表测试覆盖 capability guard、成功创建 run、run list/detail/save/replay/report 链路。
4. 它涉及 `AppState.run_in_progress`、`state.runs`、actor collaboration 和 runtime governance envelope，必须先冻结状态 owner 与锁语义。

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001I handler 层等价基线、runtime run sibling 递归 | 扩展 |
| 规范矩阵 | session start owner、run lock、in-memory record、父子通信 | 固化 |
| 引导矩阵 | `runtime.run.session_start` 白箱节点 | 扩展 |
| 模块树 | `runtime.run.session_start` | 建立单子叶基线 |

---

## 引导坐标声明

| 坐标 | 指向 |
| --- | --- |
| 引导坐标 | `root.backend.runtime.routes.run.runtime.run.session_start` |
| 全量树节点 | `markdown/10-overview/overview-full-feature-tree.md` 根2 backend 与根7 v4.16 |
| 模块树节点 | `markdown/00-matrix-governance/module-tree.md` 的 `runtime.run.session_start` |
| 真实文件 | `src/runtime/run.rs`、`src/runtime/mod.rs`、`src/backend/runtime/routes/run.rs`、`src/runtime_validation.rs`、`src/runtime_event_projection.rs`、`src/runtime_response_mapping.rs`、`src/compile_api.rs`、`src/capability_api.rs`、`src/collaboration.rs`、`src/graph_quantscript_api.rs`、`src/frontend_runtime_mapping.rs` |
| public 方法 | `start_test_run`、`validate_runtime_capability_guard`、`validate_runtime_config_capabilities`、`compile_runtime_protocol_via_qs`、`compile_runtime_protocol_config`、`build_compile_runtime_targets_from_graph`、`merge_runtime_targets`、`runtime_governance_snapshot`、`collect_frontend_events`、`prepend_capability_snapshot_event`、`attach_runtime_event_envelopes`、`validate_runtime_event_envelopes`、`account_summary`、`normalize_actor_identity`、`collaboration_with_run_actor`、`run_start_response` |
| 测试/门禁 | `cargo check -p quantpilot`、`cargo test -p quantpilot --test api_run`、`tools/check-matrix-governance.ps1`、`tools/check-full-feature-tree.ps1` |

---

## 白箱输入输出

| 方向 | 内容 | 来源/去向 | 约束 |
| --- | --- | --- | --- |
| 输入 | `UserId` | auth middleware | 仅用于 scoped run key，不迁移 auth owner |
| 输入 | `AppState` | `backend.app_state_wiring` | 只使用 `run_in_progress`、`runs`、`graph_store_dir` 等既有字段 |
| 输入 | `FrontendRunRequest.capability_context` | frontend/tests | 缺失必须返回 `capability_boundary_violation` 且不创建 run |
| 输入 | `FrontendRunRequest.runtime_config` | frontend/tests | 必须经 `validate_runtime_config_capabilities` |
| 输入 | `FrontendRunRequest.graph_json` | frontend/tests | 缺失必须按现有 bad request 拒绝 |
| 输入 | `FrontendRunRequest.runtime_targets` | frontend/tests | 与 graph targets 合并，不改 event node mapping |
| 输出 | `RunStartResponse` | frontend/tests | 不改 `run_id`、`graph_id`、`compile_id`、`event_count`、`status` schema |
| 输出 | in-memory `RunRecord` | `AppState.runs` | 不改 scoped key、governance、actor、events、account、session 写入语义 |
| 状态 | `run_in_progress` / `RunInProgressGuard` | `AppState` | 不迁移 owner，不改 AcqRel / Release 语义 |

---

## 兼容桥

```text
backend.interface_boundary
  -> backend.runtime::register_routes
  -> backend.runtime.routes::register_routes
  -> backend.runtime.routes.run::register_routes
  -> crate::runtime::start_test_run
  -> compile_runtime_protocol_via_qs
  -> compile_runtime_protocol_config
  -> RealTimeSandbox::new(RuntimeCoordinator::new(compiled))
  -> state.runs.insert(scoped run record)
  -> RunStartResponse
```

本批只固定这条链路；不允许绕过 `backend.runtime.routes.run` 新增 session start route，也不允许把 session start 与 record store、replay/status、SSE 或 v4 handoff 混成同一迁移批次。

---

## handler owner 基线

| 子域 | 当前真实 owner | 代表方法/类型 | 当前处理 |
| --- | --- | --- | --- |
| route handler | `src/runtime/run.rs` | `start_test_run` | 不移动 |
| request / response schema | `src/frontend_api_types.rs`、`src/runtime_response_mapping.rs` | `FrontendRunRequest`、`RunStartResponse`、`run_start_response` | 不改字段 |
| capability guard | `src/runtime_validation.rs` | `validate_runtime_capability_guard`、`validate_runtime_config_capabilities` | 不改 error 语义 |
| compile path | `src/compile_api.rs` | `compile_runtime_protocol_via_qs`、`compile_runtime_protocol_config` | 不绕过 QS |
| runtime execution | `src/runtime/run.rs` | `RealTimeSandbox`、`RuntimeCoordinator`、`run_session` | 不改 blocking 边界 |
| event projection | `src/runtime_event_projection.rs` | `collect_frontend_events`、`attach_runtime_event_envelopes` | 不改 envelope 语义 |
| governance / response | `src/capability_api.rs`、`src/runtime_response_mapping.rs` | `runtime_governance_snapshot`、`run_start_response` | 不改 schema |
| actor collaboration | `src/collaboration.rs` | `normalize_actor_identity`、`collaboration_with_run_actor` | 不迁移 graph audit owner |

---

## 等价证据

| 证据 | 覆盖范围 | 必须证明 |
| --- | --- | --- |
| `cargo check -p quantpilot` | handler 可见性、request/response 类型、runtime session 类型 | 基线不破坏类型 |
| `cargo test -p quantpilot --test api_run` | missing capability context、created run contract、runtime target mapping、run record/report/replay 代表链路 | session start behavior 不漂移 |
| `tools/check-matrix-governance.ps1` | 本基线、模块树、全量树锚点 | 治理入口不丢 |
| `tools/check-full-feature-tree.ps1` | 文件路径覆盖 | 新基线和真实文件可定位 |

`api_run` 中必须继续覆盖以下代表测试:

| 测试 | 覆盖 |
| --- | --- |
| `runtime_write_rejects_missing_capability_context_without_creating_run` | capability guard 与未创建 run |
| `run_endpoints_expose_service_level_contract_for_created_run` | session start、run response、save/list/detail 合同 |
| `run_endpoint_honors_runtime_targets_for_event_node_mapping` | runtime targets 与 event node mapping |
| `runtime_report_records_persist_governed_run_evidence_metadata` | created run evidence metadata |
| `run_replay_endpoint_exposes_paginated_ordered_timeline` | created run downstream replay projection |

---

## 本批次不做

- 不移动 `src/runtime/run.rs` 中的 `start_test_run` 或 helper。
- 不新建 handler 实现文件。
- 不改 `/api/runtime/test-run` route path、method、payload、response schema 或 error code。
- 不迁移 `run_in_progress`、AppState 字段 owner、runtime state owner、lock order、`state.runs` owner 或 persistence。
- 不拆 `runtime.run.record_store`、`runtime.run.replay_status`、`runtime.event_stream`、`runtime.run.v4_handoff` 或 backtest/mutation/report owner。
- 不清理旧中文字符串或历史注释乱码；这属于整理/文案治理，不属于本基线。
- 不宣称 `runtime.run.session_start` 已完成抽离。

---

## 后续判断

若继续本子叶，下一步才允许做 `runtime.run.session_start` 抽离方案，且必须满足:

1. 只迁移 session start handler 边界，不混入 record store、replay/status 或 SSE。
2. 保持 `start_test_run` 对外语义和 `RunStartResponse` schema 不变。
3. 保留 `api_run` 代表测试作为等价证据。
4. 若引入新文件，必须同步模块树、全量树和治理门禁。

---

## 验收标准

1. `58-runtime.run.session_start单子叶等价基线.md` 进入 v4.16 里程碑索引。
2. 模块树出现 `runtime.run.session_start` 白箱节点。
3. 全量树能定位本基线和真实 runtime 文件。
4. 治理门禁能发现本文件缺失。
5. 后续 session start 抽离必须引用本基线，不得绕过父模块直接迁移 handler。
