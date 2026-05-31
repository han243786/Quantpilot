# v4.16.0 runtime.backtest.experiment_sweep_import_pass 单子叶等价基线

> 版本类型: MINOR architecture / governance
> 执行档位: 标准
> 批次: BE-001DI-01
> 基准: `338-runtime.backtest_import_pass第二轮父叶残余判断.md`
> 目标子叶: `runtime.backtest.experiment_sweep_import_pass`
> 模块树坐标: `root.backend.runtime.runtime.parent_import_bridge.runtime.backtest_import_pass.runtime.backtest.experiment_sweep_import_pass`
> 代码动作: no code movement
> 下一步: BE-001DI-02 `runtime.backtest.experiment_sweep_import_pass` 抽离方案

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001DI-01 `runtime.backtest.experiment_sweep_import_pass` 单子叶等价基线 | 单子叶基线 |
| 规范矩阵 | staged explicit import pass、four-file pocket、experiment sweep import、release transition guard | 基线冻结 |
| 引导矩阵 | `root.backend.runtime.runtime.parent_import_bridge.runtime.backtest_import_pass.runtime.backtest.experiment_sweep_import_pass` | experiment sweep import 白箱 |
| 模块树 | `runtime.backtest.experiment_sweep_import_pass` | 新增基线 |

---

## 当前真实文件边界

本基线冻结四文件 import pocket:

```text
src/runtime/backtest/experiment_sweep.rs
src/runtime/backtest/parameter_grid.rs
src/runtime/backtest/record_lifecycle.rs
src/runtime/backtest/start_orchestration.rs
```

当前残余:

```rust
// src/runtime/backtest/experiment_sweep.rs
use super::*;

// src/runtime/backtest/parameter_grid.rs
use super::*;

// src/runtime/backtest/record_lifecycle.rs
use super::*;

// src/runtime/backtest/start_orchestration.rs
use super::parameter_grid::build_experiment_overrides;
use super::*;
```

`experiment_sweep.rs` 父文件当前只是 child module 壳与 public re-export。若只删除父文件的 `use super::*`，三个 child 仍会通过 `use super::*` 依赖父级转运输入面，因此后续实际抽离必须以四文件 pocket 为最小等价单元。

---

## 白箱节点

| 节点 | 输入 | 输出 | 当前 owner / 处理者 |
| --- | --- | --- | --- |
| `start_backtest_experiment` | `auth::UserId`、`State<AppState>`、`Json<FrontendExperimentRequest>` | `Json<ExperimentDetailResponse>` | `src/runtime/backtest/start_orchestration.rs` |
| `build_experiment_overrides` | `&FrontendExperimentRequest`、`&RuntimeProtocolCoreConfig` | `Vec<FrontendExecutionAssumptionOverrides>` | `src/runtime/backtest/parameter_grid.rs` |
| `list_experiments` | `State<AppState>`、`Query<PaginationQuery>` | `Json<PaginatedResponse<ExperimentListItem>>` | `src/runtime/backtest/record_lifecycle.rs` |
| `get_experiment_detail` | `auth::UserId`、`State<AppState>`、`Path<String>` | `Json<ExperimentDetailResponse>` | `src/runtime/backtest/record_lifecycle.rs` |
| `save_experiment_record` | `auth::UserId`、`State<AppState>`、`Path<String>` | `Json<ExperimentDetailResponse>` | `src/runtime/backtest/record_lifecycle.rs` |
| `discard_experiment_record` | `auth::UserId`、`State<AppState>`、`Path<String>` | `Json<DiscardRuntimeArtifactResponse>` | `src/runtime/backtest/record_lifecycle.rs` |
| `normalize_experiment_float_axis` | `&[f64]`、`base`、`field` | normalized `Vec<f64>` or `bad_request` | `src/runtime/backtest/parameter_grid.rs` private helper |
| `normalize_experiment_latency_axis` | `&[u64]`、`base` | normalized `Vec<u64>` | `src/runtime/backtest/parameter_grid.rs` private helper |

---

## 等价链路

```text
POST /api/runtime/backtests/experiments
  -> start_backtest_experiment
  -> validate_runtime_capability_guard
  -> validate_runtime_config_capabilities
  -> validate_backtest_execution_assumption_overrides
  -> compile_runtime_protocol_via_qs
  -> resolved_backtest_execution_assumptions
  -> build_experiment_overrides
  -> execute_backtest_request
  -> persist_experiment_record
  -> experiment_detail_response_from_record
```

```text
GET /api/runtime/backtests/experiments
  -> list_experiments
  -> list_experiment_records
  -> experiment_list_item_from_record
  -> paginate
```

```text
GET /api/runtime/backtests/experiments/:experiment_id
  -> get_experiment_detail
  -> load_experiment_record_from_state
  -> experiment_detail_response_from_record
```

```text
POST /api/runtime/backtests/experiments/:experiment_id/save
  -> save_experiment_record
  -> load_experiment_record_from_state
  -> load_backtest_record_from_state
  -> persist_backtest_record
  -> delete_transient_backtest_record
  -> persist_experiment_record
  -> persist_graph_audit_entry
  -> experiment_detail_response_from_record
```

```text
DELETE /api/runtime/backtests/experiments/:experiment_id
  -> discard_experiment_record
  -> load_experiment_record_from_state
  -> sanitize_storage_path_segment
  -> delete_transient_backtest_record
  -> DiscardRuntimeArtifactResponse
```

---

## 预期显式输入面

BE-001DI-03 实际抽离时，预计将 parent wildcard import 收敛为:

1. `experiment_sweep.rs`: 保留 `mod parameter_grid`、`mod record_lifecycle`、`mod start_orchestration` 与 public re-export，不再需要 `use super::*`。
2. `parameter_grid.rs`: 显式输入 `json_bad_request`、`resolved_backtest_execution_assumptions`、`MAX_EXPERIMENT_VARIANTS`、`FrontendExperimentRequest`、`FrontendExecutionAssumptionOverrides`、`RuntimeProtocolCoreConfig`、`StatusCode`。
3. `record_lifecycle.rs`: 显式输入 experiment record list/detail/save/discard 所需的 `auth`、`AppState`、pagination / response DTO、storage / audit helper、`DiscardRuntimeArtifactResponse`、`StatusCode`、`Path`、`Query`、`State`、`Json`、`fs`。
4. `start_orchestration.rs`: 保留 `super::parameter_grid::build_experiment_overrides`，并显式输入 capability validation、QS compile、execution assumption resolution、`execute_backtest_request`、experiment DTO、runtime request DTO、`State`、`Json`、`StatusCode`。

实际方案必须以 `cargo check` 校准最终 import 清单；不得在基线阶段直接改写 Rust。

---

## 等价保护点

后续实际抽离不得改变:

1. experiment route path、method、handler name 与 response schema。
2. capability guard、runtime config capability gate 与 execution assumption override validation。
3. parameter grid 的空输入报错、负数报错、去重、base fallback、variant_count 与 `MAX_EXPERIMENT_VARIANTS` 上限。
4. experiment id、created_at_ms、actor normalization、experiment_name trim 与 variants 顺序。
5. `execute_backtest_request` 复用关系与 transient/persistent record 行为。
6. save/discard 的 scoped lookup、saved conflict、path safety、state cache、transient cleanup 与 audit 行为。
7. `api_experiments` 和 `api_backtest` 的可观察行为。

---

## 排除项

- 本批不修改 Rust 代码。
- 本批不直接删除任何 `use super::*`。
- 本批不处理 `src/runtime/backtest/execution_start.rs`、`legacy_dispatch.rs` 或 `v4_*` 文件。
- 本批不处理 `src/runtime/mod.rs` root parent bridge。
- 本批不处理 `src/runtime/mutation/**` 或 test-only `src/runtime/run_guard.rs`。
- 本批不迁移 route facade、state owner、persistence owner、schema owner、frontend caller 或 test asset。
- 本批不新增 sibling horizontal link。
- 本批不启动 release transition。

---

## 验证要求

本批为 `no code movement` 基线，提交前至少执行:

```powershell
cargo fmt --check
cargo check -p quantpilot
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1
git diff --check
```

后续实际抽离至少补跑:

```powershell
cargo test -p quantpilot --test api_experiments
cargo test -p quantpilot --test api_backtest
```

---

## 幻觉检查点

AI 声称 BE-001DI-01 完成时，必须说明:

1. 本批是 `no code movement` 单子叶等价基线。
2. 目标 pocket 是四文件: `experiment_sweep.rs`、`parameter_grid.rs`、`record_lifecycle.rs`、`start_orchestration.rs`。
3. 尚未删除任何 `use super::*`。
4. 下一步只能进入 BE-001DI-02 `runtime.backtest.experiment_sweep_import_pass` 抽离方案。
5. execution_start、root bridge、mutation 子树与 test-only `src/runtime/run_guard.rs` 尚未处理。
6. release transition 未启动，未新增 sibling horizontal link。

不得宣称 experiment sweep import 已抽离、`runtime.backtest_import_pass` 已完成、parent import bridge 已清除或 `backend.runtime` 已完成。

---

## 验收标准

1. `339-runtime.backtest.experiment_sweep_import_pass单子叶等价基线.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. 基线明确四文件 pocket 与所有关键 public 方法。
3. 下一步固定为 BE-001DI-02 抽离方案。
4. 治理门禁、全量树覆盖和 `git diff --check` 均通过。
