# v4.16.0 runtime.backtest.execution_start_import_pass 抽离记录

> 版本类型: MINOR architecture / governance
> 执行档位: 标准
> 批次: BE-001DK-03
> 基准: `345-runtime.backtest.execution_start_import_pass抽离方案.md`
> 目标子叶: `runtime.backtest.execution_start_import_pass`
> 模块树坐标: `root.backend.runtime.runtime.parent_import_bridge.runtime.backtest_import_pass.runtime.backtest.execution_start_import_pass`
> 代码动作: actual Rust import rewrite
> 下一步: BE-001DK-04 `runtime.backtest.execution_start_import_pass` 单叶 closeout

---

## 三矩阵影响声明

| 矩阵 | 影响节点 | 变更类型 |
| --- | --- | --- |
| 流程矩阵 | BE-001DK-03 `runtime.backtest.execution_start_import_pass` 实际抽离 | 实际抽离 |
| 规范矩阵 | five-file explicit import rewrite、parent surface、execution_start pocket、release transition guard | 等价收敛 |
| 引导矩阵 | `root.backend.runtime.runtime.parent_import_bridge.runtime.backtest_import_pass.runtime.backtest.execution_start_import_pass` | execution_start import 白箱收敛 |
| 模块树 | `runtime.backtest.execution_start_import_pass` | 抽离记录 |

---

## 实际改动

本批只改写以下五文件 import:

```text
src/runtime/backtest/execution_start.rs
src/runtime/backtest/legacy_dispatch.rs
src/runtime/backtest/v4_projection.rs
src/runtime/backtest/v4_request_resolution.rs
src/runtime/backtest/v4_runtime_execution.rs
```

删除五文件的 parent wildcard / super import:

```rust
use super::*;
super::
```

`v4_projection.rs` test-scope `use super::*` 也已替换为显式绝对路径输入，避免该文件继续计入 parent bridge residual。

---

## 实际输入面

### `execution_start.rs`

```rust
use crate::{
    account_summary_from_portfolio, attach_runtime_event_envelopes, auth, backtest_run_response,
    build_backtest_artifact_views, build_backtest_spec, build_compile_runtime_targets_from_graph,
    collaboration_with_run_actor, collect_frontend_events_for_backtest, current_time_ms,
    internal_error, io_error, json_bad_request, json_bad_request_with_details,
    maybe_spill_transient_backtest_record, merge_runtime_targets, normalize_actor_identity,
    prepend_capability_snapshot_event, runtime_governance_snapshot,
    validate_backtest_execution_assumption_overrides, validate_runtime_capability_guard,
    validate_runtime_config_capabilities, validate_runtime_event_envelopes, AppState,
    BacktestRecord, BacktestRunResponse, FrontendRunRequest,
};
use axum::{extract::State, http::StatusCode, Json};
use serde_json::Value;
use std::sync::atomic::{AtomicU64, Ordering};
```

父级白箱 handoff 保持:

```text
prepare_legacy_backtest_dispatch
run_legacy_backtest_dispatch
LegacyBacktestDispatchOutput
build_v4_backtest_output
frontend_events_from_v4_backtest_artifact
v4_equity_curve_from_artifact
is_v4_backtest_request
resolve_v4_backtest_graph
resolve_v4_backtest_market_event_type
resolve_v4_backtest_symbols
run_v4_backtest_runtime_execution
```

### `legacy_dispatch.rs`

```rust
use crate::{
    apply_backtest_execution_assumption_overrides, build_compile_artifact_bundle,
    compile_runtime_protocol_config, compile_runtime_protocol_via_qs, internal_error,
    resolved_backtest_execution_assumptions, resolved_execution_assumption_sources,
    CompileArtifactBundle, DeterministicTestMode, ExecutionAssumptionSourceSummary,
    ExecutionAssumptionSpec, FastBacktestSandbox, FrontendBacktestReplaySource, FrontendRunRequest,
    StrategyArtifactSourceKind, BACKTEST_DETERMINISTIC_SEED,
};
use axum::http::StatusCode;
use qrpc_core::BacktestOutput;
use qrpc_runtime::Sandbox;
use serde_json::Value;
use std::collections::BTreeMap;
use tokio::task;
```

### `v4_projection.rs`

```rust
use crate::{FrontendRuntimeEvent, RuntimeEventEnvelope};
use serde_json::{json, Value};
```

test-scope 显式输入:

```rust
use crate::runtime::backtest_execution_start::v4_projection::{
    v4_equity_curve_from_artifact, v4_win_rate_from_equity_curve,
};
```

### `v4_request_resolution.rs`

```rust
use crate::{
    compile_runtime_protocol_config, compile_runtime_protocol_via_qs, internal_error,
    json_bad_request, json_bad_request_with_code, runtime::runtime_v4_static_bundle,
    FrontendRunRequest,
};
use axum::http::StatusCode;
use serde_json::Value;
```

### `v4_runtime_execution.rs`

```rust
use crate::{internal_error, runtime::runtime_simulated_v4_matrix};
use axum::http::StatusCode;
use tokio::task;
```

---

## 等价结果

- `start_backtest_run`、`execute_backtest_request`、`execute_v4_backtest_request` 行为未变。
- legacy compile、execution assumption override、deterministic mock / historical replay、sandbox replay、artifact bundle 未变。
- v4 graph resolution、symbol normalization、event type selection、runtime execution、portfolio/equity projection、frontend event ordering 未变。
- backtest id 生成、governance snapshot、capability event、event envelope、record assembly、artifact views、transient spill、state write、audit log 未变。
- 五文件不再包含 `use super::*` 或 `super::`。
- runtime parent bridge 依赖文件数从 28 降为 23。
- 当前分布为 root 1 / run 0 / backtest 0 / mutation 21 / test-only 1 / total 23。

```text
actual_parent_import_bridge_28_to_23
```

---

## 当前剩余

`runtime.backtest` 的 parent bridge import residual 已清零。剩余 parent bridge 依赖集中在:

```text
src/runtime/mod.rs
src/runtime/mutation/**
src/runtime/run_guard.rs
```

---

## 排除项

- 本批不处理 `src/runtime/mod.rs` root parent bridge。
- 本批不处理 `src/runtime/mutation/**` 或 test-only `src/runtime/run_guard.rs`。
- 本批不处理 record_store / replay / experiment_sweep 已 closeout pocket。
- 本批不迁移 route facade、state owner、persistence owner、schema owner、frontend caller 或 test asset。
- 本批不新增 sibling horizontal link。
- 本批不启动 release transition。
- 本批不恢复旧的三叶暂停目标；递归队列继续保持 `old_three_leaf_pause_target_cancelled`。

---

## 验证要求

本批提交前至少执行:

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

AI 声称 BE-001DK-03 完成时，必须说明:

1. 本批次是 actual Rust import rewrite。
2. 只改写五文件 import 和必要路径。
3. runtime parent bridge 依赖文件数从 28 降为 23，backtest residual 为 0。
4. 下一步只能进入 BE-001DK-04 `runtime.backtest.execution_start_import_pass` 单叶 closeout。
5. root bridge、mutation 子树和 test-only `src/runtime/run_guard.rs` 尚未处理。
6. release transition 未启动，未新增 sibling horizontal link。

不得宣称 `runtime.backtest_import_pass` 已 closeout、`backend.runtime` 已完成或 parent import bridge 已完全清除。

---

## 验收标准

1. `346-runtime.backtest.execution_start_import_pass抽离记录.md` 进入里程碑索引、模块树、全量树和治理门禁。
2. 五个目标 Rust 文件均不再包含 `use super::*` 或 `super::`。
3. runtime parent bridge residual 从 28 降为 23，backtest residual 为 0。
4. Rust 编译、`api_backtest`、`api_evidence_contract`、治理门禁、全量树覆盖和 `git diff --check` 均通过。
