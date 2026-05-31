# v4.16.0 runtime.backtest_import_pass 单子叶等价基线

> 版本类型: MINOR architecture / governance
> 执行档位: 标准
> 批次: BE-001DD-01
> 基准: `326-runtime.parent_import_bridge父叶残余判断.md`
> 目标子叶: `runtime.backtest_import_pass`
> 模块树坐标: `root.backend.runtime.runtime.parent_import_bridge.runtime.backtest_import_pass`
> 代码动作: no code movement
> 下一步: BE-001DD-02 `runtime.backtest_import_pass` 抽离方案

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001DD-01 `runtime.backtest_import_pass` 单子叶等价基线 | 单子叶基线 |
| 规范矩阵 | explicit import pass、backtest child import、parent surface、release transition guard | 等价冻结 |
| 引导矩阵 | `root.backend.runtime.runtime.parent_import_bridge.runtime.backtest_import_pass` | backtest import 白箱边界 |
| 模块树 | `runtime.backtest_import_pass` | 新增基线 |

---

## 引导坐标

| 项 | 指向 |
| --- | --- |
| 引导坐标 | `root.backend.runtime.runtime.parent_import_bridge.runtime.backtest_import_pass` |
| 父级白箱 | `runtime.parent_import_bridge` |
| 当前真实 owner | `src/runtime/mod.rs` 挂载 backtest 子树并通过 `pub(crate) use` 暴露 route handlers |
| 当前真实文件 | `src/runtime/backtest/execution_start.rs`、`src/runtime/backtest/experiment_sweep.rs`、`src/runtime/backtest/record_store.rs`、`src/runtime/backtest/replay.rs`、`src/runtime/backtest/legacy_dispatch.rs`、`src/runtime/backtest/parameter_grid.rs`、`src/runtime/backtest/record_lifecycle.rs`、`src/runtime/backtest/start_orchestration.rs`、`src/runtime/backtest/v4_projection.rs`、`src/runtime/backtest/v4_request_resolution.rs`、`src/runtime/backtest/v4_runtime_execution.rs` |
| public 方法 | `start_backtest_run`、`list_backtests`、`get_backtest_detail`、`save_backtest_record`、`discard_backtest_record`、`get_backtest_replay`、`start_backtest_experiment`、`list_experiments`、`get_experiment_detail`、`save_experiment_record`、`discard_experiment_record` |
| internal 方法 | `execute_backtest_request`、`prepare_legacy_backtest_dispatch`、`run_legacy_backtest_dispatch`、`build_experiment_overrides`、`build_v4_backtest_output`、`frontend_events_from_v4_backtest_artifact`、`v4_equity_curve_from_artifact`、`is_v4_backtest_request`、`resolve_v4_backtest_graph`、`resolve_v4_backtest_symbols`、`resolve_v4_backtest_market_event_type`、`run_v4_backtest_runtime_execution` |
| 测试/门禁 | `cargo check -p quantpilot`、`cargo test -p quantpilot --test api_backtest`、`cargo test -p quantpilot --test api_experiments`、`cargo test -p quantpilot --test api_evidence_contract`、`cargo fmt --check`、`tools/check-utf8.ps1`、`tools/check-matrix-governance.ps1`、`tools/check-full-feature-tree.ps1`、`git diff --check` |

---

## 当前残余文件

本基线冻结 11 个 `runtime.backtest` 残余文件。当前它们仍含 `use super::*` 或 `super::`:

```text
src/runtime/backtest/execution_start.rs
src/runtime/backtest/experiment_sweep.rs
src/runtime/backtest/legacy_dispatch.rs
src/runtime/backtest/parameter_grid.rs
src/runtime/backtest/record_lifecycle.rs
src/runtime/backtest/record_store.rs
src/runtime/backtest/replay.rs
src/runtime/backtest/start_orchestration.rs
src/runtime/backtest/v4_projection.rs
src/runtime/backtest/v4_request_resolution.rs
src/runtime/backtest/v4_runtime_execution.rs
```

计数锚点: backtest 11 / runtime parent bridge total 34。

---

## 当前挂载结构

`src/runtime/mod.rs` 直接挂载 4 个 backtest 入口模块:

```text
#[path = "backtest/execution_start.rs"] mod backtest_execution_start
#[path = "backtest/experiment_sweep.rs"] mod backtest_experiment_sweep
#[path = "backtest/record_store.rs"] mod backtest_record_store
#[path = "backtest/replay.rs"] mod backtest_replay
```

`execution_start.rs` 内部挂载:

```text
legacy_dispatch
v4_projection
v4_request_resolution
v4_runtime_execution
```

`experiment_sweep.rs` 内部挂载:

```text
parameter_grid
record_lifecycle
start_orchestration
```

---

## 等价边界

本基线只冻结 import 收敛边界，不迁移 handler、route、schema、persistence、artifact、runtime engine 或 storage owner。

| 边界 | 当前 owner | 等价要求 |
| --- | --- | --- |
| backtest start | `src/runtime/backtest/execution_start.rs` | `start_backtest_run` 与 `execute_backtest_request` 的 route path、status code、error code、artifact views 与 v4/legacy 分流不变 |
| legacy dispatch | `src/runtime/backtest/legacy_dispatch.rs` | `prepare_legacy_backtest_dispatch` 与 `run_legacy_backtest_dispatch` 的 compile/runtime 行为不变 |
| v4 projection | `src/runtime/backtest/v4_projection.rs` | v4 output、equity curve、frontend event projection 不变 |
| v4 request resolution | `src/runtime/backtest/v4_request_resolution.rs` | graph、symbols、market event type 解析不变 |
| v4 runtime execution | `src/runtime/backtest/v4_runtime_execution.rs` | deterministic replay bars/ticks、runtime matrix 与 spawn_blocking 行为不变 |
| backtest records | `src/runtime/backtest/record_store.rs` | list/detail/save/discard 的 pagination、scoped lookup、audit 与 transient deletion 不变 |
| backtest replay | `src/runtime/backtest/replay.rs` | `RuntimeReplayQuery`、`normalized_replay_options`、replay page metrics 不变 |
| experiment facade | `src/runtime/backtest/experiment_sweep.rs` | experiment route exports 和 internal child wiring 不变 |
| experiment record lifecycle | `src/runtime/backtest/record_lifecycle.rs` | list/detail/save/discard experiment 行为不变 |
| experiment start | `src/runtime/backtest/start_orchestration.rs` | sweep variants、overrides、actor、created_at 与 per-variant backtest execution 不变 |
| parameter grid | `src/runtime/backtest/parameter_grid.rs` | float/latency axes normalization 与 override generation 不变 |

---

## 父级输入面

后续 explicit import rewrite 必须把父级输入面显式化，但不得改变 owner:

- `AppState`、`auth::UserId`、Axum `State` / `Path` / `Query` / `Json` / `StatusCode`。
- request / response schema: `FrontendRunRequest`、`FrontendExperimentRequest`、`BacktestRunResponse`、`BacktestDetailResponse`、`RuntimeReplayQuery`、`RuntimeReplayResponse`、`ExperimentDetailResponse`、`PaginatedResponse`。
- record / persistence helpers: `BacktestRecord`、`list_backtest_records`、`load_backtest_record_from_state`、`persist_backtest_record`、`delete_transient_backtest_record`、`list_experiment_records`、`load_experiment_record_from_state`、`persist_experiment_record`。
- artifact / response helpers: `backtest_run_response`、`backtest_detail_response_from_record`、`backtest_replay_response_from_record`、`backtest_list_item_from_record`、`experiment_detail_response_from_record`、`experiment_list_item_from_record`。
- validation / runtime helpers: `validate_runtime_capability_guard`、`validate_runtime_config_capabilities`、`validate_backtest_execution_assumption_overrides`、`compile_runtime_protocol_via_qs`、`resolved_backtest_execution_assumptions`、`runtime_simulated_v4_matrix`。
- shared utility helpers: `current_time_ms`、`internal_error`、`io_error`、`json_bad_request`、`json_bad_request_with_details`、`normalize_actor_identity`、`paginate`、`sanitize_storage_path_segment`。

---

## 下一步约束

BE-001DD-02 只能产出抽离方案，不得直接改 Rust。方案必须先判断 11 文件是否能整批 explicit import rewrite，或是否需要继续拆成:

```text
runtime.backtest.execution_start_import_pass
runtime.backtest.experiment_sweep_import_pass
runtime.backtest.record_store_import_pass
runtime.backtest.replay_import_pass
```

若选择继续拆，必须回到单子叶等价基线；若选择整批，必须解释为什么不会扩大变更面。

---

## 排除项

- 本批不修改 Rust 代码。
- 本批不直接改写 `src/runtime/backtest/**` import。
- 本批不处理 `src/runtime/mod.rs` root parent bridge。
- 本批不处理 `src/runtime/mutation/**` 或 test-only `src/runtime/run_guard.rs`。
- 本批不迁移 route registration、schema、artifact owner、persistence owner、runtime engine 或 frontend caller。
- 本批不新增 sibling horizontal link。
- 本批不启动 release transition。

---

## 验证要求

本批为 `no code movement` 单子叶等价基线，提交前至少执行:

```powershell
cargo fmt --check
cargo check -p quantpilot
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1
git diff --check
```

后续实际抽离时至少补跑:

```powershell
cargo test -p quantpilot --test api_backtest
cargo test -p quantpilot --test api_experiments
cargo test -p quantpilot --test api_evidence_contract
```

---

## 幻觉检查点

AI 声称 BE-001DD-01 完成时，必须说明:

1. 本批次是 `no code movement` 单子叶等价基线。
2. `runtime.backtest_import_pass` 冻结 11 个 backtest 残余文件。
3. 本批没有改写任何 `use super::*` 或 `super::`。
4. 下一步只能进入 BE-001DD-02 抽离方案。
5. `src/runtime/mod.rs`、mutation 子树和 test-only `src/runtime/run_guard.rs` 尚未处理。
6. release transition 未启动，未新增 sibling horizontal link。

不得宣称 `runtime.backtest_import_pass` 已抽离、parent import bridge 已清除或 `backend.runtime` 已完成。

---

## 验收标准

1. `327-runtime.backtest_import_pass单子叶等价基线.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. 基线明确 11 个 backtest 残余文件与 public/internal 方法边界。
3. 下一步固定为 BE-001DD-02 `runtime.backtest_import_pass` 抽离方案。
4. 治理门禁、全量树覆盖和 `git diff --check` 均通过。
