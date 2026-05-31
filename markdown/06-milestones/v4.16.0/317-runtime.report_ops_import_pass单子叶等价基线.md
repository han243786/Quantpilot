# v4.16.0 runtime.report_ops_import_pass 单子叶等价基线

> 版本类型: MINOR architecture / governance
> 执行档位: 标准
> 批次: BE-001CZ-01
> 基准: `316-runtime.root_entry_import_pass单叶closeout.md`
> 目标子叶: `runtime.report_ops_import_pass`
> 模块树坐标: `root.backend.runtime.runtime.parent_import_bridge.runtime.report_ops_import_pass`
> 代码动作: no code movement

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001CZ-01 `runtime.report_ops_import_pass` 单子叶等价基线 | 基线冻结 |
| 规范矩阵 | parent import bridge、explicit import pass、transitive parent surface risk、release transition guard | 等价边界 |
| 引导矩阵 | `root.backend.runtime.runtime.parent_import_bridge.runtime.report_ops_import_pass` | report_ops 白箱依赖基线 |
| 模块树 | `runtime.report_ops_import_pass` | 新子叶登记 |

---

## 当前事实

BE-001CY-04 后，`src/runtime/**.rs` 中存在 `use super::*` 或 `super::` 依赖的文件数为 42。`runtime.report_ops_import_pass` 候选范围为:

```text
src/runtime/report_ops.rs
src/runtime/report_ops/runtime_report.rs
src/runtime/report_ops/v1_report_endpoints.rs
src/runtime/report_ops/merge_generation_health.rs
```

四个候选文件当前均存在 `use super::*`。

---

## 真实依赖判定

### `src/runtime/report_ops.rs`

该文件是 report ops parent facade，本身只声明 child module 并 re-export:

```text
merge_generation_health
runtime_report
v1_report_endpoints
```

它的 `use super::*` 主要构成 transitive parent surface risk: 3 个 child 通过 `super::*` 取得 report_ops facade 从 `src/runtime/mod.rs` 转运来的父级白箱输入。若只删除 `src/runtime/report_ops.rs` 的 parent wildcard 而不处理 child，child 的 `super::*` 入口会被抽空。

### `src/runtime/report_ops/runtime_report.rs`

真实依赖集中在 runtime report lifecycle:

- `auth::UserId`、`AppState`、`State`、`Json`、`Query`、`Path`、`StatusCode`
- `CreateRuntimeReportRequest`、`RuntimeEvidenceReportRecord`、`RuntimeEvidenceSourceKind`、`RuntimeReportLifecycleStatus`、`RuntimeReportFailureMetadata`、`RuntimeEvidenceReportArtifact`
- `PaginationQuery`、`PaginatedResponse`、`paginate`
- `current_time_ms`、`io_error`
- `load_run_record_from_state`、`load_backtest_record_from_state`
- `runtime_report_record_from_run_record`、`runtime_report_record_from_backtest_record`、`runtime_report_artifact_from_record`
- `load_runtime_report_record`、`persist_runtime_report_record`、`list_runtime_report_records`

### `src/runtime/report_ops/v1_report_endpoints.rs`

真实依赖集中在 v1 ops / audit / research reports:

- `auth::UserId`、`auth::scoped_key`、`AppState`、`State`、`Json`、`Query`、`StatusCode`
- `OpsDailyQuery`、`OpsDailyReport`、`OpsDailyReportSummary`、`OpsDataHealth`、`OpsRuntimeHealth`、`OpsAlertsSummary`、`OpsStorage`
- `AuditWeeklyQuery`、`AuditWeeklyReport`、`RuntimeApprovalReviewState`
- `ResearchMonthlyQuery`、`ResearchMonthlyReport`、`StrategyPerformanceSummary`
- `AiProposalEffectivenessSummary`、`CapacityTrend`、`CostAnalysisSummary`
- `AlertSeverity`、`current_time_ms`、`epoch_ms_to_iso8601`、`Ordering`

### `src/runtime/report_ops/merge_generation_health.rs`

真实依赖集中在 merge/config/storage health:

- `auth::UserId`、`auth::scoped_key`、`AppState`、`State`、`Json`、`StatusCode`
- `MergeRecordEntry`、`MergeRecordsResponse`
- `serde_json::Value`、`serde_json::json!`
- `std::sync::atomic::Ordering`
- `crate::storage_lifecycle::dir_size_bytes`

---

## 边界判定

本基线只冻结事实，不进行代码移动或 import 改写。BE-001CZ-02 需要明确选择以下方案之一:

1. 单批处理四个文件: 同时改写 parent facade 与 3 个 child，避免 transitive parent surface risk。
2. 先处理 3 个 child，再处理 parent facade: 需要两轮 actual import rewrite，但边界更小。
3. 暂缓 report_ops，回到 `backend.runtime` 父叶残余判断: 仅当 BE-001CZ-02 证明 report_ops batch 风险高于收益时允许。

默认倾向为 1，因为四个文件共同组成同一个 parent import bridge pocket，且 report_ops facade 本身没有业务行为体。

---

## 排除项

- 本批不修改 Rust 代码。
- 本批不删除 `src/runtime/report_ops.rs` 的 `use super::*`。
- 本批不改写 3 个 report_ops child 的 `use super::*`。
- 本批不处理 run/backtest/mutation 子树。
- 本批不处理 `src/runtime/run_guard.rs` 的 test-only super import。
- 本批不处理 `src/runtime/mod.rs` 父桥。
- 本批不新增 sibling horizontal link。
- 本批不启动 release transition。

---

## 验证要求

本批为 `no code movement` 等价基线，提交前至少执行:

```powershell
cargo fmt --check
cargo check -p quantpilot
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1
git diff --check
```

---

## 下一步

下一步只允许进入:

```text
BE-001CZ-02 runtime.report_ops_import_pass 抽离方案
```

BE-001CZ-02 只能设计 report_ops import pass 的最小批次、允许修改文件、排除项、回退点和验证门禁；不得直接改写 Rust import。

---

## 幻觉检查点

AI 声称 BE-001CZ-01 完成时，必须说明:

1. 本批次是 `no code movement` 单子叶等价基线。
2. 候选范围为 `src/runtime/report_ops.rs` 与 3 个 report_ops child。
3. `src/runtime/report_ops.rs` 存在 transitive parent surface risk，不能只删 parent wildcard 后不处理 child。
4. parent import bridge 尚未消除，剩余依赖文件数仍为 42。
5. 下一步只能进入 BE-001CZ-02 抽离方案。

不得宣称 `backend.runtime` 已完成、Rust 重构已完成、report_ops import 已改写、parent import bridge 已完全清除或 release transition 已启动。

---

## 验收标准

1. `317-runtime.report_ops_import_pass单子叶等价基线.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. 基线冻结 `src/runtime/report_ops.rs` 与 3 个 report_ops child 的真实依赖。
3. 下一步固定为 BE-001CZ-02 `runtime.report_ops_import_pass` 抽离方案。
4. 治理门禁、全量树覆盖和 `git diff --check` 均通过。
