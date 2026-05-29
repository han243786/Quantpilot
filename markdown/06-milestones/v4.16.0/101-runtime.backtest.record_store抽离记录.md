# v4.16.0 runtime.backtest.record_store 抽离记录

> 版本类型: MINOR architecture / governance。  
> 执行档位: 重型。  
> 批次: BE-001T-03。  
> 基准: `100-runtime.backtest.record_store抽离方案.md`、`99-runtime.backtest.record_store单子叶等价基线.md`、`98-runtime.backtest.execution_start父叶残余判断.md`、`77-runtime.backtest单叶closeout.md`。  
> 判定: 按方案完成 `runtime.backtest.record_store` 第一轮实际抽离；只移动四个 handler，不迁移 replay、experiment、compare、shared helper owner、state owner、persistence owner、artifact/transient owner、frontend route 或发布过渡连接。

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001T record_store 从抽离方案进入实际抽离记录 | 推进 |
| 规范矩阵 | 父级 re-export、最小迁移、shared helper owner 保留 | 落地 |
| 引导矩阵 | `root.backend.runtime.routes.runtime.backtest.record_store` | 物理抽离 |
| 模块树 | `runtime.backtest.record_store` | 标记实际抽离完成 |

---

## 引导坐标

| 项 | 指向 |
| --- | --- |
| 引导坐标 | `root.backend.runtime.routes.runtime.backtest.record_store` |
| 全量树节点 | `markdown/10-overview/overview-full-feature-tree.md` 根 2 backend 与根 7 v4.16 |
| 模块树节点 | `markdown/00-matrix-governance/module-tree.md` 的 `runtime.backtest.record_store` |
| 新真实文件 | `src/runtime/backtest/record_store.rs` |
| 保留真实文件 | `src/runtime/backtest.rs`、`src/runtime/mod.rs`、`src/backend/runtime/routes/backtest.rs`、`src/runtime_persistence.rs`、`src/runtime_response_mapping.rs`、`src/backtest_artifacts.rs`、`src/collaboration.rs`、`src/frontend_api_types.rs` |
| public 方法 | `list_backtests`、`get_backtest_detail`、`save_backtest_record`、`discard_backtest_record` |
| 保留 shared helper | `load_backtest_record_from_state`、`list_backtest_records`、`persist_backtest_record`、`delete_transient_backtest_record`、`build_backtest_artifact_views`、`backtest_list_item_from_record`、`backtest_detail_response_from_record`、`sanitize_storage_path_segment`、`persist_graph_audit_entry`、`build_graph_audit_entry` |
| 测试/门禁 | `cargo fmt --check`、`cargo check -p quantpilot`、`cargo test --no-run`、`api_backtest`、`api_evidence_contract`、`api_run`、三矩阵门禁、全量树、UTF-8、`git diff --check` |

---

## 实际移动

| 动作 | 文件 | 结果 |
| --- | --- | --- |
| 新建 record_store 子模块 | `src/runtime/backtest/record_store.rs` | 承载 `list_backtests`、`get_backtest_detail`、`save_backtest_record`、`discard_backtest_record` |
| 删除旧位置四个 handler | `src/runtime/backtest.rs` | 文件继续承载 experiment sweep、replay 和其他 backtest sibling |
| 父级兼容出口 | `src/runtime/mod.rs` | 增加 `backtest_record_store` 私有子模块和 `pub(crate) use` |
| route facade | `src/backend/runtime/routes/backtest.rs` | 未改动，仍调用 `crate::runtime::*` 四个 handler |

父级 re-export 形态:

```rust
#[path = "backtest/record_store.rs"]
mod backtest_record_store;
pub(crate) use backtest_record_store::{
    discard_backtest_record, get_backtest_detail, list_backtests, save_backtest_record,
};
```

---

## 保持不变的行为

| 行为 | 保持方式 |
| --- | --- |
| backtest list | `GET /api/runtime/backtests` 仍只列持久化 backtest records，并按 `created_at_ms` 倒序分页 |
| backtest detail | `GET /api/runtime/backtests/:backtest_id` 仍通过 scoped lookup 读取 memory、artifact directory 或 transient fallback |
| backtest save | `POST /api/runtime/backtests/:backtest_id/save` 仍写 artifact directory、回填 artifact views、清理 transient、actor 存在时写 graph audit |
| backtest discard | `DELETE /api/runtime/backtests/:backtest_id` 仍只丢弃 transient / in-memory record，已保存 artifact directory 返回 conflict |
| route facade | `src/backend/runtime/routes/backtest.rs` 未改 path、method 或 handler 调用名 |
| helper owner | persistence、response mapping、artifact/transient、graph audit helper 和 AppState owner 仍保留原 owner |

---

## 明确未迁移

- 不迁移 `runtime.backtest.replay`，即 `get_backtest_replay` 或 replay response mapping。
- 不迁移 `runtime.backtest.experiment_sweep`，即 `start_backtest_experiment`、experiment save/detail/list/discard 或 variant persistence。
- 不迁移 `backtest_compare`，即 compare core、compare narrative 或 compare route owner。
- 不迁移 `runtime.backtest.execution_start`、`v4_projection`、`v4_request_resolution`、`v4_runtime_execution` 或 `legacy_dispatch`。
- 不迁移 `state.backtests`、`backtest_store_dir`、`transient_backtest_store_dir`、`audit_store_dir` 或 AppState owner。
- 不迁移 `load_backtest_record_from_state`、`list_backtest_records`、`persist_backtest_record`、`sanitize_storage_path_segment`。
- 不迁移 `delete_transient_backtest_record`、`build_backtest_artifact_views`、artifact schema 或 transient quota owner。
- 不迁移 `backtest_list_item_from_record`、`backtest_detail_response_from_record` 或 response schema owner。
- 不迁移 `persist_graph_audit_entry`、`build_graph_audit_entry` 或 graph audit owner。
- 不改 frontend caller、route path、payload、response schema、发布过渡连接、整理或重构。

---

## 回退点

若后续发现行为回归，可将四个 handler 从 `src/runtime/backtest/record_store.rs` 放回 `src/runtime/backtest.rs`，并移除 `src/runtime/mod.rs` 中的 `backtest_record_store` 私有模块与 re-export。`src/backend/runtime/routes/backtest.rs` 不需要回退，因为本批没有修改 route facade。

---

## 验证计划

本批收口必须运行:

```powershell
cargo fmt --check
cargo check -p quantpilot
cargo test --no-run
cargo test -p quantpilot --test api_backtest
cargo test -p quantpilot --test api_evidence_contract
cargo test -p quantpilot --test api_run
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1
git diff --check
```

---

## 下一步

下一批应进入 `runtime.backtest.record_store` 单叶整理 / closeout，确认四个 handler 抽离后与原功能等价，并判断 record_store 内部是否值得继续细拆。当前不能直接拆 shared helper、persistence、audit、artifact/transient 或 response projection。

---

## 幻觉检查点

AI 声称 `runtime.backtest.record_store` 已抽离时，必须说明只迁移了 `list_backtests`、`get_backtest_detail`、`save_backtest_record`、`discard_backtest_record` 四个 handler 到 `src/runtime/backtest/record_store.rs`。不得宣称 replay、experiment、compare、shared helper owner、artifact schema owner、state owner、persistence owner、frontend caller、发布过渡、整理或重构已经完成。ASCII guard: `release transition guard`。

---

## 验收标准

1. `101-runtime.backtest.record_store抽离记录.md` 进入 v4.16 里程碑索引。
2. `src/runtime/backtest/record_store.rs` 进入全量树和模块树。
3. `src/runtime/mod.rs` 保留 `crate::runtime::{list_backtests,get_backtest_detail,save_backtest_record,discard_backtest_record}` 兼容出口。
4. `src/backend/runtime/routes/backtest.rs` route path/method 不变。
5. 治理门禁能发现本抽离记录缺失。
6. `api_backtest`、`api_evidence_contract` 和 `api_run` 证明 backtest record store 与关联 evidence 契约仍可通过。
