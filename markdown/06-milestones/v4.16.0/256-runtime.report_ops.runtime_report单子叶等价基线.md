# v4.16.0 runtime.report_ops.runtime_report 单子叶等价基线

> 版本类型: MINOR architecture / governance  
> 执行档位: 标准  
> 批次: BE-001CC-01  
> 基准: `255-runtime.report_ops单叶closeout.md`、`254-runtime.report_ops抽离记录.md`、`13-递归模块化全局根流程.md`  
> 判定: `runtime.report_ops.runtime_report` 值得作为 `runtime.report_ops` 下的下一子叶建立等价基线。当前 `no code movement`，目标文件 `src/runtime/report_ops/runtime_report.rs` 尚未创建，runtime report handler 与 helper 仍在 `src/runtime/report_ops.rs`。下一步只能进入 BE-001CC-02 抽离方案。  
> 代码动作: no code movement

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001CC-01 `runtime.report_ops.runtime_report` 单子叶等价基线 | 建基线 |
| 规范矩阵 | runtime report handler 等价、父级 re-export、禁止横向连接 | 冻结 |
| 引导矩阵 | `root.backend.runtime.runtime.report_ops.runtime_report` | 新子叶坐标 |
| 模块树 | `runtime.report_ops.runtime_report` | planned child |

---

## 当前真实边界

当前真实代码文件:

```text
src/runtime/report_ops.rs
src/runtime/mod.rs
src/backend/runtime/routes/report_ops.rs
```

计划目标文件尚未创建:

```text
src/runtime/report_ops/runtime_report.rs
```

当前 route facade 保持:

```text
GET  /api/runtime/reports
POST /api/runtime/reports
GET  /api/runtime/reports/:report_id
GET  /api/runtime/reports/:report_id/export
```

调用路径保持:

```text
src/backend/runtime/routes/report_ops.rs
  -> crate::runtime as runtime_handlers
  -> src/runtime/mod.rs pub(crate) use report_ops::{...}
  -> src/runtime/report_ops.rs
```

---

## 白箱节点

### public handler

| 方法 | 输入 | 输出 | 当前职责 |
| --- | --- | --- | --- |
| `create_runtime_report` | `auth::UserId`、`State<AppState>`、`Json<CreateRuntimeReportRequest>` | `Json<RuntimeEvidenceReportRecord>` | 从 run/backtest source 生成 runtime evidence report，去重读取既有 report，记录 metrics 并持久化 |
| `list_runtime_reports` | `auth::UserId`、`State<AppState>`、`Query<PaginationQuery>` | `Json<PaginatedResponse<RuntimeEvidenceReportRecord>>` | 读取 report store，materialize source 状态，按创建时间和 report id 排序并分页 |
| `get_runtime_report_detail` | `auth::UserId`、`State<AppState>`、`Path<String>` | `Json<RuntimeEvidenceReportRecord>` | 读取单个 report 并 materialize source 状态 |
| `export_runtime_report_artifact` | `auth::UserId`、`State<AppState>`、`Path<String>` | `Json<RuntimeEvidenceReportArtifact>` | 读取单个 report，materialize 后导出 artifact |

### private helper

| 方法 | 当前职责 |
| --- | --- |
| `report_source_metadata_matches` | 判断 saved report 与当前 source report 的 graph/source/governance/generation metadata 是否一致 |
| `source_changed_report` | 将 Ready report materialize 为 `RuntimeReportLifecycleStatus::SourceChanged` 并写入 failure metadata |
| `current_report_for_saved_source` | 按 source kind 重新加载 run/backtest source 并构造当前 report snapshot |
| `materialize_runtime_report_record` | 对 Ready report 执行 source missing/source changed 检查并记录 evidence metrics |

---

## 允许迁移清单

BE-001CC-02 只能规划迁移以下 public handler:

- `create_runtime_report`
- `list_runtime_reports`
- `get_runtime_report_detail`
- `export_runtime_report_artifact`

BE-001CC-02 只能规划迁移以下 private helper:

- `report_source_metadata_matches`
- `source_changed_report`
- `current_report_for_saved_source`
- `materialize_runtime_report_record`

允许的父级形态仅限后续方案明确:

```rust
mod runtime_report;

pub(crate) use runtime_report::{
    create_runtime_report, export_runtime_report_artifact, get_runtime_report_detail,
    list_runtime_reports,
};
```

---

## 明确排除

以下内容不得进入 BE-001CC-02:

- `list_merge_records`
- `list_config_generations`
- `get_storage_health`
- `get_ops_daily_report`
- `get_audit_weekly_report`
- `get_research_monthly_report`
- `/api/v1/merge/records`
- `/api/v1/runtime/generations`
- `/api/v1/storage/health`
- `/api/v1/reports/ops/daily`
- `/api/v1/reports/audit/weekly`
- `/api/v1/reports/research/monthly`
- `get_runtime_evidence_health`
- `cleanup_runtime_evidence`
- `runtime_report_status_counts`
- `runtime.evidence_health`
- `AppState`
- schema owner
- frontend caller
- runtime persistence owner
- storage lifecycle owner
- release transition guard

---

## 等价证据

既有后端覆盖:

- `tests/api_run.rs` 覆盖 run report create / duplicate / list / detail / export / source changed。
- `tests/api_backtest.rs` 覆盖 backtest report evidence metadata。
- `tests/api_evidence_contract.rs` 覆盖 report contract / export / cleanup preserves reports。
- `tests/api_mutation.rs` 覆盖 mutation activation 后 report export 相关路径。

既有前端覆盖:

- `frontend/src/components/RuntimeReportPanel.test.jsx`
- `frontend/src/store/graphStoreRuntimeHistoryApi.js`

本基线不运行代码级验证；BE-001CC-02/03 必须继承:

```powershell
cargo fmt --check
cargo check -p quantpilot
cargo test --no-run
cargo test -p quantpilot --test api_run
cargo test -p quantpilot --test api_backtest
cargo test -p quantpilot --test api_evidence_contract
cargo test -p quantpilot --test api_mutation
```

本基线提交前运行治理门禁:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1
git diff --check
```

---

## 下一步

下一步只允许进入:

```text
BE-001CC-02 runtime.report_ops.runtime_report 抽离方案
```

BE-001CC-02 只能规划 `src/runtime/report_ops/runtime_report.rs` 的最小物理迁移、父级 `mod runtime_report` 与受控 `pub(crate) use runtime_report::{...}` re-export。不得创建文件、迁移 handler、修改 route facade、扩大 v1 ops/report endpoints 测试缺口或启动 release transition。

---

## 幻觉检查点

AI 声称 BE-001CC-01 完成时，必须说明:

1. 当前是 `no code movement` 的等价基线。
2. `src/runtime/report_ops/runtime_report.rs` 尚未创建。
3. runtime report handler 与 helper 仍在 `src/runtime/report_ops.rs`。
4. 下一步只能进入 BE-001CC-02 抽离方案。
5. v1 ops/report endpoints、`runtime.evidence_health`、schema owner、frontend caller、runtime persistence owner、storage lifecycle owner、`AppState` 和 release transition guard 均未迁移。

---

## 验收标准

1. `256-runtime.report_ops.runtime_report单子叶等价基线.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. `runtime.report_ops.runtime_report` 白箱节点、允许迁移清单和排除项已冻结。
3. 治理门禁、全量树覆盖和 `git diff --check` 均通过。
4. 下一步固定为 BE-001CC-02 抽离方案。
