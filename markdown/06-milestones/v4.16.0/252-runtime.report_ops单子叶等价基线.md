# v4.16.0 runtime.report_ops 单子叶等价基线

> 版本类型: MINOR architecture / governance  
> 执行档位: 标准  
> 批次: BE-001CB-01  
> 基准: `251-backend.runtime父叶残余判断.md`、`250-backend.runtime.routes第六轮父叶残余判断.md`、`248-backend.runtime.routes.report_ops抽离记录.md`  
> 判定: `runtime.report_ops` 值得作为 `backend.runtime` 父叶下一个 handler 子叶建立等价基线。当前只冻结边界，目标文件尚未创建，handler 尚未迁移。  
> 代码动作: no code movement

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001CB-01 `runtime.report_ops` 单子叶等价基线 | 建基线 |
| 规范矩阵 | handler owner、route facade、state/persistence 只读边界、测试证据 | 冻结 |
| 引导矩阵 | `root.backend.runtime.runtime.report_ops` | 新子叶坐标 |
| 模块树 | `runtime.report_ops` | planned child |

---

## 边界冻结

本子叶只覆盖 `src/runtime/mod.rs` 中由 `backend.runtime.routes.report_ops` 调用的 handler 与直接 helper。

| 类别 | 当前入口 | 当前 owner | 本基线处理 |
| --- | --- | --- | --- |
| runtime report create | `create_runtime_report` | `src/runtime/mod.rs` | 冻结 |
| runtime report list | `list_runtime_reports` | `src/runtime/mod.rs` | 冻结 |
| runtime report detail | `get_runtime_report_detail` | `src/runtime/mod.rs` | 冻结 |
| runtime report export | `export_runtime_report_artifact` | `src/runtime/mod.rs` | 冻结 |
| v1 merge records | `list_merge_records` | `src/runtime/mod.rs` + `src/runtime/run.rs` response type | 冻结 |
| v1 config generations | `list_config_generations` | `src/runtime/mod.rs` | 冻结 |
| v1 storage health | `get_storage_health` | `src/runtime/mod.rs` + `src/storage_lifecycle.rs` read helper | 冻结 |
| v1 ops daily report | `get_ops_daily_report` | `src/runtime/mod.rs` + `src/runtime/mutation.rs` query type | 冻结 |
| v1 audit weekly report | `get_audit_weekly_report` | `src/runtime/mod.rs` + `src/runtime/mutation.rs` query type | 冻结 |
| v1 research monthly report | `get_research_monthly_report` | `src/runtime/mod.rs` + `src/runtime/mutation.rs` query type | 冻结 |

---

## 允许迁移清单

后续 BE-001CB-03 实际抽离若被允许，第一轮只可迁移:

- `create_runtime_report`
- `report_source_metadata_matches`
- `source_changed_report`
- `current_report_for_saved_source`
- `materialize_runtime_report_record`
- `list_runtime_reports`
- `get_runtime_report_detail`
- `export_runtime_report_artifact`
- `list_merge_records`
- `list_config_generations`
- `get_storage_health`
- `get_ops_daily_report`
- `get_audit_weekly_report`
- `get_research_monthly_report`

计划目标文件只能是:

```text
src/runtime/report_ops.rs
```

计划父级暴露方式只能是 `src/runtime/mod.rs` 中的受控 `mod report_ops` 与 `pub(crate) use report_ops::{...}` re-export。不得改变 `backend.runtime.routes.report_ops` 的 route path、method、委托顺序或 handler 调用名称。

---

## 明确排除

本基线不覆盖:

- `get_runtime_evidence_health`
- `cleanup_runtime_evidence`
- `runtime_report_status_counts`
- `RuntimeReplayQuery`
- `RuntimeParameterMutationListQuery`
- `RuntimeAiProposalListQuery`
- `clean_optional_filter`
- `normalized_replay_options`
- `RunInProgressGuard`
- `AppState` 字段、锁顺序或 state owner
- `runtime_persistence` owner
- `runtime_response_mapping` owner
- `frontend_api_types` schema owner
- frontend caller
- storage lifecycle owner
- release transition guard

`runtime.evidence_health` 后续应作为 sibling 另起父叶判断或单子叶基线，不能混入 `runtime.report_ops` 第一轮抽离。

---

## 输入输出契约

| 入口 | 输入 | 输出 | 等价要求 |
| --- | --- | --- | --- |
| `POST /api/runtime/reports` | `CreateRuntimeReportRequest` | `RuntimeEvidenceReportRecord` | report id、source metadata、generation policy、idempotent existing record 行为不变 |
| `GET /api/runtime/reports` | `PaginationQuery` | `PaginatedResponse<RuntimeEvidenceReportRecord>` | materialize、排序、分页语义不变 |
| `GET /api/runtime/reports/:report_id` | report id | `RuntimeEvidenceReportRecord` | source changed 检查与 404 行为不变 |
| `GET /api/runtime/reports/:report_id/export` | report id | `RuntimeEvidenceReportArtifact` | artifact projection 不变 |
| `GET /api/v1/merge/records` | user scope | `MergeRecordsResponse` | merge_engine event scan 与 totals 不变 |
| `GET /api/v1/runtime/generations` | AppState generation | JSON | generation history projection 不变 |
| `GET /api/v1/storage/health` | storage dirs | JSON | dir_size_bytes 调用与 layer names 不变 |
| `GET /api/v1/reports/ops/daily` | `OpsDailyQuery` | `OpsDailyReport` | metrics / alert / storage projection 不变 |
| `GET /api/v1/reports/audit/weekly` | `AuditWeeklyQuery` | `AuditWeeklyReport` | approval / proposal / mutation counts 不变 |
| `GET /api/v1/reports/research/monthly` | `ResearchMonthlyQuery` | `ResearchMonthlyReport` | strategy performance / capacity / cost projection 不变 |

---

## 真实文件

- `src/runtime/mod.rs`
- `src/backend/runtime/routes/report_ops.rs`
- `src/runtime_persistence.rs`
- `src/runtime_response_mapping.rs`
- `src/runtime/run.rs`
- `src/runtime/mutation.rs`
- `src/frontend_api_types.rs`
- `frontend/src/store/graphStoreRuntimeHistoryApi.js`
- `frontend/src/components/RuntimeReportPanel.jsx`
- `frontend/src/components/RuntimeReportPanel.test.jsx`
- `frontend/src/components/RuntimeDiagnosticsPanel.jsx`
- `frontend/src/pages/BacktestDetailPage.jsx`
- `tests/api_run.rs`
- `tests/api_backtest.rs`
- `tests/api_mutation.rs`
- `tests/api_evidence_contract.rs`

---

## 测试证据与缺口

已存在覆盖:

- `tests/api_run.rs`: runtime report create/list/detail/export。
- `tests/api_backtest.rs`: backtest report create。
- `tests/api_mutation.rs`: mutation source report create/export。
- `tests/api_evidence_contract.rs`: report create/export/detail 与 evidence health/cleanup 契约。
- `frontend/src/components/RuntimeReportPanel.test.jsx`: frontend report list/create/detail/export UI flow。

当前缺口:

- `/api/v1/merge/records`
- `/api/v1/runtime/generations`
- `/api/v1/storage/health`
- `/api/v1/reports/ops/daily`
- `/api/v1/reports/audit/weekly`
- `/api/v1/reports/research/monthly`

后续抽离方案必须明确这些 v1 ops/report endpoints 是沿用 `cargo test --no-run` 与 `cargo check` 做编译等价，还是补充专门 API 测试；不得在实际迁移时静默扩大测试缺口。

---

## 下一步

下一步只能进入:

```text
BE-001CB-02 runtime.report_ops 抽离方案
```

BE-001CB-02 只能规划最小物理迁移，不得创建 `src/runtime/report_ops.rs`，不得迁移 handler，不得改变 route order、`AppState`、schema owner、frontend caller、runtime persistence owner、storage lifecycle owner 或 release transition guard。

---

## 幻觉检查点

AI 声称 BE-001CB-01 完成时，必须说明:

1. 当前只是等价基线，`src/runtime/report_ops.rs` 尚未创建。
2. `create_runtime_report` 等 handler 仍在 `src/runtime/mod.rs`。
3. `runtime.evidence_health` 未被并入本子叶。
4. v1 ops/report endpoints 存在测试缺口，后续抽离方案必须显式处理。
5. `AppState`、schema owner、frontend caller、runtime persistence owner、storage lifecycle owner 和 release transition guard 均未迁移。

---

## 后续验证门禁

本基线批次保持 `no code movement`，提交前运行治理门禁即可。BE-001CB-02 抽离方案和 BE-001CB-03 实际抽离必须显式继承以下验证命令:

- `cargo fmt --check`
- `cargo check -p quantpilot`
- `cargo test --no-run`
- `cargo test -p quantpilot --test api_run`
- `cargo test -p quantpilot --test api_evidence_contract`
- `cargo test -p quantpilot --test api_backtest`
- `cargo test -p quantpilot --test api_mutation`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1`
- `git diff --check`

---

## 验收标准

1. `252-runtime.report_ops单子叶等价基线.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. `runtime.report_ops` 边界冻结，下一步固定为 BE-001CB-02 抽离方案。
3. 明确 `runtime.evidence_health` 不并入本批。
4. 本批保持 `no code movement`。
