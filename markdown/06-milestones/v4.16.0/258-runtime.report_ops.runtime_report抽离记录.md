# v4.16.0 runtime.report_ops.runtime_report 抽离记录

> 版本类型: MINOR architecture / governance  
> 执行档位: 标准  
> 批次: BE-001CC-03  
> 基准: `257-runtime.report_ops.runtime_report抽离方案.md`、`256-runtime.report_ops.runtime_report单子叶等价基线.md`、`13-递归模块化全局根流程.md`  
> 判定: `runtime.report_ops.runtime_report` 实际抽离已完成。`src/runtime/report_ops/runtime_report.rs` 已创建并承接四个 public handler 与四个 private helper；父级 `src/runtime/report_ops.rs` 通过受控 `mod runtime_report` 与 `pub(crate) use runtime_report::{...}` 保持 `src/runtime/mod.rs` 的既有 re-export 调用面。  
> 代码动作: actual extraction

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001CC-03 `runtime.report_ops.runtime_report` 实际抽离 | 执行抽离 |
| 规范矩阵 | handler 等价、父级 re-export、禁止横向连接、验证继承 | 冻结 |
| 引导矩阵 | `root.backend.runtime.runtime.report_ops.runtime_report` | child 文件落地 |
| 模块树 | `runtime.report_ops.runtime_report` | actual child |

---

## 实际变更

新增文件:

```text
src/runtime/report_ops/runtime_report.rs
```

父级 `src/runtime/report_ops.rs` 新增:

```rust
mod runtime_report;

pub(crate) use runtime_report::{
    create_runtime_report, export_runtime_report_artifact, get_runtime_report_detail,
    list_runtime_reports,
};
```

保持不变:

- `src/runtime/mod.rs` 的既有 `pub(crate) use report_ops::{...}` 清单。
- `src/backend/runtime/routes/report_ops.rs` route facade、route path、method、order。
- v1 ops/report handlers 仍留在 `src/runtime/report_ops.rs`。

---

## 已迁移清单

private helper:

- `report_source_metadata_matches`
- `source_changed_report`
- `current_report_for_saved_source`
- `materialize_runtime_report_record`

public handler:

- `create_runtime_report`
- `list_runtime_reports`
- `get_runtime_report_detail`
- `export_runtime_report_artifact`

---

## 保留边界

以下内容仍留在原 owner，不属于本批:

- `list_merge_records`
- `list_config_generations`
- `get_storage_health`
- `get_ops_daily_report`
- `get_audit_weekly_report`
- `get_research_monthly_report`
- v1 ops/report endpoints
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

已执行:

```powershell
cargo fmt --check
cargo check -p quantpilot
cargo test --no-run
cargo test -p quantpilot --test api_run
cargo test -p quantpilot --test api_backtest
cargo test -p quantpilot --test api_evidence_contract
cargo test -p quantpilot --test api_mutation
```

目标 API 结果:

- `api_run`: 14 passed。
- `api_backtest`: 12 passed。
- `api_evidence_contract`: 2 passed。
- `api_mutation`: 9 passed。

提交前仍需执行治理门禁:

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
BE-001CC-04 runtime.report_ops.runtime_report 单叶 closeout
```

BE-001CC-04 必须判断 `runtime.report_ops.runtime_report` 是否还值得继续细拆。不得跳过 closeout 直接处理 v1 ops/report endpoints、`runtime.evidence_health`、schema owner、frontend caller、state/persistence owner、storage lifecycle owner、`AppState` 或 release transition。

---

## 幻觉检查点

AI 声称 BE-001CC-03 完成时，必须说明:

1. `src/runtime/report_ops/runtime_report.rs` 已创建。
2. 四个 public handler 与四个 private helper 已迁入 child。
3. `src/runtime/report_ops.rs` 只新增受控 `mod runtime_report` 与 `pub(crate) use runtime_report::{...}`，并保留 v1 ops/report handlers。
4. `src/runtime/mod.rs` 与 `src/backend/runtime/routes/report_ops.rs` 未改变。
5. v1 ops/report endpoints、`runtime.evidence_health`、schema owner、frontend caller、runtime persistence owner、storage lifecycle owner、`AppState` 和 release transition guard 均未迁移。
6. 下一步必须进入 BE-001CC-04 单叶 closeout。

---

## 验收标准

1. `258-runtime.report_ops.runtime_report抽离记录.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. `src/runtime/report_ops/runtime_report.rs` 进入全量树 active file coverage。
3. Rust 编译、目标 API 测试和文档治理门禁均通过。
4. 下一步固定为 BE-001CC-04 单叶 closeout。
