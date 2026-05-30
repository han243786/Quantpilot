# v4.16.0 backend.runtime.routes.report_ops 抽离记录

> 版本类型: MINOR architecture / governance  
> 执行档位: 标准  
> 批次: BE-001BY-03  
> 基准: `247-backend.runtime.routes.report_ops抽离方案.md`、`246-backend.runtime.routes.report_ops单子叶等价基线.md`、`13-递归模块化全局根流程.md`  
> 判定: `backend.runtime.routes.report_ops` route facade 实际抽离已完成。`src/backend/runtime/routes/report_ops.rs` 已创建并承接 runtime reports、merge records、runtime generations、storage health、ops/audit/research reports 的 route registration；父级通过 `report_ops::register_runtime_report_routes(router)` 和 `report_ops::register_ops_routes(router)` 委托，并保持 `mutation -> report_ops(runtime reports) -> experiment -> report_ops(v1 ops)` 顺序。handler、schema owner、`AppState`、frontend caller、runtime persistence owner、storage lifecycle owner 和 release transition guard 均未迁移。  
> 代码动作: route facade extraction

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001BY-03 report_ops route facade 实际抽离 | 实际抽离 |
| 规范矩阵 | route facade 等价、父子通信、禁止横向连接、发布过渡保护 | 执行 |
| 引导矩阵 | `root.backend.runtime.routes.report_ops` | 更新实际文件 |
| 模块树 | `backend.runtime.routes.report_ops` | `stop_split: pending` |

---

## 代码变更

| 文件 | 动作 | 说明 |
| --- | --- | --- |
| `src/backend/runtime/routes/report_ops.rs` | 新增 | 新增 route child facade，提供 `MODULE_ID`、`register_runtime_report_routes` 与 `register_ops_routes` |
| `src/backend/runtime/routes.rs` | 更新 | 声明 `pub mod report_ops`，以两段委托保持 experiment 前后 route order，并移除父级直接 `get` import |
| `src/runtime/mod.rs` | 保持不变 | report / ops handlers 仍由 runtime facade owner 暴露 |

---

## 抽离后结构

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

实际迁移的 route registration:

| route | method | handler | 当前 owner |
| --- | --- | --- | --- |
| `/api/runtime/reports` | GET | `runtime_handlers::list_runtime_reports` | `src/backend/runtime/routes/report_ops.rs` |
| `/api/runtime/reports` | POST | `runtime_handlers::create_runtime_report` | `src/backend/runtime/routes/report_ops.rs` |
| `/api/runtime/reports/:report_id` | GET | `runtime_handlers::get_runtime_report_detail` | `src/backend/runtime/routes/report_ops.rs` |
| `/api/runtime/reports/:report_id/export` | GET | `runtime_handlers::export_runtime_report_artifact` | `src/backend/runtime/routes/report_ops.rs` |
| `/api/v1/merge/records` | GET | `runtime_handlers::list_merge_records` | `src/backend/runtime/routes/report_ops.rs` |
| `/api/v1/runtime/generations` | GET | `runtime_handlers::list_config_generations` | `src/backend/runtime/routes/report_ops.rs` |
| `/api/v1/storage/health` | GET | `runtime_handlers::get_storage_health` | `src/backend/runtime/routes/report_ops.rs` |
| `/api/v1/reports/ops/daily` | GET | `runtime_handlers::get_ops_daily_report` | `src/backend/runtime/routes/report_ops.rs` |
| `/api/v1/reports/audit/weekly` | GET | `runtime_handlers::get_audit_weekly_report` | `src/backend/runtime/routes/report_ops.rs` |
| `/api/v1/reports/research/monthly` | GET | `runtime_handlers::get_research_monthly_report` | `src/backend/runtime/routes/report_ops.rs` |

---

## 等价边界

本批只迁移 route registration，不迁移:

- runtime report handler body。
- `list_runtime_reports`、`create_runtime_report`、`get_runtime_report_detail`、`export_runtime_report_artifact`。
- `list_merge_records`、`list_config_generations`、`get_storage_health`。
- `get_ops_daily_report`、`get_audit_weekly_report`、`get_research_monthly_report`。
- report materialization、pagination、artifact schema 或 evidence metrics。
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

抽离后固定为:

```text
backend.runtime
  -> backend.runtime.routes
  -> backend.runtime.routes.report_ops
  -> crate::runtime::{report_ops handlers}
```

`backend.runtime.routes.report_ops` 只作为 route facade。不得横向接管 run/backtest/event_stream/evidence/mutation/experiment route child、runtime report handler implementation、frontend caller、runtime persistence owner、storage lifecycle owner 或 executor。开发者未明确进入发布版本过渡前，AI 不得主动提出横向连接或性能旁路。

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
BE-001BY-04 backend.runtime.routes.report_ops 单叶 closeout
```

BE-001BY-04 必须判断 `backend.runtime.routes.report_ops` 是否值得继续细拆。不得跳过 closeout 直接处理 handler、schema、state owner 或发布过渡。

---

## 幻觉检查点

AI 声称 BE-001BY-03 完成时，必须说明只完成 route facade 抽离；report handlers、schema owner、`AppState`、frontend caller、runtime persistence owner、storage lifecycle owner 和 release transition guard 均未改变。不得宣称 `backend.runtime.routes` 父叶完成、发布过渡已启动、整理或重构已经完成。

---

## 验收标准

1. `src/backend/runtime/routes/report_ops.rs` 已创建并承接 report_ops route registration。
2. `src/backend/runtime/routes.rs` 通过 `report_ops::register_runtime_report_routes(router)` 与 `report_ops::register_ops_routes(router)` 委托，并保持 `mutation -> report_ops(runtime reports) -> experiment -> report_ops(v1 ops)` 顺序。
3. `248-backend.runtime.routes.report_ops抽离记录.md` 进入里程碑索引、模块树、全量树和治理门禁。
4. 下一步固定为 BE-001BY-04 单叶 closeout。
5. 本批不迁移 handler、schema、`AppState`、frontend caller、runtime persistence owner、storage lifecycle owner 或 release transition guard。
