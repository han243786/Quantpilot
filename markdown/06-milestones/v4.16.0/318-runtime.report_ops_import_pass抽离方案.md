# v4.16.0 runtime.report_ops_import_pass 抽离方案

> 版本类型: MINOR architecture / governance
> 执行档位: 标准
> 批次: BE-001CZ-02
> 基准: `317-runtime.report_ops_import_pass单子叶等价基线.md`
> 目标子叶: `runtime.report_ops_import_pass`
> 模块树坐标: `root.backend.runtime.runtime.parent_import_bridge.runtime.report_ops_import_pass`
> 代码动作: no code movement

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001CZ-02 `runtime.report_ops_import_pass` 抽离方案 | 方案优化 |
| 规范矩阵 | parent import bridge、explicit import pass、four-file pocket、transitive parent surface risk、release transition guard | 执行边界 |
| 引导矩阵 | `root.backend.runtime.runtime.parent_import_bridge.runtime.report_ops_import_pass` | report_ops import plan |
| 模块树 | `runtime.report_ops_import_pass` | 方案登记 |

---

## 方案判定

采用 four-file pocket 同批处理，不拆成 parent-first 或 child-first 两轮。

BE-001CZ-03 只允许处理:

```text
src/runtime/report_ops.rs
src/runtime/report_ops/runtime_report.rs
src/runtime/report_ops/v1_report_endpoints.rs
src/runtime/report_ops/merge_generation_health.rs
```

选择同批的原因:

1. `src/runtime/report_ops.rs` 本身是 parent facade，没有业务行为体；其 `use super::*` 主要把 `src/runtime/mod.rs` 的父级白箱输入转运给 3 个 child。
2. 若只删 parent facade 的 `use super::*`，3 个 child 的 `super::*` 会失去 transitive parent surface。
3. 若只改 child 而保留 parent wildcard，report_ops pocket 仍保持残余 bridge 噪声。
4. 四个文件共同组成同一个 import pocket，边界小于 run/backtest/mutation 子树。

---

## 允许改写清单

### `src/runtime/report_ops.rs`

允许删除:

```rust
use super::*;
```

不需要补充新的 import；只保留 module declaration 与 `pub(crate) use` facade。

### `src/runtime/report_ops/runtime_report.rs`

允许将 `use super::*` 改为显式 import，目标依赖包括:

```text
auth
AppState
State
Json
Query
Path
StatusCode
CreateRuntimeReportRequest
RuntimeEvidenceReportRecord
RuntimeEvidenceSourceKind
RuntimeReportLifecycleStatus
RuntimeReportFailureMetadata
RuntimeEvidenceReportArtifact
PaginationQuery
PaginatedResponse
paginate
current_time_ms
io_error
load_run_record_from_state
load_backtest_record_from_state
runtime_report_record_from_run_record
runtime_report_record_from_backtest_record
runtime_report_artifact_from_record
load_runtime_report_record
persist_runtime_report_record
list_runtime_report_records
```

### `src/runtime/report_ops/v1_report_endpoints.rs`

允许将 `use super::*` 改为显式 import，目标依赖包括:

```text
auth
AppState
State
Json
Query
StatusCode
OpsDailyQuery
OpsDailyReport
OpsDailyReportSummary
OpsDataHealth
OpsRuntimeHealth
OpsAlertsSummary
OpsStorage
AuditWeeklyQuery
AuditWeeklyReport
RuntimeApprovalReviewState
ResearchMonthlyQuery
ResearchMonthlyReport
StrategyPerformanceSummary
AiProposalEffectivenessSummary
CapacityTrend
CostAnalysisSummary
AlertSeverity
current_time_ms
epoch_ms_to_iso8601
Ordering
```

### `src/runtime/report_ops/merge_generation_health.rs`

允许将 `use super::*` 改为显式 import，目标依赖包括:

```text
auth
AppState
State
Json
StatusCode
MergeRecordEntry
MergeRecordsResponse
serde_json::Value
serde_json::json!
std::sync::atomic::Ordering
crate::storage_lifecycle::dir_size_bytes
```

---

## 明确排除

BE-001CZ-03 不允许处理:

```text
src/runtime/mod.rs
src/runtime/run_guard.rs
src/runtime/run/**
src/runtime/backtest/**
src/runtime/mutation/**
```

不允许迁移 runtime report schema、runtime persistence helpers、storage lifecycle owner、metrics owner、`AppState` owner、frontend caller 或 route facade。

不允许新增 sibling horizontal link，不允许启动 release transition。

---

## 回退点

若 BE-001CZ-03 失败，回退范围仅限:

1. 四个 report_ops pocket 文件的显式 import 改写。
2. 与 BE-001CZ-03 同批新增的治理文档和门禁锚点。

不得回退 `runtime.root_entry_import_pass`、`runtime.root_support_import_pilot`、`runtime.parent_include_cleanup` 或任何已 closeout 子模块。

---

## 验证要求

本批为 `no code movement` 抽离方案，提交前至少执行:

```powershell
cargo fmt --check
cargo check -p quantpilot
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1
git diff --check
```

BE-001CZ-03 实际 import rewrite 后至少补跑:

```powershell
cargo test -p quantpilot --test api_run
cargo test -p quantpilot --test api_backtest
cargo test -p quantpilot --test api_evidence_contract
cargo test -p quantpilot --test api_v1_reports
cargo test -p quantpilot --test api_v1_ops_health
```

---

## 下一步

下一步只允许进入:

```text
BE-001CZ-03 runtime.report_ops_import_pass 实际抽离
```

BE-001CZ-03 只允许改写 report_ops four-file pocket 的 parent wildcard import，不得顺手处理 `src/runtime/mod.rs`、test-only `src/runtime/run_guard.rs`、run/backtest/mutation 子树或 release transition。

---

## 幻觉检查点

AI 声称 BE-001CZ-02 完成时，必须说明:

1. 本批次是 `no code movement` 抽离方案。
2. BE-001CZ-03 只允许处理 report_ops four-file pocket。
3. 选择同批处理是为了消除 transitive parent surface risk。
4. parent import bridge 尚未消除，剩余依赖文件数仍为 42。
5. 下一步只能进入 BE-001CZ-03 实际抽离。

不得宣称 `backend.runtime` 已完成、Rust 重构已完成、report_ops import 已改写、parent import bridge 已完全清除或 release transition 已启动。

---

## 验收标准

1. `318-runtime.report_ops_import_pass抽离方案.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. 方案固定 BE-001CZ-03 只处理 report_ops four-file pocket。
3. 明确保留 `src/runtime/mod.rs`、test-only `src/runtime/run_guard.rs` 与 run/backtest/mutation 子树。
4. 治理门禁、全量树覆盖和 `git diff --check` 均通过。
