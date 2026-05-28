# v4.16.0 runtime.run.replay_status 单子叶等价基线

> 版本类型: MINOR architecture / governance。  
> 执行档位: 重型。  
> 批次: BE-001K-01。  
> 前置: `65-runtime.run.record_store单叶closeout.md`。  
> 判定: 本批只建立 `runtime.run.replay_status` 单子叶等价基线，不移动代码，`no code movement`，不抽离 handler，不迁移 response mapping、state owner、persistence owner 或 SSE。

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | R5 handler sibling 队列从 `runtime.run.record_store` 转向 `runtime.run.replay_status` | 推进 |
| 规范矩阵 | replay/status route、父级出口、response mapping owner、SSE 排除边界 | 冻结 |
| 引导矩阵 | `runtime.run.replay_status` 白箱节点 | 新增基线 |
| 模块树 | `runtime.run.replay_status` | 新增 |

---

## 选择理由

1. `runtime.run.v4_handoff`、`runtime.run.session_start`、`runtime.run.record_store` 已完成单叶 closeout，均停止内部细拆。
2. `src/runtime/run.rs` 当前还保留 `get_run_replay`、`stream_run_events`、`get_run_status` 和后续 legacy blocks。
3. replay/status 是同一个 run record 的读侧投影：都经 `load_run_record_from_state` 读取 record，并经 response mapping 生成 API 响应。
4. SSE 是事件流 route，虽然相邻但输出协议、stream 生命周期和 keep-alive 行为不同，必须留给 `runtime.event_stream` 单独处理。

---

## 引导坐标

| 项 | 指向 |
| --- | --- |
| 引导坐标 | `root.backend.runtime.routes.run.runtime.run.replay_status` |
| 全量树节点 | `markdown/10-overview/overview-full-feature-tree.md` 根 2 backend 与根 7 v4.16 |
| 模块树节点 | `markdown/00-matrix-governance/module-tree.md` 的 `runtime.run.replay_status` |
| 当前真实文件 | `src/runtime/run.rs` |
| route facade | `src/backend/runtime/routes/run.rs` |
| 父级出口 | `src/runtime/mod.rs` 通过 `include!("run.rs")` 暴露 `crate::runtime::{get_run_replay,get_run_status}` |
| response mapping owner | `src/runtime_response_mapping.rs` |
| API schema owner | `src/frontend_api_types.rs` |
| metrics owner | `src/lib.rs` 的 `RuntimeEvidenceMetrics::record_replay_page` |
| 代表测试 | `tests/api_run.rs`、`tests/api_evidence_contract.rs` |

---

## public 方法边界

| 方法 | 输入 | 输出 | 调用方 | 禁止事项 |
| --- | --- | --- | --- | --- |
| `get_run_replay` | `auth::UserId`、`AppState`、`run_id`、`RuntimeReplayQuery` | `Json<RuntimeReplayResponse>` 或 `(StatusCode, String)` | `GET /api/runtime/runs/:run_id/replay` | 不迁移 SSE，不私有化 replay response mapping，不改变 cursor/filter 语义 |
| `get_run_status` | `auth::UserId`、`AppState`、`run_id` | `Json<RunStatusResponse>` 或 `(StatusCode, String)` | `GET /api/runtime/runs/:run_id/status` | 不迁移 detail/list/save/discard，不改变 run record state owner |

---

## 等价冻结项

| 行为 | 当前语义 | 等价证据 |
| --- | --- | --- |
| replay route | `GET /api/runtime/runs/:run_id/replay` 进入 `runtime_handlers::get_run_replay` | `src/backend/runtime/routes/run.rs` |
| status route | `GET /api/runtime/runs/:run_id/status` 进入 `runtime_handlers::get_run_status` | `src/backend/runtime/routes/run.rs` |
| record 读取 | 两个 handler 都通过 `load_run_record_from_state` 读取 current runtime 或 manifest fallback | `src/runtime/run.rs` |
| replay query | `RuntimeReplayQuery` 经 `normalized_replay_options` 处理 cursor、limit、sequence_cursor、stage、severity、retention_class、module_key、key_only | `src/runtime/mod.rs` |
| replay response | `run_replay_response_from_record` 负责排序、过滤、cursor、timeline、checkpoint、next/previous cursor | `src/runtime_response_mapping.rs` |
| bad cursor | replay response mapping 返回错误时转成 `json_bad_request("bad_replay_cursor", message)` | `src/runtime/run.rs` |
| replay metrics | replay 成功后调用 `state.evidence_metrics.record_replay_page(...)` | `src/runtime/run.rs`、`src/lib.rs` |
| status response | `run_status_response_from_record` 返回 `run_id`、`graph_id`、`compile_id`、`event_count`、`account` | `src/runtime_response_mapping.rs` |

---

## 不迁移边界

- 不迁移 `runtime.event_stream`，即 `stream_run_events` 或 `/api/runtime/runs/:run_id/events`。
- 不迁移 `runtime.run.record_store`，即 `list_runs`、`get_run_detail`、`save_run_record`、`discard_run_record`。
- 不迁移 `runtime.run.session_start` 或 `runtime.run.v4_handoff`。
- 不迁移 `runtime_response_mapping` owner，包括 `run_replay_response_from_record` 和 `run_status_response_from_record`。
- 不迁移 `RuntimeReplayQuery`、`RuntimeReplayOptions`、`RuntimeReplayFilters`、`RuntimeReplayResponse`、`RunStatusResponse` schema owner。
- 不迁移 `state.runs`、`run_store_dir`、`audit_store_dir`、`AppState` 或 `RuntimeEvidenceMetrics` owner。
- 不迁移 backtest replay；`get_backtest_replay` 仍属于 backtest route。
- 不改 frontend route 或 API caller。

---

## 等价验证计划

| 命令 | 覆盖 |
| --- | --- |
| `cargo fmt --check` | 文档批次不应留下格式漂移 |
| `cargo check -p quantpilot` | handler 可见性、route facade、schema 类型 |
| `cargo test -p quantpilot --test api_run` | run replay/status 代表链路、bad cursor、governance loaded record |
| `cargo test -p quantpilot --test api_evidence_contract` | replay response contract 与 evidence metrics 代表链路 |
| `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1` | 本基线、模块树、全量树锚点 |
| `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1` | 新文档路径覆盖 |
| `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1` | UTF-8 文档治理 |

---

## 后续入口

本基线通过后，下一批若继续，应进入 `BE-001K-02 runtime.run.replay_status 抽离方案`。方案只能讨论是否把 `get_run_replay` 与 `get_run_status` 迁入计划目标文件，例如 `src/runtime/run/replay_status.rs`；不得直接移动代码，不得混入 SSE、response mapping owner、schema owner、state owner 或 persistence owner。

---

## 幻觉检查点

AI 声称 `runtime.run.replay_status` 已建立基线时，必须说明这只是 replay/status 两个 handler 的等价基线；当前没有迁移代码，没有迁移 `stream_run_events`，没有迁移 `run_replay_response_from_record`、`run_status_response_from_record`、state owner、persistence owner、schema owner 或 frontend route。不得宣称 runtime run handler 全部完成。

---

## 验收标准

1. `66-runtime.run.replay_status单子叶等价基线.md` 进入 v4.16 里程碑索引。
2. 模块树新增 `runtime.run.replay_status` 白箱节点。
3. 全量树覆盖本基线文档。
4. 治理门禁能发现本基线文档缺失。
5. 代表测试继续通过。
