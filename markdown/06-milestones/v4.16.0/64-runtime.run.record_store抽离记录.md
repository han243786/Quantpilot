# v4.16.0 runtime.run.record_store 抽离记录

> 版本类型: MINOR architecture / governance。  
> 执行档位: 重型。  
> 批次: BE-001J-04。  
> 基准: `61-runtime.run.record_store单子叶等价基线.md`、`62-runtime.run.record_store真实边界梳理.md`、`63-runtime.run.record_store抽离方案.md`。  
> 判定: 按方案完成 `runtime.run.record_store` 第一轮实际抽离；只移动四个 handler，不迁移 replay/status、SSE、shared helper owner、state owner、persistence owner 或 frontend route。

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001J record_store 从抽离方案进入实际抽离记录 | 推进 |
| 规范矩阵 | 父级 re-export、最小迁移、shared helper owner 保留 | 落地 |
| 引导矩阵 | `runtime.run.record_store` 白箱节点 | 更新 |
| 模块树 | `runtime.run.record_store` | 标记实际抽离完成 |

---

## 引导坐标

| 项 | 指向 |
| --- | --- |
| 引导坐标 | `root.backend.runtime.routes.run.runtime.run.record_store` |
| 全量树节点 | `markdown/10-overview/overview-full-feature-tree.md` 根 2 backend 与根 7 v4.16 |
| 模块树节点 | `markdown/00-matrix-governance/module-tree.md` 的 `runtime.run.record_store` |
| 新真实文件 | `src/runtime/run/record_store.rs` |
| 保留真实文件 | `src/runtime/run.rs`、`src/runtime/mod.rs`、`src/backend/runtime/routes/run.rs`、`src/runtime_persistence.rs`、`src/runtime_response_mapping.rs`、`src/collaboration.rs`、`frontend/src/store/graphStoreRuntimeHistoryApi.js`、`frontend/src/store/graphStoreRuntimeHistoryFlow.js` |
| public 方法 | `list_runs`、`get_run_detail`、`save_run_record`、`discard_run_record` |
| 保留 shared helper | `load_run_record_from_state`、`persist_run_record`、`list_run_records`、`sanitize_storage_path_segment`、`run_list_item_from_record`、`run_detail_response_from_record`、`persist_graph_audit_entry`、`build_graph_audit_entry` |
| 测试/门禁 | `cargo check -p quantpilot`、`cargo test -p quantpilot --test api_run`、`cargo fmt --check`、`tools/check-matrix-governance.ps1`、`tools/check-full-feature-tree.ps1`、`tools/check-utf8.ps1` |

---

## 实际移动

| 动作 | 文件 | 结果 |
| --- | --- | --- |
| 新建 record_store 子模块 | `src/runtime/run/record_store.rs` | 承载 `list_runs`、`get_run_detail`、`save_run_record`、`discard_run_record` |
| 删除旧位置四个 handler | `src/runtime/run.rs` | 文件开头直接进入 `get_run_replay`；replay/status/SSE sibling 保留原位 |
| 父级兼容出口 | `src/runtime/mod.rs` | 增加 `run_record_store` 私有子模块和 `pub(crate) use` |
| route facade | `src/backend/runtime/routes/run.rs` | 未改动，仍调用 `crate::runtime::*` 四个 handler |

---

## 保持不变的行为

| 行为 | 保持方式 |
| --- | --- |
| run list | `GET /api/runtime/runs` 仍分页并按 `created_at_ms` 倒序 |
| run detail | `GET /api/runtime/runs/:run_id` 仍通过 `load_run_record_from_state` 做 current runtime 优先、manifest fallback |
| run save | `POST /api/runtime/runs/:run_id/save` 仍写 `run_store_dir`，actor 存在时写 graph audit |
| run discard | `DELETE /api/runtime/runs/:run_id` 仍只删除 transient in-memory record，已保存 manifest 返回 conflict |
| frontend path | `discardRunRecord` 仍调用 `/runtime/runs/${runId}`，不引入 `/discard` |
| helper owner | persistence、response mapping、graph audit helper 仍保留原 owner |

---

## 明确未迁移

- 不迁移 `runtime.run.replay_status`，即 `get_run_replay`、`get_run_status`。
- 不迁移 `runtime.event_stream`，即 `stream_run_events` 或 `/api/runtime/runs/:run_id/events`。
- 不迁移 `state.runs`、`run_store_dir`、`audit_store_dir` 或 AppState owner。
- 不迁移 `load_run_record_from_state`、`persist_run_record`、`list_run_records`、`sanitize_storage_path_segment`。
- 不迁移 `run_list_item_from_record`、`run_detail_response_from_record`。
- 不迁移 `persist_graph_audit_entry`、`build_graph_audit_entry`。
- 不改 `frontend/src/store/graphStoreRuntimeHistoryApi.js` 或 `frontend/src/store/graphStoreRuntimeHistoryFlow.js`。

---

## 回退点

若后续发现行为回归，可将四个 handler 从 `src/runtime/run/record_store.rs` 放回 `src/runtime/run.rs`，并移除 `src/runtime/mod.rs` 中的 `run_record_store` 私有模块与 re-export。route facade 不需要回退，因为本批没有修改 `src/backend/runtime/routes/run.rs`。

---

## 验证计划

本批收口必须运行:

```powershell
cargo fmt --check
cargo check -p quantpilot
cargo test -p quantpilot --test api_run
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1
git diff --check
```

---

## 下一步

下一批应进入 `runtime.run.record_store` 单叶整理 / closeout，确认四个 handler 抽离后与原功能等价，并判断 persistence/audit/response projection 是否值得继续细拆。当前不能直接拆 shared helper。

---

## 验收标准

1. `64-runtime.run.record_store抽离记录.md` 进入 v4.16 里程碑索引。
2. `src/runtime/run/record_store.rs` 进入全量树和模块树。
3. `src/runtime/mod.rs` 保留 `crate::runtime::{list_runs,get_run_detail,save_run_record,discard_run_record}` 兼容出口。
4. `src/backend/runtime/routes/run.rs` route path/method 不变。
5. 治理门禁能发现本抽离记录缺失。
6. `api_run` 证明 run list/detail/save/discard 相关服务级契约仍可通过。
