# v4.16.0 runtime.run.v4_handoff 单子叶等价基线

> 版本类型: MINOR architecture / governance。
> 执行档位: 重型。
> 批次: BE-001H-01。
> 基准: `54-backend.runtime.routes.run单叶closeout.md`。
> 判定: 建立 `runtime.run.v4_handoff` handler 层单子叶等价基线；本批只固定 v4 handoff 的输入、输出、真实 owner、等价证据和禁止迁移边界，不移动 `src/runtime/run.rs` 中的 handler 或 helper。

---

## 选择理由

`runtime.run.v4_handoff` 是 `src/runtime/run.rs` 内部最适合作为 handler 层第一片继续递归的子叶:

1. 它聚焦 `/api/runtime/v4/run`，边界比 `start_test_run`、record store 和 SSE 更清楚。
2. 它围绕 v4 QS source / preparsed graph / initial event / handoff report / paper simulated runtime 一组职责，测试覆盖强。
3. 它不直接拥有 saved run record、SSE streaming、report source、mutation approval 或 legacy merge record。
4. 它的状态边界相对窄，只使用 `run_in_progress` guard 和 v4 paper simulated runtime，不迁移 AppState owner 或 persistence owner。

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001H handler 层等价基线、runtime run 局部递归 | 扩展 |
| 规范矩阵 | handler owner 冻结、v4 handoff 输入输出、父子通信 | 固化 |
| 引导矩阵 | `runtime.run.v4_handoff` 白箱节点 | 扩展 |
| 模块树 | `runtime.run.v4_handoff` | 建立单子叶基线 |

---

## 引导坐标声明

| 坐标 | 指向 |
| --- | --- |
| 全量树节点 | `markdown/10-overview/overview-full-feature-tree.md` 根2 backend runtime、根7.6 v4.16 |
| 模块树节点 | `runtime.run.v4_handoff` |
| 真实文件 | `src/runtime/run.rs`、`src/backend/runtime/routes/run.rs`、`src/runtime_validation.rs`、`src/runtime_response_mapping.rs` |
| public 方法 | `start_v4_runtime_run`、`resolve_v4_runtime_run_graph`、`handoff_initial_event`、`v4_runtime_handoff_response`、`default_v4_payload_value`、`runtime_v4_static_bundle`、`runtime_simulated_v4_matrix` |
| route 坐标 | `/api/runtime/v4/run` |
| 测试/门禁 | `cargo check -p quantpilot`、`cargo test -p quantpilot --test api_run`、`tools/check-matrix-governance.ps1`、`tools/check-full-feature-tree.ps1` |

---

## 白箱输入输出

| 方向 | 内容 | 来源/去向 | 约束 |
| --- | --- | --- | --- |
| 输入 | Axum request | `backend.runtime.routes.run` | 只接受 `/api/runtime/v4/run` |
| 输入 | `V4RuntimeRunRequest.source` | frontend、tests、local API caller | 空 source 必须按现有 error code 拒绝 |
| 输入 | `V4RuntimeRunRequest.graph` | frontend、tests、compiled graph caller | 必须通过 `validate_static_contract` |
| 输入 | `V4RuntimeRunRequest.initial_event` | frontend、tests | 缺省时从 event catalog 派生 |
| 输出 | `V4RuntimeRunResponse` | frontend、tests | 不改 `run_id`、`graph_id`、`event_count`、`output`、`handoff`、`diagnostics` schema |
| 输出 | error JSON | frontend、tests | 不改 `v4_source_missing`、`v4_runtime_handoff_rejected`、`v4_graph_missing`、`v4_event_catalog_missing` 语义 |
| 状态 | `run_in_progress` guard | `AppState` | 不迁移 owner，不改 AcqRel 语义 |

---

## handler owner 基线

| 子域 | 当前真实 owner | 代表方法/类型 | 当前处理 |
| --- | --- | --- | --- |
| route handler | `src/runtime/run.rs` | `start_v4_runtime_run` | 不移动 |
| request / response schema | `src/runtime/run.rs` | `V4RuntimeRunRequest`、`V4RuntimeRunResponse`、`V4RuntimeRunDiagnostic`、`V4RuntimeRunHandoff` | 不改字段 |
| source / graph resolution | `src/runtime/run.rs` | `resolve_v4_runtime_run_graph` | 不改 validation / audit flow |
| event derivation | `src/runtime/run.rs` | `handoff_initial_event`、`default_v4_payload_value` | 不改 default payload |
| handoff projection | `src/runtime/run.rs` | `v4_runtime_handoff_response` | 不改 response mapping |
| runtime capability matrix | `src/runtime/run.rs` | `runtime_v4_static_bundle`、`runtime_simulated_v4_matrix` | 不改 supported modes |

---

## 兼容桥

本基线建立时的兼容桥为:

```text
backend.interface_boundary
  -> backend.runtime::register_routes
  -> backend.runtime.routes::register_routes
  -> backend.runtime.routes.run::register_routes
  -> start_v4_runtime_run
  -> resolve_v4_runtime_run_graph
  -> qrpc_runtime::V4PaperSimulatedRuntime
  -> V4RuntimeRunResponse
```

本批只固定这条链路；不允许绕过 `backend.runtime.routes.run` 直接暴露新的 v4 run route，也不允许把 v4 handoff 与 legacy `start_test_run`、SSE、record store 混成同一迁移批次。

---

## 等价证据

| 证据 | 覆盖范围 | 必须证明 |
| --- | --- | --- |
| `cargo check -p quantpilot` | handler 可见性、request/response 类型、v4 runtime 类型 | handler 层基线不破坏类型 |
| `cargo test -p quantpilot --test api_run` | `/api/runtime/v4/run` happy path、graph path、initial event、missing source、handoff reject、event catalog missing | v4 handoff behavior 不漂移 |
| `tools/check-matrix-governance.ps1` | 本基线、模块树、全量树锚点 | 治理入口不丢 |
| `tools/check-full-feature-tree.ps1` | 文件路径覆盖 | 新基线和真实文件可定位 |

`api_run` 中必须继续覆盖以下代表测试:

| 测试 | 覆盖 |
| --- | --- |
| `start_v4_runtime_run_accepts_paper_simulated_qs_source` | QS source handoff happy path |
| `start_v4_runtime_run_uses_default_catalog_event_for_source` | event catalog default event |
| `start_v4_runtime_run_accepts_preparsed_graph_without_handoff` | graph request path |
| `start_v4_runtime_run_accepts_initial_event_override` | explicit initial event |
| `start_v4_runtime_run_rejects_missing_source_and_graph` | missing input error |
| `start_v4_runtime_run_rejects_non_paper_simulated_handoff` | rejected handoff |
| `start_v4_runtime_run_rejects_graph_without_runtime_event_catalog` | event catalog guard |

---

## 本批次不做

- 不移动 `src/runtime/run.rs` 中的 v4 handoff handler 或 helper。
- 不新建 handler 实现文件。
- 不改 `/api/runtime/v4/run` route path、method、payload、response schema 或 error code。
- 不迁移 `run_in_progress`、AppState 字段 owner、runtime state owner、lock order 或 persistence。
- 不扩大 provider 支持，不引入 provider 真连接，也不把 RuntimeSimulated 能力登记解释为真实 provider 可用。
- 不拆 `runtime.run.session_start`、`runtime.run.record_store`、`runtime.run.replay_status` 或 `runtime.event_stream`。
- 不清理 `include!("run.rs")` 带来的 sibling type 共用问题。
- 不宣称 `runtime.run.v4_handoff` 已完成抽离。

---

## 后续判断

若继续本子叶，下一步才允许做 `runtime.run.v4_handoff` 抽离方案，且必须满足:

1. 只迁移 v4 handoff helper 边界，不混入 `start_test_run`。
2. 保持 `start_v4_runtime_run` 的 route handler 对外语义不变。
3. 保留 `api_run` 代表测试作为等价证据。
4. 若引入新文件，必须同步模块树、全量树和治理门禁。

---

## 验收标准

1. `55-runtime.run.v4_handoff单子叶等价基线.md` 进入 v4.16 里程碑索引。
2. 模块树出现 `runtime.run.v4_handoff` 白箱节点。
3. 全量树能定位本基线和真实 runtime 文件。
4. 治理门禁能发现本文件缺失。
5. 后续 v4 handoff 抽离必须引用本基线，不得绕过父模块直接迁移 handler。
