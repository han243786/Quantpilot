# v4.16.0 runtime.run.record_store 抽离方案

> 版本类型: MINOR architecture / governance。  
> 执行档位: 重型。  
> 批次: BE-001J-03。  
> 基准: `61-runtime.run.record_store单子叶等价基线.md`、`62-runtime.run.record_store真实边界梳理.md`。  
> 判定: 建立 `runtime.run.record_store` 实际抽离方案；本批只落方案和门禁要求，no code movement，不移动代码。

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001J record_store 从真实边界进入实际抽离方案 | 推进 |
| 规范矩阵 | 父级 re-export、shared helper owner、route method、最小迁移边界 | 固化 |
| 引导矩阵 | `runtime.run.record_store` 白箱节点 | 细化 |
| 模块树 | `runtime.run.record_store` | 补充实施计划 |

---

## 引导坐标

| 项 | 指向 |
| --- | --- |
| 引导坐标 | `root.backend.runtime.routes.run.runtime.run.record_store` |
| 全量树节点 | `markdown/10-overview/overview-full-feature-tree.md` 根 2 backend 与根 7 v4.16 |
| 模块树节点 | `markdown/00-matrix-governance/module-tree.md` 的 `runtime.run.record_store` |
| 真实文件 | `src/runtime/run.rs`、`src/runtime/mod.rs`、`src/backend/runtime/routes/run.rs`、`src/runtime_persistence.rs`、`src/runtime_response_mapping.rs`、`src/collaboration.rs`、`frontend/src/store/graphStoreRuntimeHistoryApi.js`、`frontend/src/store/graphStoreRuntimeHistoryFlow.js` |
| 计划目标文件 | `src/runtime/run/record_store.rs` |
| public 方法 | `list_runs`、`get_run_detail`、`save_run_record`、`discard_run_record` |
| 保留 shared helper | `load_run_record_from_state`、`persist_run_record`、`list_run_records`、`sanitize_storage_path_segment`、`run_list_item_from_record`、`run_detail_response_from_record`、`persist_graph_audit_entry`、`build_graph_audit_entry` |
| 测试/门禁 | `cargo check -p quantpilot`、`cargo test -p quantpilot --test api_run`、`cargo fmt --check`、`tools/check-matrix-governance.ps1`、`tools/check-full-feature-tree.ps1`、`tools/check-utf8.ps1` |

---

## 抽离目标

第一轮实际抽离只移动四个 record store route handler:

| handler | route | 计划归属 | 保持不变 |
| --- | --- | --- | --- |
| `list_runs` | `GET /api/runtime/runs` | `src/runtime/run/record_store.rs` | 分页、`created_at_ms` 倒序、`run_list_item_from_record` 调用 |
| `get_run_detail` | `GET /api/runtime/runs/:run_id` | `src/runtime/run/record_store.rs` | scoped lookup、in-memory 优先、manifest fallback |
| `save_run_record` | `POST /api/runtime/runs/:run_id/save` | `src/runtime/run/record_store.rs` | persistence 写入、actor 存在时写 graph audit |
| `discard_run_record` | `DELETE /api/runtime/runs/:run_id` | `src/runtime/run/record_store.rs` | 已保存 manifest 返回 conflict，只允许丢弃 transient record |

本方案不引入 `DELETE /api/runtime/runs/:run_id/discard` 或任何 `/discard` 后缀。

---

## 实施方案

1. 新建 `src/runtime/run/record_store.rs`，只承载四个 handler。
2. 从 `src/runtime/run.rs` 移出 `list_runs`、`get_run_detail`、`save_run_record`、`discard_run_record`。
3. 在 `src/runtime/mod.rs` 增加私有子模块:

```rust
#[path = "run/record_store.rs"]
mod run_record_store;
pub(crate) use run_record_store::{
    discard_run_record, get_run_detail, list_runs, save_run_record,
};
```

4. 保持 `src/backend/runtime/routes/run.rs` 不变；route facade 继续调用 `crate::runtime::{list_runs,get_run_detail,save_run_record,discard_run_record}`。
5. 保持 `src/runtime/run.rs` 继续拥有 `get_run_replay`、`get_run_status`、SSE/legacy blocks 和后续 sibling。
6. 保持 `src/runtime_persistence.rs`、`src/runtime_response_mapping.rs`、`src/collaboration.rs` owner 不变。
7. 代码移动后再补实际抽离记录，并通过 `api_run` 证明行为等价。

---

## 明确排除

| 排除项 | 原因 |
| --- | --- |
| `runtime.run.replay_status` | `get_run_replay`、`get_run_status` 属于 replay/status response projection，不进入 record_store 第一轮 |
| `runtime.event_stream` | `/api/runtime/runs/:run_id/events` 仍在 `backend.runtime.routes` 父级，不属于本子叶 |
| AppState owner | `state.runs`、`run_store_dir`、`audit_store_dir` 不迁移 |
| persistence owner | `load_run_record_from_state`、`persist_run_record`、`list_run_records`、`sanitize_storage_path_segment` 不私有化 |
| response mapping owner | `run_list_item_from_record`、`run_detail_response_from_record` 不迁移 |
| graph audit owner | `persist_graph_audit_entry`、`build_graph_audit_entry` 不迁移 |
| frontend API | `fetchRunHistoryList`、`fetchRunDetail`、`saveRunRecord`、`discardRunRecord` 不改 path 或 flow |
| 整理/重构 | 不做目录美化、schema 改名、旧实现删除或测试资产汰换 |

---

## 适配性风险与处理

| 风险 | 处理 |
| --- | --- |
| `include!("run.rs")` 与 re-export 重名 | 先移除 `src/runtime/run.rs` 中四个 handler，再在父级 re-export，避免 duplicate definition |
| 子模块导入缺失 | `record_store.rs` 使用显式 `use` 或 `use super::*` 后由 `cargo check -p quantpilot` 校验 |
| shared helper 可见性不足 | 优先保持既有 `pub(crate)` / module 可见性，不能为了抽离而迁移 owner |
| discard 语义误改 | 以 `DELETE /api/runtime/runs/:run_id` 和 `409 CONFLICT` 作为硬约束 |
| frontend 流程漂移 | 本批不改 `graphStoreRuntimeHistoryApi.js` 与 `graphStoreRuntimeHistoryFlow.js` |

---

## 中止条件

进入代码移动时，只要出现以下任一情况，应中止并回到方案讨论:

1. 需要改变 route method、route path、response schema 或 error code。
2. 需要把 shared helper owner 搬进 record_store 私有模块。
3. 需要改 `runtime.run.replay_status`、`runtime.event_stream`、backtest、mutation 或 frontend state。
4. `cargo check -p quantpilot` 暴露的可见性问题无法通过父级 re-export 或显式 import 解决。
5. `cargo test -p quantpilot --test api_run` 出现行为回归。

---

## 验证计划

实际抽离批次必须至少运行:

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

下一批应进入 BE-001J-04 `runtime.run.record_store` 实际抽离记录: 按本方案移动四个 handler 到 `src/runtime/run/record_store.rs`，保持父级 re-export、route facade、shared helper owner、state owner 和 frontend route 不变。完成后再做单叶 closeout，并判断 persistence/audit/response projection 是否值得继续细拆。

---

## 验收标准

1. `63-runtime.run.record_store抽离方案.md` 进入 v4.16 里程碑索引。
2. 模块树 `runtime.run.record_store` 节点标记实际抽离方案已建立，但代码尚未移动。
3. 全量树能定位本方案、真实文件和下一步计划目标。
4. 治理门禁能发现本方案文档缺失。
5. 后续 BE-001J 实际抽离必须引用本方案，不得把 `runtime.run.replay_status`、`runtime.event_stream`、shared helper owner、state owner 或 persistence owner 混入第一轮迁移。
