# v4.16.0 backend.runtime.routes.experiment 抽离方案

> 版本类型: MINOR architecture / governance
> 执行档位: 标准
> 批次: BE-001BS-02
> 基准: `231-backend.runtime.routes.experiment单子叶等价基线.md`、`230-backend.runtime.routes第二轮父叶残余判断.md`
> 判定: 建立 `backend.runtime.routes.experiment` 抽离方案。下一批只允许创建 planned route child `src/backend/runtime/routes/experiment.rs`，迁移五个 experiment route registration，并在父级 `src/backend/runtime/routes.rs` 保留原相对注册顺序的委托；当前 `no code movement`。下一步只能进入 BE-001BS-03 实际抽离。
> 代码动作: no code movement

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001BS-02 experiment route facade 抽离方案 | 方案优化 |
| 规范矩阵 | 最小迁移、父子通信、route order 等价、发布过渡保护 | 固化 |
| 引导矩阵 | `root.backend.runtime.routes.experiment` | 抽离方案 |
| 模块树 | `backend.runtime.routes.experiment` | planned route child |

---

## 计划目标

BE-001BS-03 只做 route facade 最小物理迁移:

1. 新建 `src/backend/runtime/routes/experiment.rs`。
2. 在 child 中提供 `pub const MODULE_ID: &str = "backend.runtime.routes.experiment";`。
3. 在 child 中提供 `pub(crate) fn register_routes(router: Router<AppState>) -> Router<AppState>`。
4. 将五个 experiment route registration 从 `src/backend/runtime/routes.rs` 迁入 child。
5. 父级新增 `pub mod experiment;`，并在原 reports 与 ops routes 之间调用 `experiment::register_routes(router)`，保持当前相对 route order。

---

## 迁移清单

| route | method | handler | 目标 |
| --- | --- | --- | --- |
| `/api/runtime/experiments/backtest-sweep` | POST | `runtime_handlers::start_backtest_experiment` | move route registration only |
| `/api/runtime/experiments` | GET | `runtime_handlers::list_experiments` | move route registration only |
| `/api/runtime/experiments/:experiment_id/save` | POST | `runtime_handlers::save_experiment_record` | move route registration only |
| `/api/runtime/experiments/:experiment_id` | GET | `runtime_handlers::get_experiment_detail` | move route registration only |
| `/api/runtime/experiments/:experiment_id` | DELETE | `runtime_handlers::discard_experiment_record` | move route registration only |

目标 child skeleton:

```rust
use axum::{
    routing::{get, post},
    Router,
};

use crate::{runtime as runtime_handlers, AppState};

pub const MODULE_ID: &str = "backend.runtime.routes.experiment";

pub(crate) fn register_routes(router: Router<AppState>) -> Router<AppState> {
    router
        .route(
            "/api/runtime/experiments/backtest-sweep",
            post(runtime_handlers::start_backtest_experiment),
        )
        .route(
            "/api/runtime/experiments",
            get(runtime_handlers::list_experiments),
        )
        .route(
            "/api/runtime/experiments/:experiment_id/save",
            post(runtime_handlers::save_experiment_record),
        )
        .route(
            "/api/runtime/experiments/:experiment_id",
            get(runtime_handlers::get_experiment_detail)
                .delete(runtime_handlers::discard_experiment_record),
        )
}
```

父级接线目标:

```rust
pub mod experiment;

let router = router
    .route(...)
    .route(...); // reports stay before experiment

let router = experiment::register_routes(router);

router
    .route(...) // merge/config/storage/ops stay after experiment
```

---

## 保留边界

BE-001BS-03 不迁移、不修改:

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

## 回归计划

BE-001BS-02 为方案批次，只运行治理门禁:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1
git diff --check
```

BE-001BS-03 实际抽离后必须运行:

```powershell
cargo fmt --check
cargo check -p quantpilot
cargo test --no-run
cargo test -p quantpilot --test api_experiments
cargo test -p quantpilot --test api_backtest
cargo test -p quantpilot --test api_evidence_contract
cargo test -p quantpilot --test api_run
```

---

## 回退点

若 BE-001BS-03 出现编译或回归问题，回退只允许撤销:

- `pub mod experiment;`
- `experiment::register_routes(router)` 委托
- `src/backend/runtime/routes/experiment.rs`
- 父级中 route registration 的迁出

不得回退或改写 handler、schema、state、persistence、frontend caller 或其它 route child。

---

## 下一步

下一步只能进入:

```text
BE-001BS-03 backend.runtime.routes.experiment 实际抽离
```

不得跳过实际抽离进入 closeout，也不得顺手迁移 evidence、report_ops、event_stream 或 handler 域。

---

## 幻觉检查点

AI 声称 BE-001BS-02 完成时，必须说明当前仍是 `no code movement`；`src/backend/runtime/routes/experiment.rs` 尚未创建，experiment route 尚未迁移。不得宣称 route facade 已抽离、handler 已迁移、`backend.runtime.routes` 父叶完成、release transition guard 已启动。

---

## 验收标准

1. `232-backend.runtime.routes.experiment抽离方案.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. 方案只允许下一批迁移五个 experiment route registration。
3. 方案明确父级委托位置必须保持 reports -> experiment -> ops 的相对 route order。
4. 方案明确 BE-001BS-03 的代码验证矩阵。
