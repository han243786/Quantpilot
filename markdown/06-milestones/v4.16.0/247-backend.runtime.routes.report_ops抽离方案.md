# v4.16.0 backend.runtime.routes.report_ops 抽离方案

> 版本类型: MINOR architecture / governance  
> 执行档位: 标准  
> 批次: BE-001BY-02  
> 基准: `246-backend.runtime.routes.report_ops单子叶等价基线.md`、`245-backend.runtime.routes第五轮父叶残余判断.md`、`13-递归模块化全局根流程.md`  
> 判定: `backend.runtime.routes.report_ops` 抽离方案已建立。当前 `no code movement`，只规划 route facade 迁移；下一步 BE-001BY-03 才允许创建 `src/backend/runtime/routes/report_ops.rs` 并迁移 runtime reports、merge records、runtime generations、storage health、ops/audit/research reports 的 route registration。handler、schema owner、`AppState`、frontend caller、runtime persistence owner、storage lifecycle owner 和 release transition guard 均不得迁移。  
> 代码动作: no code movement

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001BY-02 report_ops route facade 抽离方案 | 新建方案 |
| 规范矩阵 | route facade 等价、父子通信、禁止横向连接、发布过渡保护 | 冻结 |
| 引导矩阵 | `root.backend.runtime.routes.report_ops` | 更新下一步 |
| 模块树 | `backend.runtime.routes.report_ops` | `stop_split: pending` |

---

## 引导坐标声明

| 坐标 | 指向 |
| --- | --- |
| 引导坐标 | `root.backend.runtime.routes.report_ops` |
| 全量树节点 | `markdown/10-overview/overview-full-feature-tree.md` -> `src/backend/runtime/routes.rs`、`src/runtime/mod.rs` |
| 模块树节点 | `markdown/00-matrix-governance/module-tree.md` -> `root.backend.runtime.routes.report_ops` |
| 当前真实文件 | `src/backend/runtime/routes.rs`、`src/runtime/mod.rs`、`src/runtime_persistence.rs`、`src/frontend_api_types.rs`、`src/storage_lifecycle.rs`、`tests/api_run.rs`、`tests/api_evidence_contract.rs`、`tests/api_backtest.rs`、`tests/api_mutation.rs`、`markdown/05-testing/手动全量实机测试检查单.md` |
| 计划目标文件 | `src/backend/runtime/routes/report_ops.rs` |
| public 方法 | `backend.runtime.routes::register_routes`、`report_ops::register_runtime_report_routes(router)`、`report_ops::register_ops_routes(router)`、`list_runtime_reports`、`create_runtime_report`、`get_runtime_report_detail`、`export_runtime_report_artifact`、`list_merge_records`、`list_config_generations`、`get_storage_health`、`get_ops_daily_report`、`get_audit_weekly_report`、`get_research_monthly_report` |
| 测试/门禁 | `cargo fmt --check`、`cargo check -p quantpilot`、`cargo test --no-run`、`cargo test -p quantpilot --test api_run`、`cargo test -p quantpilot --test api_evidence_contract`、`cargo test -p quantpilot --test api_backtest`、`cargo test -p quantpilot --test api_mutation`、manual smoke coverage、`tools\check-matrix-governance.ps1`、`tools\check-full-feature-tree.ps1`、`tools\check-utf8.ps1`、`git diff --check` |

---

## 目标形态

BE-001BY-03 的唯一目标是把 report_ops route registration 从父 aggregate 移入 route child facade:

```text
src/backend/runtime/routes.rs
  pub mod report_ops;
  register_routes:
    backtest -> run -> event_stream -> evidence -> mutation
    -> report_ops::register_runtime_report_routes
    -> experiment
    -> report_ops::register_ops_routes

src/backend/runtime/routes/report_ops.rs
  pub const MODULE_ID: &str = "backend.runtime.routes.report_ops";
  pub(crate) fn register_runtime_report_routes(router: Router<AppState>) -> Router<AppState>
  pub(crate) fn register_ops_routes(router: Router<AppState>) -> Router<AppState>
```

采用两个父级委托入口是为了保持现有 route registration 顺序: runtime report routes 当前位于 `mutation` 之后、`experiment` 之前；v1 ops routes 当前位于 `experiment` 之后。该形态仍属于同一个 `backend.runtime.routes.report_ops` 白箱节点，不形成横向连接。

必须保持 route order:

```text
backtest -> run -> event_stream -> evidence -> mutation -> report_ops(runtime reports) -> experiment -> report_ops(v1 ops)
```

---

## 允许迁移清单

BE-001BY-03 只能执行以下代码动作:

1. 创建 `src/backend/runtime/routes/report_ops.rs`。
2. 在 `src/backend/runtime/routes.rs` 中声明 `pub mod report_ops`。
3. 把 `/api/runtime/reports` GET/POST、`/api/runtime/reports/:report_id` GET、`/api/runtime/reports/:report_id/export` GET route registration 移入 `report_ops::register_runtime_report_routes(router)`。
4. 把 `/api/v1/merge/records`、`/api/v1/runtime/generations`、`/api/v1/storage/health`、`/api/v1/reports/ops/daily`、`/api/v1/reports/audit/weekly`、`/api/v1/reports/research/monthly` route registration 移入 `report_ops::register_ops_routes(router)`。
5. 在父级中以 `report_ops::register_runtime_report_routes(router)` 和 `report_ops::register_ops_routes(router)` 委托，并保持 `mutation -> report_ops(runtime reports) -> experiment -> report_ops(v1 ops)` 的相对顺序。
6. 若父级 `src/backend/runtime/routes.rs` 不再直接调用 `get`，允许把父级 import 收敛为 `use axum::Router;`；不得做其他无关 import 风格调整。

允许迁移的 route registration 仅限:

| route | method | handler |
| --- | --- | --- |
| `/api/runtime/reports` | GET | `runtime_handlers::list_runtime_reports` |
| `/api/runtime/reports` | POST | `runtime_handlers::create_runtime_report` |
| `/api/runtime/reports/:report_id` | GET | `runtime_handlers::get_runtime_report_detail` |
| `/api/runtime/reports/:report_id/export` | GET | `runtime_handlers::export_runtime_report_artifact` |
| `/api/v1/merge/records` | GET | `runtime_handlers::list_merge_records` |
| `/api/v1/runtime/generations` | GET | `runtime_handlers::list_config_generations` |
| `/api/v1/storage/health` | GET | `runtime_handlers::get_storage_health` |
| `/api/v1/reports/ops/daily` | GET | `runtime_handlers::get_ops_daily_report` |
| `/api/v1/reports/audit/weekly` | GET | `runtime_handlers::get_audit_weekly_report` |
| `/api/v1/reports/research/monthly` | GET | `runtime_handlers::get_research_monthly_report` |

---

## 非目标边界

本方案和下一步实际抽离均不得迁移或修改:

- runtime report handler body。
- `list_runtime_reports`、`create_runtime_report`、`get_runtime_report_detail`、`export_runtime_report_artifact`。
- `list_merge_records`、`list_config_generations`、`get_storage_health`。
- `get_ops_daily_report`、`get_audit_weekly_report`、`get_research_monthly_report`。
- `load_run_record_from_state`、`load_backtest_record_from_state`、report materialization、pagination、artifact schema。
- `persist_runtime_report_record`、`list_runtime_report_records`、`load_runtime_report_record`。
- `storage_lifecycle::dir_size_bytes`。
- `auth::UserId`、`State<AppState>`、`Path(report_id)`、`Query<OpsDailyQuery>`、`Query<AuditWeeklyQuery>`、`Query<ResearchMonthlyQuery>` extractor 组合。
- `AppState`、state owner、store dir owner 或锁顺序。
- schema owner。
- frontend caller。
- runtime persistence owner。
- release transition guard。
- run/backtest/event_stream/evidence/mutation/experiment route child。

---

## 父子通信规则

固定为:

```text
backend.runtime
  -> backend.runtime.routes
  -> backend.runtime.routes.report_ops
  -> crate::runtime::{report_ops handlers}
```

`backend.runtime.routes.report_ops` 只能作为 route facade。不得横向接管 run/backtest/event_stream/evidence/mutation/experiment route child、runtime report handler implementation、frontend caller、runtime persistence owner、storage lifecycle owner 或 executor。开发者未明确进入发布版本过渡前，AI 不得主动提出横向连接或性能旁路。

---

## 回退点

若 BE-001BY-03 验证失败，回退只允许:

1. 删除 `src/backend/runtime/routes/report_ops.rs`。
2. 移除父级 `pub mod report_ops`。
3. 把 runtime reports route registration 放回 `src/backend/runtime/routes.rs` 的 `mutation` 与 `experiment` 之间。
4. 把 v1 ops route registration 放回 `src/backend/runtime/routes.rs` 的 `experiment` 之后。
5. 若父级 import 曾收敛为 `use axum::Router;`，恢复为 `use axum::{routing::get, Router};`。

不得用回退作为理由迁移 handler、schema、`AppState`、frontend caller、runtime persistence owner、storage lifecycle owner 或 release transition guard。

---

## 验证计划

```powershell
cargo fmt --check
cargo check -p quantpilot
cargo test --no-run
cargo test -p quantpilot --test api_run
cargo test -p quantpilot --test api_evidence_contract
cargo test -p quantpilot --test api_backtest
cargo test -p quantpilot --test api_mutation
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1
git diff --check
```

---

## 下一步

下一步只能进入:

```text
BE-001BY-03 backend.runtime.routes.report_ops 实际抽离
```

BE-001BY-03 完成后必须进入单叶 closeout，判断 `backend.runtime.routes.report_ops` 是否还值得继续细拆。不得跳过 closeout 直接处理 handler、schema、state owner 或发布过渡。

---

## 幻觉检查点

AI 声称 BE-001BY-02 完成时，必须说明当前仍是 `no code movement` 的抽离方案，`src/backend/runtime/routes/report_ops.rs` 尚未创建，handler、schema owner、`AppState`、frontend caller、runtime persistence owner、storage lifecycle owner 和 release transition guard 均未改变。不得宣称 report_ops route 已迁移、`backend.runtime.routes` 父叶完成、发布过渡已启动、整理或重构已经完成。

---

## 验收标准

1. `247-backend.runtime.routes.report_ops抽离方案.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. 模块树 `backend.runtime.routes.report_ops` 节点更新为 BE-001BY-02 抽离方案已建立。
3. 方案明确 BE-001BY-03 的目标文件、两个父级委托入口、route order、允许迁移清单和非目标边界。
4. 下一步固定为 BE-001BY-03 实际抽离。
5. 本批保持 `no code movement`。
