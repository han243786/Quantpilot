# v4.16.0 runtime.backtest.record_store 单叶 closeout

> 版本类型: MINOR architecture / governance。  
> 执行档位: 重型。  
> 批次: BE-001T-04。  
> 基准: `101-runtime.backtest.record_store抽离记录.md`、`100-runtime.backtest.record_store抽离方案.md`、`99-runtime.backtest.record_store单子叶等价基线.md`、`77-runtime.backtest单叶closeout.md`。  
> 判定: `runtime.backtest.record_store` 已完成单叶整理 / closeout；本叶在当前抽离阶段停止继续细拆，`stop_split: true`。后续应回到 `runtime.backtest` sibling 队列，默认下一候选为 `runtime.backtest.replay`。

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001T record_store 从实际抽离进入单叶 closeout | 收口 |
| 规范矩阵 | 父级 re-export、shared helper owner、state/persistence/artifact owner、细分停止条件 | 固化 |
| 引导矩阵 | `root.backend.runtime.routes.runtime.backtest.record_store` | 单叶 closeout |
| 模块树 | `runtime.backtest.record_store` | 设置停止细拆 |

---

## 引导坐标

| 项 | 指向 |
| --- | --- |
| 引导坐标 | `root.backend.runtime.routes.runtime.backtest.record_store` |
| 全量树节点 | `markdown/10-overview/overview-full-feature-tree.md` 根 2 backend 与根 7 v4.16 |
| 模块树节点 | `markdown/00-matrix-governance/module-tree.md` 的 `runtime.backtest.record_store` |
| 真实文件 | `src/runtime/backtest/record_store.rs`、`src/runtime/backtest.rs`、`src/runtime/mod.rs`、`src/backend/runtime/routes/backtest.rs`、`src/runtime_persistence.rs`、`src/runtime_response_mapping.rs`、`src/backtest_artifacts.rs`、`src/collaboration.rs`、`src/frontend_api_types.rs` |
| public 方法 | `list_backtests`、`get_backtest_detail`、`save_backtest_record`、`discard_backtest_record` |
| 保留 shared helper | `load_backtest_record_from_state`、`list_backtest_records`、`persist_backtest_record`、`delete_transient_backtest_record`、`build_backtest_artifact_views`、`backtest_list_item_from_record`、`backtest_detail_response_from_record`、`sanitize_storage_path_segment`、`persist_graph_audit_entry`、`build_graph_audit_entry` |
| closeout 判定 | `stop_split: true` |
| 测试/门禁 | `cargo fmt --check`、`cargo check -p quantpilot`、`cargo test --no-run`、`cargo test -p quantpilot --test api_backtest`、`cargo test -p quantpilot --test api_evidence_contract`、`cargo test -p quantpilot --test api_run`、`tools\check-utf8.ps1`、`tools\check-matrix-governance.ps1`、`tools\check-full-feature-tree.ps1`、`git diff --check` |

---

## 等价整理结论

| 维度 | 结论 | 证据 |
| --- | --- | --- |
| route 入口 | 等价 | `GET /api/runtime/backtests`、`GET /api/runtime/backtests/:backtest_id`、`POST /api/runtime/backtests/:backtest_id/save`、`DELETE /api/runtime/backtests/:backtest_id` 仍经 `backend.runtime.routes.backtest -> crate::runtime::*` 暴露 |
| 父级出口 | 等价 | `src/runtime/mod.rs` 保留 `backtest_record_store` 私有子模块与 `pub(crate)` re-export |
| handler 文件 | 已抽离 | `list_backtests`、`get_backtest_detail`、`save_backtest_record`、`discard_backtest_record` 已迁入 `src/runtime/backtest/record_store.rs` |
| record list | 等价 | 仍由 `list_backtest_records` 读取 artifact directory，并通过 `backtest_list_item_from_record`、`paginate`、`created_at_ms` 倒序返回 |
| record detail | 等价 | 仍由 `load_backtest_record_from_state` 处理 scoped memory、artifact directory 与 transient fallback |
| record save | 等价 | 仍写 `backtest_store_dir`，回填 artifact views，清理 transient，actor 存在时继续写 graph audit |
| record discard | 等价 | 仍只丢弃 transient / in-memory record；已保存 artifact directory 返回 conflict；真实 route 仍是 `DELETE /api/runtime/backtests/:backtest_id` |
| shared helper owner | 保留 | persistence、response mapping、artifact/transient、graph audit、path sanitize helper 均保留原 owner |
| sibling 边界 | 保留 | replay、experiment、compare、execution_start sibling 均未迁移 |

---

## 细分价值判断

**最终判定**: `runtime.backtest.record_store` 当前不继续细拆，设置 `stop_split: true`。

它已经是围绕 backtest record list/detail/save/discard 的完整 handler 叶子。继续拆成更小 handler 文件会增加父级 re-export 和导入面，但不会产生新的 owner 清晰度；真正复杂的 persistence、artifact/transient、audit、response mapping 和 AppState owner 都应继续保持共享边界。

| 候选内部子叶 | 判定 | 理由 |
| --- | --- | --- |
| `runtime.backtest.record_store.list_query` | 不拆 | `list_backtests` 只是读取、映射、排序、分页，真实 owner 是 persistence 与 response mapping helper |
| `runtime.backtest.record_store.detail_projection` | 不拆 | `get_backtest_detail` 是薄 handler，projection owner 在 `runtime_response_mapping` |
| `runtime.backtest.record_store.save_persistence` | 不拆 | `persist_backtest_record`、artifact views 与 transient cleanup 是共享 owner，私有化会破坏 replay/report/experiment 复用边界 |
| `runtime.backtest.record_store.save_audit` | 不拆 | graph audit helper owner 在 `collaboration`，本叶只是条件调用 |
| `runtime.backtest.record_store.discard_transient` | 不拆 | discard 的 saved-conflict 检查、in-memory remove 和 transient cleanup 构成同一事务，拆出只会增加桥接成本 |
| `runtime.backtest.persistence` | 暂不在本叶内拆 | 若未来要重构 persistence owner，应另起父级共享节点方案，并经过存储/安全边界校验 |

---

## 父子通信收口

```text
backend.runtime.routes.backtest
  -> crate::runtime::{list_backtests,get_backtest_detail,save_backtest_record,discard_backtest_record}
  -> runtime::backtest_record_store::{list_backtests,get_backtest_detail,save_backtest_record,discard_backtest_record}
  -> runtime_persistence / runtime_response_mapping / backtest_artifacts / collaboration helpers
  -> AppState::{backtests,backtest_store_dir,transient_backtest_store_dir,audit_store_dir}
```

本叶只能经父级 `runtime` re-export 和 `backend.runtime.routes.backtest` 暴露，不得横向接管 `runtime.backtest.execution_start`、`runtime.backtest.replay`、`runtime.backtest.experiment_sweep`、`backtest_compare`、persistence owner、frontend caller 或其他 sibling。发布过渡前不得主动提出缓存旁路或性能优化。ASCII guard: `release transition guard`。

---

## 后续 sibling 队列

| 候选 | 判断 | 进入条件 |
| --- | --- | --- |
| `runtime.backtest.replay` | 默认下一候选 | 覆盖 `get_backtest_replay`，需先建立单子叶等价基线 |
| `runtime.backtest.experiment_sweep` | 值得后续判断 | 覆盖 experiment list/detail/save/discard 和 variant persistence，必须另起基线 |
| `backtest_compare` | 独立 owner | compare core 在 `src/backtest_compare.rs`，若抽离应按 compare owner 单独处理 |
| `runtime.backtest.record_store` | 停止 | 本叶已 closeout，当前不继续细拆 |
| `runtime.backtest.execution_start` | 停止当前轮 | 已完成内部子叶 closeout 与父叶残余判断，不回流 record/state/persistence |

---

## 本批次不做

- 不迁移 `runtime.backtest.replay`，即 `get_backtest_replay` 或 replay response mapping。
- 不迁移 `runtime.backtest.experiment_sweep`，即 `start_backtest_experiment`、experiment save/detail/list/discard 或 variant persistence。
- 不迁移 `backtest_compare`，即 compare core、compare narrative 或 compare route owner。
- 不迁移 `state.backtests`、`backtest_store_dir`、`transient_backtest_store_dir`、`audit_store_dir` 或 AppState owner。
- 不私有化 `load_backtest_record_from_state`、`list_backtest_records`、`persist_backtest_record`、`sanitize_storage_path_segment`。
- 不私有化 `delete_transient_backtest_record`、`build_backtest_artifact_views`、`backtest_list_item_from_record`、`backtest_detail_response_from_record`。
- 不私有化 `persist_graph_audit_entry`、`build_graph_audit_entry`。
- 不改 frontend caller、route path、payload、response schema。
- 不主动提出发布版本过渡或横向连接。

---

## 幻觉检查点

AI 声称 `runtime.backtest.record_store` 已完成时，必须说明只是 backtest record list/detail/save/discard handler 子模块完成抽离与 closeout；`src/runtime/backtest.rs` 仍拥有 replay、experiment 和其他 sibling；state owner、shared helper owner、persistence owner、artifact/transient owner、frontend route、发布版本过渡、整理和重构均未完成。不得宣称 backtest handler 全部完成。

---

## 验收标准

1. `102-runtime.backtest.record_store单叶closeout.md` 进入 v4.16 里程碑索引。
2. 模块树标记 `runtime.backtest.record_store` closeout 完成并设置 `stop_split: true`。
3. 全量树覆盖本 closeout 文档与 `src/runtime/backtest/record_store.rs`。
4. 治理门禁能发现本 closeout 文档、`stop_split: true`、禁止迁移边界、下一候选和回归证据缺失。
5. `api_backtest`、`api_evidence_contract` 和 `api_run` 代表测试继续通过。
