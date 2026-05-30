# v4.16.0 runtime.report_ops 抽离方案

> 版本类型: MINOR architecture / governance  
> 执行档位: 标准  
> 批次: BE-001CB-02  
> 基准: `252-runtime.report_ops单子叶等价基线.md`、`251-backend.runtime父叶残余判断.md`、`13-递归模块化全局根流程.md`  
> 判定: `runtime.report_ops` 抽离方案已建立。当前 `no code movement`，只规划 handler 子叶物理迁移；下一步 BE-001CB-03 才允许创建 `src/runtime/report_ops.rs` 并迁移 runtime report / v1 ops report handler。`runtime.evidence_health`、schema owner、frontend caller、runtime persistence owner、storage lifecycle owner、`AppState` 和 release transition guard 均不得迁移。  
> 代码动作: no code movement

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001CB-02 `runtime.report_ops` 抽离方案 | 新建方案 |
| 规范矩阵 | handler 等价、父级 re-export、禁止横向连接、测试缺口显式处理 | 冻结 |
| 引导矩阵 | `root.backend.runtime.runtime.report_ops` | 更新下一步 |
| 模块树 | `runtime.report_ops` | `stop_split: pending` |

---

## 引导坐标声明

| 坐标 | 指向 |
| --- | --- |
| 引导坐标 | `root.backend.runtime.runtime.report_ops` |
| 全量树节点 | `markdown/10-overview/overview-full-feature-tree.md` -> `src/runtime/mod.rs` |
| 模块树节点 | `markdown/00-matrix-governance/module-tree.md` -> `root.backend.runtime.runtime.report_ops` |
| 当前真实文件 | `src/runtime/mod.rs`、`src/backend/runtime/routes/report_ops.rs`、`src/runtime_persistence.rs`、`src/runtime_response_mapping.rs`、`src/runtime/run.rs`、`src/runtime/mutation.rs`、`src/frontend_api_types.rs`、`frontend/src/store/graphStoreRuntimeHistoryApi.js`、`frontend/src/components/RuntimeReportPanel.jsx`、`tests/api_run.rs`、`tests/api_backtest.rs`、`tests/api_mutation.rs`、`tests/api_evidence_contract.rs` |
| 计划目标文件 | `src/runtime/report_ops.rs` |
| 父级出口 | `src/runtime/mod.rs` 中新增受控 `mod report_ops;` 与 `pub(crate) use report_ops::{...};` |
| route facade | `src/backend/runtime/routes/report_ops.rs` 继续调用 `crate::runtime` re-export，不改 route path、method 或 order |
| 测试/门禁 | `cargo fmt --check`、`cargo check -p quantpilot`、`cargo test --no-run`、`cargo test -p quantpilot --test api_run`、`cargo test -p quantpilot --test api_evidence_contract`、`cargo test -p quantpilot --test api_backtest`、`cargo test -p quantpilot --test api_mutation`、`tools\check-utf8.ps1`、`tools\check-matrix-governance.ps1`、`tools\check-full-feature-tree.ps1`、`git diff --check` |

---

## 目标形态

BE-001CB-03 的唯一目标是把 `runtime.report_ops` handler 从 `src/runtime/mod.rs` 移入 child module:

```text
src/runtime/mod.rs
  mod report_ops;
  pub(crate) use report_ops::{
      create_runtime_report,
      export_runtime_report_artifact,
      get_audit_weekly_report,
      get_ops_daily_report,
      get_research_monthly_report,
      get_runtime_report_detail,
      get_storage_health,
      list_config_generations,
      list_merge_records,
      list_runtime_reports,
  };

src/runtime/report_ops.rs
  use super::*;
  pub(crate) async fn create_runtime_report(...)
  pub(crate) async fn list_runtime_reports(...)
  pub(crate) async fn get_runtime_report_detail(...)
  pub(crate) async fn export_runtime_report_artifact(...)
  pub(crate) async fn list_merge_records(...)
  pub(crate) async fn list_config_generations(...)
  pub(crate) async fn get_storage_health(...)
  pub(crate) async fn get_ops_daily_report(...)
  pub(crate) async fn get_audit_weekly_report(...)
  pub(crate) async fn get_research_monthly_report(...)
```

`src/backend/runtime/routes/report_ops.rs` 不需要知道 child 文件存在；它仍通过 `crate::runtime as runtime_handlers` 调用父级 re-export。父子通信保持:

```text
backend.runtime
  -> backend.runtime.routes
  -> backend.runtime.routes.report_ops
  -> runtime.report_ops via crate::runtime re-export
```

route facade 的 path / method 保持不变:

| route | method | handler |
| --- | --- | --- |
| `/api/runtime/reports` | GET | `list_runtime_reports` |
| `/api/runtime/reports` | POST | `create_runtime_report` |
| `/api/runtime/reports/:report_id` | GET | `get_runtime_report_detail` |
| `/api/runtime/reports/:report_id/export` | GET | `export_runtime_report_artifact` |
| `/api/v1/merge/records` | GET | `list_merge_records` |
| `/api/v1/runtime/generations` | GET | `list_config_generations` |
| `/api/v1/storage/health` | GET | `get_storage_health` |
| `/api/v1/reports/ops/daily` | GET | `get_ops_daily_report` |
| `/api/v1/reports/audit/weekly` | GET | `get_audit_weekly_report` |
| `/api/v1/reports/research/monthly` | GET | `get_research_monthly_report` |

---

## 允许迁移清单

BE-001CB-03 只能迁移以下 private helper:

- `report_source_metadata_matches`
- `source_changed_report`
- `current_report_for_saved_source`
- `materialize_runtime_report_record`

BE-001CB-03 只能迁移以下 public handler:

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

允许的父级改动仅限:

1. 在 `src/runtime/mod.rs` 增加 `mod report_ops;`。
2. 在 `src/runtime/mod.rs` 增加受控 `pub(crate) use report_ops::{...};`。
3. 从 `src/runtime/mod.rs` 删除已迁出的 helper / handler 原定义。
4. 新建 `src/runtime/report_ops.rs`，文件顶部只使用既有子叶模式 `use super::*;`。
5. 因迁出导致 `src/runtime/mod.rs` import 出现未使用项时，只做最小 import 收敛。

---

## 明确排除

BE-001CB-03 不得迁移或修改:

- `get_runtime_evidence_health`
- `cleanup_runtime_evidence`
- `runtime_report_status_counts`
- `RuntimeReplayQuery`
- `RuntimeParameterMutationListQuery`
- `RuntimeAiProposalListQuery`
- `clean_optional_filter`
- `normalized_replay_options`
- `RunInProgressGuard`
- `AppState` 字段、锁顺序、state owner 或 store dir owner
- `runtime_persistence` owner
- `runtime_response_mapping` owner
- `frontend_api_types` schema owner
- frontend caller
- storage lifecycle owner
- release transition guard
- `src/backend/runtime/routes/report_ops.rs` 的 route path、method、handler name 或 route order

`runtime.evidence_health` 后续应作为 sibling 另起父叶判断或单子叶基线；不得在 BE-001CB-03 顺手合并。

---

## v1 ops/report 测试缺口处理

当前已有测试覆盖 runtime report create/list/detail/export:

- `tests/api_run.rs`
- `tests/api_backtest.rs`
- `tests/api_mutation.rs`
- `tests/api_evidence_contract.rs`
- `frontend/src/components/RuntimeReportPanel.test.jsx`

当前缺口仍然存在:

- `/api/v1/merge/records`
- `/api/v1/runtime/generations`
- `/api/v1/storage/health`
- `/api/v1/reports/ops/daily`
- `/api/v1/reports/audit/weekly`
- `/api/v1/reports/research/monthly`

BE-001CB-03 是纯物理迁移，不新增测试资产；必须以 `cargo check -p quantpilot`、`cargo test --no-run` 和既有 API 测试证明编译等价。BE-001CB-04 单叶 closeout 必须再次声明这些 v1 endpoint 测试缺口，并判断是否需要另起测试补强子叶；不得在实际迁移时静默掩盖缺口。

---

## 父子通信规则

`runtime.report_ops` 只能作为 `backend.runtime` 父级下的 handler 子叶。route facade、frontend caller、schema owner、runtime persistence owner 和 storage lifecycle owner 均保持外部 owner。开发者未明确进入发布版本过渡前，AI 不得主动提出横向连接、缓存旁路、跨子叶调用或性能优化。

---

## 回退点

若 BE-001CB-03 验证失败，回退只允许:

1. 删除 `src/runtime/report_ops.rs`。
2. 移除 `src/runtime/mod.rs` 中的 `mod report_ops;`。
3. 移除 `src/runtime/mod.rs` 中的 `pub(crate) use report_ops::{...};`。
4. 将本批迁出的 helper / handler 原样放回 `src/runtime/mod.rs` 原区域。
5. 恢复因本批产生的最小 import 收敛。

不得用回退作为理由迁移 `get_runtime_evidence_health`、`cleanup_runtime_evidence`、`runtime_report_status_counts`、schema owner、frontend caller、runtime persistence owner、storage lifecycle owner、`AppState` 或 release transition guard。

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
BE-001CB-03 runtime.report_ops 实际抽离
```

BE-001CB-03 完成后必须进入 BE-001CB-04 单叶 closeout，判断 `runtime.report_ops` 是否还值得继续细拆。不得跳过 closeout 直接处理 `runtime.evidence_health`、schema owner、state owner、runtime persistence owner、frontend caller 或发布过渡。

---

## 幻觉检查点

AI 声称 BE-001CB-02 完成时，必须说明当前仍是 `no code movement` 的抽离方案，`src/runtime/report_ops.rs` 尚未创建，handler 尚未迁移，v1 ops/report endpoints 测试缺口仍存在。不得宣称代码已抽离、`runtime.evidence_health` 已处理、发布过渡已启动、整理或重构已经完成。

---

## 验收标准

1. `253-runtime.report_ops抽离方案.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. 方案明确 BE-001CB-03 的目标文件、父级 re-export、允许迁移清单、非目标边界、测试缺口和回退点。
3. `runtime.evidence_health` 继续排除在本叶第一轮抽离之外。
4. 下一步固定为 BE-001CB-03 实际抽离。
5. 本批保持 `no code movement`。
