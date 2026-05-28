# v4.16.0 runtime.run.record_store 真实边界梳理

> 版本类型: MINOR architecture / governance。  
> 执行档位: 重型。  
> 批次: BE-001J-02。  
> 基准: `61-runtime.run.record_store单子叶等价基线.md`。  
> 判定: 梳理 `runtime.run.record_store` 当前真实边界，为后续实际抽离方案做准备；本批只校正和登记边界，不移动代码。

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001J record_store 抽离前真实边界复核 | 扩展 |
| 规范矩阵 | route method、state/persistence/audit owner、共享 helper 调用者 | 固化 |
| 引导矩阵 | `runtime.run.record_store` 白箱节点 | 细化 |
| 模块树 | `runtime.run.record_store` | 补充真实边界 |

---

## 引导坐标

| 项 | 指向 |
| --- | --- |
| 引导坐标 | `root.backend.runtime.routes.run.runtime.run.record_store` |
| 全量树节点 | `markdown/10-overview/overview-full-feature-tree.md` 根 2 backend 与根 7 v4.16 |
| 模块树节点 | `markdown/00-matrix-governance/module-tree.md` 的 `runtime.run.record_store` |
| 真实文件 | `src/runtime/run.rs`、`src/backend/runtime/routes/run.rs`、`src/runtime_persistence.rs`、`src/runtime_response_mapping.rs`、`src/collaboration.rs`、`frontend/src/store/graphStoreRuntimeHistoryApi.js`、`frontend/src/store/graphStoreRuntimeHistoryFlow.js` |
| public 方法 | `list_runs`、`get_run_detail`、`save_run_record`、`discard_run_record`、`fetchRunHistoryList`、`fetchRunDetail`、`saveRunRecord`、`discardRunRecord` |
| 测试/门禁 | `cargo check -p quantpilot`、`cargo test -p quantpilot --test api_run`、`tools/check-matrix-governance.ps1`、`tools/check-full-feature-tree.ps1` |

---

## 边界校正结论

`runtime.run.record_store` 当前真实边界是四个 handler，而不是一个独立文件或完整 persistence owner:

| handler | 真实 route | 当前文件 | 真实职责 |
| --- | --- | --- | --- |
| `list_runs` | `GET /api/runtime/runs` | `src/runtime/run.rs` | 从 `run_store_dir` 读取已保存 manifest，映射为 list item，按 `created_at_ms` 倒序分页 |
| `get_run_detail` | `GET /api/runtime/runs/:run_id` | `src/runtime/run.rs` | 通过 `load_run_record_from_state` 读取 current runtime 或 saved manifest，再映射 detail response |
| `save_run_record` | `POST /api/runtime/runs/:run_id/save` | `src/runtime/run.rs` | 读取 current/saved run record，写入 `run_store_dir`，若存在 actor 则写 graph audit |
| `discard_run_record` | `DELETE /api/runtime/runs/:run_id` | `src/runtime/run.rs` | 仅允许丢弃 transient in-memory record；若 manifest 已存在则返回 conflict |

防幻觉校正点: 当前没有 `DELETE /api/runtime/runs/:run_id/discard` 或 `/api/runtime/runs/:run_id/discard` route。前端真实调用同样是 `deleteJson(`/runtime/runs/${runId}`)`。

---

## 当前真实调用链

```text
frontend/src/store/graphStoreRuntimeHistoryApi.js
  -> fetchRunHistoryList / fetchRunDetail / saveRunRecord / discardRunRecord
  -> backend.runtime.routes.run::register_routes
  -> crate::runtime::{list_runs,get_run_detail,save_run_record,discard_run_record}
  -> runtime_persistence / runtime_response_mapping / collaboration audit / AppState
```

| 调用侧 | 真实文件 | 说明 |
| --- | --- | --- |
| route facade | `src/backend/runtime/routes/run.rs` | 注册 run group；record_store 四条 route 与 replay/status 共处同一 facade |
| handler | `src/runtime/run.rs` | 1-80 行附近是 record_store 四个 handler；后续 replay/SSE/status 不是本子叶 |
| frontend API | `frontend/src/store/graphStoreRuntimeHistoryApi.js` | list/detail/save/delete 四个函数直接对应 record_store |
| frontend flow | `frontend/src/store/graphStoreRuntimeHistoryFlow.js` | save 后刷新历史并重载 detail；discard 后 reset runtime state |
| tests | `tests/api_run.rs` | 覆盖 created run contract、save、list、detail、legacy manifest governance |

---

## 共享 helper 真实归属

record_store 会调用多个 helper，但它们当前不是 record_store 私有 owner:

| helper | 当前文件 | 其他调用者 | 本轮判断 |
| --- | --- | --- | --- |
| `load_run_record_from_state` | `src/runtime_persistence.rs` | replay/status、report、mutation、AI proposal 相关路径 | 共享 lookup，不应在第一轮抽离中私有搬入 record_store |
| `persist_run_record` | `src/runtime_persistence.rs` | mutation 激活路径也会写 run record | 共享 persistence helper，不应改变 owner |
| `list_run_records` | `src/runtime_persistence.rs` | 当前主要服务 `list_runs` | 可继续保留 persistence owner；是否随 handler 迁移需另行判断 |
| `sanitize_storage_path_segment` | `src/runtime_persistence.rs` | graph、backtest、experiment、report、proposal 多处复用 | 全局 storage safety helper，不得搬入 record_store |
| `run_list_item_from_record` | `src/runtime_response_mapping.rs` | 当前主要服务 run list | response mapping owner 保留 |
| `run_detail_response_from_record` | `src/runtime_response_mapping.rs` | run detail 与 response mapping 内部复用 | response mapping owner 保留 |
| `persist_graph_audit_entry` / `build_graph_audit_entry` | `src/collaboration.rs` | graph/backtest 等 audit 路径 | graph audit owner 保留 |

因此后续第一轮实际抽离的最窄边界应优先只移动四个 handler。共享 helper 可以通过父级 `runtime` 模块或现有 sibling owner 调用，不应为了目录漂亮而一并迁移。

---

## 状态与副作用边界

| 状态/副作用 | 当前 owner | record_store 使用方式 | 禁止变更 |
| --- | --- | --- | --- |
| `state.runs` | `AppState` | detail/save/replay/status 读取；discard 删除 transient record | 不迁移 AppState owner，不改 scoped key |
| `run_store_dir` | `AppState` + runtime persistence | list/read/save manifest | 不改保存路径、bounded read、atomic write |
| `audit_store_dir` | `AppState` + collaboration | save 时 actor 存在才写 audit | 不改 audit action 或 actor guard |
| `PaginationQuery` / `paginate` | frontend API types | list runs 分页 | 不改 limit/offset 上限 |
| saved manifest conflict | `discard_run_record` | manifest 已存在时 discard 返回 `409 CONFLICT` | 不允许删除已保存 run manifest |

---

## 排除边界

| 非本子叶 | 真实入口 | 原因 |
| --- | --- | --- |
| `runtime.run.replay_status` | `get_run_replay`、`get_run_status` | 虽共享 `load_run_record_from_state`，但 response projection、metrics 和 replay cursor 属独立职责 |
| `runtime.event_stream` | `GET /api/runtime/runs/:run_id/events` | route 在 `backend.runtime.routes` 父级，不属于 run route facade 的 record_store |
| `runtime.run.session_start` | `POST /api/runtime/test-run` | 已 closeout，不能继续混入 record_store |
| `runtime.run.v4_handoff` | `POST /api/runtime/v4/run` | 已 closeout，不能混入 record_store |
| mutation/report/AI proposal | `src/runtime/mutation.rs`、`src/runtime/mod.rs` | 会读/写 run record，但不是 record_store route handler |

---

## 后续抽离建议

下一步若进入实际抽离，应采用最小迁移:

1. 新建 `src/runtime/run/record_store.rs`。
2. 只迁移 `list_runs`、`get_run_detail`、`save_run_record`、`discard_run_record` 四个 handler。
3. `src/runtime/mod.rs` 增加私有子模块和 `pub(crate) use` 兼容出口。
4. `src/runtime/run.rs` 保留 replay/status/SSE 和后续 legacy blocks。
5. 不迁移 `load_run_record_from_state`、`persist_run_record`、`sanitize_storage_path_segment`、response mapping 或 audit helper owner。
6. 不改前端 API route，尤其不得引入 `/discard` 后缀。

---

## 验收标准

1. `62-runtime.run.record_store真实边界梳理.md` 进入 v4.16 里程碑索引。
2. 模块树 `runtime.run.record_store` 节点标记真实 route/method 校正完成。
3. 全量树能定位本梳理文档和真实 frontend/backend 文件。
4. 治理门禁能发现本梳理文档缺失。
5. 后续 BE-001J 实际抽离必须引用本梳理，不得把 shared helper owner 或 replay/status/SSE 混入第一轮迁移。
