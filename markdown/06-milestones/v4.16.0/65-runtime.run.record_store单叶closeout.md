# v4.16.0 runtime.run.record_store 单叶 closeout

> 版本类型: MINOR architecture / governance。  
> 执行档位: 重型。  
> 批次: BE-001J-05。  
> 基准: `61-runtime.run.record_store单子叶等价基线.md`、`62-runtime.run.record_store真实边界梳理.md`、`63-runtime.run.record_store抽离方案.md`、`64-runtime.run.record_store抽离记录.md`。  
> 判定: `runtime.run.record_store` 已完成单叶整理 / closeout；本叶在当前抽离阶段停止继续细拆，`stop_split: true`。后续应回到 `runtime.run` sibling 队列，优先为 `runtime.run.replay_status` 另起等价基线。

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001J record_store 从实际抽离进入单叶 closeout | 收口 |
| 规范矩阵 | 父级 re-export、shared helper owner、state/persistence owner、细分停止条件 | 固化 |
| 引导矩阵 | `runtime.run.record_store` 白箱节点 | closeout |
| 模块树 | `runtime.run.record_store` | closeout |

---

## 引导坐标

| 项 | 指向 |
| --- | --- |
| 引导坐标 | `root.backend.runtime.routes.run.runtime.run.record_store` |
| 全量树节点 | `markdown/10-overview/overview-full-feature-tree.md` 根 2 backend 与根 7 v4.16 |
| 模块树节点 | `markdown/00-matrix-governance/module-tree.md` 的 `runtime.run.record_store` |
| 真实文件 | `src/runtime/run/record_store.rs`、`src/runtime/mod.rs`、`src/runtime/run.rs`、`src/backend/runtime/routes/run.rs`、`src/runtime_persistence.rs`、`src/runtime_response_mapping.rs`、`src/collaboration.rs` |
| public 方法 | `list_runs`、`get_run_detail`、`save_run_record`、`discard_run_record` |
| 保留 shared helper | `load_run_record_from_state`、`persist_run_record`、`list_run_records`、`sanitize_storage_path_segment`、`run_list_item_from_record`、`run_detail_response_from_record`、`persist_graph_audit_entry`、`build_graph_audit_entry` |
| 测试/门禁 | `cargo fmt --check`、`cargo check -p quantpilot`、`cargo test -p quantpilot --test api_run`、`tools/check-matrix-governance.ps1`、`tools/check-full-feature-tree.ps1`、`tools/check-utf8.ps1` |

---

## 等价整理结论

| 维度 | 结论 | 证据 |
| --- | --- | --- |
| route 入口 | 等价 | `GET /api/runtime/runs`、`GET /api/runtime/runs/:run_id`、`POST /api/runtime/runs/:run_id/save`、`DELETE /api/runtime/runs/:run_id` 仍经 `backend.runtime.routes.run -> crate::runtime::*` 暴露 |
| 父级出口 | 等价 | `src/runtime/mod.rs` 保留 `run_record_store` 私有子模块与 `pub(crate)` re-export |
| handler 文件 | 已抽离 | `list_runs`、`get_run_detail`、`save_run_record`、`discard_run_record` 已迁入 `src/runtime/run/record_store.rs` |
| record list | 等价 | 仍由 `list_run_records` 读取 manifest，并通过 `run_list_item_from_record`、`paginate`、`created_at_ms` 倒序返回 |
| record detail | 等价 | 仍由 `load_run_record_from_state` 处理 current runtime 优先与 manifest fallback |
| record save | 等价 | 仍写 `run_store_dir`，且 actor 存在时继续通过 graph audit helper 写审计 |
| record discard | 等价 | 仍只丢弃 transient in-memory record；已保存 manifest 返回 conflict；真实 route 仍是 `DELETE /api/runtime/runs/:run_id` |
| shared helper owner | 保留 | persistence、response mapping、graph audit、path sanitize helper 均保留原 owner |
| sibling 边界 | 保留 | `runtime.run.replay_status` 与 `runtime.event_stream` 未迁移 |

---

## 细分价值判断

**最终判定**: `runtime.run.record_store` 当前不继续细拆。它已经是围绕 run record list/detail/save/discard 的完整 handler 叶子；继续拆会把 shared helper 包装成更小 facade，但不会产生新的 owner 清晰度。

| 候选内部子叶 | 判定 | 理由 |
| --- | --- | --- |
| `runtime.run.record_store.list_query` | 不拆 | `list_runs` 只是读取、映射、排序、分页，真实 owner 是 persistence 与 response mapping helper |
| `runtime.run.record_store.detail_projection` | 不拆 | `get_run_detail` 是薄 handler，projection owner 在 `runtime_response_mapping` |
| `runtime.run.record_store.save_persistence` | 不拆 | `persist_run_record` 是共享 persistence owner；把它私有化会破坏后续 replay/report 复用边界 |
| `runtime.run.record_store.save_audit` | 不拆 | graph audit helper owner 在 `collaboration`，本叶只是条件调用 |
| `runtime.run.record_store.discard_transient` | 不拆 | discard 的安全路径检查和 in-memory remove 构成同一事务；当前拆出只会增加父子桥成本 |
| `runtime.record_persistence` | 暂不在本叶内拆 | 若未来要重构 persistence owner，应另起父级共享节点方案，并经过安全/存储决策暂停 |

因此本叶 closeout 后，递归流程应回到 `runtime.run` sibling 队列，而不是继续切 `src/runtime/run/record_store.rs`。

---

## 父子通信收口

```text
backend.runtime.routes.run
  -> crate::runtime::{list_runs,get_run_detail,save_run_record,discard_run_record}
  -> runtime::run_record_store::{list_runs,get_run_detail,save_run_record,discard_run_record}
  -> runtime_persistence / runtime_response_mapping / collaboration helpers
  -> AppState::{runs,run_store_dir,audit_store_dir}
```

本叶只能经父级 `runtime` re-export 和 `backend.runtime.routes.run` 暴露，不得横向接管 `runtime.run.replay_status`、`runtime.event_stream`、`runtime.run.session_start`、`runtime.run.v4_handoff`、backtest、mutation、executor 或 frontend state。

---

## 后续 sibling 队列

| 候选 | 判断 | 进入条件 |
| --- | --- | --- |
| `runtime.run.replay_status` | 默认下一候选 | 覆盖 `get_run_replay`、`get_run_status`，需先建立单子叶等价基线 |
| `runtime.event_stream` | 值得但属父级 route 子叶 | SSE 仍由 `backend.runtime.routes` 直接拥有，不属于 `backend.runtime.routes.run` facade |
| `runtime.run.record_store` | 停止 | 本叶已 closeout，当前不继续细拆 |
| `runtime.run.session_start` | 停止 | 已在 BE-001I-03 closeout，当前不继续细拆 |
| `runtime.run.v4_handoff` | 停止 | 已在 BE-001H-03 closeout，当前不继续细拆 |

---

## 本批次不做

- 不迁移 `runtime.run.replay_status`，即 `get_run_replay`、`get_run_status`。
- 不迁移 `runtime.event_stream`，即 `stream_run_events` 或 `/api/runtime/runs/:run_id/events`。
- 不迁移 `state.runs`、`run_store_dir`、`audit_store_dir` 或 AppState owner。
- 不私有化 `load_run_record_from_state`、`persist_run_record`、`list_run_records`、`sanitize_storage_path_segment`。
- 不私有化 `run_list_item_from_record`、`run_detail_response_from_record`、`persist_graph_audit_entry`、`build_graph_audit_entry`。
- 不改 `frontend/src/store/graphStoreRuntimeHistoryApi.js` 或 `frontend/src/store/graphStoreRuntimeHistoryFlow.js`。
- 不主动提出发布版本过渡或横向连接。

---

## 幻觉检查点

AI 声称 `runtime.run.record_store` 已完成时，必须说明只是 run record list/detail/save/discard handler 子模块完成抽离与 closeout；discard 真实 route 是 `DELETE /api/runtime/runs/:run_id`；`src/runtime/run.rs` 仍拥有 replay/status/SSE sibling；state owner、shared helper owner、persistence owner、frontend route 和发布版本过渡均未完成。不得宣称 runtime run handler 全部完成。

---

## 验收标准

1. `65-runtime.run.record_store单叶closeout.md` 进入 v4.16 里程碑索引。
2. 模块树标记 `runtime.run.record_store` closeout 完成并停止内部细分。
3. 全量树覆盖本 closeout 文档与 `src/runtime/run/record_store.rs`。
4. 治理门禁能发现本 closeout 文档缺失。
5. `api_run` 代表测试继续通过。
