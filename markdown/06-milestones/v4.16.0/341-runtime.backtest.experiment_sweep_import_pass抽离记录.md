# v4.16.0 runtime.backtest.experiment_sweep_import_pass 抽离记录

> 版本类型: MINOR architecture / governance
> 执行档位: 标准
> 批次: BE-001DI-03
> 基准: `340-runtime.backtest.experiment_sweep_import_pass抽离方案.md`
> 目标子叶: `runtime.backtest.experiment_sweep_import_pass`
> 模块树坐标: `root.backend.runtime.runtime.parent_import_bridge.runtime.backtest_import_pass.runtime.backtest.experiment_sweep_import_pass`
> 代码动作: actual Rust import rewrite
> 下一步: BE-001DI-04 `runtime.backtest.experiment_sweep_import_pass` 单叶 closeout

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001DI-03 `runtime.backtest.experiment_sweep_import_pass` 实际抽离 | 实际抽离 |
| 规范矩阵 | four-file explicit import rewrite、parent whitebox handoff、release transition guard | 等价收敛 |
| 引导矩阵 | `root.backend.runtime.runtime.parent_import_bridge.runtime.backtest_import_pass.runtime.backtest.experiment_sweep_import_pass` | experiment sweep import 白箱收敛 |
| 模块树 | `runtime.backtest.experiment_sweep_import_pass` | 抽离记录 |

---

## 实际改动

本批只改写:

```text
src/runtime/backtest/experiment_sweep.rs
src/runtime/backtest/parameter_grid.rs
src/runtime/backtest/record_lifecycle.rs
src/runtime/backtest/start_orchestration.rs
```

删除四文件的 parent wildcard / sibling super import:

```rust
use super::*;
use super::parameter_grid::build_experiment_overrides;
```

---

## 实际输入面

### `experiment_sweep.rs`

```rust
mod parameter_grid;
mod record_lifecycle;
mod start_orchestration;

use parameter_grid::build_experiment_overrides;

pub(crate) use record_lifecycle::{
    discard_experiment_record, get_experiment_detail, list_experiments, save_experiment_record,
};
pub(crate) use start_orchestration::start_backtest_experiment;
```

### `parameter_grid.rs`

```rust
use crate::{
    json_bad_request, resolved_backtest_execution_assumptions, runtime::MAX_EXPERIMENT_VARIANTS,
    FrontendExecutionAssumptionOverrides, FrontendExperimentRequest,
};
use axum::http::StatusCode;
use qrpc_core::RuntimeProtocolCoreConfig;
```

### `record_lifecycle.rs`

```rust
use crate::{
    auth, build_graph_audit_entry, delete_transient_backtest_record,
    experiment_detail_response_from_record, experiment_list_item_from_record, io_error,
    list_experiment_records, load_backtest_record_from_state, load_experiment_record_from_state,
    paginate, persist_backtest_record, persist_experiment_record, persist_graph_audit_entry,
    runtime::DiscardRuntimeArtifactResponse, sanitize_storage_path_segment, AppState,
    ExperimentDetailResponse, ExperimentListItem, GraphAuditAction, PaginatedResponse,
    PaginationQuery,
};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use tokio::fs;
```

### `start_orchestration.rs`

```rust
use crate::{
    auth, compile_runtime_protocol_via_qs, current_time_ms, experiment_detail_response_from_record,
    io_error, json_bad_request, json_bad_request_with_details, normalize_actor_identity,
    persist_experiment_record, resolved_backtest_execution_assumptions,
    runtime::{backtest_experiment_sweep::build_experiment_overrides, execute_backtest_request},
    validate_backtest_execution_assumption_overrides, validate_runtime_capability_guard,
    validate_runtime_config_capabilities, AppState, ExperimentDefinitionSummary,
    ExperimentDetailResponse, ExperimentRecord, ExperimentVariantSummary, FrontendBacktestOptions,
    FrontendBacktestReplaySource, FrontendExecutionAssumptionOverrides, FrontendExperimentRequest,
    FrontendRunRequest,
};
use axum::{extract::State, http::StatusCode, Json};
```

`build_experiment_overrides` 通过 `experiment_sweep.rs` 父级白箱输入面转交给 `start_orchestration.rs`，未新增 sibling horizontal link。

---

## 等价结果

- `start_backtest_experiment` 行为未变。
- `build_experiment_overrides` 的 grid validation、dedupe、base fallback、variant expansion order 与 `MAX_EXPERIMENT_VARIANTS` guard 未变。
- `list_experiments`、`get_experiment_detail`、`save_experiment_record`、`discard_experiment_record` 行为未变。
- route path、method、handler name、status code、response schema、state cache、persistence owner、audit owner 与 transient cleanup 均未改变。
- runtime parent bridge 依赖文件数从 32 降为 28。
- 当前分布为 root 1 / run 0 / backtest 5 / mutation 21 / test-only 1 / total 28。

---

## 当前剩余

`runtime.backtest` 剩余 parent bridge 依赖已收敛到 execution_start 组:

```text
src/runtime/backtest/execution_start.rs
src/runtime/backtest/legacy_dispatch.rs
src/runtime/backtest/v4_projection.rs
src/runtime/backtest/v4_request_resolution.rs
src/runtime/backtest/v4_runtime_execution.rs
```

---

## 排除项

- 本批不处理 `src/runtime/backtest/execution_start.rs`、`legacy_dispatch.rs` 或 `v4_*` 文件。
- 本批不处理 `src/runtime/mod.rs` root parent bridge。
- 本批不处理 `src/runtime/mutation/**` 或 test-only `src/runtime/run_guard.rs`。
- 本批不迁移 route facade、state owner、persistence owner、schema owner、frontend caller 或 test asset。
- 本批不新增 sibling horizontal link。
- 本批不启动 release transition。

---

## 验证要求

本批提交前至少执行:

```powershell
cargo fmt --check
cargo check -p quantpilot
cargo test -p quantpilot --test api_experiments
cargo test -p quantpilot --test api_backtest
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1
git diff --check
```

---

## 幻觉检查点

AI 声称 BE-001DI-03 完成时，必须说明:

1. 本批次是 actual Rust import rewrite。
2. 只改写四文件 import 和父级白箱 handoff。
3. runtime parent bridge 依赖文件数从 32 降为 28。
4. 下一步只能进入 BE-001DI-04 `runtime.backtest.experiment_sweep_import_pass` 单叶 closeout。
5. execution_start、root bridge、mutation 子树与 test-only `src/runtime/run_guard.rs` 尚未处理。
6. release transition 未启动，未新增 sibling horizontal link。

不得宣称 `runtime.backtest_import_pass` 已完成、parent import bridge 已清除或 `backend.runtime` 已完成。

---

## 验收标准

1. `341-runtime.backtest.experiment_sweep_import_pass抽离记录.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. 四个目标 Rust 文件不再包含 `use super::*` 或 `super::`。
3. runtime parent bridge 残余计数为 28。
4. Rust 编译、`api_experiments`、`api_backtest`、治理门禁、全量树覆盖和 `git diff --check` 均通过。
