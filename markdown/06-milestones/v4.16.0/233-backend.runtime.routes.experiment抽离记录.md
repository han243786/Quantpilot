# v4.16.0 backend.runtime.routes.experiment 抽离记录

> 版本类型: MINOR architecture / governance
> 执行档位: 标准
> 批次: BE-001BS-03
> 基准: `231-backend.runtime.routes.experiment单子叶等价基线.md`、`232-backend.runtime.routes.experiment抽离方案.md`
> 判定: `backend.runtime.routes.experiment` 实际抽离完成。已创建 `src/backend/runtime/routes/experiment.rs`，五个 experiment route registration 已迁入 child；父级 `src/backend/runtime/routes.rs` 通过 `experiment::register_routes(router)` 委托，并保持 reports -> experiment -> ops 的相对 route order。下一步只能进入 BE-001BS-04 单叶 closeout。
> 代码动作: code movement

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001BS-03 experiment route facade 实际抽离 | 实施 |
| 规范矩阵 | route facade、父子通信、route order 等价 | 固化 |
| 引导矩阵 | `root.backend.runtime.routes.experiment` | 实际抽离 |
| 模块树 | `backend.runtime.routes.experiment` | child file created |

---

## 代码变更

新增 route child:

```text
src/backend/runtime/routes/experiment.rs
```

child 暴露:

```rust
pub const MODULE_ID: &str = "backend.runtime.routes.experiment";
pub(crate) fn register_routes(router: Router<AppState>) -> Router<AppState>
```

父级接线:

```rust
pub mod experiment;
let router = experiment::register_routes(router);
```

---

## 迁移结果

| route | method | handler | 结果 |
| --- | --- | --- | --- |
| `/api/runtime/experiments/backtest-sweep` | POST | `runtime_handlers::start_backtest_experiment` | moved to child |
| `/api/runtime/experiments` | GET | `runtime_handlers::list_experiments` | moved to child |
| `/api/runtime/experiments/:experiment_id/save` | POST | `runtime_handlers::save_experiment_record` | moved to child |
| `/api/runtime/experiments/:experiment_id` | GET | `runtime_handlers::get_experiment_detail` | moved to child |
| `/api/runtime/experiments/:experiment_id` | DELETE | `runtime_handlers::discard_experiment_record` | moved to child |

父级 `src/backend/runtime/routes.rs` 仍保留:

- event_stream route。
- evidence health / cleanup routes。
- mutation route child delegate。
- report routes before experiment delegate。
- merge / config / storage / ops report routes after experiment delegate。

---

## 等价边界

本批未迁移、未修改:

- `start_backtest_experiment`
- `list_experiments`
- `get_experiment_detail`
- `save_experiment_record`
- `discard_experiment_record`
- `src/runtime/backtest/experiment_sweep.rs`
- `src/runtime/backtest/start_orchestration.rs`
- `src/runtime/backtest/record_lifecycle.rs`
- `src/runtime/mod.rs`
- `AppState`
- schema owner
- frontend caller
- runtime persistence owner
- artifact schema owner
- compare owner
- evidence / report_ops / event_stream routes
- release transition guard

---

## 父子通信规则

当前固定:

```text
backend.runtime
  -> backend.runtime.routes
  -> backend.runtime.routes.experiment
  -> crate::runtime::{start_backtest_experiment, list_experiments, get_experiment_detail, save_experiment_record, discard_experiment_record}
```

`backend.runtime.routes.experiment` 只拥有 route registration；handler owner 仍在 `runtime.backtest.experiment_sweep` 子树，状态与持久化 owner 仍在 `AppState` / runtime persistence。

---

## 回归计划

BE-001BS-03 必须运行:

```powershell
cargo fmt --check
cargo check -p quantpilot
cargo test --no-run
cargo test -p quantpilot --test api_experiments
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

下一步只能进入:

```text
BE-001BS-04 backend.runtime.routes.experiment 单叶 closeout
```

不得跳到 evidence/report_ops/event_stream，也不得继续细拆 experiment route child。

---

## 幻觉检查点

AI 声称 BE-001BS-03 完成时，必须说明只完成 experiment route facade 抽离；handler、`AppState`、schema owner、frontend caller、runtime persistence owner 和 release transition guard 均未改变。不得宣称 `backend.runtime.routes` 父叶完成、experiment handler 已迁移、整理或重构已经完成。

---

## 验收标准

1. `src/backend/runtime/routes/experiment.rs` 创建并进入全量树。
2. `src/backend/runtime/routes.rs` 只保留 `pub mod experiment` 与 `experiment::register_routes(router)` 委托。
3. 五个 experiment route registration 均迁入 child。
4. 验证矩阵通过。
