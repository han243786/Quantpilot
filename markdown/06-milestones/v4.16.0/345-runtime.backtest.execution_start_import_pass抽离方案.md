# v4.16.0 runtime.backtest.execution_start_import_pass 抽离方案

> 版本类型: MINOR architecture / governance
> 执行档位: 标准
> 批次: BE-001DK-02
> 基准: `344-runtime.backtest.execution_start_import_pass单子叶等价基线.md`
> 目标子叶: `runtime.backtest.execution_start_import_pass`
> 模块树坐标: `root.backend.runtime.runtime.parent_import_bridge.runtime.backtest_import_pass.runtime.backtest.execution_start_import_pass`
> 代码动作: no code movement
> 下一步: BE-001DK-03 actual Rust import rewrite

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001DK-02 `runtime.backtest.execution_start_import_pass` 抽离方案 | 方案优化 |
| 规范矩阵 | five-file explicit import rewrite、parent surface、execution_start pocket、release transition guard | 抽离方案 |
| 引导矩阵 | `root.backend.runtime.runtime.parent_import_bridge.runtime.backtest_import_pass.runtime.backtest.execution_start_import_pass` | execution_start import 收敛方案 |
| 模块树 | `runtime.backtest.execution_start_import_pass` | 抽离方案 |

---

## 方案结论

BE-001DK-03 可以进入 actual Rust import rewrite，范围仅限:

```text
src/runtime/backtest/execution_start.rs
src/runtime/backtest/legacy_dispatch.rs
src/runtime/backtest/v4_projection.rs
src/runtime/backtest/v4_request_resolution.rs
src/runtime/backtest/v4_runtime_execution.rs
```

结论:

1. 五文件仍属于同一个 execution_start import pocket，可以一批收敛。
2. 本批不是业务 owner 迁移；既不重做 BE-001N~BE-001S 的物理抽离，也不把 record write、artifact views、state write、audit log 再拆新 owner。
3. `v4_projection.rs` 的 test-scope `use super::*` 与顶层 parent wildcard 同批清理，避免实际抽离后该文件继续计入 residual。
4. 预期 runtime parent bridge 依赖文件数从 28 降为 23。

```text
expected_parent_import_bridge_28_to_23
```

---

## 允许改动

BE-001DK-03 只允许:

1. 删除五文件的顶层 `use super::*`。
2. 删除 `v4_projection.rs` test module 内的 `use super::*`，并补齐显式测试输入。
3. 为五文件补充 `crate::{...}`、`axum::{...}`、`serde_json::{...}`、`std::{...}`、`tokio::task` 等显式 import。
4. 保留 `execution_start.rs` 对 `legacy_dispatch`、`v4_projection`、`v4_request_resolution`、`v4_runtime_execution` 的父级白箱 handoff。
5. 如 `cargo check` 要求，允许在五文件范围内做最小 visibility / path 调整。

必须保留的父级白箱 handoff 名称:

```text
prepare_legacy_backtest_dispatch
run_legacy_backtest_dispatch
build_v4_backtest_output
frontend_events_from_v4_backtest_artifact
v4_equity_curve_from_artifact
is_v4_backtest_request
resolve_v4_backtest_graph
resolve_v4_backtest_symbols
resolve_v4_backtest_market_event_type
run_v4_backtest_runtime_execution
```

---

## 预期输入面

### `execution_start.rs`

预期显式输入至少覆盖:

```text
auth
account_summary_from_portfolio
attach_runtime_event_envelopes
backtest_run_response
build_backtest_artifact_views
build_backtest_spec
build_compile_runtime_targets_from_graph
collect_frontend_events_for_backtest
collaboration_with_run_actor
current_time_ms
internal_error
io_error
json_bad_request
json_bad_request_with_details
maybe_spill_transient_backtest_record
merge_runtime_targets
normalize_actor_identity
prepend_capability_snapshot_event
runtime_governance_snapshot
safe_eprintln
validate_backtest_execution_assumption_overrides
validate_runtime_capability_guard
validate_runtime_config_capabilities
validate_runtime_event_envelopes
AppState
BacktestRecord
BacktestRunResponse
FrontendRunRequest
```

外部 crate 输入至少覆盖:

```text
axum::{extract::State, http::StatusCode, Json}
serde_json::Value
std::sync::atomic::{AtomicU64, Ordering}
```

### `legacy_dispatch.rs`

预期显式输入至少覆盖:

```text
apply_backtest_execution_assumption_overrides
build_compile_artifact_bundle
compile_runtime_protocol_config
compile_runtime_protocol_via_qs
internal_error
resolved_backtest_execution_assumptions
resolved_execution_assumption_sources
BACKTEST_DETERMINISTIC_SEED
CompileArtifactBundle
ExecutionAssumptionSourceSummary
ExecutionAssumptionSpec
FrontendBacktestReplaySource
FrontendRunRequest
StrategyArtifactSourceKind
```

外部 crate 输入至少覆盖:

```text
axum::http::StatusCode
serde_json::Value
std::collections::BTreeMap
tokio::task
qrpc_core
qrpc_runtime::{slippage, DeterministicTestMode, FastBacktestSandbox}
```

### `v4_projection.rs`

预期显式输入至少覆盖:

```text
RuntimeEventEnvelope
FrontendRuntimeEvent
serde_json::{json, Value}
qrpc_core
qrpc_core_ir
```

test-scope 必须显式引入被测试函数与类型，不能继续使用 wildcard。

### `v4_request_resolution.rs`

预期显式输入至少覆盖:

```text
compile_runtime_protocol_config
compile_runtime_protocol_via_qs
internal_error
json_bad_request_with_code
runtime_v4_static_bundle
FrontendRunRequest
```

外部 crate 输入至少覆盖:

```text
axum::http::StatusCode
serde_json::Value
qrpc_core_ir
qrpc_runtime
quantscript
crate::error_codes
```

### `v4_runtime_execution.rs`

预期显式输入至少覆盖:

```text
internal_error
runtime_simulated_v4_matrix
```

外部 crate 输入至少覆盖:

```text
axum::http::StatusCode
qrpc_core_ir
qrpc_runtime
tokio::task
```

---

## 等价保持

BE-001DK-03 不得改变:

1. `start_backtest_run` / `execute_backtest_request` / `execute_v4_backtest_request` 的行为。
2. legacy compile、execution assumption override、deterministic mock / historical replay、sandbox replay、artifact bundle。
3. v4 graph resolution、symbol normalization、event type selection、runtime execution、portfolio/equity projection、frontend event ordering。
4. backtest id 生成、governance snapshot、capability event、event envelope、record assembly、artifact views、transient spill、state write、audit log。
5. route path、handler name、status code、response schema、persistence owner、state owner、frontend caller。

---

## 排除项

- BE-001DK-03 不得处理 `src/runtime/mod.rs` root parent bridge。
- BE-001DK-03 不得处理 `src/runtime/mutation/**` 或 test-only `src/runtime/run_guard.rs`。
- BE-001DK-03 不得处理 record_store / replay / experiment_sweep 已 closeout pocket。
- BE-001DK-03 不得迁移 route facade、state owner、persistence owner、schema owner、frontend caller 或 test asset。
- BE-001DK-03 不得新增 sibling horizontal link。
- BE-001DK-03 不得启动 release transition。
- BE-001DK-03 不得恢复旧的三叶暂停目标；递归队列继续保持 `old_three_leaf_pause_target_cancelled`。

---

## 验证要求

BE-001DK-03 实际抽离后至少执行:

```powershell
cargo fmt --check
cargo check -p quantpilot
cargo test -p quantpilot --test api_backtest
cargo test -p quantpilot --test api_evidence_contract
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-utf8.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-matrix-governance.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File tools\check-full-feature-tree.ps1
git diff --check
```

---

## 幻觉检查点

AI 声称 BE-001DK-02 完成时，必须说明:

1. 本批次是 `no code movement` 抽离方案。
2. BE-001DK-03 只允许改写五文件 import 和必要 visibility。
3. 预期 parent bridge 依赖文件数从 28 降为 23。
4. root bridge、mutation 子树、test-only `src/runtime/run_guard.rs` 不在范围内。
5. release transition 未启动，未新增 sibling horizontal link。

不得宣称 execution_start import 已经改写、`runtime.backtest_import_pass` 已完成、`backend.runtime` 已完成或 parent import bridge 已完全清除。

---

## 验收标准

1. `345-runtime.backtest.execution_start_import_pass抽离方案.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. 方案固定 BE-001DK-03 五文件 import rewrite 范围。
3. 方案明确 test-scope `use super::*` 同批清理。
4. 治理门禁、全量树覆盖和 `git diff --check` 均通过。
