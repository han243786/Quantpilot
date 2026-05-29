# v4.16.0 runtime.backtest.record_store 抽离方案

> 版本类型: MINOR architecture / governance。  
> 执行档位: 重型。  
> 批次: BE-001T-02。  
> 基准: `99-runtime.backtest.record_store单子叶等价基线.md`、`98-runtime.backtest.execution_start父叶残余判断.md`、`77-runtime.backtest单叶closeout.md`。  
> 判定: 建立 `runtime.backtest.record_store` 实际抽离方案；本批只落方案和门禁要求，`no code movement`，不移动代码。  
> 下一步: BE-001T-03 实际抽离记录。

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001T 从等价基线进入实际抽离方案 | 推进 |
| 规范矩阵 | 父级 re-export、route facade、shared helper owner、artifact/transient owner、最小迁移边界 | 固化 |
| 引导矩阵 | `root.backend.runtime.routes.runtime.backtest.record_store` | 细化 |
| 模块树 | `runtime.backtest.record_store` | 补充实施计划 |

---

## 引导坐标

| 项 | 指向 |
| --- | --- |
| 引导坐标 | `root.backend.runtime.routes.runtime.backtest.record_store` |
| 全量树节点 | `markdown/10-overview/overview-full-feature-tree.md` 根 2 backend 与根 7 v4.16 |
| 模块树节点 | `markdown/00-matrix-governance/module-tree.md` 的 `runtime.backtest.record_store` |
| 真实文件 | `src/runtime/backtest.rs`、`src/runtime/mod.rs`、`src/backend/runtime/routes/backtest.rs`、`src/runtime_persistence.rs`、`src/runtime_response_mapping.rs`、`src/backtest_artifacts.rs`、`src/collaboration.rs`、`src/frontend_api_types.rs` |
| 计划目标文件 | `src/runtime/backtest/record_store.rs` |
| public 方法 | `list_backtests`、`get_backtest_detail`、`save_backtest_record`、`discard_backtest_record` |
| 保留 shared helper | `load_backtest_record_from_state`、`list_backtest_records`、`persist_backtest_record`、`delete_transient_backtest_record`、`build_backtest_artifact_views`、`backtest_list_item_from_record`、`backtest_detail_response_from_record`、`sanitize_storage_path_segment`、`persist_graph_audit_entry`、`build_graph_audit_entry` |
| 测试/门禁 | `cargo fmt --check`、`cargo check -p quantpilot`、`cargo test --no-run`、`cargo test -p quantpilot --test api_backtest`、`cargo test -p quantpilot --test api_evidence_contract`、`cargo test -p quantpilot --test api_run`、`tools/check-matrix-governance.ps1`、`tools/check-full-feature-tree.ps1`、`tools/check-utf8.ps1` |

---

## 抽离目标

第一轮实际抽离只移动四个 backtest record store route handler:

| handler | route | 计划归属 | 保持不变 |
| --- | --- | --- | --- |
| `list_backtests` | `GET /api/runtime/backtests` | `src/runtime/backtest/record_store.rs` | 只列持久化 artifact directory，分页、`created_at_ms` 倒序、`backtest_list_item_from_record` 调用不变 |
| `get_backtest_detail` | `GET /api/runtime/backtests/:backtest_id` | `src/runtime/backtest/record_store.rs` | scoped lookup、memory -> artifact directory -> transient fallback 不变 |
| `save_backtest_record` | `POST /api/runtime/backtests/:backtest_id/save` | `src/runtime/backtest/record_store.rs` | artifact persistence、artifact views、transient cleanup、actor 存在时 graph audit 不变 |
| `discard_backtest_record` | `DELETE /api/runtime/backtests/:backtest_id` | `src/runtime/backtest/record_store.rs` | 已保存 artifact directory 返回 conflict，只允许丢弃 transient / in-memory record |

本方案不引入 `DELETE /api/runtime/backtests/:backtest_id/discard` 或任何 `/discard` 后缀。

---

## 实施方案

1. 新建 `src/runtime/backtest/record_store.rs`，只承载四个 handler。
2. 从 `src/runtime/backtest.rs` 移出 `list_backtests`、`get_backtest_detail`、`save_backtest_record`、`discard_backtest_record`。
3. 在 `src/runtime/mod.rs` 增加父级私有子模块和受控 re-export:

```rust
#[path = "backtest/record_store.rs"]
mod backtest_record_store;
pub(crate) use backtest_record_store::{
    discard_backtest_record, get_backtest_detail, list_backtests, save_backtest_record,
};
```

4. 保持 `src/backend/runtime/routes/backtest.rs` 不变；route facade 继续调用 `crate::runtime::{list_backtests,get_backtest_detail,save_backtest_record,discard_backtest_record}`。
5. 保持 `src/runtime/backtest.rs` 继续拥有 experiment sweep、experiment record store、backtest replay 和后续 sibling。
6. 保持 `src/runtime_persistence.rs`、`src/runtime_response_mapping.rs`、`src/backtest_artifacts.rs`、`src/collaboration.rs`、`src/frontend_api_types.rs` owner 不变。
7. 代码移动后再补实际抽离记录，并通过 `api_backtest`、`api_evidence_contract`、`api_run` 证明行为等价。

---

## 明确排除

| 排除项 | 原因 |
| --- | --- |
| `runtime.backtest.replay` | `get_backtest_replay` 与 `backtest_replay_response_from_record` 属于 replay window / response mapping，不进入 record_store 第一轮 |
| `runtime.backtest.experiment_sweep` | `start_backtest_experiment` 复用 `execute_backtest_request`，不属于 record list/detail/save/discard |
| `backtest_compare` | compare 读取 backtest record，但 compare owner 在 `src/backtest_compare.rs`，不迁移 |
| `runtime.backtest.execution_start` | backtest 创建路径已独立 closeout，不回流到 record_store |
| AppState owner | `state.backtests`、`backtest_store_dir`、`transient_backtest_store_dir`、`audit_store_dir` 不迁移 |
| persistence owner | `load_backtest_record_from_state`、`list_backtest_records`、`persist_backtest_record`、`sanitize_storage_path_segment` 不私有化 |
| artifact/transient owner | `build_backtest_artifact_views`、`delete_transient_backtest_record`、artifact schema、transient quota 不迁移 |
| response mapping owner | `backtest_list_item_from_record`、`backtest_detail_response_from_record`、`normalize_backtest_record` 不迁移 |
| graph audit owner | `persist_graph_audit_entry`、`build_graph_audit_entry` 不迁移 |
| frontend API | 不改 path、payload、flow 或 response schema |
| 整理/重构 | 不做目录美化、schema 改名、旧实现删除或测试资产汰换 |

---

## 适配性风险与处理

| 风险 | 处理 |
| --- | --- |
| `include!("backtest.rs")` 与 re-export 重名 | 先移除 `src/runtime/backtest.rs` 中四个 handler，再在 `src/runtime/mod.rs` re-export，避免 duplicate definition |
| 子模块导入缺失 | `record_store.rs` 使用 `use super::*`，再由 `cargo check -p quantpilot` 校验 |
| shared helper 可见性不足 | 优先保持既有 `pub(super)` / module 可见性，不能为了抽离迁移 owner |
| saved conflict 语义误改 | 以 `DELETE /api/runtime/backtests/:backtest_id` 对已保存 artifact directory 返回 conflict 作为硬约束 |
| transient cleanup 漏掉 | `save_backtest_record` 和 `discard_backtest_record` 必须继续调用 `delete_transient_backtest_record` |
| artifact view 语义漂移 | `save_backtest_record` 必须继续从 `persist_backtest_record` 返回 views，并写回 `record.backtest_artifacts` |
| downstream replay/compare 漂移 | `api_backtest` 与 `api_evidence_contract` 必须覆盖 replay、compare、legacy governance fallback |

---

## 中止条件

进入代码移动时，只要出现以下任一情况，应中止并回到方案讨论:

1. 需要改变 route method、route path、response schema 或 error code。
2. 需要把 persistence、artifact/transient、response mapping、audit 或 schema owner 搬进 record_store 私有模块。
3. 需要改 `runtime.backtest.replay`、`runtime.backtest.experiment_sweep`、`backtest_compare`、execution_start 或 frontend state。
4. `cargo check -p quantpilot` 暴露的可见性问题无法通过父级 re-export 或显式 import 解决。
5. `cargo test -p quantpilot --test api_backtest`、`api_evidence_contract` 或 `api_run` 出现行为回归。

---

## 验证计划

实际抽离批次必须至少运行:

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

下一批应进入 BE-001T-03 `runtime.backtest.record_store` 实际抽离记录: 按本方案移动四个 handler 到计划目标文件，保持父级 re-export、route facade、shared helper owner、state owner、artifact/transient owner、persistence owner 和 frontend route 不变。完成后再做单叶 closeout，并判断 record_store 内部是否值得继续细拆。

---

## 幻觉检查点

AI 声称 `runtime.backtest.record_store` 已有抽离方案时，必须说明本批 `no code movement`。不得宣称 `list_backtests`、`get_backtest_detail`、`save_backtest_record`、`discard_backtest_record` 已迁移，不得宣称 replay、experiment、compare、artifact schema owner、state owner、persistence owner、frontend caller、发布过渡、整理或重构已经完成。ASCII guard: `release transition guard`。

---

## 验收标准

1. `100-runtime.backtest.record_store抽离方案.md` 进入 v4.16 里程碑索引。
2. 模块树 `runtime.backtest.record_store` 节点标记实际抽离方案已建立，但代码尚未移动。
3. 全量树能定位本方案、真实文件和下一步计划目标。
4. 治理门禁能发现本方案文档、`no code movement`、下一步 BE-001T-03、禁止迁移边界和回归证据缺失。
5. 后续 BE-001T 实际抽离必须引用本方案，不得把 replay、experiment、compare、shared helper owner、state owner、persistence owner、artifact/transient owner 或 frontend route 混入第一轮迁移。
