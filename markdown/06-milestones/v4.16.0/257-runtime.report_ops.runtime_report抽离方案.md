# v4.16.0 runtime.report_ops.runtime_report 抽离方案

> 版本类型: MINOR architecture / governance  
> 执行档位: 标准  
> 批次: BE-001CC-02  
> 基准: `256-runtime.report_ops.runtime_report单子叶等价基线.md`、`255-runtime.report_ops单叶closeout.md`、`13-递归模块化全局根流程.md`  
> 判定: `runtime.report_ops.runtime_report` 抽离方案已建立。当前 `no code movement`，只规划 runtime report handler/helper 的最小物理迁移；下一步 BE-001CC-03 才允许创建 `src/runtime/report_ops/runtime_report.rs` 并迁移四个 public handler 与四个 private helper。  
> 代码动作: no code movement

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001CC-02 `runtime.report_ops.runtime_report` 抽离方案 | 建方案 |
| 规范矩阵 | 父级 re-export、handler 等价、禁止横向连接、回退点 | 冻结 |
| 引导矩阵 | `root.backend.runtime.runtime.report_ops.runtime_report` | 固定下一步 |
| 模块树 | `runtime.report_ops.runtime_report` | planned extraction |

---

## 目标

BE-001CC-03 的唯一目标是把 runtime report handler/helper 从 `src/runtime/report_ops.rs` 移入 child module:

```text
src/runtime/report_ops/runtime_report.rs
```

父级 `src/runtime/report_ops.rs` 只允许新增受控 module 声明与 re-export:

```rust
mod runtime_report;

pub(crate) use runtime_report::{
    create_runtime_report, export_runtime_report_artifact, get_runtime_report_detail,
    list_runtime_reports,
};
```

`src/runtime/mod.rs` 既有 `pub(crate) use report_ops::{...}` 出口不改变。`src/backend/runtime/routes/report_ops.rs` route facade 不改变。

---

## 迁移清单

BE-001CC-03 只允许迁移以下 private helper:

- `report_source_metadata_matches`
- `source_changed_report`
- `current_report_for_saved_source`
- `materialize_runtime_report_record`

BE-001CC-03 只允许迁移以下 public handler:

- `create_runtime_report`
- `list_runtime_reports`
- `get_runtime_report_detail`
- `export_runtime_report_artifact`

迁移后调用路径必须保持:

```text
src/backend/runtime/routes/report_ops.rs
  -> crate::runtime as runtime_handlers
  -> src/runtime/mod.rs
  -> src/runtime/report_ops.rs re-export
  -> src/runtime/report_ops/runtime_report.rs
```

---

## 实施顺序

1. 在 `src/runtime/report_ops.rs` 顶部保留 `use super::*;`，新增 `mod runtime_report;` 与受控 `pub(crate) use runtime_report::{...}`。
2. 新建 `src/runtime/report_ops/runtime_report.rs`，优先通过 `use super::*;` 复用父级可见上下文；若编译器要求显式导入，只能补充当前迁移清单已使用的既有类型/函数导入，不得引入新 owner。
3. 将四个 private helper 和四个 public handler 原样迁入 child，保留 async signature、返回类型、错误形态、metrics side effect、sort/paginate 逻辑和 source changed materialization 行为。
4. 从 `src/runtime/report_ops.rs` 删除已迁出的函数，保留 v1 ops/report handlers 原地不动。
5. 不修改 `src/runtime/mod.rs` 的既有 report_ops re-export 清单。
6. 不修改 route facade、schema、frontend caller、runtime persistence owner、storage lifecycle owner、`AppState` 或 release transition guard。

---

## 明确排除

以下内容不得进入 BE-001CC-03:

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

## 等价验证

BE-001CC-03 完成后必须执行:

```powershell
cargo fmt --check
cargo check -p quantpilot
cargo test --no-run
cargo test -p quantpilot --test api_run
cargo test -p quantpilot --test api_backtest
cargo test -p quantpilot --test api_evidence_contract
cargo test -p quantpilot --test api_mutation
```

提交前必须执行治理门禁:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1
git diff --check
```

---

## 回退点

若 BE-001CC-03 验证失败，回退只允许:

1. 删除 `src/runtime/report_ops/runtime_report.rs`。
2. 将四个 public handler 与四个 private helper 放回 `src/runtime/report_ops.rs`。
3. 移除 `mod runtime_report` 与 `pub(crate) use runtime_report::{...}`。
4. 保持 `src/runtime/mod.rs`、route facade、schema、frontend caller、state/persistence owner 不变。

不得通过改 route、改 schema、改状态结构或启动 release transition 来绕过失败。

---

## 下一步

下一步只允许进入:

```text
BE-001CC-03 runtime.report_ops.runtime_report 实际抽离
```

BE-001CC-03 必须严格执行本方案的迁移清单和排除项。完成后必须进入 BE-001CC-04 单叶 closeout，判断 `runtime.report_ops.runtime_report` 是否还值得继续细拆，不得跳过 closeout 直接处理 v1 ops/report endpoints、`runtime.evidence_health`、schema owner、frontend caller、state/persistence owner 或 release transition。

---

## 幻觉检查点

AI 声称 BE-001CC-02 完成时，必须说明:

1. 当前仍是 `no code movement` 的抽离方案。
2. `src/runtime/report_ops/runtime_report.rs` 尚未创建。
3. runtime report handler/helper 尚未迁移。
4. 下一步 BE-001CC-03 才允许实际抽离。
5. v1 ops/report endpoints、`runtime.evidence_health`、schema owner、frontend caller、runtime persistence owner、storage lifecycle owner、`AppState` 和 release transition guard 均不属于本批。

---

## 验收标准

1. `257-runtime.report_ops.runtime_report抽离方案.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. 方案明确 BE-001CC-03 的目标文件、父级 re-export、允许迁移清单、排除项、验证命令和回退点。
3. 治理门禁、全量树覆盖和 `git diff --check` 均通过。
4. 下一步固定为 BE-001CC-03 实际抽离。
