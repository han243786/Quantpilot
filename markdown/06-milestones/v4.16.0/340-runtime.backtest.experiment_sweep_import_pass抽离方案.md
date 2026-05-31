# v4.16.0 runtime.backtest.experiment_sweep_import_pass 抽离方案

> 版本类型: MINOR architecture / governance
> 执行档位: 标准
> 批次: BE-001DI-02
> 基准: `339-runtime.backtest.experiment_sweep_import_pass单子叶等价基线.md`
> 目标子叶: `runtime.backtest.experiment_sweep_import_pass`
> 模块树坐标: `root.backend.runtime.runtime.parent_import_bridge.runtime.backtest_import_pass.runtime.backtest.experiment_sweep_import_pass`
> 代码动作: no code movement
> 下一步: BE-001DI-03 actual Rust import rewrite

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001DI-02 `runtime.backtest.experiment_sweep_import_pass` 抽离方案 | 方案优化 |
| 规范矩阵 | four-file explicit import rewrite、parent whitebox handoff、release transition guard | 实施方案 |
| 引导矩阵 | `root.backend.runtime.runtime.parent_import_bridge.runtime.backtest_import_pass.runtime.backtest.experiment_sweep_import_pass` | experiment sweep import 收敛方案 |
| 模块树 | `runtime.backtest.experiment_sweep_import_pass` | 抽离方案 |

---

## 允许进入实际抽离

BE-001DI-03 可以进入 actual Rust import rewrite，范围仅限:

```text
src/runtime/backtest/experiment_sweep.rs
src/runtime/backtest/parameter_grid.rs
src/runtime/backtest/record_lifecycle.rs
src/runtime/backtest/start_orchestration.rs
```

允许动作:

1. 删除四文件中的 `use super::*`。
2. 删除 `start_orchestration.rs` 中的 `use super::parameter_grid::build_experiment_overrides`。
3. 在 `experiment_sweep.rs` 建立父级白箱输入面，保留 module declaration、public re-export，并按需提供 parent-mediated `build_experiment_overrides` handoff。
4. 在 `parameter_grid.rs`、`record_lifecycle.rs`、`start_orchestration.rs` 改为显式 import。
5. 仅在 `cargo check` 要求时微调 visibility，且不得改变 public API、route path、response schema 或 persistence owner。

---

## 计划显式输入面

### `experiment_sweep.rs`

目标:

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

该 `build_experiment_overrides` 输入只作为父级白箱 handoff，供 `start_orchestration.rs` 通过 parent coordinate 使用；不得新增 sibling horizontal link。

### `parameter_grid.rs`

计划输入:

```rust
use crate::{
    json_bad_request, resolved_backtest_execution_assumptions,
    runtime::MAX_EXPERIMENT_VARIANTS, FrontendExecutionAssumptionOverrides,
    FrontendExperimentRequest,
};
use axum::http::StatusCode;
use qrpc_core::RuntimeProtocolCoreConfig;
```

### `record_lifecycle.rs`

计划输入:

```rust
use crate::{
    auth, build_graph_audit_entry, delete_transient_backtest_record,
    experiment_detail_response_from_record, experiment_list_item_from_record, io_error,
    list_experiment_records, load_backtest_record_from_state, load_experiment_record_from_state,
    paginate, persist_backtest_record, persist_experiment_record, persist_graph_audit_entry,
    sanitize_storage_path_segment, AppState, DiscardRuntimeArtifactResponse,
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

计划输入:

```rust
use crate::{
    auth, compile_runtime_protocol_via_qs, current_time_ms, execute_backtest_request,
    experiment_detail_response_from_record, json_bad_request, json_bad_request_with_details,
    normalize_actor_identity, resolved_backtest_execution_assumptions,
    runtime::backtest_experiment_sweep::build_experiment_overrides,
    validate_backtest_execution_assumption_overrides, validate_runtime_capability_guard,
    validate_runtime_config_capabilities, AppState, ExperimentDefinitionSummary,
    ExperimentDetailResponse, ExperimentRecord, ExperimentVariantSummary, FrontendBacktestOptions,
    FrontendBacktestReplaySource, FrontendExecutionAssumptionOverrides, FrontendExperimentRequest,
    FrontendRunRequest,
};
use axum::{extract::State, http::StatusCode, Json};
```

最终 import 清单以 BE-001DI-03 的 `cargo check` 为准；方案阶段不改 Rust。

---

## 等价保护点

BE-001DI-03 不得改变以下行为:

1. `start_backtest_experiment` 的 route path、method、handler name、status code 与 `ExperimentDetailResponse` schema。
2. `build_experiment_overrides` 的 empty grid、negative value、dedupe、base fallback、variant expansion order 与 `MAX_EXPERIMENT_VARIANTS` guard。
3. capability guard、runtime config capability gate、execution assumption override validation 与 QS compile。
4. `execute_backtest_request` 复用关系、variant_id 顺序、summary / execution_assumptions_tag 生成。
5. experiment metadata 持久化、state cache、save/discard lifecycle、transient cleanup、audit write 与 path safety。
6. `api_experiments` 与 `api_backtest` 的可观察行为。

---

## 预期计数

实际抽离完成后，预期:

```text
runtime parent bridge 依赖文件数: 32 -> 28
root 1 / run 0 / backtest 5 / mutation 21 / test-only 1 / total 28
```

若 `cargo check` 证明 parent-mediated `build_experiment_overrides` handoff 需要不同 visibility 或路径，BE-001DI-03 可以在四文件范围内调整，但必须保持父子通信规则，不得新增 sibling horizontal link。

---

## 排除项

- BE-001DI-03 不得处理 `src/runtime/backtest/execution_start.rs`、`legacy_dispatch.rs` 或 `v4_*` 文件。
- BE-001DI-03 不得处理 `src/runtime/mod.rs` root parent bridge。
- BE-001DI-03 不得处理 `src/runtime/mutation/**` 或 test-only `src/runtime/run_guard.rs`。
- BE-001DI-03 不得迁移 route facade、state owner、persistence owner、schema owner、frontend caller 或 test asset。
- BE-001DI-03 不得新增 sibling horizontal link。
- BE-001DI-03 不得启动 release transition。

---

## 验证要求

BE-001DI-03 实际抽离后至少执行:

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

AI 声称 BE-001DI-02 完成时，必须说明:

1. 本批是 `no code movement` 抽离方案。
2. BE-001DI-03 只允许改写四文件 import 和必要 visibility。
3. 尚未删除任何 `use super::*`。
4. 预期 runtime parent bridge 依赖文件数从 32 降到 28。
5. execution_start、root bridge、mutation 子树与 test-only `src/runtime/run_guard.rs` 尚未处理。
6. release transition 未启动，未新增 sibling horizontal link。

不得宣称 experiment sweep import 已抽离、`runtime.backtest_import_pass` 已完成、parent import bridge 已清除或 `backend.runtime` 已完成。

---

## 验收标准

1. `340-runtime.backtest.experiment_sweep_import_pass抽离方案.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. 方案固定 BE-001DI-03 四文件 import rewrite 范围。
3. 方案明确 parent-mediated `build_experiment_overrides` handoff，不新增 sibling horizontal link。
4. 治理门禁、全量树覆盖和 `git diff --check` 均通过。
