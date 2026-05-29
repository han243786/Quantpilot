# v4.16.0 runtime.backtest.experiment_sweep.record_lifecycle 抽离记录

> 版本类型: MINOR architecture / governance。  
> 执行档位: 重型。  
> 批次: BE-001AA-03。  
> 基准: `122-runtime.backtest.experiment_sweep.record_lifecycle抽离方案.md`、`121-runtime.backtest.experiment_sweep.record_lifecycle单子叶等价基线.md`、`120-runtime.backtest.experiment_sweep第二轮父叶残余判断.md`。  
> 判定: 按方案完成 `runtime.backtest.experiment_sweep.record_lifecycle` 第一轮实际抽离；只迁移 `list_experiments`、`get_experiment_detail`、`save_experiment_record`、`discard_experiment_record`，不迁移 route registration、parameter_grid、start_orchestration、schema、state、persistence、response mapping、audit、frontend caller 或发布过渡连接。

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001AA record_lifecycle 从抽离方案进入实际抽离记录 | 推进 |
| 规范矩阵 | 父级私有子模块、受控 re-export、父子通信、禁止横向连接 | 落地 |
| 引导矩阵 | `root.backend.runtime.routes.runtime.backtest.experiment_sweep.record_lifecycle` | 物理抽离 |
| 模块树 | `runtime.backtest.experiment_sweep.record_lifecycle` | 标记实际抽离完成 |

---

## 引导坐标

| 项 | 指向 |
| --- | --- |
| 引导坐标 | `root.backend.runtime.routes.runtime.backtest.experiment_sweep.record_lifecycle` |
| 父模块 | `runtime.backtest.experiment_sweep` |
| 全量树节点 | `markdown/10-overview/overview-full-feature-tree.md` 根 2 backend 与根 7 v4.16 |
| 模块树节点 | `markdown/00-matrix-governance/module-tree.md` 的 `runtime.backtest.experiment_sweep.record_lifecycle` |
| 新真实文件 | `src/runtime/backtest/record_lifecycle.rs` |
| 父级真实文件 | `src/runtime/backtest/experiment_sweep.rs` |
| 保留 sibling | `src/runtime/backtest/parameter_grid.rs`、`src/runtime/backtest/start_orchestration.rs` |
| 保留 shared owner | `src/runtime/backtest/execution_start.rs`、`src/runtime/backtest.rs`、`src/runtime/mod.rs`、`src/backend/runtime/routes.rs`、`src/runtime_persistence.rs`、`src/runtime_response_mapping.rs`、`src/frontend_api_types.rs`、`src/backtest_artifacts.rs` |
| public 方法 | `list_experiments`、`get_experiment_detail`、`save_experiment_record`、`discard_experiment_record` |
| 已迁移方法 | `list_experiments`、`get_experiment_detail`、`save_experiment_record`、`discard_experiment_record` |
| 父级 re-export | `pub(crate) use record_lifecycle::{discard_experiment_record,get_experiment_detail,list_experiments,save_experiment_record};` |
| 子模块导入 | `use super::*;` |
| 保留 route owner | `backend.runtime.routes` / backtest route facade |
| 测试/门禁 | `cargo fmt --check`、`cargo check -p quantpilot`、`cargo test --no-run`、`cargo test -p quantpilot --test api_experiments`、`cargo test -p quantpilot --test api_backtest`、`cargo test -p quantpilot --test api_evidence_contract`、`tools\check-matrix-governance.ps1`、`tools\check-full-feature-tree.ps1`、`tools\check-utf8.ps1`、`git diff --check` |

---

## 实际移动

| 动作 | 文件 | 结果 |
| --- | --- | --- |
| 新建 record_lifecycle 子模块 | `src/runtime/backtest/record_lifecycle.rs` | 承载 experiment list/detail/save/discard lifecycle handlers |
| 父级声明私有模块 | `src/runtime/backtest/experiment_sweep.rs` | 增加 `mod record_lifecycle;` |
| 父级受控出口 | `src/runtime/backtest/experiment_sweep.rs` | 增加 `pub(crate) use record_lifecycle::{...};` |
| 保留 start orchestration | `src/runtime/backtest/start_orchestration.rs` | `start_backtest_experiment` 仍归 start_orchestration |
| 保留 parameter grid | `src/runtime/backtest/parameter_grid.rs` | `build_experiment_overrides` 仍归 parameter_grid |
| 保留兼容出口 | `src/runtime/mod.rs` | `crate::runtime::{list_experiments,get_experiment_detail,save_experiment_record,discard_experiment_record}` 语义不变 |

父级形态:

```rust
use super::*;

mod parameter_grid;
mod record_lifecycle;
mod start_orchestration;

pub(crate) use record_lifecycle::{
    discard_experiment_record, get_experiment_detail, list_experiments, save_experiment_record,
};
pub(crate) use start_orchestration::start_backtest_experiment;
```

子模块形态:

```rust
use super::*;

pub(crate) async fn list_experiments(
    State(state): State<AppState>,
    Query(pagination): Query<PaginationQuery>,
) -> Result<Json<PaginatedResponse<ExperimentListItem>>, (StatusCode, String)>
```

---

## 保持不变的行为

| 行为 | 保持方式 |
| --- | --- |
| list read | 继续调用 `list_experiment_records(state.experiment_store_dir.as_ref())` |
| list projection | 继续 `.map(experiment_list_item_from_record)` |
| list order | 继续按 `created_at_ms` 倒序，再调用 `paginate(items, pagination)` |
| detail lookup | 继续 `load_experiment_record_from_state(&state, &user_id, &experiment_id)` |
| detail response | 继续 `experiment_detail_response_from_record(record)` |
| save variant persistence | 每个 variant 继续加载 backtest record 并调用 `persist_backtest_record` |
| transient cleanup | save/discard 继续调用 `delete_transient_backtest_record` |
| state cache | save 写 `state.experiments`，discard 清 `state.experiments` 和 transient `state.backtests` |
| audit | actor 存在时继续写 `GraphAuditAction::ExperimentCreated`，失败仍冒泡 |
| saved conflict | saved experiment discard 继续返回 `StatusCode::CONFLICT` |
| safe path | discard 继续使用 `sanitize_storage_path_segment` |
| discard response | `discarded_kind` 继续固定为 `experiment` |

---

## 明确未迁移

- 不迁移 `start_backtest_experiment`、`build_experiment_overrides`、`execute_backtest_request`。
- 不迁移 route registration；route owner 仍是 `backend.runtime.routes` / backtest route facade。
- 不迁移 `runtime.backtest.execution_start`、`runtime.backtest.record_store`、`runtime.backtest.replay`、`backtest_compare`。
- 不迁移 `MAX_EXPERIMENT_VARIANTS`、request/response schema、artifact schema 或 frontend caller。
- 不迁移 persistence owner、response mapping owner、state owner、audit owner、整理、重构或发布过渡连接。ASCII guard: `release transition guard`。

---

## 回退点

若后续发现行为回归，可将 `list_experiments`、`get_experiment_detail`、`save_experiment_record`、`discard_experiment_record` 从 `src/runtime/backtest/record_lifecycle.rs` 放回 `src/runtime/backtest/experiment_sweep.rs`，并移除父级的 `mod record_lifecycle;` 与 `pub(crate) use record_lifecycle::{...};`。不需要回退 route、schema、state、persistence、response mapping、audit 或 frontend 文件，因为本批未修改这些 owner。

---

## 验证计划

本批收口必须运行:

```powershell
cargo fmt --check
cargo check -p quantpilot
cargo test --no-run
cargo test -p quantpilot --test api_experiments
cargo test -p quantpilot --test api_backtest
cargo test -p quantpilot --test api_evidence_contract
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1
git diff --check
```

---

## 下一步

下一批应进入 BE-001AA-04 `runtime.backtest.experiment_sweep.record_lifecycle` 单叶 closeout，确认四个 lifecycle handler 抽离后与原功能等价，并判断 `record_lifecycle` 是否设置 `stop_split: true`。当前不能直接继续细拆 save/discard、迁移 route/schema/state/persistence/response mapping/audit/frontend caller，或启动发布过渡连接。ASCII marker: `next closeout marker`。

---

## 幻觉检查点

AI 声称 `runtime.backtest.experiment_sweep.record_lifecycle` 已抽离时，必须说明只迁移了 `list_experiments`、`get_experiment_detail`、`save_experiment_record`、`discard_experiment_record` 到 `src/runtime/backtest/record_lifecycle.rs`，并通过父级 `pub(crate) use` 保持 `crate::runtime::*` 兼容出口。不得宣称 record lifecycle 已 closeout、`stop_split: true` 已设置、route registration、schema、state、persistence、response mapping、audit、frontend caller、发布过渡、整理或重构已经完成。

---

## 验收标准

1. `123-runtime.backtest.experiment_sweep.record_lifecycle抽离记录.md` 进入 v4.16 里程碑索引。
2. `src/runtime/backtest/record_lifecycle.rs` 进入全量树和模块树。
3. `src/runtime/backtest/experiment_sweep.rs` 保留父级私有模块声明、受控 re-export、parameter_grid sibling 和 start_orchestration sibling。
4. `src/runtime/mod.rs` 与 route registration 行为不变。
5. 治理门禁能发现本抽离记录、实际文件、四个 lifecycle handler、禁止迁移边界和回归证据。
6. `api_experiments`、`api_backtest` 和 `api_evidence_contract` 代表测试继续通过。
