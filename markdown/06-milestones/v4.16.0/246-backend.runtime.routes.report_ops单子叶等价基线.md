# v4.16.0 backend.runtime.routes.report_ops 单子叶等价基线

> 版本类型: MINOR architecture / governance  
> 执行档位: 标准  
> 批次: BE-001BY-01  
> 基准: `245-backend.runtime.routes第五轮父叶残余判断.md`、`244-backend.runtime.routes.event_stream单叶closeout.md`、`13-递归模块化全局根流程.md`。  
> 判定: `backend.runtime.routes.report_ops` 单子叶等价基线已建立。当前 `no code movement`，只冻结 report_ops route group 的 path、method、handler owner、父级注册位置、状态/持久化读取边界和回归证据。下一步只能进入 BE-001BY-02 抽离方案，不得创建 `src/backend/runtime/routes/report_ops.rs` 或迁移 handler。  
> 代码动作: no code movement

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001BY-01 report_ops route facade 单子叶基线 | 新建基线 |
| 规范矩阵 | route facade 等价、父子通信、禁止横向连接、发布过渡保护 | 冻结 |
| 引导矩阵 | `root.backend.runtime.routes.report_ops` | 新增单子叶基线 |
| 模块树 | `backend.runtime.routes.report_ops` | `stop_split: pending` |

---

## 引导坐标声明

| 坐标 | 指向 |
| --- | --- |
| 引导坐标 | `root.backend.runtime.routes.report_ops` |
| 全量树节点 | `markdown/10-overview/overview-full-feature-tree.md` -> `src/backend/runtime/routes.rs`、`src/runtime/mod.rs` |
| 模块树节点 | `markdown/00-matrix-governance/module-tree.md` -> `root.backend.runtime.routes.report_ops` |
| 真实文件 | `src/backend/runtime/routes.rs`、`src/runtime/mod.rs`、`src/runtime_persistence.rs`、`src/frontend_api_types.rs`、`src/storage_lifecycle.rs`、`tests/api_run.rs`、`tests/api_evidence_contract.rs`、`tests/api_backtest.rs`、`tests/api_mutation.rs`、`markdown/05-testing/手动全量实机测试检查单.md` |
| public 方法 | `backend.runtime.routes::register_routes`、`list_runtime_reports`、`create_runtime_report`、`get_runtime_report_detail`、`export_runtime_report_artifact`、`list_merge_records`、`list_config_generations`、`get_storage_health`、`get_ops_daily_report`、`get_audit_weekly_report`、`get_research_monthly_report` |
| 测试/门禁 | `cargo fmt --check`、`cargo check -p quantpilot`、`cargo test --no-run`、`cargo test -p quantpilot --test api_run`、`cargo test -p quantpilot --test api_evidence_contract`、`cargo test -p quantpilot --test api_backtest`、`cargo test -p quantpilot --test api_mutation`、`tools/check-matrix-governance.ps1`、`tools/check-full-feature-tree.ps1`、`tools/check-utf8.ps1`、`git diff --check` |

---

## 当前 route owner 基线

| route | method | handler | 当前 owner |
| --- | --- | --- | --- |
| `/api/runtime/reports` | GET | `runtime_handlers::list_runtime_reports` | `src/backend/runtime/routes.rs` |
| `/api/runtime/reports` | POST | `runtime_handlers::create_runtime_report` | `src/backend/runtime/routes.rs` |
| `/api/runtime/reports/:report_id` | GET | `runtime_handlers::get_runtime_report_detail` | `src/backend/runtime/routes.rs` |
| `/api/runtime/reports/:report_id/export` | GET | `runtime_handlers::export_runtime_report_artifact` | `src/backend/runtime/routes.rs` |
| `/api/v1/merge/records` | GET | `runtime_handlers::list_merge_records` | `src/backend/runtime/routes.rs` |
| `/api/v1/runtime/generations` | GET | `runtime_handlers::list_config_generations` | `src/backend/runtime/routes.rs` |
| `/api/v1/storage/health` | GET | `runtime_handlers::get_storage_health` | `src/backend/runtime/routes.rs` |
| `/api/v1/reports/ops/daily` | GET | `runtime_handlers::get_ops_daily_report` | `src/backend/runtime/routes.rs` |
| `/api/v1/reports/audit/weekly` | GET | `runtime_handlers::get_audit_weekly_report` | `src/backend/runtime/routes.rs` |
| `/api/v1/reports/research/monthly` | GET | `runtime_handlers::get_research_monthly_report` | `src/backend/runtime/routes.rs` |

父级注册片段:

```rust
.route(
    "/api/runtime/reports",
    get(runtime_handlers::list_runtime_reports)
        .post(runtime_handlers::create_runtime_report),
)
.route(
    "/api/runtime/reports/:report_id",
    get(runtime_handlers::get_runtime_report_detail),
)
.route(
    "/api/runtime/reports/:report_id/export",
    get(runtime_handlers::export_runtime_report_artifact),
)
```

```rust
.route(
    "/api/v1/merge/records",
    get(runtime_handlers::list_merge_records),
)
.route(
    "/api/v1/runtime/generations",
    get(runtime_handlers::list_config_generations),
)
.route(
    "/api/v1/storage/health",
    get(runtime_handlers::get_storage_health),
)
.route(
    "/api/v1/reports/ops/daily",
    get(runtime_handlers::get_ops_daily_report),
)
.route(
    "/api/v1/reports/audit/weekly",
    get(runtime_handlers::get_audit_weekly_report),
)
.route(
    "/api/v1/reports/research/monthly",
    get(runtime_handlers::get_research_monthly_report),
)
```

---

## Handler 等价边界

| handler | 输入 | 输出 | 关键依赖 | 禁止事项 |
| --- | --- | --- | --- | --- |
| `create_runtime_report` | `auth::UserId`、`State<AppState>`、`Json<CreateRuntimeReportRequest>` | `Json<RuntimeEvidenceReportRecord>` | `load_run_record_from_state`、`load_backtest_record_from_state`、`runtime_report_record_from_*`、`persist_runtime_report_record` | 不得迁移 report generation、report store 或 evidence metrics owner |
| `list_runtime_reports` | `auth::UserId`、`State<AppState>`、`Query<PaginationQuery>` | `Json<PaginatedResponse<RuntimeEvidenceReportRecord>>` | `list_runtime_report_records`、`materialize_runtime_report_record`、`paginate` | 不得改变 sorting、pagination 或 source changed materialization |
| `get_runtime_report_detail` | `auth::UserId`、`State<AppState>`、`Path(report_id)` | `Json<RuntimeEvidenceReportRecord>` | `load_runtime_report_record`、`materialize_runtime_report_record` | 不得绕过 report store lookup |
| `export_runtime_report_artifact` | `auth::UserId`、`State<AppState>`、`Path(report_id)` | `Json<RuntimeEvidenceReportArtifact>` | `load_runtime_report_record`、`runtime_report_artifact_from_record` | 不得改变 artifact schema |
| `list_merge_records` | `auth::UserId`、`State<AppState>` | `Json<MergeRecordsResponse>` | `state.runs`、`auth::scoped_key` | 不得迁移 run record state owner |
| `list_config_generations` | `State<AppState>` | JSON generation history | `state.config_generation`、`state.config_generation_history` | 不得迁移 generation state owner |
| `get_storage_health` | `State<AppState>` | JSON storage health | `storage_lifecycle::dir_size_bytes`、store dirs | 不得迁移 storage lifecycle owner |
| `get_ops_daily_report` | `auth::UserId`、`State<AppState>`、`Query<OpsDailyQuery>` | `Json<OpsDailyReport>` | runs、alerts、evidence metrics | 不得迁移 ops report schema or state owner |
| `get_audit_weekly_report` | `auth::UserId`、`State<AppState>`、`Query<AuditWeeklyQuery>` | `Json<AuditWeeklyReport>` | approval records、AI proposals、hotswap records | 不得迁移 audit report schema or state owner |
| `get_research_monthly_report` | `auth::UserId`、`State<AppState>`、`Query<ResearchMonthlyQuery>` | `Json<ResearchMonthlyReport>` | backtests、experiments、runtime metrics | 不得迁移 research report schema or state owner |

---

## 保留边界

BE-001BY-01 不迁移、不修改:

- `src/backend/runtime/routes.rs` 中任何 route registration。
- planned `src/backend/runtime/routes/report_ops.rs`。
- runtime report handler bodies。
- report persistence helpers in `src/runtime_persistence.rs`。
- `storage_lifecycle::dir_size_bytes`。
- report / ops / audit / research schema owner。
- `AppState`、state lock order、store dir owner。
- frontend caller。
- runtime persistence owner。
- release transition guard。

---

## 父子通信规则

当前固定:

```text
backend.runtime
  -> backend.runtime.routes
  -> crate::runtime::{report_ops handlers}
```

BE-001BY-01 只登记计划中的 route child 坐标。下一步若进入抽离方案，也只能规划 route registration facade，不得迁移 report handler、schema owner、AppState、frontend caller、runtime persistence owner 或发布过渡连接。

---

## 回归证据

| 证据 | 覆盖 |
| --- | --- |
| `cargo test -p quantpilot --test api_run` | runtime report create/list/detail/export 与 run evidence 侧效应 |
| `cargo test -p quantpilot --test api_evidence_contract` | evidence report retention / export / cleanup contract |
| `cargo test -p quantpilot --test api_backtest` | backtest source report compatibility |
| `cargo test -p quantpilot --test api_mutation` | mutation source evidence and report export compatibility |
| `markdown/05-testing/手动全量实机测试检查单.md` | `/api/v1/merge/records`、`/api/v1/runtime/generations`、`/api/v1/storage/health`、`/api/v1/reports/*` manual smoke coverage |
| `tools/check-matrix-governance.ps1` | 模块树 / 里程碑 / gate token 覆盖 |
| `tools/check-full-feature-tree.ps1` | 全量树路径覆盖 |

---

## 验证计划

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1
git diff --check
```

---

## 下一步

下一步只能进入:

```text
BE-001BY-02 backend.runtime.routes.report_ops 抽离方案
```

该方案只允许规划 `src/backend/runtime/routes/report_ops.rs` route facade 与父级 `report_ops::register_routes(router)` 委托；不得直接创建目标文件、不得迁移 handler、不得改变 `AppState` / schema owner / frontend caller / runtime persistence owner / release transition guard。

---

## 幻觉检查点

AI 声称 BE-001BY-01 完成时，必须说明当前只是 `backend.runtime.routes.report_ops` 等价基线，`src/backend/runtime/routes/report_ops.rs` 尚未创建，route registration 与 handler 仍在原 owner。不得宣称 report_ops route 已迁移、report handlers 已迁移、runtime persistence owner 已迁移、`backend.runtime.routes` 父叶完成、发布过渡已启动、整理或重构已经完成。

---

## 验收标准

1. `246-backend.runtime.routes.report_ops单子叶等价基线.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. 模块树新增 `backend.runtime.routes.report_ops` 白箱节点，并标记 `stop_split: pending`。
3. 基线明确 report_ops route path、method、handler owner 和非目标边界。
4. 下一步固定为 BE-001BY-02 抽离方案。
5. 本批保持 `no code movement`。
