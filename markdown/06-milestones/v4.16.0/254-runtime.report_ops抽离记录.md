# v4.16.0 runtime.report_ops 抽离记录

> 版本类型: MINOR architecture / governance  
> 执行档位: 标准  
> 批次: BE-001CB-03  
> 基准: `253-runtime.report_ops抽离方案.md`、`252-runtime.report_ops单子叶等价基线.md`、`13-递归模块化全局根流程.md`  
> 判定: `runtime.report_ops` 实际抽离已完成。`src/runtime/report_ops.rs` 已创建，runtime report / v1 ops report handler 与四个 report helper 已迁入 child；`src/runtime/mod.rs` 通过受控 `mod report_ops` 与 `pub(crate) use report_ops::{...}` 保持 route facade 调用面。`runtime.evidence_health`、schema owner、frontend caller、runtime persistence owner、storage lifecycle owner、`AppState` 和 release transition guard 均未迁移。  
> 代码动作: actual extraction

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001CB-03 `runtime.report_ops` 实际抽离 | 执行抽离 |
| 规范矩阵 | handler 等价、父级 re-export、禁止横向连接、测试缺口显式保留 | 冻结 |
| 引导矩阵 | `root.backend.runtime.runtime.report_ops` | child 文件落地 |
| 模块树 | `runtime.report_ops` | actual child |

---

## 实际变更

新增文件:

```text
src/runtime/report_ops.rs
```

父级 `src/runtime/mod.rs` 新增:

```rust
mod report_ops;

pub(crate) use report_ops::{
    create_runtime_report, export_runtime_report_artifact, get_audit_weekly_report,
    get_ops_daily_report, get_research_monthly_report, get_runtime_report_detail,
    get_storage_health, list_config_generations, list_merge_records, list_runtime_reports,
};
```

route facade `src/backend/runtime/routes/report_ops.rs` 未改变，仍通过 `crate::runtime as runtime_handlers` 调用父级 re-export。route path / method / order 未改变。

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
- `list_merge_records`
- `list_config_generations`
- `get_storage_health`
- `get_ops_daily_report`
- `get_audit_weekly_report`
- `get_research_monthly_report`

---

## 保留边界

以下项仍留在原 owner，不属于本批:

- `get_runtime_evidence_health`
- `cleanup_runtime_evidence`
- `runtime_report_status_counts`
- `RuntimeReplayQuery`
- `RuntimeParameterMutationListQuery`
- `RuntimeAiProposalListQuery`
- `clean_optional_filter`
- `normalized_replay_options`
- `RunInProgressGuard`
- `AppState`
- `runtime_persistence`
- `runtime_response_mapping`
- `frontend_api_types`
- frontend caller
- storage lifecycle owner
- release transition guard

`runtime.evidence_health` 仍应作为 sibling 另起父叶判断或单子叶基线，不得视为已并入 `runtime.report_ops`。

---

## 等价证据

已执行:

```powershell
cargo fmt --check
cargo check -p quantpilot
cargo test --no-run
cargo test -p quantpilot --test api_run
cargo test -p quantpilot --test api_evidence_contract
cargo test -p quantpilot --test api_backtest
cargo test -p quantpilot --test api_mutation
```

提交前仍必须执行治理门禁:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1
git diff --check
```

---

## v1 ops/report 测试缺口

本批不新增测试资产，以下 endpoint 仍缺少专门 API 测试:

- `/api/v1/merge/records`
- `/api/v1/runtime/generations`
- `/api/v1/storage/health`
- `/api/v1/reports/ops/daily`
- `/api/v1/reports/audit/weekly`
- `/api/v1/reports/research/monthly`

BE-001CB-04 单叶 closeout 必须再次声明该缺口，并判断是否另起测试补强子叶；不得把本次编译等价误判为 v1 endpoint 行为全覆盖。

---

## 下一步

下一步只能进入:

```text
BE-001CB-04 runtime.report_ops 单叶 closeout
```

BE-001CB-04 必须判断 `runtime.report_ops` 是否值得继续拆成 runtime report、merge/generation/storage health、ops/audit/research report 等更细子叶；不得跳过 closeout 直接处理 `runtime.evidence_health`、schema owner、state owner、runtime persistence owner、frontend caller 或发布过渡。

---

## 幻觉检查点

AI 声称 BE-001CB-03 完成时，必须说明:

1. `src/runtime/report_ops.rs` 已创建并承载十个 public handler 与四个 private helper。
2. `src/runtime/mod.rs` 只通过 `mod report_ops` 和 `pub(crate) use report_ops::{...}` 保持兼容出口。
3. `src/backend/runtime/routes/report_ops.rs` route facade 未改变。
4. `runtime.evidence_health`、schema owner、frontend caller、runtime persistence owner、storage lifecycle owner、`AppState` 和 release transition guard 均未迁移。
5. v1 ops/report endpoints 的专门测试缺口仍存在，下一步必须进入 BE-001CB-04 单叶 closeout。

---

## 验收标准

1. `254-runtime.report_ops抽离记录.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. `src/runtime/report_ops.rs` 进入全量树 active file coverage。
3. Rust 编译、目标 API 测试和文档治理门禁均通过。
4. 下一步固定为 BE-001CB-04 单叶 closeout。
